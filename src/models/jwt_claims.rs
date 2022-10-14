use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    #[serde(alias = "session_id")]
    pub session_id: uuid::Uuid,
}