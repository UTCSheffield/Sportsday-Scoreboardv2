use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error,
};
use futures::future::{ok, Ready};
use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

pub struct DefaultHtmlContentType;

impl<S, B> Transform<S, ServiceRequest> for DefaultHtmlContentType
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = DefaultHtmlContentTypeMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(DefaultHtmlContentTypeMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct DefaultHtmlContentTypeMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for DefaultHtmlContentTypeMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            let headers = res.headers_mut();

            if !headers.contains_key(header::CONTENT_TYPE) {
                headers.insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("text/html; charset=utf-8"),
                );
            }

            if headers.get(header::CONTENT_TYPE)
                != Some(&header::HeaderValue::from_static(
                    "text/html; charset=utf-8",
                ))
            {
                headers.insert(
                    header::CACHE_CONTROL,
                    header::HeaderValue::from_static("max-age=600"),
                );
            }

            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().body("test")
    }

    async fn test_handler_with_content_type() -> HttpResponse {
        HttpResponse::Ok()
            .content_type("application/json")
            .body("{}")
    }

    #[actix_web::test]
    async fn test_default_html_content_type_middleware_adds_header() {
        let app = test::init_service(
            App::new()
                .wrap(DefaultHtmlContentType)
                .route("/", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
    }

    #[actix_web::test]
    async fn test_default_html_content_type_middleware_respects_existing() {
        let app = test::init_service(
            App::new()
                .wrap(DefaultHtmlContentType)
                .route("/json", web::get().to(test_handler_with_content_type)),
        )
        .await;

        let req = test::TestRequest::get().uri("/json").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
        // Should add cache control for non-HTML responses
        assert!(resp.headers().contains_key(header::CACHE_CONTROL));
    }

    #[actix_web::test]
    async fn test_cache_control_added_for_non_html() {
        let app = test::init_service(
            App::new()
                .wrap(DefaultHtmlContentType)
                .route("/json", web::get().to(test_handler_with_content_type)),
        )
        .await;

        let req = test::TestRequest::get().uri("/json").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(
            resp.headers().get(header::CACHE_CONTROL).unwrap(),
            "max-age=600"
        );
    }
}
