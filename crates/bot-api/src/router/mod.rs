use axum::routing::post;
use axum::{extract::Extension, routing::get, Json, Router};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use http::Method;
use std::sync::Arc;
use tower_http::cors::{any, CorsLayer};
use tracing::info;

mod orders;
mod swap;
mod wallet;

type PGPool = Pool<ConnectionManager<PgConnection>>;

pub struct AppState {
    db: PGPool,
}

fn get_app_state() -> Arc<AppState> {
    let manager = ConnectionManager::<PgConnection>::new(std::env::var("DATABASE_URL").unwrap());
    let db = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    let app_state: Arc<AppState> = Arc::new(AppState { db });
    return app_state;
}

pub async fn start_server() {
    let app_state = get_app_state();
    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(any());

    let app = Router::new()
        .route("/wallet", get(wallet::wallet))
        // .route("/wallet/import", post(wallet::import_wallet))
        .route("/orders", get(orders::orders))
        .route("/swap", get(swap::swap))
        .layer(cors)
        .layer(Extension(app_state));

    let bind = std::env::var("BIND").unwrap_or("0.0.0.0:8080".to_string());
    info!("Server bind on prot: {} ", bind);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
