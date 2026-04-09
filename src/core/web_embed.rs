use axum::{
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use percent_encoding::percent_decode_str;
use rust_embed::RustEmbed;
use tracing::{debug, warn};

#[derive(RustEmbed)]
#[folder = "web/dist/"]
struct WebAssets;

/// Serve embedded frontend static files (single-binary deployment).
pub async fn web_embed_file_handler(uri: Uri) -> impl IntoResponse {
    let raw_path = uri.path().trim_start_matches('/');
    let decoded_path = decode_uri_path(raw_path);
    serve_embedded_files(&decoded_path).await
}

fn decode_uri_path(path: &str) -> String {
    percent_decode_str(path).decode_utf8_lossy().into_owned()
}
/// check if the path is a static resource path
fn is_static_resource_path(path: &str) -> bool {
    // if the path contains a file extension, it is a static resource
    if path.contains('.') {
        return true;
    }

    // special static resource paths
    if path.starts_with("assets/")
        || path.starts_with("static/")
        || path.starts_with("public/")
        || path.starts_with("images/")
        || path.starts_with("css/")
        || path.starts_with("js/")
        || path.starts_with("doc/")
    {
        return true;
    }

    // other cases are considered SPA routes
    false
}

/// use embedded static files
async fn serve_embedded_files(path: &str) -> Response {
    debug!("[static file] handle request: {}", path);

    // if the path is the root path, return index.html
    if path.is_empty() || path == "index.html" {
        debug!("[static file] return root path index.html");
        return serve_embedded_index_html().await;
    }

    // check if the path is a static resource
    let is_static = is_static_resource_path(path);
    debug!("[static file] path '{}' is static resource: {}", path, is_static);

    if is_static {
        // try to get the static resource file
        if let Some(file) = WebAssets::get(path) {
            // set Content-Type based on the file extension
            let content_type = get_content_type(path);
            let contents = file.data;

            debug!(
                "[static file] find embedded file: {}, Content-Type: {}, size: {} bytes",
                path,
                content_type,
                contents.len()
            );

            return Response::builder()
                .status(StatusCode::OK)
                .header("content-type", content_type)
                .header("cache-control", "public, max-age=604800") // static resource cache 1 week
                .body(axum::body::Body::from(contents.into_owned()))
                .unwrap();
        } else {
            // static resource file not found
            warn!("[static file] embedded file not found: {}", path);
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("content-type", "text/plain; charset=utf-8")
                .body(axum::body::Body::from(format!("File not found: {}", path)))
                .unwrap();
        }
    }

    // for non-static resource paths (SPA routes), return index.html
    // this is especially important for hash routes, because all routes should return index.html
    debug!("[static file] SPA routes, return embedded index.html: {}", path);
    serve_embedded_index_html().await
}

/// serve embedded index.html file
async fn serve_embedded_index_html() -> Response {
    if let Some(index_file) = WebAssets::get("index.html") {
        debug!("[static file] serve embedded index.html");
        Html(String::from_utf8_lossy(&index_file.data).into_owned()).into_response()
    } else {
        warn!("[static file] embedded index.html file not found");

        // if there is no embedded index.html, return a simple default page
        let default_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>RustZen Admin</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; text-align: center; }
                .logo { font-size: 48px; margin-bottom: 20px; }
                .info { color: #666; }
            </style>
        </head>
        <body>
            <div class="logo">🖥️</div>
            <h1>RustZen Admin</h1>
            <p class="info">Web interface is loading...</p>
            <p class="info">If you see this page, it means the static files may not be correctly embedded.</p>
        </body>
        </html>
        "#;

        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html; charset=utf-8")
            .body(axum::body::Body::from(default_html))
            .unwrap()
    }
}

/// get Content-Type based on the file extension
fn get_content_type(path: &str) -> &'static str {
    if let Some(extension) = path.split('.').last() {
        match extension.to_lowercase().as_str() {
            "html" => "text/html; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "js" | "mjs" => "application/javascript; charset=utf-8",
            "jsx" => "application/javascript; charset=utf-8",
            "ts" => "application/typescript; charset=utf-8",
            "tsx" => "application/typescript; charset=utf-8",
            "json" => "application/json; charset=utf-8",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "eot" => "application/vnd.ms-fontobject",
            "webp" => "image/webp",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "pdf" => "application/pdf",
            "xml" => "application/xml; charset=utf-8",
            "txt" => "text/plain; charset=utf-8",
            "map" => "application/json; charset=utf-8", // source maps
            _ => "application/octet-stream",
        }
    } else {
        "application/octet-stream"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::Uri;

    #[test]
    fn static_resource_path_detection_matches_spa_rules() {
        assert!(is_static_resource_path("index.html"));
        assert!(is_static_resource_path("assets/app.js"));
        assert!(is_static_resource_path("images/logo"));
        assert!(!is_static_resource_path("dashboard"));
        assert!(!is_static_resource_path("system/users"));
    }

    #[test]
    fn uri_path_is_percent_decoded_before_asset_lookup() {
        let encoded =
            "doc/by/%E6%AF%8F%2024000%20%E4%B8%AA%E5%B7%A5%E4%BD%9C%E5%B0%8F%E6%97%B6%E6%88%96%E6%AF%8F%203%20%E5%B9%B4/%E6%AF%8F%2024000%20%E4%B8%AA%E5%B7%A5%E4%BD%9C%E5%B0%8F%E6%97%B6%E6%88%96%E6%AF%8F%203%20%E5%B9%B4(1).html";
        let decoded = decode_uri_path(encoded);

        assert_eq!(
            decoded,
            "doc/by/每 24000 个工作小时或每 3 年/每 24000 个工作小时或每 3 年(1).html"
        );
    }

    #[test]
    fn content_type_detection_covers_common_extensions() {
        assert_eq!(get_content_type("app.js"), "application/javascript; charset=utf-8");
        assert_eq!(get_content_type("styles.css"), "text/css; charset=utf-8");
        assert_eq!(get_content_type("icon.svg"), "image/svg+xml");
        assert_eq!(get_content_type("blob.unknown"), "application/octet-stream");
        assert_eq!(get_content_type("no-extension"), "application/octet-stream");
    }

    #[tokio::test]
    async fn root_path_serves_embedded_index_html() {
        let response = serve_embedded_files("").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/html; charset=utf-8");

        let body =
            to_bytes(response.into_body(), usize::MAX).await.expect("body should be readable");
        let body = String::from_utf8(body.to_vec()).expect("html should be utf-8");
        assert!(body.contains("<!DOCTYPE html") || body.contains("<!doctype html"));
    }

    #[tokio::test]
    async fn spa_route_falls_back_to_embedded_index_html() {
        let response = serve_embedded_files("dashboard/overview").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/html; charset=utf-8");
    }

    #[tokio::test]
    async fn missing_static_resource_returns_not_found() {
        let response = serve_embedded_files("assets/does-not-exist.js").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/plain; charset=utf-8");

        let body =
            to_bytes(response.into_body(), usize::MAX).await.expect("body should be readable");
        let body = String::from_utf8(body.to_vec()).expect("plain text should be utf-8");
        assert!(body.contains("File not found"));
    }

    #[tokio::test]
    async fn explicit_index_file_is_served_as_static_resource() {
        let response = serve_embedded_files("index.html").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/html; charset=utf-8");
    }

    #[tokio::test]
    async fn embedded_static_asset_is_served_with_cache_headers() {
        let css_path = WebAssets::iter()
            .find(|p| p.ends_with(".css"))
            .expect("no CSS asset found in web/dist");
        let response = serve_embedded_files(&css_path).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/css; charset=utf-8");
        assert_eq!(response.headers().get("cache-control").unwrap(), "public, max-age=604800");

        let body =
            to_bytes(response.into_body(), usize::MAX).await.expect("body should be readable");
        assert!(!body.is_empty());
    }

    #[tokio::test]
    async fn web_embed_file_handler_routes_root_and_spa_paths_to_index() {
        let root = web_embed_file_handler(Uri::from_static("/")).await.into_response();
        assert_eq!(root.status(), StatusCode::OK);
        assert_eq!(root.headers().get("content-type").unwrap(), "text/html; charset=utf-8");

        let spa = web_embed_file_handler(Uri::from_static("/dashboard")).await.into_response();
        assert_eq!(spa.status(), StatusCode::OK);
        assert_eq!(spa.headers().get("content-type").unwrap(), "text/html; charset=utf-8");
    }

    #[tokio::test]
    async fn web_embed_file_handler_serves_static_files_from_uri() {
        let js_path =
            WebAssets::iter().find(|p| p.ends_with(".js")).expect("no JS asset found in web/dist");
        let uri_str = format!("/{}", js_path);
        let uri = uri_str.parse::<Uri>().expect("valid URI");
        let response = web_embed_file_handler(uri).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(response.headers().get("cache-control").unwrap(), "public, max-age=604800");
    }

    #[tokio::test]
    async fn web_embed_file_handler_serves_percent_encoded_doc_paths() {
        let uri = Uri::from_static(
            "/doc/by/%E6%AF%8F%2024000%20%E4%B8%AA%E5%B7%A5%E4%BD%9C%E5%B0%8F%E6%97%B6%E6%88%96%E6%AF%8F%203%20%E5%B9%B4/%E6%AF%8F%2024000%20%E4%B8%AA%E5%B7%A5%E4%BD%9C%E5%B0%8F%E6%97%B6%E6%88%96%E6%AF%8F%203%20%E5%B9%B4(1).html",
        );

        let response = web_embed_file_handler(uri).await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/html; charset=utf-8");
        assert_eq!(response.headers().get("cache-control").unwrap(), "public, max-age=604800");
    }
}
