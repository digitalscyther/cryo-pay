use std::env;
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

fn web_base_url() -> Result<String, String> {
    get_env_var("WEB_BASE_URL")
}

pub fn get_invoice_url(invoice_id: Uuid) -> Result<String, String> {
    Ok(format!("{}/invoices/{}", web_base_url()?, invoice_id.to_string()))
}