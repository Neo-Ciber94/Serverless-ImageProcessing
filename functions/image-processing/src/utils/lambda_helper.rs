use axum::extract::{FromRequestParts, Path, Query};
use axum::response::Response;
use axum::routing::MethodFilter;
use axum::{response::IntoResponse, Router};
use hyper::{Method, StatusCode};
use lambda_http::aws_lambda_events::query_map::QueryMap;
use lambda_http::{Request, RequestExt};
use std::future::Future;
use tower_http::trace::TraceLayer;

pub async fn handle_request<H, Fut>(path: &str, method: Method, handler: H)
where
    H: Fn(Request) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<Response<lambda_http::Body>, lambda_http::Error>> + Send,
{
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let app = Router::new().layer(TraceLayer::new_for_http()).route(
        path,
        axum::routing::any(
            move |req: axum::http::Request<axum::body::Body>| async move {
                let req_method = req.method().clone();
                let (mut parts, axum_body) = req.into_parts();
                let Query(query) = Query::<QueryMap>::from_request_parts(&mut parts, &())
                    .await
                    .unwrap();
                let Path(params) = Path::<QueryMap>::from_request_parts(&mut parts, &())
                    .await
                    .unwrap();
                let req_method = MethodFilter::try_from(req_method).unwrap();
                let method = MethodFilter::try_from(method).unwrap();

                if method != req_method {
                    return axum::http::Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(axum::body::BoxBody::default())
                        .unwrap();
                }

                let bytes = match hyper::body::to_bytes(axum_body).await {
                    Ok(x) => x,
                    Err(err) => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
                            .into_response();
                    }
                };

                let request = Request::from_parts(parts, lambda_http::Body::Binary(bytes.to_vec()))
                    .with_query_string_parameters(query)
                    .with_path_parameters(params);

                let res = match handler(request).await {
                    Ok(x) => x,
                    Err(err) => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
                    }
                };

                let (parts, body) = res.into_parts();
                let axum_body = axum::body::Body::from(body.to_vec());
                axum::response::Response::from_parts(parts, axum::body::boxed(axum_body))
            },
        ),
    );

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
