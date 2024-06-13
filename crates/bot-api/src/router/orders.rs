use tracing::instrument;
#[instrument(name = "orders")]
pub async fn orders() -> &'static str {
    //req: user_id
    //res: orders table data
    "orders"
}
