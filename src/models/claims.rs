use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(alias = "session_id")]
    pub session_id: uuid::Uuid,
}