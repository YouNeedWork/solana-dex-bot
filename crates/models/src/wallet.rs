use crate::schema::wallets;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Insertable, Queryable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = wallets)]
pub struct Wallet {
    pub private_key: String,
    pub wallet_address: String,
    pub user_id: i32,
    pub is_default: bool,
    pub create_at: Option<chrono::NaiveDateTime>,
    pub update_at: Option<chrono::NaiveDateTime>,
}
