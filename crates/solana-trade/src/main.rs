use anyhow::Result;
use solana_trade::trade::{
    trade_proto::trade_service_server::TradeServiceServer, Trade,
};
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let trade = Trade::new();

    let addr = "[::1]:3000".parse()?;
    info!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(TradeServiceServer::new(trade))
        .serve(addr)
        .await?;

    Ok(())
}
