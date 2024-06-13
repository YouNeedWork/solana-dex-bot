use tracing::instrument;

#[instrument(name = "swap")]
pub async fn swap() -> &'static str {
    //req: user_id,
    //ca: Token contract address,
    //amount: amount to buy or sell,
    "Hello, World!"
}
