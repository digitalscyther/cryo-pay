use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use redis::{aio::MultiplexedConnection, AsyncCommands, ConnectionLike, RedisResult};
use rs_firebase_admin_sdk::{credentials_provider, App, GcpCredentials, auth::token::{error::TokenVerificationError, jwt::JWToken, TokenVerifier}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::migrate::MigrateError;
use sqlx::PgPool;
use uuid::Uuid;
use crate::db::{self, ApiKey, CallbackUrl, Invoice, User, Webhook};
use crate::db::billing::{self, Payment, Subscription};
use crate::network::Network;
use crate::telegram::TelegramClient;
use crate::utils;

const SUGGESTED_GAS_FEE_TIMEOUT: u64 = 60 * 10;

pub async fn setup_app_state(networks: Vec<Network>, db: DB, telegram_client: TelegramClient) -> Result<AppState, String> {
    let gc = GC::new().await?;
    let jwt = JWT::new()?;
    let redis = Redis::new().await?;
    let infura_token = utils::get_env_var("INFURA_TOKEN")?;

    Ok(AppState { db, telegram_client, networks, gc, jwt, redis, infura_token })
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
pub struct Redis {
    connection: MultiplexedConnection,
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

    pub async fn list_invoices(&self, limit: i64, offset: i64, user_id: Option<Uuid>) -> Result<Vec<Invoice>, String> {
        db::list_invoices(&self.pg_pool, limit, offset, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get invoices"))
    }

    pub async fn user_own_invoices(&self, limit: i64, offset: i64, user_id: &Uuid) -> Result<Vec<Invoice>, String> {
        db::user_own_invoices(&self.pg_pool, limit, offset, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get invoices"))
    }

    pub async fn create_invoice(
        &self,
        amount: BigDecimal,
        seller: &str,
        networks: &Vec<i32>,
        user_id: Option<Uuid>,
        external_id: Option<String>,
        is_private: bool,
    ) -> Result<Invoice, String> {
        db::create_invoice(&self.pg_pool, amount, seller, networks, user_id, external_id, is_private)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create invoice"))
    }

    pub async fn get_invoice(&self, id: &Uuid) -> Result<Option<Invoice>, String> {
        db::get_invoice(&self.pg_pool, id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get invoice"))
    }

    pub async fn get_is_owner(&self, id: &Uuid, user_id: &Uuid) -> Result<bool, String> {
        db::get_is_owner(&self.pg_pool, id, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get is owner"))
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
            .map_err(|err| utils::make_err(Box::new(err), "set block number"))
    }

    pub async fn get_user_by_id(&self, id: &Uuid) -> Result<User, String> {
        db::get_user_by_id(&self.pg_pool, id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get user by id"))
    }

    pub async fn get_or_create_user(&self, firebase_user_id: &str, email: Option<String>) -> Result<User, String> {
        db::get_or_create_user(&self.pg_pool, firebase_user_id, email)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get or create user"))
    }

    pub async fn update_user(
        &self,
        user_id: &Uuid,
        email_notification: Option<bool>,
        telegram_notification: Option<bool>,
    ) -> Result<User, String> {
        db::update_user(&self.pg_pool, user_id, email_notification, telegram_notification)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "update user"))
    }

    pub async fn set_user_telegram_chat_id(
        &self,
        user_id: &Uuid,
        telegram_chat_id: Option<String>,
    ) -> Result<(), String> {
        db::set_user_telegram_chat_id(&self.pg_pool, user_id, telegram_chat_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "set user telegram chat id"))
    }

    pub async fn delete_own_invoice(&self, id: &Uuid, user_id: &Uuid) -> Result<bool, String> {
        db::delete_own_invoice(&self.pg_pool, id, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "delete own invoice"))
    }

    pub async fn delete_api_key(&self, id: &Uuid, user_id: &Uuid) -> Result<bool, String> {
        db::delete_api_key(&self.pg_pool, id, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "soft delete API key"))
    }

    pub async fn create_api_key(
        &self,
        user_id: &Uuid,
        hashed_api_key: &str,
    ) -> Result<ApiKey, String> {
        db::create_api_key(&self.pg_pool, user_id, hashed_api_key)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create API key"))
    }

    pub async fn get_api_key(&self, id: &Uuid, user_id: &Uuid) -> Result<Option<ApiKey>, String> {
        db::get_api_key(&self.pg_pool, id, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get API key by ID"))
    }

    pub async fn get_api_key_by_api_key(
        &self,
        hashed_api_key: &str,
    ) -> Result<Option<ApiKey>, String> {
        db::get_api_key_by_api_key(&self.pg_pool, hashed_api_key)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get API key by api_key"))
    }

    pub async fn list_api_key(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<ApiKey>, String> {
        db::list_api_key(&self.pg_pool, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get active API keys by user ID"))
    }

    pub async fn update_api_key_last_used(&self, id: &Uuid) -> Result<(), String> {
        db::update_api_key_last_used(&self.pg_pool, id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "update last used timestamp"))
    }

    pub async fn count_api_keys_by_user_id(&self, user_id: &Uuid) -> Result<usize, String> {
        match db::count_api_keys_by_user_id(&self.pg_pool, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "update last used timestamp"))? {
            None => Err("count_api_keys_by_user_id didn't return value".to_string()),
            Some(count) => Ok(count as usize)
        }
    }

    pub async fn create_callback_url(&self, url: &str, user_id: &Uuid) -> Result<CallbackUrl, String> {
        db::create_callback_url(&self.pg_pool, url, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create callback url"))
    }

    pub async fn list_callback_urls(&self, user_id: &Uuid) -> Result<Vec<CallbackUrl>, String> {
        db::list_callback_urls_by_user_id(&self.pg_pool, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "list callback urls"))
    }

    pub async fn delete_callback_url(&self, callback_url_id: &Uuid, user_id: &Uuid) -> Result<bool, String> {
        db::delete_callback_url_by_id_and_user_id(&self.pg_pool, callback_url_id, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "delete callback url"))
    }

    pub async fn count_callback_urls(&self, user_id: &Uuid) -> Result<usize, String> {
        match db::count_callback_urls_by_user_id(&self.pg_pool, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "list callback urls"))? {
            None => Err("count_callback_urls didn't return value".to_string()),
            Some(cnt) => Ok(cnt as usize)
        }
    }

    pub async fn exists_callback_url(&self, url: &str, user_id: &Uuid) -> Result<bool, String> {
        db::exists_callback_url(&self.pg_pool, url, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "exists callback url"))
    }

    pub async fn get_payment(&self, id: &Uuid) -> Result<Option<Payment>, String> {
        billing::get_payment(&self.pg_pool, id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get payment"))
    }

    pub async fn user_list_payment(&self, user_id: &Uuid, limit: i64, offset: i64) -> Result<Vec<Payment>, String> {
        billing::user_list_payment(&self.pg_pool, user_id, limit, offset)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "user list payment"))
    }

    pub async fn list_payment(&self, payment_type: &str, limit: i64, offset: i64) -> Result<Vec<Payment>, String> {
        billing::list_payment(&self.pg_pool, payment_type, limit, offset)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "list payment"))
    }

    pub async fn create_payment(&self, id: &Uuid, user_id: Option<Uuid>, data: &Value) -> Result<Payment, String> {
        billing::create_payment(&self.pg_pool, id, user_id, data)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create payment"))
    }

    pub async fn list_user_subscriptions(&self, user_id: &Uuid) -> Result<Vec<Subscription>, String> {
        billing::list_subscriptions(&self.pg_pool, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "list user subscription"))
    }

    pub async fn get_user_active_subscription(&self, user_id: &Uuid, target: &str) -> Result<Option<Subscription>, String> {
        billing::get_active_subscription(&self.pg_pool, user_id, target)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get user active subscription"))
    }

    pub async fn create_or_update_subscription(&self, user_id: &Uuid, target: &str, data: Option<Value>, until: NaiveDateTime) -> Result<(), String> {
        billing::create_or_update_subscription(&self.pg_pool, user_id, target, data, until)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create or update subscription"))
    }

    pub async fn set_payment_paid(&self, id: &Uuid) -> Result<(), String> {
        billing::set_payment_paid(&self.pg_pool, id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "set payment paid"))
    }

    pub async fn create_webhook(&self, url: &str, user_id: &Uuid) -> Result<Webhook, String> {
        db::create_webhook(&self.pg_pool, url, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create webhook"))
    }

    pub async fn list_webhooks(&self, user_id: &Uuid) -> Result<Vec<Webhook>, String> {
        db::list_webhooks_by_user_id(&self.pg_pool, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "list webhooks"))
    }

    pub async fn delete_webhook(&self, webhook_id: &Uuid, user_id: &Uuid) -> Result<bool, String> {
        db::delete_webhook_by_id_and_user_id(&self.pg_pool, webhook_id, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "delete webhook"))
    }

    pub async fn count_webhooks(&self, user_id: &Uuid) -> Result<usize, String> {
        match db::count_webhooks_by_user_id(&self.pg_pool, user_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "list webhooks"))? {
            None => Err("count_webhooks didn't return value".to_string()),
            Some(cnt) => Ok(cnt as usize)
        }
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

impl Redis {
    async fn new() -> Result<Self, String> {
        let mut client = redis::Client::open(utils::get_env_var("REDIS_URL")?)
            .map_err(|e| utils::make_err(Box::new(e), "get redis client"))?;

        assert!(client.check_connection());

        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| utils::make_err(Box::new(e), "get redis connection"))?;

        Ok(Self { connection })
    }

    async fn get(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.connection
            .clone()
            .get(key)
            .await
            .map_err(|e| utils::make_err(Box::new(e), "get redis value"))?)
    }

    async fn set(&self, key: &str, value: String, timeout: u64) -> Result<(), String> {
        let set_result: RedisResult<()> = self.connection
            .clone()
            .set_ex(key, value, timeout)
            .await;

        set_result.map_err(|e| utils::make_err(Box::new(e), "get redis value"))
    }

    pub async fn incr(&self, key: &str, timeout: u64) -> Result<u64, String> {
        let count: u64 = self.connection
            .clone()
            .incr(&key, 1)
            .await
            .map_err(|e| utils::make_err(Box::new(e), "increment redis value"))?;

        let _: bool = self.connection
            .clone()
            .expire(&key, timeout as i64)
            .await
            .map_err(|e| utils::make_err(Box::new(e), "set redis key expiration"))?;

        Ok(count)
    }

    pub async fn get_suggested_gas_fees(&self, network: &i64) -> Result<Option<String>, String> {
        let redis_key = get_suggested_gas_fees_key(network);
        self.get(&redis_key).await
    }

    pub async fn set_suggested_gas_fees(&self, network: &i64, value: String) -> Result<(), String> {
        let redis_key = get_suggested_gas_fees_key(network);
        self.set(&redis_key, value, SUGGESTED_GAS_FEE_TIMEOUT).await
    }
}

fn get_suggested_gas_fees_key(network: &i64) -> String {
    format!("network-suggested-gas-fees:{}", network)
}