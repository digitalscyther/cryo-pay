use tgbot::api::Client;
use tgbot::types::{ChatId, GetBot, InlineKeyboardButton, InlineKeyboardMarkup, SendMessage};
use crate::db::Invoice;
use crate::utils;

pub async fn get_client() -> Result<Client, String> {
    let token = utils::get_env_var("TGBOT_TOKEN")?;
    Client::new(token)
        .map_err(|err| utils::make_err(Box::new(err), "create telegram client"))
}

pub async fn send_invoice_paid(client: &Client, chat_id: ChatId, invoice: &Invoice) -> Result<(), String> {
    let mut reply_markup = InlineKeyboardMarkup::default();
    reply_markup = reply_markup.add_row(
        vec![InlineKeyboardButton::for_url("Check", &invoice.web_url()?)]
    );

    let mut lines = vec![
        "Invoice".to_string(),
        format!("ID: {}", &invoice.id.to_string()),
        "Paid".to_string(),
    ];
    if let Some(external_id) = &invoice.external_id {
        lines.insert(2, format!("External ID: {}", external_id))
    }
    let message = lines.join("\n");

    client
        .execute(SendMessage::new(
            chat_id, &message)
                .with_reply_markup(reply_markup))
        .await
        .map_err(|err| utils::make_err(Box::new(err), "send telegram message"))?;

    Ok(())
}

pub async fn send_message(client: &Client, chat_id: ChatId, text: &str) -> Result<(), String> {
    client
        .execute(SendMessage::new(chat_id, text))
        .await
        .map_err(|err| utils::make_err(Box::new(err), "send telegram message"))?;

    Ok(())
}

pub async fn get_bot_name(client: &Client) -> Result<String, String> {
    client
        .execute(GetBot)
        .await
        .map_err(|err| utils::make_err(Box::new(err), "get telegram bot name"))
        .map(|bot| bot.username)
}