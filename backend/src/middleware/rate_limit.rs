use dashmap::DashMap;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub general_limit: u32,
    pub general_window: Duration,
    pub login_limit: u32,
    pub login_window: Duration,
    pub ai_limit: u32,
    pub ai_window: Duration,
    pub upload_limit: u32,
    pub upload_window: Duration,
}

impl RateLimitConfig {
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

#[derive(Debug)]
pub struct RateLimiter {
    store: DashMap<String, VecDeque<Instant>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            store: DashMap::new(),
        }
    }

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
