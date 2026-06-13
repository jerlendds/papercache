use actix_web::{HttpRequest, HttpResponse, http::header};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "ui/dist/"]
struct UiAssets;

pub async fn serve(req: HttpRequest) -> HttpResponse {
    let path = req.path().trim_start_matches('/');
    if path.starts_with("api/") {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "not found"
        }));
    }

    let asset_path = if path.is_empty() { "index.html" } else { path };
    let asset_path = if is_safe_asset_path(asset_path) {
        asset_path
    } else {
        return HttpResponse::BadRequest().finish();
    };

    let resolved_path = if UiAssets::get(asset_path).is_some() {
        asset_path
    } else if should_fallback_to_index(asset_path) {
        "index.html"
    } else {
        return HttpResponse::NotFound().finish();
    };

    match UiAssets::get(resolved_path) {
        Some(asset) => {
            let content_type = content_type(resolved_path);
            let mut response = HttpResponse::Ok();
            response.insert_header((header::CONTENT_TYPE, content_type));
            if resolved_path.starts_with("assets/") {
                response
                    .insert_header((header::CACHE_CONTROL, "public, max-age=31536000, immutable"));
            } else {
                response.insert_header((header::CACHE_CONTROL, "no-cache"));
            }
            response.body(asset.data.into_owned())
        }
        None => HttpResponse::NotFound().finish(),
    }
}

fn is_safe_asset_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.split('/').any(|part| part == ".." || part.is_empty())
}

fn should_fallback_to_index(path: &str) -> bool {
    !path.starts_with("assets/") && !path.contains('.')
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}
