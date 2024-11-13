use std::env;

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
    let true_values = vec!["0", "false", "n", "no", "not"];

    contains_value(&true_values, &val.to_lowercase())
}

fn contains_value(list: &Vec<&str>, value: &str) -> bool {
    list.contains(&value)
}