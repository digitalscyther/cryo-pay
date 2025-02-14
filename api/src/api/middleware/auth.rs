use uuid::Uuid;
use crate::db::User;

#[derive(Clone, Debug)]
pub enum AuthType {
    API,
    WEB,
}

impl AuthType {
    pub fn is_web(&self) -> bool {
        match self {
            AuthType::WEB => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Auth {
    pub auth_type: AuthType,
    pub user: User,
}

impl Auth {
    pub fn new(auth_type: AuthType, user: User) -> Self {
        Self { auth_type, user }
    }

    fn redis_key(&self) -> String {
        format!("{:?}:{}", self.auth_type, self.user.id)
    }
}


#[derive(Clone, Debug)]
pub struct AppUser {
    ip: String,
    pub auth: Option<Auth>,
}

impl AppUser {
    pub fn new(ip: String, auth: Option<Auth>) -> Self {
        Self { ip, auth }
    }

    pub fn user_id(&self) -> Option<Uuid> {
        self.auth.clone().map(|auth| auth.user.id)
    }

    pub fn redis_key(&self) -> String {
        match &self.auth {
            Some(auth) => auth.redis_key(),
            None => format!("anonymus:{}", self.ip),
        }
    }
}
