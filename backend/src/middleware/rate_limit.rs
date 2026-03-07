//! Sliding-window request rate limiting.

use dashmap::DashMap;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Environment-backed limits for the middleware's request buckets.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests allowed for general API traffic per window.
    pub general_limit: u32,
    /// Window size for general API traffic.
    pub general_window: Duration,
    /// Requests allowed for login attempts per window.
    pub login_limit: u32,
    /// Window size for login attempts.
    pub login_window: Duration,
    /// Requests allowed for AI endpoints per window.
    pub ai_limit: u32,
    /// Window size for AI endpoints.
    pub ai_window: Duration,
    /// Requests allowed for uploads and submissions per window.
    pub upload_limit: u32,
    /// Window size for uploads and submissions.
    pub upload_window: Duration,
}

impl RateLimitConfig {
    /// Builds rate-limit settings from environment variables with safe defaults.
    pub fn from_env() -> Self {
        let general_limit = env_u32("RATE_LIMIT_GENERAL_PER_MINUTE", 120);
        let login_limit = env_u32("RATE_LIMIT_LOGIN_PER_5_MIN", 20);
        let ai_limit = env_u32("RATE_LIMIT_AI_PER_MINUTE", 30);
        let upload_limit = env_u32("RATE_LIMIT_UPLOAD_PER_MINUTE", 20);

        Self {
            general_limit,
            general_window: Duration::from_secs(60),
            login_limit,
            login_window: Duration::from_secs(5 * 60),
            ai_limit,
            ai_window: Duration::from_secs(60),
            upload_limit,
            upload_window: Duration::from_secs(60),
        }
    }
}

/// In-memory sliding-window rate limiter keyed by bucket and client identity.
#[derive(Debug)]
pub struct RateLimiter {
    store: DashMap<String, VecDeque<Instant>>,
}

impl RateLimiter {
    /// Creates an empty limiter store.
    pub fn new() -> Self {
        Self {
            store: DashMap::new(),
        }
    }

    /// Returns `true` when a request fits inside the configured window.
    pub fn check(&self, key: &str, limit: u32, window: Duration) -> bool {
        let now = Instant::now();
        let mut entry = self
            .store
            .entry(key.to_string())
            .or_insert_with(VecDeque::new);
        while let Some(front) = entry.front() {
            if now.duration_since(*front) > window {
                entry.pop_front();
            } else {
                break;
            }
        }
        if entry.len() >= limit as usize {
            return false;
        }
        entry.push_back(now);
        true
    }
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_blocks_when_limit_reached() {
        let limiter = RateLimiter::new();
        let key = "login:127.0.0.1";
        let window = Duration::from_secs(60);

        assert!(limiter.check(key, 1, window));
        assert!(!limiter.check(key, 1, window));
    }

    #[tokio::test]
    async fn check_allows_after_window_expires() {
        let limiter = RateLimiter::new();
        let key = "ai:127.0.0.1";
        let window = Duration::from_millis(1);

        assert!(limiter.check(key, 1, window));
        tokio::time::sleep(Duration::from_millis(3)).await;
        assert!(limiter.check(key, 1, window));
    }

    #[test]
    fn env_u32_parses_and_falls_back() {
        let key = "RATE_LIMIT_TEST_KEY";

        std::env::set_var(key, "42");
        assert_eq!(env_u32(key, 7), 42);

        std::env::set_var(key, "not-a-number");
        assert_eq!(env_u32(key, 7), 7);

        std::env::remove_var(key);
    }

    #[test]
    fn from_env_populates_windows() {
        let config = RateLimitConfig::from_env();
        assert_eq!(config.general_window, Duration::from_secs(60));
        assert_eq!(config.login_window, Duration::from_secs(5 * 60));
        assert_eq!(config.ai_window, Duration::from_secs(60));
        assert_eq!(config.upload_window, Duration::from_secs(60));
    }
}
