use image_processing::error::ResponseError;
use lambda_http::{run, service_fn, Body, Error, IntoResponse, Request, Response};
use reqwest::header;

async fn function_handler(request: Request) -> Result<Response<Body>, Error> {
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
    use axum::{response::IntoResponse, Router};
    use hyper::StatusCode;
    use lambda_http::aws_lambda_events::query_map::QueryMap;
    use lambda_http::RequestExt;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    async fn endpoint(
        axum::extract::Query(query): axum::extract::Query<QueryMap>,
        req: axum::http::Request<axum::body::Body>,
    ) -> axum::response::Response {
        let (parts, axum_body) = req.into_parts();
        let bytes = match hyper::body::to_bytes(axum_body).await {
            Ok(x) => x,
            Err(err) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
            }
        };

        let request = Request::from_parts(parts, Body::Binary(bytes.to_vec()))
            .with_query_string_parameters(query);

        let res = match function_handler(request).await {
            Ok(x) => x,
            Err(err) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
            }
        };

        let (parts, body) = res.into_parts();
        let axum_body = axum::body::Body::from(body.to_vec());
        axum::response::Response::from_parts(parts, axum::body::boxed(axum_body))
    }

    let app = Router::new().route("/api/image", axum::routing::get(endpoint));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000);

    let addr: std::net::SocketAddr = format!("0.0.0.0:{port}").parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
