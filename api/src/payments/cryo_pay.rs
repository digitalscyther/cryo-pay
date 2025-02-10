use bigdecimal::BigDecimal;
use reqwest::Client;
use serde_json::{json, Value};
use url::Url;
use uuid::Uuid;
use crate::api::{get_invoice_full_path, get_target_invoice_path};
use crate::api::state::DB;
use crate::db::billing::Payment;
use crate::network::Network;
use crate::utils;

fn get_url(path: &str) -> Result<String, String> {
    utils::get_self_url()
        .and_then(|base_url| Url::parse(&base_url)
            .map_err(|err| utils::make_err(Box::new(err), "parse base url")))
        .and_then(|base| base.join(path)
            .map_err(|err| utils::make_err(Box::new(err), "join path")))
        .map(|url| url.to_string())
}

pub struct CryoPayRecipient {
    pub seller: String,
    pub networks: Vec<i64>
}

impl CryoPayRecipient {
    pub fn default(available_networks: &Vec<Network>) -> Result<Self, String> {
        let seller = utils::get_env_var("CRYO_PAY_SELF_ADDRESS")?;
        let networks_names: Vec<String> = serde_json::from_str(
            &utils::get_env_var("CRYO_PAY_RECIEVE_FROM_NETWORKS")?)
            .map_err(|err| utils::make_err(
                Box::new(err), "parse CRYO_PAY_RECIEVE_FROM_NETWORKS env"))?;
        let networks = available_networks
            .into_iter()
            .filter(|n| networks_names.contains(&n.name))
            .map(|n| n.id)
            .collect::<Vec<i64>>();

        Ok(Self::new(seller, networks))
    }

    fn new(seller: String, networks: Vec<i64>) -> Self {
        Self { seller, networks }
    }
}

pub struct CryoPayApi {
    api_key: Option<String>,
}

impl CryoPayApi {
    pub fn default() -> Self {
        let api_key = utils::get_env_var("CRYO_PAY_API_KEY").ok();
        CryoPayApi::new(api_key)
    }

    fn new(api_key: Option<String>) -> Self {
        CryoPayApi{ api_key }
    }

    pub async fn create_invoice(
        &self,
        seller: &str,
        networks: &Vec<i64>,
        custom_id: Option<String>,
        price: &BigDecimal,
    ) -> Result<Uuid, String> {
        let email_data = json!({
            "amount": price,
            "seller": seller,
            "networks": networks,
            "external_id": custom_id,
        });

        let url = get_url(&get_invoice_full_path())?;
        let mut request = Client::new().post(url);
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", api_key)
        }

        let response = request
            .header("content-type", "application/json")
            .json(&email_data)
            .send()
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create invoice"))?;

        match response.status().is_success() {
            false => Err(
                format!(
                    "Failed to create invoice: Non-200 response received: text={:?}",
                    response.text()
                        .await
                        .map_err(|err| utils::make_err(Box::new(err), "parse response text"))?
                )
            ),
            true => {
                let json_response: Value = response
                    .json()
                    .await
                    .map_err(|err| utils::make_err(Box::new(err), "parse response"))?;
                match json_response.get("id").and_then(|v| v.as_str()) {
                    Some(id) => Ok(Uuid::parse_str(id)
                        .map_err(|err| utils::make_err(Box::new(err), "parse invoice id"))?),
                    None => Err("Failed to create invoice: 'id' field missing in response".to_string())
                }
            }
        }
    }

    pub async fn is_invoice_paid(&self, id: &Uuid) -> Result<bool, String> {
        let mut request = Client::new().get(get_url(&get_target_invoice_path(id))?);
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", api_key)
        }

        let response = request
            .send()
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get invoice"))?;

        match response.status().is_success() {
            false => Err("Failed to get invoice: Non-200 response received".to_string()),
            true => {
                let json_response: Value = response
                    .json()
                    .await
                    .map_err(|err| utils::make_err(Box::new(err), "parse response"))?;
                match json_response.get("invoice") {
                    None => Err("Failed to parse invoice".to_string()),
                    Some(invoice) => match invoice.get("paid_at") {
                        None => Err("Failed to parse paid_at".to_string()),
                        Some(paid_at) => Ok(match paid_at {
                            Value::Null => false,
                            _ => true,
                        })
                    }
                }
            }
        }
    }
}

pub fn get_payment_path(invoice_id: &Uuid, callback_url: Option<String>) -> Result<String, String> {
    let web_base_url_len = utils::web_base_url()?.len();
    let mut url = Url::parse(&utils::get_invoice_url(invoice_id)?)
        .map_err(|err| utils::make_err(Box::new(err), "parse invoice url"))?;

    if let Some(callback) = callback_url {
        url.query_pairs_mut().append_pair("callback_url", &callback);
    }

    let url_str = url.to_string();
    let trimmed = &url_str[web_base_url_len..];

    Ok(trimmed.to_string())
}

pub enum PaidPayableResult {
    NotPaid,
    NotFound,
    Payment(Payment),
}

pub async fn get_paid_payable(db: &DB, invoice_id: &Uuid) -> Result<PaidPayableResult, String> {
    Ok(match db.get_payment(invoice_id).await? {
        None => PaidPayableResult::NotFound,
        Some(payment) => match CryoPayApi::default().is_invoice_paid(invoice_id).await? {
            false => PaidPayableResult::NotPaid,
            true => PaidPayableResult::Payment(payment)
        }
    })
}
