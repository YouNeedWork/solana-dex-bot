use anyhow::Result;
use axum::Json;
use clap::Parser;
use config::Config;
use solana_bot::config;

use futures::StreamExt;
use lapin::options::BasicConsumeOptions;
use lapin::options::BasicQosOptions;
use lapin::types::FieldTable;
use lapin::ConnectionProperties;

use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;

use serde::Deserialize;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::parse();

    let conn = lapin::Connection::connect(
        //&std::env::var("RABBIT_MQ_URL").unwrap(),
        &config.rabbit_mq_url,
        ConnectionProperties::default(),
    )
    .await?;

    //TODO: api service for generate unsign message for client,
    for _ in 0..5 {
        let channel = conn.create_channel().await?;
        channel.basic_qos(1, BasicQosOptions::default()).await?;

        tokio::spawn(async move {
            let mut ch = channel
                .basic_consume(
                    "trade",
                    "consumer",
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .expect("failed to consume");

            while let Some(delivery) = ch.next().await {
                let delivery = delivery.expect("error in consumer");

                // TODO: 得到UUID& signature
                // 从UUID拿到订单unsign交易
                // 替换unsign交易的签名
                // 发送交易
                // 等待交易确认
                // 通知用户
                // 确认交易

                channel
                    .basic_ack(
                        delivery.delivery_tag,
                        lapin::options::BasicAckOptions { multiple: false },
                    )
                    .await
                    .expect("failed to ack");
            }
        });
    }

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/trade", post(trade));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[derive(Deserialize, Debug)]
struct TradeParams {
    user_id: i32,
    token_a: String,
    token_b: String,
    amount: u64,
    tips: u64,
    gas: Option<u64>,
}

async fn trade(Json(trade): Json<TradeParams>) -> (StatusCode, ()) {
    info!("{:?}", trade);

    (StatusCode::CREATED, ())
}
