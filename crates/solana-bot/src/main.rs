use anyhow::Result;
use solana_bot::trade::{
    trade_proto::trade_service_server::TradeServiceServer, Trade,
};
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "[::1]:3000".parse()?;
    let trade = Trade::default();

    info!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(TradeServiceServer::new(trade))
        .serve(addr)
        .await?;

    Ok(())
}
