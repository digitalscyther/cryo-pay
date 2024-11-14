use tgbot::api::Client;
use tgbot::types::{ChatId, GetBot, InlineKeyboardButton, InlineKeyboardMarkup, SendMessage};
use crate::utils;

pub async fn get_client() -> Result<Client, String> {
    let token = utils::get_env_var("TGBOT_TOKEN")?;
    Client::new(token)
        .map_err(|err| utils::make_err(Box::new(err), "create telegram client"))
}

pub async fn send_invoice_paid(client: &Client, chat_id: ChatId, invoice_url: &str) -> Result<(), String> {
    let mut reply_markup = InlineKeyboardMarkup::default();
    reply_markup = reply_markup.add_row(
        vec![InlineKeyboardButton::for_url("Check", invoice_url)]
    );

    client
        .execute(SendMessage::new(
            chat_id, "Invoice paid")
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