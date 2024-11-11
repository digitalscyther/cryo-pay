pub enum Notifier {
    Email(EmailNotifier),
    Telegram(TelegramNotifier)
}

struct EmailNotifier {
    email: String
}

struct TelegramNotifier {
    chat_id: String
}

impl Notifier {
    pub fn from_email(email: String) -> Self {
        Self::Email(EmailNotifier::new(email))
    }

    pub fn from_telegram_data(chat_id: String) -> Self {
        Self::Telegram(TelegramNotifier::new(chat_id))
    }

    pub async fn notify(&self) -> Result<(), String> {
        match self {
            Notifier::Email(email) => email.notify().await,
            Notifier::Telegram(telegram) => telegram.notify().await,
        }
    }
}

trait Notify {
    async fn notify(&self) -> Result<(), String>;
}

impl EmailNotifier {
    fn new(email: String) -> Self {
        Self{ email }
    }
}

impl TelegramNotifier {
    fn new(chat_id: String) -> Self {
        Self{ chat_id }
    }
}

impl Notify for EmailNotifier {
    async fn notify(&self) ->  Result<(), String>{
        todo!()
    }
}

impl Notify for TelegramNotifier {
    async fn notify(&self) ->  Result<(), String>{
        todo!()
    }
}
