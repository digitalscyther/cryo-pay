use std::sync::Arc;
use bigdecimal::BigDecimal;
use hex;
use sha2;
use chrono::NaiveDateTime;
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use redis::{aio::ConnectionManager, AsyncCommands, RedisResult};
use rs_firebase_admin_sdk::{credentials_provider, App, auth::token::{cache::{HttpCache, PubKeys}, error::TokenVerificationError, jwt::JWToken, LiveTokenVerifier, TokenVerifier}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::migrate::MigrateError;
use sqlx::PgPool;
use uuid::Uuid;
use crate::db::{self, ApiKey, CallbackUrl, Invoice, User, Webhook};
use crate::db::billing::{self, Payment, Subscription};
use crate::error::AppError;
use crate::monitoring::health::DaemonHealth;
use crate::network::Network;
use crate::telegram::TelegramClient;
use crate::utils;

const SUGGESTED_GAS_FEE_TIMEOUT: u64 = 60 * 10;

pub async fn setup_app_state(
    networks: Vec<Network>, db: DB, telegram_client: TelegramClient,
    daemon_health: Arc<DaemonHealth>,
) -> Result<AppState, String> {
    let gc = GC::new().await?;
    let jwt = JWT::new()?;
    let redis = Redis::new().await?;
    let infura_token = utils::get_env_var("INFURA_TOKEN")?;

    Ok(AppState { db, telegram_client, networks, gc, jwt, redis, infura_token, daemon_health })
}

#[derive(Clone)]
pub struct DB {
    pg_pool: PgPool,
}

type FirebaseVerifier = LiveTokenVerifier<HttpCache<reqwest::Client, PubKeys>>;

#[derive(Clone)]
pub struct GC {
    verifier: Arc<FirebaseVerifier>,
}

#[derive(Clone)]
pub struct JWT {
    secret: String,
}

#[derive(Clone)]
pub struct Redis {
    connection: ConnectionManager,
}

#[derive(Clone)]
pub struct AppState {
    pub db: DB,
    pub telegram_client: TelegramClient,
    pub networks: Vec<Network>,
    pub gc: GC,
    pub jwt: JWT,
    pub redis: Redis,
    pub infura_token: String,
    pub daemon_health: Arc<DaemonHealth>,
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
        let secret = utils::get_env_var("APP_SECRET")?;

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

    pub fn internal_token(&self) -> String {
        use sha2::{Sha256, Digest};
        hex::encode(Sha256::digest(format!("internal:{}", self.secret)))
    }
}

impl DB {
    pub async fn new() -> Result<Self, String> {
        let db_url = utils::get_env_var("POSTGRES_URL")?;

        let pg_pool = db::get_db_connection(&db_url).await.map_err(|_| "Failed to connect to database".to_string())?;

        Ok(Self { pg_pool })
    }

    pub async fn run_migrations(&self) -> Result<(), MigrateError> {
        sqlx::migrate!("./migrations")
            .run(&self.pg_pool)
            .await
    }

    pub async fn health_check(&self) -> Result<(), AppError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pg_pool)
            .await
            .map_err(AppError::Db)?;
        Ok(())
    }

    pub async fn list_invoices(&self, limit: i64, offset: i64, user_id: Option<Uuid>) -> Result<Vec<Invoice>, AppError> {
        db::invoice::list_invoices(&self.pg_pool, limit, offset, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn user_own_invoices(&self, limit: i64, offset: i64, user_id: &Uuid) -> Result<Vec<Invoice>, AppError> {
        db::invoice::user_own_invoices(&self.pg_pool, limit, offset, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn create_invoice(
        &self,
        amount: BigDecimal,
        seller: &str,
        networks: &Vec<i32>,
        user_id: Option<Uuid>,
        external_id: Option<String>,
        is_private: bool,
    ) -> Result<Invoice, AppError> {
        db::invoice::create_invoice(&self.pg_pool, amount, seller, networks, user_id, external_id, is_private)
            .await
            .map_err(AppError::Db)
    }

    pub async fn get_invoice(&self, id: &Uuid) -> Result<Option<Invoice>, AppError> {
        db::invoice::get_invoice(&self.pg_pool, id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn get_is_owner(&self, id: &Uuid, user_id: &Uuid) -> Result<bool, AppError> {
        db::invoice::get_is_owner(&self.pg_pool, id, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn set_invoice_paid(&self, id: Uuid, seller: &str, amount: BigDecimal, buyer: &str, paid_at: NaiveDateTime) -> Result<Invoice, AppError> {
        db::invoice::set_invoice_paid(&self.pg_pool, id, seller, amount, buyer, paid_at)
            .await
            .map_err(AppError::Db)
    }

    pub async fn get_block_number(&self, network: &str) -> Result<Option<i64>, AppError> {
        db::blockchain::get_block_number(&self.pg_pool, network)
            .await
            .map_err(AppError::Db)
    }

    pub async fn set_block_number(&self, network: &str, block_number: i64) -> Result<(), AppError> {
        db::blockchain::create_or_update_block_number(&self.pg_pool, network, block_number)
            .await
            .map_err(AppError::Db)
    }

    pub async fn get_user_by_id(&self, id: &Uuid) -> Result<User, AppError> {
        db::user::get_user_by_id(&self.pg_pool, id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn get_or_create_user(&self, firebase_user_id: &str, email: Option<String>) -> Result<User, AppError> {
        db::user::get_or_create_user(&self.pg_pool, firebase_user_id, email)
            .await
            .map_err(AppError::Db)
    }

    pub async fn update_user(
        &self,
        user_id: &Uuid,
        email_notification: Option<bool>,
        telegram_notification: Option<bool>,
    ) -> Result<User, AppError> {
        db::user::update_user(&self.pg_pool, user_id, email_notification, telegram_notification)
            .await
            .map_err(AppError::Db)
    }

    pub async fn set_user_telegram_chat_id(
        &self,
        user_id: &Uuid,
        telegram_chat_id: Option<String>,
    ) -> Result<(), AppError> {
        db::user::set_user_telegram_chat_id(&self.pg_pool, user_id, telegram_chat_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn delete_own_invoice(&self, id: &Uuid, user_id: &Uuid) -> Result<bool, AppError> {
        db::invoice::delete_own_invoice(&self.pg_pool, id, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn delete_api_key(&self, id: &Uuid, user_id: &Uuid) -> Result<bool, AppError> {
        db::api_key::delete_api_key(&self.pg_pool, id, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn create_api_key(
        &self,
        user_id: &Uuid,
        hashed_api_key: &str,
    ) -> Result<ApiKey, AppError> {
        db::api_key::create_api_key(&self.pg_pool, user_id, hashed_api_key)
            .await
            .map_err(AppError::Db)
    }

    pub async fn get_api_key(&self, id: &Uuid, user_id: &Uuid) -> Result<Option<ApiKey>, AppError> {
        db::api_key::get_api_key(&self.pg_pool, id, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn get_api_key_by_api_key(
        &self,
        hashed_api_key: &str,
    ) -> Result<Option<ApiKey>, AppError> {
        db::api_key::get_api_key_by_api_key(&self.pg_pool, hashed_api_key)
            .await
            .map_err(AppError::Db)
    }

    pub async fn list_api_key(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<ApiKey>, AppError> {
        db::api_key::list_api_key(&self.pg_pool, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn update_api_key_last_used(&self, id: &Uuid) -> Result<(), AppError> {
        db::api_key::update_api_key_last_used(&self.pg_pool, id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn count_api_keys_by_user_id(&self, user_id: &Uuid) -> Result<usize, AppError> {
        let count = db::api_key::count_api_keys_by_user_id(&self.pg_pool, user_id)
            .await
            .map_err(AppError::Db)?;
        Ok(count.unwrap_or(0) as usize)
    }

    pub async fn create_callback_url(&self, url: &str, user_id: &Uuid) -> Result<CallbackUrl, AppError> {
        db::callback_url::create_callback_url(&self.pg_pool, url, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn list_callback_urls(&self, user_id: &Uuid) -> Result<Vec<CallbackUrl>, AppError> {
        db::callback_url::list_callback_urls_by_user_id(&self.pg_pool, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn delete_callback_url(&self, callback_url_id: &Uuid, user_id: &Uuid) -> Result<bool, AppError> {
        db::callback_url::delete_callback_url_by_id_and_user_id(&self.pg_pool, callback_url_id, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn count_callback_urls(&self, user_id: &Uuid) -> Result<usize, AppError> {
        let count = db::callback_url::count_callback_urls_by_user_id(&self.pg_pool, user_id)
            .await
            .map_err(AppError::Db)?;
        Ok(count.unwrap_or(0) as usize)
    }

    pub async fn exists_callback_url(&self, url: &str, user_id: &Uuid) -> Result<bool, AppError> {
        db::callback_url::exists_callback_url(&self.pg_pool, url, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn get_payment(&self, id: &Uuid) -> Result<Option<Payment>, AppError> {
        billing::get_payment(&self.pg_pool, id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn user_list_payment(&self, user_id: &Uuid, limit: i64, offset: i64) -> Result<Vec<Payment>, AppError> {
        billing::user_list_payment(&self.pg_pool, user_id, limit, offset)
            .await
            .map_err(AppError::Db)
    }

    pub async fn list_payment(&self, payment_type: &str, limit: i64, offset: i64) -> Result<Vec<Payment>, AppError> {
        billing::list_payment(&self.pg_pool, payment_type, limit, offset)
            .await
            .map_err(AppError::Db)
    }

    pub async fn create_payment(&self, id: &Uuid, user_id: Option<Uuid>, data: &Value) -> Result<Payment, AppError> {
        billing::create_payment(&self.pg_pool, id, user_id, data)
            .await
            .map_err(AppError::Db)
    }

    pub async fn list_user_subscriptions(&self, user_id: &Uuid) -> Result<Vec<Subscription>, AppError> {
        billing::list_subscriptions(&self.pg_pool, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn get_user_active_subscription(&self, user_id: &Uuid, target: &str) -> Result<Option<Subscription>, AppError> {
        billing::get_active_subscription(&self.pg_pool, user_id, target)
            .await
            .map_err(AppError::Db)
    }

    pub async fn create_or_update_subscription(&self, user_id: &Uuid, target: &str, data: Option<Value>, until: NaiveDateTime) -> Result<(), AppError> {
        billing::create_or_update_subscription(&self.pg_pool, user_id, target, data, until)
            .await
            .map_err(AppError::Db)
    }

    pub async fn set_payment_paid(&self, id: &Uuid) -> Result<(), AppError> {
        billing::set_payment_paid(&self.pg_pool, id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn sync_payment_paid_at(&self, id: &Uuid, paid_at: &chrono::NaiveDateTime) -> Result<(), AppError> {
        billing::sync_payment_paid_at(&self.pg_pool, id, paid_at)
            .await
            .map_err(AppError::Db)
    }

    pub async fn create_webhook(&self, url: &str, secret: &str, user_id: &Uuid) -> Result<Webhook, AppError> {
        db::webhook::create_webhook(&self.pg_pool, url, secret, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn list_webhooks(&self, user_id: &Uuid) -> Result<Vec<Webhook>, AppError> {
        db::webhook::list_webhooks_by_user_id(&self.pg_pool, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn delete_webhook(&self, webhook_id: &Uuid, user_id: &Uuid) -> Result<bool, AppError> {
        db::webhook::delete_webhook_by_id_and_user_id(&self.pg_pool, webhook_id, user_id)
            .await
            .map_err(AppError::Db)
    }

    pub async fn count_webhooks(&self, user_id: &Uuid) -> Result<usize, AppError> {
        let count = db::webhook::count_webhooks_by_user_id(&self.pg_pool, user_id)
            .await
            .map_err(AppError::Db)?;
        Ok(count.unwrap_or(0) as usize)
    }

    pub async fn invoice_stats_by_day(
        &self,
        user_id: &Uuid,
        since: NaiveDateTime,
    ) -> Result<Vec<db::analytics::InvoicePeriodStats>, AppError> {
        db::analytics::invoice_stats_by_day(&self.pg_pool, user_id, since)
            .await
            .map_err(AppError::Db)
    }

    pub async fn invoice_summary(
        &self,
        user_id: &Uuid,
        since: NaiveDateTime,
    ) -> Result<db::analytics::InvoiceSummary, AppError> {
        db::analytics::invoice_summary(&self.pg_pool, user_id, since)
            .await
            .map_err(AppError::Db)
    }

    #[cfg(test)]
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pg_pool: pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;

    async fn setup_test_db() -> DB {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for integration tests");
        let pool = PgPool::connect(&url).await
            .expect("Failed to connect to test database");
        let db = DB::from_pool(pool);
        db.run_migrations().await
            .expect("Failed to run migrations");
        db
    }

    #[ignore]
    #[tokio::test]
    async fn test_invoice_lifecycle() {
        let db = setup_test_db().await;

        let invoice = db.create_invoice(
            BigDecimal::from(10), "0xseller", &vec![10], None, None, false,
        ).await.unwrap();

        let fetched = db.get_invoice(&invoice.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, invoice.id);
        assert!(fetched.paid_at.is_none());

        let paid = db.set_invoice_paid(
            invoice.id, "0xseller", BigDecimal::from(10), "0xbuyer",
            chrono::Utc::now().naive_utc(),
        ).await.unwrap();
        assert!(paid.paid_at.is_some());
    }

    #[ignore]
    #[tokio::test]
    async fn test_user_get_or_create_idempotent() {
        let db = setup_test_db().await;
        let uid = format!("test-{}", Uuid::new_v4());
        let u1 = db.get_or_create_user(&uid, None).await.unwrap();
        let u2 = db.get_or_create_user(&uid, None).await.unwrap();
        assert_eq!(u1.id, u2.id);
    }

    #[ignore]
    #[tokio::test]
    async fn test_api_key_crud() {
        let db = setup_test_db().await;
        let uid = format!("test-{}", Uuid::new_v4());
        let user = db.get_or_create_user(&uid, None).await.unwrap();

        db.create_api_key(&user.id, "test_hashed_key_for_integration_test").await.unwrap();

        let keys = db.list_api_key(&user.id).await.unwrap();
        assert_eq!(keys.len(), 1);

        let deleted = db.delete_api_key(&keys[0].id, &user.id).await.unwrap();
        assert!(deleted);

        let keys_after = db.list_api_key(&user.id).await.unwrap();
        assert_eq!(keys_after.len(), 0);
    }
}

impl GC {
    async fn new() -> Result<Self, String> {
        let credentials = credentials_provider()
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get google cloud provider"))?;
        let app = App::live(credentials)
            .await
            .map_err(|err| utils::make_err(Box::new(err.current_context().clone()), "build live app"))?;
        let verifier = app.id_token_verifier()
            .await
            .map_err(|err| utils::make_err(Box::new(err.current_context().clone()), "get id token verifier"))?;
        Ok(Self { verifier: Arc::new(verifier) })
    }

    pub async fn get_verified_token(&self, token: &str) -> Result<JWToken, VerifyError> {
        match self.verifier.verify_token(token).await {
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

impl Redis {
    async fn new() -> Result<Self, String> {
        let client = redis::Client::open(utils::get_env_var("REDIS_URL")?)
            .map_err(|e| utils::make_err(Box::new(e), "get redis client"))?;

        let connection = ConnectionManager::new(client)
            .await
            .map_err(|e| utils::make_err(Box::new(e), "get redis connection"))?;

        Ok(Self { connection })
    }

    pub async fn health_check(&self) -> Result<(), AppError> {
        redis::cmd("PING")
            .query_async::<String>(&mut self.connection.clone())
            .await
            .map_err(AppError::Redis)?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>, AppError> {
        self.connection
            .clone()
            .get(key)
            .await
            .map_err(AppError::Redis)
    }

    async fn set(&self, key: &str, value: String, timeout: u64) -> Result<(), AppError> {
        let set_result: RedisResult<()> = self.connection
            .clone()
            .set_ex(key, value, timeout)
            .await;

        set_result.map_err(AppError::Redis)
    }

    pub async fn incr(&self, key: &str, timeout: u64) -> Result<u64, AppError> {
        let script = redis::Script::new(r#"
            local current = redis.call('INCR', KEYS[1])
            if current == 1 then
              redis.call('EXPIRE', KEYS[1], ARGV[1])
            end
            return current
        "#);
        script
            .key(key)
            .arg(timeout)
            .invoke_async::<u64>(&mut self.connection.clone())
            .await
            .map_err(AppError::Redis)
    }

    pub async fn get_suggested_gas_fees(&self, network: &i64) -> Result<Option<String>, AppError> {
        let redis_key = get_suggested_gas_fees_key(network);
        self.get(&redis_key).await
    }

    pub async fn set_suggested_gas_fees(&self, network: &i64, value: String) -> Result<(), AppError> {
        let redis_key = get_suggested_gas_fees_key(network);
        self.set(&redis_key, value, SUGGESTED_GAS_FEE_TIMEOUT).await
    }
}

fn get_suggested_gas_fees_key(network: &i64) -> String {
    format!("network-suggested-gas-fees:{}", network)
}