use std::{net::SocketAddr, sync::Arc};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod routes;
pub mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "exposureiq_backend=debug,tower_http=debug,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Iniciando ExposureIQ Backend (Rust + Axum)...");

    let config = Arc::new(config::Config::from_env());

    let pool = match db::init_pool(&config.database_url, config.database_max_connections).await {
        Ok(p) => {
            if let Err(e) = db::run_migrations(&p).await {
                error!("Aviso: no se pudieron correr migraciones automáticas (podrían estar ya aplicadas): {}", e);
            }
            p
        }
        Err(e) => {
            error!("No se pudo conectar a PostgreSQL en {}: {}", config.database_url, e);
            error!("El servidor intentará iniciar en modo de espera o reintento.");
            panic!("Error fatal de conexión a PostgreSQL: {}", e);
        }
    };

    let app = routes::create_router(pool, config.clone());

    let addr = format!("{}:{}", config.host, config.port).parse::<SocketAddr>()?;
    info!("🚀 Servidor escuchando en http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
