use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::rc::Rc;

use actix_web::cookie::Cookie;
use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};

use crate::db::user_sessions::UserSessions;

/// Configuration for the authentication middleware
#[derive(Clone)]
pub struct AuthConfig {
    /// Require the user to have `has_admin` permission
    pub require_admin: bool,
    /// Require the user to have `has_set_score` permission
    pub require_set_score: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            require_admin: false,
            require_set_score: false,
        }
    }
}

impl AuthConfig {
    /// Create a new AuthConfig requiring admin permissions
    pub fn require_admin() -> Self {
        Self {
            require_admin: true,
            require_set_score: false,
        }
    }

    /// Create a new AuthConfig requiring set_score permissions
    pub fn require_set_score() -> Self {
        Self {
            require_admin: false,
            require_set_score: true,
        }
    }

    /// Create a new AuthConfig requiring both admin and set_score permissions
    pub fn require_both() -> Self {
        Self {
            require_admin: true,
            require_set_score: true,
        }
    }

    /// Create a new AuthConfig with no permission requirements (just verify session exists)
    pub fn require_authenticated() -> Self {
        Self::default()
    }
}

/// Authentication middleware
pub struct Authentication {
    config: AuthConfig,
}

impl Authentication {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
    config: AuthConfig,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let config = self.config.clone();

        Box::pin(async move {
            // Extract the session_data cookie
            let session_data = req.cookie("session_data").map(|c| c.value().to_string());

            if session_data.is_none() {
                log::debug!("No session_data cookie found");
                let oauth_client_id = std::env::var("GITHUB_OAUTH_CLIENT_ID").unwrap();
                let redirect_url = format!(
                    "https://github.com/login/oauth/authorize?client_id={}&scope=user:email",
                    oauth_client_id
                );
                let res = HttpResponse::Found()
                    .append_header(("Location", redirect_url))
                    .cookie(Cookie::build("redirect-to", req.path()).path("/").finish())
                    .finish();
                return Ok(req.into_response(res).map_into_right_body());
            }

            log::debug!(
                "Found session_data cookie: {}",
                session_data.as_ref().unwrap()
            );

            let session_data = session_data.unwrap();

            // Get the database pool from app data
            let pool = match req.app_data::<actix_web::web::Data<crate::AppState>>() {
                Some(state) => state.pool.clone(),
                None => {
                    log::error!("Could not get database pool from app state");
                    return Ok(req.into_response(
                        HttpResponse::InternalServerError()
                            .body("Internal server error")
                            .map_into_right_body(),
                    ));
                }
            };

            // Verify the session
            let verified_session = match UserSessions::verify(&pool, session_data).await {
                Ok(session) => session,
                Err(e) => {
                    log::error!("Error verifying session: {}", e);
                    return Ok(req.into_response(
                        HttpResponse::InternalServerError()
                            .body("Internal server error")
                            .map_into_right_body(),
                    ));
                }
            };

            // Check if session is verified
            if !verified_session.verified {
                log::debug!("Session not verified");
                return Ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .body("Invalid session")
                        .map_into_right_body(),
                ));
            }

            // Check permissions based on config
            if config.require_admin && !verified_session.has_admin {
                log::debug!("User does not have admin permission");
                return Ok(req.into_response(
                    HttpResponse::Forbidden()
                        .body("Admin permission required")
                        .map_into_right_body(),
                ));
            }

            if config.require_set_score && !verified_session.has_set_score {
                log::debug!("User does not have set_score permission");
                return Ok(req.into_response(
                    HttpResponse::Forbidden()
                        .body("Set score permission required")
                        .map_into_right_body(),
                ));
            }

            // Store the verified session in request extensions for access in handlers
            req.extensions_mut().insert(verified_session);

            // Call the next service
            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}
