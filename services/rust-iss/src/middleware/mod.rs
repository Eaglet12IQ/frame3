use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let mut map = self.inner.lock().unwrap();
        let now = Instant::now();

        let timestamps = map.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove timestamps outside the window
        timestamps.retain(|&t| now.duration_since(t) < self.window);

        // Check if under limit
        if timestamps.len() < self.max_requests {
            timestamps.push(now);
            true
        } else {
            false
        }
    }
}

pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Simple in-memory rate limiter (10 requests per minute per IP)
    static RATE_LIMITER: std::sync::OnceLock<RateLimiter> = std::sync::OnceLock::new();

    let limiter = RATE_LIMITER.get_or_init(|| RateLimiter::new(100, 60));

    // Use X-Forwarded-For header if available, otherwise use peer IP
    let headers = request.headers();
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or("unknown")
        .to_string();

    if limiter.check(&client_ip) {
        next.run(request).await
    } else {
        (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response()
    }
}
