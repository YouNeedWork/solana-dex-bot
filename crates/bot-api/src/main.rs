use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_stdout as stdout;
use std::sync::Arc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
mod router;
type PGPool = Pool<ConnectionManager<PgConnection>>;

pub struct AppState {
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

    let app_state: Arc<AppState> = Arc::new(AppState { db });

    router::start_server(app_state).await;
}
