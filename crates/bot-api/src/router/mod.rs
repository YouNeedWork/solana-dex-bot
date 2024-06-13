use axum::{extract::Extension, routing::get, Json, Router};
use http::Method;
use std::sync::Arc;
use tower_http::cors::{any, CorsLayer};
use tracing::info;
use tracing::instrument;

use crate::AppState;

pub async fn start_server(app_state: Arc<AppState>) {
    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(any());
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/wallet", get(wallet))
        .route("/orders", get(orders))
        .layer(cors)
        .layer(Extension(app_state));

    let bind = std::env::var("BIND").unwrap_or("0.0.0.0:8080".to_string());
    info!("Server bind on prot: {} ", bind);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[instrument(name = "root")]
async fn root() -> &'static str {
    "Hello, World!"
}

#[instrument(name = "wallet")]
async fn wallet() -> &'static str {
    // user_id
    "Hello, World!"
}

#[instrument(name = "orders")]
async fn orders() -> &'static str {
    //req: user_id
    //
    //res: orders table data
    "Hello, World!"
}

#[instrument(name = "swap")]
async fn swap() -> &'static str {
    //req: user_id,
    //ca: Token contract address,
    //amount: amount to buy or sell,
    "Hello, World!"
}
