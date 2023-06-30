mod handlers;

use lambda_http::{run, service_fn, Body, Error, Request, Response};

async fn function_handler(request: Request) -> Result<Response<Body>, Error> {
    handlers::image_handler(request).await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
