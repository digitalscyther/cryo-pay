use serde::Deserialize;
use crate::api::response_error::ResponseError;

fn default_limit() -> i64 {
    10
}

fn default_offset() -> i64 {
    0
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default = "default_offset")]
    offset: i64,
}

impl Pagination {
    pub fn get_valid(&self, max_limit: i64) -> Result<(i64, i64), ResponseError> {
        match self.limit > max_limit {
            false => Ok((self.limit, self.offset)),
            true => {
                Err(ResponseError::Bad(format!("max limit = {max_limit}")))
            }
        }
    }
}
