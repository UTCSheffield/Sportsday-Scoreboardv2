// Library exports for integration tests

pub mod configurator;
pub mod db;
pub mod logger;
pub mod middleware;
pub mod prometheus;
pub mod routes;
pub mod templates;
pub mod utils;
pub mod websocket;

#[cfg(test)]
pub mod test_harness;

// Re-export commonly used items
pub use db::create_tables;

use async_sqlite::Pool;
use configurator::parser::Configuration;
use logger::LogCollector;

pub struct AppState {
    pub client: reqwest::Client,
    pub config: Configuration,
    pub log_collector: LogCollector,
    pub oauth_creds: OauthCreds,
    pub pool: Pool,
}

pub struct OauthCreds {
    pub client_id: String,
    pub client_secret: String,
}
