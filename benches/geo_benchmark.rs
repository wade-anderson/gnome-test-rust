use criterion::{Criterion, criterion_group, criterion_main};
use httpmock::prelude::*;
use tokio::runtime::Runtime;

#[allow(dead_code, unused_imports)]
#[path = "../src/main.rs"]
mod app;

use app::GeoService;

fn fetch_location_benchmark(c: &mut Criterion) {
    let server = MockServer::start();

    // Mock the successful response
    server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"latitude": 40.7128, "longitude": -74.0060}"#);
    });

    let providers = vec![server.url("/json")];
    let geo_service = GeoService::new(providers);

    // Create a Tokio runtime for executing the async benchmark
    let rt = Runtime::new().unwrap();

    c.bench_function("fetch_location", |b| {
        b.to_async(&rt).iter(|| geo_service.fetch_location());
    });
}

criterion_group!(benches, fetch_location_benchmark);
criterion_main!(benches);
