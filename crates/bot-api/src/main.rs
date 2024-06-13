mod open_telemetry;
mod router;

#[tokio::main]
async fn main() {
    open_telemetry::subscriber_telemetry();
    router::start_server().await;
}
