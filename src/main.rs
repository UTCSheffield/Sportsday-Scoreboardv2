use std::io::{Error, ErrorKind};

use actix::{Actor, Addr};
use actix_files::Files;
use actix_web::{middleware as ActixMiddleware, web, App, HttpServer};
use async_sqlite::PoolBuilder;
use log::debug;

use crate::{configurator::parser::Configuration, websocket::ChannelsActor};

mod configurator;
mod db;
mod routes;
mod templates;
mod utils;
mod websocket;

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
            }))
            .app_data(web::Data::new(ws_channels.clone()))
            .service(Files::new("assets/", "assets/"))
            .service(routes::index::get)
            .service(routes::scoreboard::get)
            .service(routes::set_scores::post)
            .service(routes::set_scores::get)
            .service(routes::results::get)
            .service(routes::ws::get)
    })
    .bind((host, port))?
    .run()
    .await?;
    Ok(())
}

struct AppState {
    client: reqwest::Client,
    config: Configuration,
    pool: async_sqlite::Pool,
}
