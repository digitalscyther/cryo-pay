use serde::{Deserialize, Serialize};
use crate::utils;

#[derive(Clone, Deserialize, Serialize)]
pub struct Network {
    pub name: String,
    pub id: i64,
    pub link: String,
    pub addresses: Addresses
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Addresses {
    erc20: String,
    pub contract: String,
}

impl Network {
    pub fn vec_from_env(env_name: &str) -> Result<Vec<Self>, String> {
        let env_val = utils::get_env_var(env_name)?;

        serde_json::from_str(&env_val)
            .map_err(|err| utils::make_err(Box::new(err), "parse env networks"))
    }
}