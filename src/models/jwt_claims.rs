use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    #[serde(alias = "session_id")]
    pub session_id: uuid::Uuid,
    exp: usize,
}

impl JwtClaims {
    pub fn new(session_id: uuid::Uuid) -> Self {
        let exp = (Utc::now() + Duration::days(15))
            .timestamp() as usize;

        Self {
            session_id,
            exp
        }
    }
}