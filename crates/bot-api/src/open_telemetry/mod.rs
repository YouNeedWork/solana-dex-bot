use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_stdout as stdout;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

pub fn subscriber_telemetry() {
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
}
