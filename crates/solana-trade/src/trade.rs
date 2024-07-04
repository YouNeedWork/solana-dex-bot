use tonic::{Request, Response, Status};
use tracing::info;
use trade_proto::{
    trade_service_server::TradeService, TradeReply, TradeRequest,
};

use anyhow::Result;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub mod trade_proto {
    tonic::include_proto!("trade");
}

struct Work((String, TradeRequest));

#[derive(Clone)]
pub struct Trade {
    work_sender: Sender<Work>,
}

impl Trade {
    pub fn new() -> Self {
        let (tx, rx) = channel::<Work>(1000);

        tokio::spawn(async move { Self::swap_work(rx).await });

        Self { work_sender: tx }
    }
}

//TODO: Add one channel for do the trade onchain. and wait for success
impl Trade {
    async fn swap_work(mut works: Receiver<Work>) -> Result<()> {
        while let Some(work) = works.recv().await {
            let Work((id, request)) = work;
            info!("handle request id: {} user_id {:?}", id, request);

            //TODO: query database
            //TODO: do the trade onchain
            //TODO: save the result to database
            //TODO: return the result
        }

        Ok(())
    }
}

#[tonic::async_trait]
impl TradeService for Trade {
    async fn trade(
        &self, request: Request<TradeRequest>,
    ) -> Result<Response<TradeReply>, Status> {
        info!("Got a request: {:?}", request);
        //TODO: check all params.
        let id = uuid::Uuid::new_v4().to_string();

        self.work_sender
            .send(Work((id.clone(), request.into_inner())))
            .await
            .unwrap();

        let reply = TradeReply {
            code: 0,
            message: id,
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

#[tokio::test]
async fn test_trade() {}
