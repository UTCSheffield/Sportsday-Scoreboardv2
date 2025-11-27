use std::io::{Error, ErrorKind};

use actix::{Actor, Addr};
use actix_files::Files;
use actix_web::{middleware as ActixMiddleware, web, App, HttpServer};
use async_sqlite::PoolBuilder;
use log::debug;

use crate::{
    configurator::parser::Configuration,
    middleware::authentication::{AuthConfig, Authentication},
    websocket::ChannelsActor,
};

mod configurator;
mod db;
mod middleware;
mod routes;
mod templates;
mod utils;
mod websocket;

#[cfg(test)]
mod test_harness;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Init Logging/Environment
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    let db_url = std::env::var("DB_URL").unwrap_or_else(|_| "./db.sqlite".to_string());

    let oauth_client_id = std::env::var("GITHUB_OAUTH_CLIENT_ID").unwrap();
    let oauth_client_secret = std::env::var("GITHUB_OAUTH_CLIENT_SECRET").unwrap();

    // Create the DB
    let pool = match PoolBuilder::new().path(db_url).open().await {
        Ok(pool) => {
            log::info!("Established DB pool");
            pool
        }
        Err(e) => {
            log::error!("Error estalishing DB pool {e}");
            return Err(Error::new(
                ErrorKind::Other,
                "database pool could not be established",
            ));
        }
    };

    match db::create_tables(&pool).await {
        Ok(_) => log::info!("Ran Migrations"),
        Err(e) => {
            log::error!("Database Migrations failed {e}");
            return Err(Error::new(ErrorKind::Other, "database migrations failed"));
        }
    }

    // Reqwest Client
    let client = reqwest::Client::builder()
        .user_agent("SportsDayScore")
        .build()
        .unwrap();

    // Create the Plan & Run it
    let config = match configurator::parser::Configuration::from_yaml_file("./config.yaml") {
        Ok(config) => {
            let plan = configurator::build::build_plan(config.clone());
            // Check if the version has already been built
            if std::fs::exists("./version.txt").unwrap() {
                if std::fs::read_to_string("./version.txt").unwrap() == config.get_version() {
                    debug!("Config Version matches DB, not rebuilding");
                } else {
                    debug!("Config Version doesn't match DB, rebuilding");
                    configurator::run::run(plan, &pool).await.unwrap();
                    std::fs::write("./version.txt", config.get_version())?;
                }
            } else {
                debug!("Version state doesn't exist, rebuilding");
                configurator::run::run(plan, &pool).await.unwrap();
                std::fs::write("./version.txt", config.get_version())?;
            }
            config
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let ws_channels: Addr<ChannelsActor> = ChannelsActor::new().start();

    HttpServer::new(move || {
        App::new()
            .wrap(ActixMiddleware::Logger::default())
            .app_data(web::Data::new(AppState {
                client: client.clone(),
                config: config.clone(),
                pool: pool.clone(),
                oauth_creds: OauthCreds {
                    client_id: oauth_client_id.clone(),
                    client_secret: oauth_client_secret.clone(),
                },
            }))
            .app_data(web::Data::new(ws_channels.clone()))
            .service(Files::new("assets/", "assets/"))
            .service(routes::index::get)
            .service(routes::scoreboard::get)
            .service(routes::results::get)
            .service(routes::ws::get)
            .service(routes::oauth::callback_get)
            .service(
                web::scope("/set_scores")
                    .wrap(Authentication::new(AuthConfig::require_set_score()))
                    .service(routes::set_scores::get)
                    .service(routes::set_scores::post),
            )
            .service(
                web::scope("/admin")
                    .wrap(Authentication::new(AuthConfig::require_admin()))
                    .service(routes::admin::get)
                    .service(
                        web::scope("/users")
                            .service(routes::admin::users::list)
                            .service(routes::admin::users::create)
                            .service(routes::admin::users::edit)
                            .service(routes::admin::users::update)
                            .service(routes::admin::users::new),
                    ),
            )
    })
    .bind((host, port))?
    .run()
    .await?;
    Ok(())
}

struct AppState {
    client: reqwest::Client,
    config: Configuration,
    oauth_creds: OauthCreds,
    pool: async_sqlite::Pool,
}

struct OauthCreds {
    client_id: String,
    client_secret: String,
}
