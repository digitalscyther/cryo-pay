use tgbot::api::Client;
use tgbot::types::{ChatId, Integer, SendMessage};
use crate::utils;

pub async fn get_client() -> Result<Client, String> {
    let token = utils::get_env_var("TGBOT_TOKEN")?;
    Client::new(token)
        .map_err(|err| utils::make_err(Box::new(err), "create telegram client"))
}

pub async fn send_message(client: &Client, chat_id: &str, text: &str) -> Result<(), String> {
    let chat_id: ChatId = chat_id
        .parse::<Integer>()
        .map_err(|err| utils::make_err(Box::new(err), "parse chat id"))?
        .into();

    client
        .execute(SendMessage::new(chat_id.clone(), text))
        .await
        .map_err(|err| utils::make_err(Box::new(err), "send telegram message"))?;

    Ok(())
}