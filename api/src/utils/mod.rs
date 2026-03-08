use std::env;
use hex::encode;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde_json::Value;
use sha2::{Digest, Sha256};
use url::Url;
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

    retry(1, || {
        let url = api_url.clone();
        async move {
            reqwest::get(&url).await
                .map_err(|err| make_err(Box::new(err), "make get reqwest for get_suggested_gas_fees"))?
                .json().await
                .map_err(|err| make_err(Box::new(err), "parse reqwest for get_suggested_gas_fees"))
        }
    }).await
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

pub fn generate_webhook_secret() -> String {
    encode(thread_rng().gen::<[u8; 32]>())
}

pub async fn retry<F, Fut, T>(max_retries: u32, operation: F) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut last_err = String::new();
    for attempt in 0..=max_retries {
        match operation().await {
            Ok(val) => return Ok(val),
            Err(err) => {
                last_err = err;
                if attempt < max_retries {
                    tracing::warn!("Retry attempt {}/{}: {}", attempt + 1, max_retries + 1, last_err);
                    let backoff = std::cmp::min(1u64 << attempt, 8);
                    tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                }
            }
        }
    }
    Err(last_err)
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

#[derive(Debug, PartialEq)]
enum ApiGlobalUrlType {
    Full,
    Path
}

pub struct ApiGlobalUrl {
    url_type: ApiGlobalUrlType,
    value: String,
}

impl ApiGlobalUrl {
    pub fn get() -> Result<Self, String> {
        Self::from_str(get_env_var("API_GLOBAL_URL")?)
    }

    fn from_str(env_value: String) -> Result<Self, String> {
        Ok(Self {
            url_type: match Url::parse(&env_value) {
                Err(_) => match env_value.starts_with("/") {
                    true => ApiGlobalUrlType::Path,
                    false => return Err(format!("Failed to parse ApiGlobalUrlType: {env_value}")),
                },
                _ => ApiGlobalUrlType::Full,
            },
            value: env_value
        })
    }

    pub fn url(&self) -> Result<String, String> {
        Ok(match self.url_type {
            ApiGlobalUrlType::Full => self.value.clone(),
            ApiGlobalUrlType::Path => combine_paths(&[&web_base_url()?, &self.value.clone()])
        })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;

    #[rstest]
    #[case("http://example.com", ApiGlobalUrlType::Full, "http://example.com")]
    #[case("/api", ApiGlobalUrlType::Path, "/api")]
    #[case("invalid_url", ApiGlobalUrlType::Full, "error")]
    fn test_api_global_url(#[case] input: String, #[case] expected_type: ApiGlobalUrlType, #[case] expected_value: &str) {
        let result = ApiGlobalUrl::from_str(input);

        match expected_value == "error" {
            true => {
                assert!(result.is_err());
            },
            false => {
                let url = result.expect("Expected valid URL");
                assert_eq!(url.url_type, expected_type);
                assert_eq!(url.value, expected_value);
            }
        }
    }

    #[rstest]
    #[case("", true)]
    #[case("0", true)]
    #[case("false", true)]
    #[case("n", true)]
    #[case("no", true)]
    #[case("not", true)]
    #[case("1", false)]
    #[case("true", false)]
    #[case("yes", false)]
    #[case("TRUE", false)]
    fn test_is_false(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(is_false(input), expected);
    }

    #[test]
    fn test_combine_paths() {
        assert_eq!(combine_paths(&["/a", "/b", "/c"]), "/a/b/c");
        assert_eq!(combine_paths(&[]), "");
    }

    #[test]
    fn test_generate_webhook_secret_length() {
        let secret = generate_webhook_secret();
        assert_eq!(secret.len(), 64); // 32 bytes = 64 hex chars
    }
}
