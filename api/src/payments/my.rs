use bigdecimal::BigDecimal;
use reqwest::Client;
use serde_json::{json, Value};
use url::Url;
use crate::api::get_invoice_full_path;
use crate::utils;


fn get_base_url() -> Result<String, String> {
    utils::get_bind_address().map(|addr| format!("http://{}", addr))
}

fn get_url(path: &str) -> Result<String, String> {
    get_base_url()
        .and_then(|base_url| Url::parse(&base_url)
            .map_err(|err| utils::make_err(Box::new(err), "parse base url")))
        .and_then(|base| base.join(path)
            .map_err(|err| utils::make_err(Box::new(err), "join path")))
        .map(|url| url.to_string())
}

async fn create_invoice(
    seller: &str,
    networks: &Vec<i32>,
    custom_id: Option<String>,
    price: &BigDecimal,
    api_key: Option<String>,
) -> Result<String, String> {
    let email_data = json!({
        "amount": price,
        "seller": seller,
        "networks": networks,
        "external_id": custom_id,
    });

    let mut request = Client::new().post(get_url(&get_invoice_full_path())?);
    if let Some(api_key) = api_key {
        request = request.header("Authorization", api_key)
    }

    let response = request
        .header("content-type", "application/json")
        .json(&email_data)
        .send()
        .await
        .map_err(|err| utils::make_err(Box::new(err), "create invoice"))?;

    match response.status().is_success() {
        false => Err("Failed to create invoice: Non-200 response received".to_string()),
        true => {
            let json_response: Value = response
                .json()
                .await
                .map_err(|err| utils::make_err(Box::new(err), "parse response"))?;
            match json_response.get("id").and_then(|v| v.as_str()) {
                Some(id) => Ok(id.to_string()),
                None => Err("Failed to create invoice: 'id' field missing in response".to_string())
            }
        },
    }
}

fn get_payment_path(invoice_id: &str, callback_url: Option<String>) -> Result<String, String> {
    let dummy_base = "http://dummy";
    let path = format!("/invoices/{}", invoice_id);

    let mut url = Url::parse(dummy_base)
        .map_err(|err| utils::make_err(Box::new(err), "parse dummy"))
        .and_then(|base| base.join(&path)
            .map_err(|err| utils::make_err(Box::new(err), "join path")))?;

    if let Some(callback) = callback_url {
        url.query_pairs_mut().append_pair("callback_url", &callback);
    }

    let url_str = url.to_string();
    let trimmed = &url_str[dummy_base.len()..];

    Ok(trimmed.to_string())
}
