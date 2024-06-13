use tracing::instrument;

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

// import wallet
pub async fn import_wallet() {}
