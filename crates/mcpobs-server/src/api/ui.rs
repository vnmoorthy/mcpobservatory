//! Embedded UI assets. The `ui/dist` directory is bundled into the binary
//! at build time via `rust-embed`.

use axum::{
    body::Body,
    extract::Path,
    http::{header, HeaderValue, Response, StatusCode},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../ui/dist/"]
struct Asset;

pub async fn serve(uri: axum::http::Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');
    let file = if path.is_empty() { "index.html" } else { path };

    if let Some(asset) = Asset::get(file) {
        let mime = mime_guess::from_path(file).first_or_octet_stream();
        return Response::builder()
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime.as_ref())
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            )
            .body(Body::from(asset.data.into_owned()))
            .unwrap();
    }

    // SPA fallback: anything not matching a real asset returns index.html.
    if let Some(index) = Asset::get("index.html") {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(index.data.into_owned()))
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(
            "UI bundle missing — build the ui/ directory before this binary.",
        ))
        .unwrap()
}

// Unused parameter helper to keep axum's handler signature happy in older
// type-inference paths; not exposed.
#[allow(dead_code)]
fn _path_dummy(_: Path<String>) {}
