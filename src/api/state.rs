use sqlx::migrate::MigrateError;
use sqlx::PgPool;
use crate::api::db;
use crate::api::db::Invoice;
use crate::utils;
use crate::utils::get_env_var;

pub async fn setup_app_state() -> Result<AppState, String> {
    let db_url = get_env_var("POSTGRES_URL")?;

    let pg_pool = db::get_db_connection(&db_url).await.map_err(|_| "Failed to connect to database".to_string())?;
    Ok(AppState { pg_pool })
}

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: PgPool,
}

impl AppState {
    pub async fn run_migrations(&self) -> Result<(), MigrateError> {
        sqlx::migrate!("./migrations")
            .run(&self.pg_pool)
            .await
    }

    pub async fn get_invoices(&self) -> Result<Vec<Invoice>, String> {
        db::get_invoices(&self.pg_pool)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get invoices"))
    }
}