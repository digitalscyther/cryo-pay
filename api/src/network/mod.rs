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
    pub fn default_vec() -> Result<Vec<Self>, String> {
        Self::vec_from_env("NETWORKS")
    }
    pub fn vec_from_env(env_name: &str) -> Result<Vec<Self>, String> {
        Self::parse_networks(&utils::get_env_var(env_name)?)
    }

    fn parse_networks(json: &str) -> Result<Vec<Self>, String> {
        serde_json::from_str(json)
            .map_err(|err| utils::make_err(Box::new(err), "parse env networks"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_networks() {
        let networks = Network::parse_networks(
            r#"[{"name":"Optimism","id":10,"link":"https://rpc.example.com","addresses":{"erc20":"0xabc","contract":"0xdef"}}]"#
        ).unwrap();
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].name, "Optimism");
        assert_eq!(networks[0].id, 10);
    }

    #[test]
    fn test_parse_invalid_json() {
        assert!(Network::parse_networks("not json").is_err());
    }

    #[test]
    fn test_missing_env_var() {
        assert!(Network::vec_from_env("TEST_NETWORKS_MISSING_12345").is_err());
    }
}
