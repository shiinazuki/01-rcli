use std::{
    fmt::Write as _,
    net::SocketAddr,
    path::{Component, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use tokio::{fs, net::TcpListener};
use tokio_util::io::ReaderStream;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{info, warn};

#[derive(Debug, Clone)]
struct HttpServeState {
    dir: PathBuf,
}

/// # Errors
pub async fn process_http_serve(dir: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Serving {} on {addr}", dir.display());

    let dir_service = ServeDir::new(dir)
        .append_index_html_on_directories(true)
        .precompressed_gzip()
        .precompressed_br();

    let router = Router::new()
        .fallback_service(dir_service)
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, router).await?;
    Ok(())
}

// ====================================================================
// 下面手动实现
#[allow(
    clippy::allow_attributes,
    reason = "下面这条压制的是条件编译下才出现的 lint，expect 不适用"
)]
#[allow(
    dead_code,
    reason = "手写的 handler 还没接进 router，只有 #[cfg(test)] 的测试在调；换成 expect 会在 \
              --all-targets 下因为测试用到了它而落空"
)]
async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> Response {
    serve_path(&state.dir, &path).await
}

async fn serve_path(base: &std::path::Path, requested: &str) -> Response {
    let Some(full) = safe_join(base, requested) else {
        warn!("Rejected suspicious path: {requested}");
        return (StatusCode::NOT_FOUND, "Not found".to_owned()).into_response();
    };

    let Ok(meta) = fs::metadata(&full).await else {
        return (StatusCode::NOT_FOUND, format!("/{requested} not found")).into_response();
    };

    if meta.is_dir() {
        return match render_dir(&full, requested).await {
            Ok(html) => Html(html).into_response(),
            Err(e) => {
                warn!("Error listing directory: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        };
    }

    match fs::File::open(&full).await {
        Ok(file) => {
            info!("Serving {} ({} bytes)", full.display(), meta.len());
            (
                [(header::CONTENT_TYPE, content_type(&full))],
                Body::from_stream(ReaderStream::new(file)),
            )
                .into_response()
        }
        Err(e) => {
            warn!("Error opening file: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn render_dir(full: &std::path::Path, requested: &str) -> std::io::Result<String> {
    let mut read_dir = fs::read_dir(full).await?;
    let mut items = String::new();

    while let Some(entry) = read_dir.next_entry().await? {
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry.file_type().await.is_ok_and(|t| t.is_dir());

        let href = if requested.is_empty() {
            format!("/{name}")
        } else {
            format!("/{}/{name}", requested.trim_end_matches('/'))
        };

        let _ = write!(
            items,
            "<li><a href=\"{}\">{}{}</a></li>",
            escape(&href),
            escape(&name),
            if is_dir { "/" } else { "" }
        );
    }
    Ok(format!(
        "<!doctype html><meta \
         charset=\"utf-8\"><title>/{req}</title><h1>/{req}</h1><ul>{items}</ul>",
        req = escape(requested)
    ))
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn safe_join(base: &std::path::Path, requested: &str) -> Option<PathBuf> {
    let mut out = base.to_path_buf();

    for comp in std::path::Path::new(requested).components() {
        match comp {
            Component::Normal(seg) => {
                let seg = seg.to_str()?;
                if seg.contains('/') || seg.contains('\\') {
                    return None;
                }
                out.push(seg);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(out)
}

fn content_type(path: &std::path::Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);

    match ext.as_deref() {
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("pdf") => "application/pdf",
        Some("wasm") => "application/wasm",
        Some("txt" | "md" | "toml" | "csv" | "yaml" | "yml") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_handler() {
        let state = Arc::new(HttpServeState {
            dir: PathBuf::from("."),
        });
        let resp = file_handler(State(state), Path("Cargo.toml".to_owned())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("text/plain; charset=utf-8")
        );
    }

    #[tokio::test]
    async fn test_missing_file_is_404() {
        let state = Arc::new(HttpServeState {
            dir: PathBuf::from("."),
        });
        let resp = file_handler(State(state), Path("no-such-file".to_owned())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_dir_listing() {
        let state = Arc::new(HttpServeState {
            dir: PathBuf::from("."),
        });
        let resp = file_handler(State(state), Path("src".to_owned())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("text/html; charset=utf-8")
        );
    }

    #[test]
    fn test_safe_join_rejects_traversal() {
        let base = std::path::Path::new("fixtures");
        assert!(safe_join(base, "../Cargo.toml").is_none());
        assert!(safe_join(base, "a/../../etc/passwd").is_none());
        assert!(safe_join(base, "/etc/passwd").is_none());
        assert_eq!(
            safe_join(base, "./b64.txt"),
            Some(PathBuf::from("fixtures/b64.txt"))
        );
    }

    #[test]
    fn test_escape() {
        assert_eq!(escape("<b>&\"x\""), "&lt;b&gt;&amp;&quot;x&quot;");
    }
}
