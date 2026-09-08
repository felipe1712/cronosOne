use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::info;

pub type DbPool = PgPool;

pub async fn init_pool(database_url: &str, max_connections: u32) -> Result<DbPool, sqlx::Error> {
    info!("Conectando al pool de PostgreSQL (ExposureIQ)...");
    
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;

    info!("Conexión a PostgreSQL establecida exitosamente.");
    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    info!("Ejecutando migraciones de base de datos...");
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    info!("Migraciones ejecutadas exitosamente.");
    Ok(())
}
