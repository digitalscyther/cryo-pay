#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Auth,

    #[error("network error: {0}")]
    Network(String),

    #[error("internal: {0}")]
    Internal(String),
}
