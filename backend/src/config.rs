use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub upload_dir: String,
    pub worker_base_url: String,
    pub waha_session: String,
    pub director_whatsapp: String,
    pub n8n_restart_webhook_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/exposureiq_db".to_string()),
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev_secret_key_exposureiq_2026".to_string()),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            upload_dir: env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()),
            worker_base_url: env::var("WORKER_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8001".to_string()),
            waha_session: env::var("WAHA_SESSION").unwrap_or_else(|_| "default".to_string()),
            director_whatsapp: env::var("DIRECTOR_WHATSAPP_PHONE")
                .unwrap_or_else(|_| "5215512345678".to_string()),
            n8n_restart_webhook_url: env::var("N8N_RESTART_WEBHOOK_URL").ok(),
        }
    }
}
