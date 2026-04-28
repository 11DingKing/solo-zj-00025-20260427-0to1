use crate::config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
    pub config: Config,
}

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    // Migrations are handled by PostgreSQL init scripts via docker-entrypoint-initdb.d
    // Just verify the connection works
    sqlx::query("SELECT 1").execute(pool).await?;
    tracing::info!("Database connection verified");
    Ok(())
}
