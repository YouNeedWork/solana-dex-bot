use tracing::instrument;

#[instrument(name = "wallet")]
pub async fn wallet() -> &'static str {
    // user_id
    "wallet"
}
