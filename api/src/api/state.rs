use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use sqlx::migrate::MigrateError;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;
use crate::api::db;
use crate::api::db::Invoice;
use crate::utils;
use crate::utils::get_env_var;

pub async fn setup_app_state() -> Result<AppState, String> {
    let db = DB::new().await?;
    Ok(AppState { db })
}

#[derive(Clone)]
pub struct DB {
    pg_pool: PgPool,
}

#[derive(Clone)]
pub struct AppState {
    pub db: DB,
}

impl DB {
    pub async fn new() -> Result<Self, String> {
        let db_url = get_env_var("POSTGRES_URL")?;

        let pg_pool = db::get_db_connection(&db_url).await.map_err(|_| "Failed to connect to database".to_string())?;

        Ok(Self { pg_pool })
    }

    pub async fn run_migrations(&self) -> Result<(), MigrateError> {
        sqlx::migrate!("./migrations")
            .run(&self.pg_pool)
            .await
    }

    pub async fn list_invoices(&self, limit: i64, offset: i64) -> Result<Vec<Invoice>, String> {
        db::list_invoices(&self.pg_pool, limit, offset)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get invoices"))
    }

    pub async fn create_invoice(&self, amount: BigDecimal, seller: &str) -> Result<Invoice, String> {
        db::create_invoice(&self.pg_pool, amount, seller)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create invoice"))
    }

    pub async fn get_invoice(&self, id: Uuid) -> Result<Invoice, String> {
        db::get_invoice(&self.pg_pool, id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create invoice"))
    }

    pub async fn set_invoice_paid(&self, id: Uuid, seller: &str, amount: BigDecimal, buyer: &str, paid_at: NaiveDateTime) -> Result<Invoice, String> {
        let invoice = db::set_invoice_paid(&self.pg_pool, id, seller, amount, buyer, paid_at)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "set invoice paid"))?;

        info!("Invoice {} set paid", id);

        return Ok(invoice);
    }
}