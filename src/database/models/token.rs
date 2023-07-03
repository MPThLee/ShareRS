use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids;

#[serde_with::serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub id: ids::TokenId,
    pub user_id: ids::UserId,
    pub expires: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
