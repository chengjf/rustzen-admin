use figment::{
    Figment,
    providers::{Env, Serialized},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// application port
    pub app_port: u16,
    /// application host
    pub app_host: String,
    /// database URL
    pub db_url: String,
    /// database maximum connection count
    pub db_max_conn: u32,
    /// database minimum connection count
    pub db_min_conn: u32,
    /// database connection timeout
    pub db_conn_timeout: u64,
    /// database idle connection timeout
    pub db_idle_timeout: u64,
    /// session expiration time in seconds
    pub session_expiration_secs: i64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_port: 8007,
            app_host: "0.0.0.0".into(),
            db_url: "postgres://postgres:postgres@localhost:5432/rustzen".into(),
            db_max_conn: 10,
            db_min_conn: 1,
            db_conn_timeout: 10,
            db_idle_timeout: 0,
            session_expiration_secs: 60 * 60 * 8, // 8 hours
        }
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config: Config = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Env::prefixed("RUSTZEN_"))
        .extract()
        .expect("Failed to load configuration");

    tracing::info!(
        "CONFIG: host={}:{}, db_max_conn={}, db_min_conn={}, session_expiration_secs={}",
        config.app_host,
        config.app_port,
        config.db_max_conn,
        config.db_min_conn,
        config.session_expiration_secs,
    );

    config
});
