use reqwest::{self, Client};
use serde_json::json;
use crate::utils;

#[derive(Clone)]
pub struct Mailer {
    api_key: String,
}

impl Mailer {
    pub fn new() -> Result<Self, String> {
        let api_key = utils::get_env_var("BREVO_API_KEY")?;
        Ok(Self { api_key })
    }

    pub async fn send_invoice_paid(&self, recipient_email: &str, invoice_url: &str) -> Result<(), String> {
        send_invoice_paid(&self.api_key, recipient_email, invoice_url).await
    }
}

async fn send_invoice_paid(api_key: &str, recipient_email: &str, invoice_link: &str) -> Result<(), String> {
    let sender_email = utils::get_env_var("EMAIL_SENDER")?;
    let email_data = json!({
        "sender": {
            "email": sender_email
        },
        "to": [
            {
                "email": recipient_email
            }
        ],
        "textContent": format!(
            "Hello, \n\nYour invoice has been successfully paid. \
            You can view the invoice at the following link: {}\n\nBest regards, \nCryoPay",
            invoice_link
        ),
        "subject": "Your Invoice Has Been Paid",
        "tags": ["InvoiceNotification"]
    });

    let client = Client::new();
    let response = client
        .post("https://api.brevo.com/v3/smtp/email")
        .header("accept", "application/json")
        .header("api-key", api_key)
        .header("content-type", "application/json")
        .json(&email_data)
        .send()
        .await
        .map_err(|err| utils::make_err(Box::new(err), "send email"))?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await;
        return Err(
            format!(
                "Failed to send notification email: status={:?}, text={:?}",
                status,
                text,
            )
        )
    }

    Ok(())
}
