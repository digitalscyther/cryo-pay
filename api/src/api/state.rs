use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use rs_firebase_admin_sdk::{credentials_provider, App, GcpCredentials, auth::token::{error::TokenVerificationError, jwt::JWToken, TokenVerifier}};
use serde::{Deserialize, Serialize};
use sqlx::migrate::MigrateError;
use sqlx::PgPool;
use uuid::Uuid;
use crate::api::db::{self, Invoice, User};
use crate::network::Network;
use crate::utils;
use crate::utils::get_env_var;

pub async fn setup_app_state(networks: Vec<Network>) -> Result<AppState, String> {
    let db = DB::new().await?;
    let gc = GC::new().await?;
    let jwt = JWT::new()?;
    Ok(AppState { db, networks, gc, jwt })
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
pub struct JWT {
    secret: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: DB,
    pub networks: Vec<Network>,
    pub gc: GC,
    pub jwt: JWT,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    exp: usize,
}

impl Claims {
    fn from_jwt(token: &str, secret: &str) -> Result<Self, String> {
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        let token_data = decode::<Claims>(token, &decoding_key, &Validation::new(Algorithm::HS256))
            .map_err(|err| utils::make_err(Box::new(err), "decode jwt"))?;
        Ok(token_data.claims)
    }
}

impl JWT {
    fn new() -> Result<Self, String> {
        let secret = get_env_var("APP_SECRET")?;

        Ok(JWT { secret })
    }

    pub fn generate(&self, user_id: String, email: Option<String>) -> Result<String, jsonwebtoken::errors::Error> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id,
            email,
            exp: expiration,
        };

        encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_bytes()))
    }

    pub fn claims_from_jwt(&self, jwt: &str) -> Result<Claims, String> {
        Claims::from_jwt(jwt, &self.secret)
    }
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

    pub async fn list_invoices(&self, limit: i64, offset: i64, user_id: Option<Uuid>) -> Result<Vec<Invoice>, String> {
        db::list_invoices(&self.pg_pool, limit, offset, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get invoices"))
    }

    pub async fn create_invoice(&self, amount: BigDecimal, seller: &str, networks: &Vec<i32>, user_id: Option<Uuid>) -> Result<Invoice, String> {
        db::create_invoice(&self.pg_pool, amount, seller, networks, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create invoice"))
    }

    async fn get_invoice(&self, id: Uuid) -> Result<Invoice, String> {
        db::get_invoice(&self.pg_pool, id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get invoice"))
    }

    pub async fn get_own_invoice(&self, id: Uuid, user_id: Option<Uuid>) -> Result<(bool, Invoice), String> {
        let invoice = self.get_invoice(id).await?;

        let own = match user_id {
            None => false,
            Some(user_id) => db::get_is_owner(&self.pg_pool, id, user_id)
                .await
                .map_err(|err| utils::make_err(Box::new(err), "get invoice"))?
        };

        Ok((own, invoice))
    }

    pub async fn set_invoice_paid(&self, id: Uuid, seller: &str, amount: BigDecimal, buyer: &str, paid_at: NaiveDateTime) -> Result<Invoice, String> {
        db::set_invoice_paid(&self.pg_pool, id, seller, amount, buyer, paid_at)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "set invoice paid"))
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

    pub async fn get_or_create_user(&self, firebase_user_id: &str, email: Option<String>) -> Result<User, String> {
        db::get_or_create_user(&self.pg_pool, firebase_user_id, email)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get or create user"))
    }

    pub async fn get_email_to_notify(&self, _: &Uuid) -> Option<String> {
        None
    }
    pub async fn get_telegram_chat_id_to_notify(&self, _: &Uuid) -> Option<String> {
        None
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