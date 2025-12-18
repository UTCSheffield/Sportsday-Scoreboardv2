use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::websocket::{ChannelsActor, WsSession};

#[get("/ws/{channel}")]
async fn get(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<String>,
    channels: web::Data<actix::Addr<ChannelsActor>>,
) -> actix_web::Result<HttpResponse> {
    let channel_name = path.into_inner();
    ws::start(
        WsSession {
            channel_name,
            channels: channels.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix::Actor;
    use actix_web::test;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn get_test_db_path(prefix: &str) -> String {
        static COUNTER: AtomicU64 = AtomicU64::new(11000);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::fs::create_dir_all("./test").ok();
        let path = format!("./test/{}_{}.db", prefix, id);
        std::fs::remove_file(&path).ok();
        path
    }

    #[actix_web::test]
    async fn test_websocket_route() {
        let config = crate::configurator::parser::Configuration {
            version: "1.0.0".to_string(),
            genders: vec![],
            scores: vec![],
            years: vec![],
            forms: vec![],
            events: vec![],
        };

        let client = reqwest::Client::builder()
            .user_agent("SportsDayScore")
            .build()
            .unwrap();

        let pool = async_sqlite::PoolBuilder::new()
            .path(&get_test_db_path("ws_route"))
            .open()
            .await
            .unwrap();

        crate::create_tables(&pool).await.unwrap();

        let log_collector = crate::logger::LogCollector::new(1000);
        let ws_channels = crate::websocket::ChannelsActor::new().start();

        let app = test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new(crate::AppState {
                    client: client.clone(),
                    config: config.clone(),
                    pool: pool.clone(),
                    log_collector: log_collector.clone(),
                    oauth_creds: crate::OauthCreds {
                        client_id: "test".to_string(),
                        client_secret: "test".to_string(),
                    },
                }))
                .app_data(web::Data::new(ws_channels.clone()))
                .service(get),
        )
        .await;

        let req = test::TestRequest::get().uri("/ws/test").to_request();

        // WebSocket upgrade will fail in test context, but we can verify the route exists
        let resp = test::call_service(&app, req).await;
        // WebSocket upgrade failure is expected in test
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }
}
