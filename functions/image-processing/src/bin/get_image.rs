use image_processing::error::ResponseError;
use lambda_http::{Body, Error, Request, Response};
use reqwest::header;

async fn function_handler(request: Request) -> Result<Response<Body>, Error> {
    use lambda_http::IntoResponse;

    match image_processing::api::get_image_endpoint(request).await {
        Ok(mut res) => {
            let seconds = 60 * 60 * 24 * 365; // 1 year
            let value = header::HeaderValue::from_str(&format!("max-age={seconds}")).unwrap();
            res.headers_mut().append(header::CACHE_CONTROL, value);
            Ok(res)
        }
        Err(err) => match err.downcast::<ResponseError>() {
            Ok(x) => Ok((*x).into_response().await),
            Err(err) => Err(err),
        },
    }
}

#[tokio::main]
#[cfg(not(feature = "local"))]
async fn main() -> Result<(), Error> {
    use lambda_http::{run, service_fn};

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

#[tokio::main]
#[cfg(feature = "local")]
async fn main() {
    use hyper::Method;

    image_processing::utils::handle_local_request("/", Method::GET, function_handler).await
}
