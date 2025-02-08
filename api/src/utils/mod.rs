use std::env;
use axum::http::StatusCode;
use hex::encode;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tracing::error;
use uuid::Uuid;

pub fn make_err(err: Box<dyn std::error::Error>, process: &str) -> String {
    format!("Failed {}: {:?}", process, err)
}

pub fn get_env_var(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("{} must be set", key))
}

pub fn get_env_or(key: &str, default: String) -> Result<String, String> {
    get_env_var(key).or(Ok(default))
}

pub fn is_false(val: &str) -> bool {
    let true_values = vec!["", "0", "false", "n", "no", "not"];

    contains_value(&true_values, &val.to_lowercase())
}

fn contains_value(list: &Vec<&str>, value: &str) -> bool {
    list.contains(&value)
}

pub fn web_base_url() -> Result<String, String> {
    get_env_var("WEB_BASE_URL")
}

pub fn get_invoice_url(invoice_id: &Uuid) -> Result<String, String> {
    Ok(format!("{}/invoices/{}", web_base_url()?, invoice_id.to_string()))
}

pub async fn get_suggested_gas_fees(infura_token: &str, network_id: i64) -> Result<Value, String> {
    let api_url = format!(
        "https://gas.api.infura.io/v3/{}/networks/{}/suggestedGasFees",
        infura_token, network_id
    );

    Ok(
        reqwest::get(api_url).await
        .map_err(|err| make_err(Box::new(err), "make get reqwest for get_suggested_gas_fees"))?
        .json().await
        .map_err(|err| make_err(Box::new(err), "parse reqwest for get_suggested_gas_fees"))?
    )
}

pub struct ApiKey {
    pub value: String
}

impl ApiKey {
    fn new(user_id: Uuid) -> Self {
        let random_suffix: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        let value = format!("{}.{}", user_id, random_suffix);

        Self { value }
    }

    pub fn hash_value(api_key: &str) -> String {
        let app_secret = env::var("APP_SECRET").expect("APP_SECRET must be set");
        let mut hasher = Sha256::new();
        hasher.update(app_secret);
        hasher.update(api_key);
        let result = hasher.finalize();

        encode(result)
    }

    pub fn hashed_value(&self) -> String {
        ApiKey::hash_value(&self.value)
    }
}

pub fn new_api_key(user_id: Uuid) -> ApiKey {
    ApiKey::new(user_id)
}

pub fn get_bind_address() -> Result<String, String> {
    let host = get_env_var("HOST")?;
    let port = get_env_var("PORT")?;
    Ok(format!("{}:{}", host, port))
}

pub fn combine_paths(paths: &[&str]) -> String {
    paths.concat()
}

pub fn get_self_url() -> Result<String, String> {
    get_bind_address().map(|addr| format!("http://{}", addr))
}

pub fn log_and_error(err: String) -> StatusCode {
    error!("{err}");
    StatusCode::INTERNAL_SERVER_ERROR
}