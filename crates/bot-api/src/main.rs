use axum::{extract::Extension, routing::get, Json, Router};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use http::Method;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_stdout as stdout;
use std::sync::Arc;
use tower_http::cors::{any, CorsLayer};
use tracing::info;
use tracing::instrument;
use tracing::{error, span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

type PGPool = Pool<ConnectionManager<PgConnection>>;

struct AppState {
    db: PGPool,
}

#[tokio::main]
async fn main() {
    // Create a new OpenTelemetry trace pipeline that prints to stdout
    let provider = TracerProvider::builder()
        .with_simple_exporter(stdout::SpanExporter::default())
        .build();
    let tracer = provider.tracer("readme_example");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default().with(telemetry);
    //subscriber.into();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let manager = ConnectionManager::<PgConnection>::new(std::env::var("DATABASE_URL").unwrap());
    let db = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    let app_state = Arc::new(AppState { db });

    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(any());

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
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
