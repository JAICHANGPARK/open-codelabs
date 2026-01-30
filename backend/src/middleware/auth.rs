use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use cookie::time::Duration as CookieDuration;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::utils::error::{forbidden, unauthorized};
use crate::infrastructure::database::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Admin,
    Attendee,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Attendee => "attendee",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "admin" => Some(Role::Admin),
            "attendee" => Some(Role::Attendee),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionClaims {
    pub sub: String,
    pub role: String,
    pub codelab_id: Option<String>,
    pub iss: String,
    pub aud: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub issuer: String,
    pub audience: String,
    pub secrets: Vec<String>,
    pub admin_ttl: Duration,
    pub attendee_ttl: Duration,
    pub cookie_name: String,
    pub csrf_cookie_name: String,
    pub cookie_secure: bool,
    pub cookie_same_site: SameSite,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        let issuer = std::env::var("AUTH_ISSUER").unwrap_or_else(|_| "open-codelabs".to_string());
        let audience =
            std::env::var("AUTH_AUDIENCE").unwrap_or_else(|_| "open-codelabs".to_string());

        let secrets_env = std::env::var("AUTH_SECRETS")
            .or_else(|_| std::env::var("AUTH_SECRET"))
            .unwrap_or_default();
        let mut secrets: Vec<String> = secrets_env
            .split(',')
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();

        if secrets.is_empty() {
            let fallback = std::env::var("ADMIN_PW").unwrap_or_else(|_| "insecure".to_string());
            tracing::warn!(
                "AUTH_SECRETS not set; falling back to ADMIN_PW for signing. Set AUTH_SECRETS to enable rotation."
            );
            secrets.push(fallback);
        }

        let cookie_secure = std::env::var("COOKIE_SECURE")
            .ok()
            .map(|value| value == "true")
            .unwrap_or(false);

        let cookie_same_site = match std::env::var("COOKIE_SAMESITE")
            .unwrap_or_else(|_| "lax".to_string())
            .to_lowercase()
            .as_str()
        {
            "strict" => SameSite::Strict,
            "none" => {
                if !cookie_secure {
                    tracing::warn!(
                        "COOKIE_SAMESITE=none requires COOKIE_SECURE=true; falling back to Lax."
                    );
                    SameSite::Lax
                } else {
                    SameSite::None
                }
            }
            _ => SameSite::Lax,
        };

        let admin_ttl = Duration::from_secs(
            std::env::var("ADMIN_SESSION_TTL_SECONDS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(8 * 60 * 60),
        );
        let attendee_ttl = Duration::from_secs(
            std::env::var("ATTENDEE_SESSION_TTL_SECONDS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(12 * 60 * 60),
        );

        let cookie_prefix = if cookie_secure { "__Host-" } else { "" };

        Self {
            issuer,
            audience,
            secrets,
            admin_ttl,
            attendee_ttl,
            cookie_name: format!("{}oc_session", cookie_prefix),
            csrf_cookie_name: format!("{}oc_csrf", cookie_prefix),
            cookie_secure,
            cookie_same_site,
        }
    }

    pub fn issue_token(
        &self,
        claims: &SessionClaims,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let header = Header::new(Algorithm::HS256);
        encode(
            &header,
            claims,
            &EncodingKey::from_secret(self.secrets[0].as_bytes()),
        )
    }

    pub fn verify_token(&self, token: &str) -> Option<SessionClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[self.issuer.clone()]);
        validation.set_audience(&[self.audience.clone()]);
        validation.validate_exp = true;
        validation.leeway = 30;

        for secret in &self.secrets {
            let key = DecodingKey::from_secret(secret.as_bytes());
            if let Ok(data) = decode::<SessionClaims>(token, &key, &validation) {
                return Some(data.claims);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub claims: Option<SessionClaims>,
}

impl AuthSession {
    pub fn require_admin(&self) -> Result<SessionClaims, (StatusCode, String)> {
        match &self.claims {
            Some(claims) if Role::from_str(&claims.role) == Some(Role::Admin) => Ok(claims.clone()),
            Some(_) => Err(forbidden()),
            None => Err(unauthorized()),
        }
    }

    pub fn require_attendee(&self) -> Result<SessionClaims, (StatusCode, String)> {
        match &self.claims {
            Some(claims) if Role::from_str(&claims.role) == Some(Role::Attendee) => {
                Ok(claims.clone())
            }
            Some(_) => Err(forbidden()),
            None => Err(unauthorized()),
        }
    }
}

impl<S> FromRequestParts<S> for AuthSession
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let State(state) = State::<Arc<AppState>>::from_request_parts(parts, state)
            .await
            .map_err(|_| unauthorized())?;

        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar
            .get(&state.auth.cookie_name)
            .map(|cookie| cookie.value().to_string());

        if token.is_none() {
            eprintln!("AuthSession: No session cookie found (cookie_name={})", state.auth.cookie_name);
        }

        let claims = token.and_then(|value| {
            let result = state.auth.verify_token(&value);
            if result.is_none() {
                eprintln!("AuthSession: Token verification failed");
            }
            result
        });
        Ok(Self { claims })
    }
}

pub fn build_session_cookie(
    config: &AuthConfig,
    token: String,
    max_age: Duration,
) -> Cookie<'static> {
    Cookie::build((config.cookie_name.clone(), token))
        .path("/")
        .http_only(true)
        .secure(config.cookie_secure)
        .same_site(config.cookie_same_site)
        .max_age(CookieDuration::seconds(max_age.as_secs() as i64))
        .build()
}

pub fn build_csrf_cookie(config: &AuthConfig, token: String, max_age: Duration) -> Cookie<'static> {
    Cookie::build((config.csrf_cookie_name.clone(), token))
        .path("/")
        .http_only(false)
        .secure(config.cookie_secure)
        .same_site(config.cookie_same_site)
        .max_age(CookieDuration::seconds(max_age.as_secs() as i64))
        .build()
}

pub fn clear_cookie(name: &str) -> Cookie<'static> {
    Cookie::build((name.to_string(), ""))
        .path("/")
        .max_age(CookieDuration::seconds(0))
        .build()
}

pub fn generate_csrf_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub fn now_epoch_seconds() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_extra::extract::cookie::SameSite;

    fn test_config() -> AuthConfig {
        AuthConfig {
            issuer: "test-issuer".to_string(),
            audience: "test-audience".to_string(),
            secrets: vec!["secret1".to_string(), "oldsecret".to_string()],
            admin_ttl: Duration::from_secs(60),
            attendee_ttl: Duration::from_secs(60),
            cookie_name: "oc_session".to_string(),
            csrf_cookie_name: "oc_csrf".to_string(),
            cookie_secure: false,
            cookie_same_site: SameSite::Lax,
        }
    }

    #[test]
    fn issue_and_verify_token() {
        let config = test_config();
        let now = now_epoch_seconds();
        let claims = SessionClaims {
            sub: "admin".to_string(),
            role: Role::Admin.as_str().to_string(),
            codelab_id: None,
            iss: config.issuer.clone(),
            aud: config.audience.clone(),
            iat: now,
            exp: now + 60,
        };
        let token = config.issue_token(&claims).expect("token");
        let decoded = config.verify_token(&token).expect("claims");
        assert_eq!(decoded.sub, "admin");
        assert_eq!(decoded.role, "admin");
    }

    #[test]
    fn verify_token_rejects_wrong_secret() {
        let config = test_config();
        let now = now_epoch_seconds();
        let claims = SessionClaims {
            sub: "attendee".to_string(),
            role: Role::Attendee.as_str().to_string(),
            codelab_id: Some("codelab".to_string()),
            iss: config.issuer.clone(),
            aud: config.audience.clone(),
            iat: now,
            exp: now + 60,
        };
        let token = config.issue_token(&claims).expect("token");

        let mut other_config = test_config();
        other_config.secrets = vec!["different".to_string()];
        assert!(other_config.verify_token(&token).is_none());
    }

    #[test]
    fn csrf_token_has_expected_length() {
        let token = generate_csrf_token();
        assert_eq!(token.len(), 32);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
