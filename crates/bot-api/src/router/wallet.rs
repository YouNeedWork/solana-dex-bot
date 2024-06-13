use axum::{extract::Query, Json};
use serde::Deserialize;
use tracing::instrument;
use validator::Validate;
#[instrument(name = "wallet")]
pub async fn wallet() -> &'static str {
    // user_id
    "wallet"
}

// create solana wallet
pub async fn create_wallet() {}

// remove solana wallet
pub async fn remove_wallet() {}

// set default wallet
pub async fn set_default_wallet() {}

#[derive(Debug, Validate, Deserialize)]
pub struct ImportWalletParams {
    #[validate(length(equal = 87))]
    pub private_key: String,
}

pub async fn import_wallet(Json(frm): Json<ImportWalletParams>) -> String {
    // frm.private_key.len()
    format!("{}", frm.private_key)
}

pub struct SetWalletName {
    pub name: String,
}
pub async fn set_wallet_name() {}
