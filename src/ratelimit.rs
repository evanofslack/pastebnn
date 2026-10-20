use axum::{
    extract::{ConnectInfo, Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const WINDOW: Duration = Duration::from_secs(60);

/// Fixed window per-IP rate limiter
pub struct RateLimiter {
    limit_per_minute: u32,
    trust_proxy: bool,
    windows: Mutex<HashMap<IpAddr, (Instant, u32)>>,
}

impl RateLimiter {
    pub fn new(limit_per_minute: u32, trust_proxy: bool) -> Self {
        Self {
            limit_per_minute,
            trust_proxy,
            windows: Mutex::new(HashMap::new()),
        }
    }

    /// Record a request from ip, returning seconds until retry if over the limit
    fn check(&self, ip: IpAddr, now: Instant) -> Result<(), u64> {
        if self.limit_per_minute == 0 {
            return Ok(());
        }
        let mut windows = self.windows.lock().unwrap();
        let (start, count) = windows.entry(ip).or_insert((now, 0));
        if now.duration_since(*start) >= WINDOW {
            *start = now;
            *count = 0;
        }
        if *count >= self.limit_per_minute {
            let retry = WINDOW.saturating_sub(now.duration_since(*start));
            return Err(retry.as_secs().max(1));
        }
        *count += 1;
        Ok(())
    }

    /// Drop windows that have elapsed so the map does not grow unbounded
    pub fn sweep(&self) {
        let now = Instant::now();
        self.windows
            .lock()
            .unwrap()
            .retain(|_, (start, _)| now.duration_since(*start) < WINDOW);
    }

    pub async fn sweep_periodically(&self) {
        let mut interval = tokio::time::interval(WINDOW);
        loop {
            interval.tick().await;
            self.sweep();
        }
    }

    fn client_ip<B>(&self, req: &axum::http::Request<B>) -> Option<IpAddr> {
        if self.trust_proxy {
            // rightmost entry is the one appended by our own reverse proxy
            let forwarded = req
                .headers()
                .get("x-forwarded-for")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.rsplit(',').next())
                .and_then(|ip| ip.trim().parse().ok());
            if forwarded.is_some() {
                return forwarded;
            }
        }
        req.extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip())
    }
}

pub async fn limit(State(limiter): State<Arc<RateLimiter>>, req: Request, next: Next) -> Response {
    if let Some(ip) = limiter.client_ip(&req)
        && let Err(retry_after) = limiter.check(ip, Instant::now())
    {
        tracing::debug!(%ip, "rate limited");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
        )
            .into_response();
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    const IP: IpAddr = IpAddr::V4(std::net::Ipv4Addr::new(1, 2, 3, 4));

    #[test]
    fn limits_after_threshold() {
        let limiter = RateLimiter::new(2, false);
        let now = Instant::now();
        assert!(limiter.check(IP, now).is_ok());
        assert!(limiter.check(IP, now).is_ok());
        assert!(limiter.check(IP, now).is_err());
        // other clients are unaffected
        assert!(limiter.check(IpAddr::V4([5, 6, 7, 8].into()), now).is_ok());
        // window resets
        assert!(limiter.check(IP, now + WINDOW).is_ok());
    }

    #[test]
    fn zero_disables() {
        let limiter = RateLimiter::new(0, false);
        for _ in 0..100 {
            assert!(limiter.check(IP, Instant::now()).is_ok());
        }
    }

    #[test]
    fn forwarded_for_only_when_trusted() {
        let req = axum::http::Request::builder()
            .header("x-forwarded-for", "9.9.9.9, 1.2.3.4")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 80))))
            .body(())
            .unwrap();
        assert_eq!(RateLimiter::new(1, true).client_ip(&req), Some(IP));
        assert_eq!(
            RateLimiter::new(1, false).client_ip(&req),
            Some(IpAddr::V4([127, 0, 0, 1].into()))
        );
    }
}
