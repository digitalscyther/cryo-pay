use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use rs_firebase_admin_sdk::{credentials_provider, App, GcpCredentials, auth::token::{error::TokenVerificationError, jwt::JWToken, TokenVerifier}};
use sqlx::migrate::MigrateError;
use sqlx::PgPool;
use uuid::Uuid;
use crate::api::db;
use crate::api::db::Invoice;
use crate::network::Network;
use crate::utils;
use crate::utils::get_env_var;

pub async fn setup_app_state(networks: Vec<Network>) -> Result<AppState, String> {
    let db = DB::new().await?;
    let gc = GC::new().await?;
    Ok(AppState { db, networks, gc, jwt_secret: "your_secret_key".to_string() })
}

#[derive(Clone)]
pub struct DB {
    pg_pool: PgPool,
}

#[derive(Clone)]
pub struct GC {
    credentials: GcpCredentials,
}

#[derive(Clone)]
pub struct AppState {
    pub db: DB,
    pub networks: Vec<Network>,
    pub gc: GC,
    pub jwt_secret: String,
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

    pub async fn create_invoice(&self, amount: BigDecimal, seller: &str, networks: &Vec<i32>) -> Result<Invoice, String> {
        db::create_invoice(&self.pg_pool, amount, seller, networks)
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

        return Ok(invoice);
    }

    pub async fn get_block_number(&self, network: &str) -> Result<Option<i64>, String> {
        db::get_block_number(&self.pg_pool, network)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get block number"))
    }

    pub async fn set_block_number(&self, network: &str, block_number: i64) -> Result<(), String> {
        db::create_or_update_block_number(&self.pg_pool, network, block_number)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get block number"))
    }
}

impl GC {
    async fn new() -> Result<Self, String> {
        let credentials = credentials_provider()
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get google cloud provider"))?;
        return Ok(Self { credentials });
    }
    pub async fn get_verified_token(&self, token: &str) -> Result<JWToken, VerifyError> {
        let live_app = App::live(self.credentials.to_owned())
            .await
            .map_err(|err| VerifyError::Unexpected(
                utils::make_err(Box::new(err.current_context().clone()),
                                "build live app")
            ))?;
        let verifier = live_app.id_token_verifier()
            .await
            .map_err(|err| VerifyError::Unexpected(
                utils::make_err(Box::new(err.current_context().clone()),
                                "get verifier")
            ))?;

        match verifier.verify_token(token).await {
            Ok(token) => Ok(token),
            Err(err) => Err(VerifyError::Verification(err.current_context().clone()))
        }
    }
}

#[derive(Debug)]
pub enum VerifyError {
    Verification(TokenVerificationError),
    Unexpected(String),
}