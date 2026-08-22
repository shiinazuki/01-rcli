use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
};

use anyhow::Result;
use tokio::fs;
use tracing::info;

/// 递归为目录树生成 index.html，返回生成的文件数。
///
/// # Errors
pub async fn process_http_index(root: PathBuf, force: bool) -> Result<usize> {
    let mut generated = 0;
    let mut stack = vec![root.clone()];

    while let Some(dir) = stack.pop() {
        let mut entries: Vec<(String, bool)> = Vec::new();

        let mut read_dir = fs::read_dir(&dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "index.html" {
                continue;
            }
            let is_dir = entry.file_type().await.is_ok_and(|t| t.is_dir());
            if is_dir {
                stack.push(entry.path());
            }
            entries.push((name, is_dir));
        }

        let index = dir.join("index.html");
        if index.exists() && !force {
            info!(
                "Skip {} (exists, use --force to overwrite)",
                index.display()
            );
            continue;
        }

        // 目录排前面，同类按名字排
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let rel = dir.strip_prefix(&root).unwrap_or(Path::new(""));
        let html = render_index(rel, &entries, dir != root);

        fs::write(&index, html).await?;
        info!("Wrote {}", index.display());
        generated += 1;
    }

    Ok(generated)
}

fn render_index(rel: &Path, entries: &[(String, bool)], has_parent: bool) -> String {
    let title = if rel.as_os_str().is_empty() {
        "/".to_owned()
    } else {
        format!("/{}", rel.display())
    };

    let mut items = String::new();
    if has_parent {
        items.push_str("<li class=\"up\"><a href=\"../\">../</a></li>");
    }
    for (name, is_dir) in entries {
        let slash = if *is_dir { "/" } else { "" };
        let _ = write!(
            items,
            "<li><a href=\"{n}{slash}\">{n}{slash}</a></li>",
            n = escape(name)
        );
    }

    format!(
        r#"<!doctype html>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<style>
  body {{ font: 16px/1.7 ui-monospace, SFMono-Regular, Menlo, monospace;
         max-width: 48rem; margin: 3rem auto; padding: 0 1rem; }}
  h1 {{ font-size: 1.2rem; border-bottom: 1px solid #8884; padding-bottom: .5rem; }}
  ul {{ list-style: none; padding: 0; }}
  li {{ padding: .15rem 0; }}
  a {{ text-decoration: none; }}
  a:hover {{ text-decoration: underline; }}
  .up a {{ opacity: .6; }}
</style>
<h1>{title}</h1>
<ul>{items}</ul>
"#
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
