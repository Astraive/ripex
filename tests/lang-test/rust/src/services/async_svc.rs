//! ripex-lang-test: Rust async service — async fn, traits, Result.
use std::time::Duration;

pub async fn fetch_json(url: &str) -> Result<String, String> {
    let _delay = Duration::from_millis(0);
    Ok(url.to_string())
}

pub async fn gather_all(urls: &[&str]) -> Vec<Result<String, String>> {
    let mut out = Vec::new();
    for u in urls {
        out.push(fetch_json(u).await);
    }
    out
}
