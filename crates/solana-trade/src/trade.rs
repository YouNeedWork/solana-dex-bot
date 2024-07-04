use tonic::{Request, Response, Status};
use tracing::info;
use trade_proto::{
    trade_service_server::TradeService, TradeReply, TradeRequest,
};

pub mod trade_proto {
    tonic::include_proto!("trade");
}

#[derive(Debug, Default)]
pub struct Trade {}

impl Trade {
    pub fn new() -> Self {
        Self {}
    }
}

//TODO: Add one channel for do the trade onchain. and wait for success
/*
impl Trade {
    async fn swap_work(&self) -> Result<()> {
        Ok(())
    }
}
*/

#[tonic::async_trait]
impl TradeService for Trade {
    async fn trade(
        &self, request: Request<TradeRequest>,
    ) -> Result<Response<TradeReply>, Status> {
        info!("Got a request: {:?}", request);

        let reply = TradeReply {
            code: 0,
            message: format!("Hello From Grpc!"),
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}
