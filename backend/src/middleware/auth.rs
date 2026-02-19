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

use crate::infrastructure::database::AppState;
use crate::utils::error::{forbidden, unauthorized};

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
    pub attendee_cookie_name: String,
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
            attendee_cookie_name: format!("{}oc_attendee_session", cookie_prefix),
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
    pub admin_claims: Option<SessionClaims>,
    pub attendee_claims: Option<SessionClaims>,
}

impl AuthSession {
    pub fn require_admin(&self) -> Result<SessionClaims, (StatusCode, String)> {
        if let Some(claims) = &self.admin_claims {
            return Ok(claims.clone());
        }
        match &self.claims {
            Some(claims) if Role::from_str(&claims.role) == Some(Role::Admin) => Ok(claims.clone()),
            Some(_) => Err(forbidden()),
            None => Err(unauthorized()),
        }
    }

    pub fn require_attendee(&self) -> Result<SessionClaims, (StatusCode, String)> {
        if let Some(claims) = &self.attendee_claims {
            return Ok(claims.clone());
        }
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
        let admin_token = jar
            .get(&state.auth.cookie_name)
            .map(|cookie| cookie.value().to_string());
        let attendee_token = jar
            .get(&state.auth.attendee_cookie_name)
            .map(|cookie| cookie.value().to_string());

        if admin_token.is_none() && attendee_token.is_none() {
            eprintln!(
                "AuthSession: No session cookie found (cookie_name={}, attendee_cookie_name={})",
                state.auth.cookie_name, state.auth.attendee_cookie_name
            );
        }

        let admin_claims = admin_token.and_then(|value| {
            let result = state.auth.verify_token(&value);
            if result.is_none() {
                eprintln!("AuthSession: Admin token verification failed");
            }
            result
        });

        let attendee_claims = attendee_token.and_then(|value| {
            let result = state.auth.verify_token(&value);
            if result.is_none() {
                eprintln!("AuthSession: Attendee token verification failed");
            }
            result
        });
        let claims = admin_claims.clone().or(attendee_claims.clone());
        Ok(Self {
            claims,
            admin_claims,
            attendee_claims,
        })
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

pub fn build_attendee_session_cookie(
    config: &AuthConfig,
    token: String,
    max_age: Duration,
) -> Cookie<'static> {
    Cookie::build((config.attendee_cookie_name.clone(), token))
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
    use crate::infrastructure::database::{AppState, DbKind};
    use axum::{body::Body, http::Request};
    use axum_extra::extract::cookie::SameSite;
    use sqlx::any::AnyPoolOptions;

    fn claims(role: Role, config: &AuthConfig) -> SessionClaims {
        let now = now_epoch_seconds();
        SessionClaims {
            sub: "user-1".to_string(),
            role: role.as_str().to_string(),
            codelab_id: Some("codelab-1".to_string()),
            iss: config.issuer.clone(),
            aud: config.audience.clone(),
            iat: now,
            exp: now + 60,
        }
    }

    fn test_config() -> AuthConfig {
        AuthConfig {
            issuer: "test-issuer".to_string(),
            audience: "test-audience".to_string(),
            secrets: vec!["secret1".to_string(), "oldsecret".to_string()],
            admin_ttl: Duration::from_secs(60),
            attendee_ttl: Duration::from_secs(60),
            cookie_name: "oc_session".to_string(),
            attendee_cookie_name: "oc_attendee_session".to_string(),
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

    #[test]
    fn role_from_str_and_as_str_cover_all_cases() {
        assert_eq!(Role::Admin.as_str(), "admin");
        assert_eq!(Role::Attendee.as_str(), "attendee");
        assert_eq!(Role::from_str("admin"), Some(Role::Admin));
        assert_eq!(Role::from_str("attendee"), Some(Role::Attendee));
        assert_eq!(Role::from_str("other"), None);
    }

    #[test]
    fn from_env_parses_samesite_and_cookie_prefix() {
        struct EnvRestore {
            key: &'static str,
            value: Option<String>,
        }
        impl EnvRestore {
            fn new(key: &'static str) -> Self {
                Self {
                    key,
                    value: std::env::var(key).ok(),
                }
            }
        }
        impl Drop for EnvRestore {
            fn drop(&mut self) {
                if let Some(value) = &self.value {
                    std::env::set_var(self.key, value);
                } else {
                    std::env::remove_var(self.key);
                }
            }
        }

        std::env::set_var("AUTH_ISSUER", "pre-existing");
        let _issuer = EnvRestore::new("AUTH_ISSUER");
        let _aud = EnvRestore::new("AUTH_AUDIENCE");
        let _secret = EnvRestore::new("AUTH_SECRETS");
        let _cookie_secure = EnvRestore::new("COOKIE_SECURE");
        let _cookie_samesite = EnvRestore::new("COOKIE_SAMESITE");
        let _admin_pw = EnvRestore::new("ADMIN_PW");

        std::env::set_var("AUTH_ISSUER", "issuer-x");
        std::env::set_var("AUTH_AUDIENCE", "aud-x");
        std::env::set_var("AUTH_SECRETS", "s1,s2");
        std::env::set_var("COOKIE_SECURE", "false");
        std::env::set_var("COOKIE_SAMESITE", "none");
        std::env::set_var("ADMIN_PW", "pw-x");

        let cfg = AuthConfig::from_env();
        assert_eq!(cfg.issuer, "issuer-x");
        assert_eq!(cfg.audience, "aud-x");
        assert_eq!(cfg.secrets, vec!["s1".to_string(), "s2".to_string()]);
        // none without secure should fall back to Lax.
        assert_eq!(cfg.cookie_same_site, SameSite::Lax);
        assert_eq!(cfg.cookie_name, "oc_session");

        std::env::remove_var("AUTH_SECRETS");
        std::env::set_var("COOKIE_SECURE", "true");
        std::env::set_var("COOKIE_SAMESITE", "none");
        let secure_none_cfg = AuthConfig::from_env();
        assert_eq!(secure_none_cfg.cookie_same_site, SameSite::None);
        assert!(secure_none_cfg.cookie_secure);
        assert_eq!(secure_none_cfg.cookie_name, "__Host-oc_session");

        std::env::set_var("COOKIE_SAMESITE", "strict");
        let secure_cfg = AuthConfig::from_env();
        assert_eq!(secure_cfg.cookie_same_site, SameSite::Strict);
        assert!(secure_cfg.cookie_secure);
        assert_eq!(secure_cfg.cookie_name, "__Host-oc_session");
    }

    #[test]
    fn require_admin_and_attendee_cover_rejections() {
        let config = test_config();

        let admin = claims(Role::Admin, &config);
        let attendee = claims(Role::Attendee, &config);

        let admin_session = AuthSession {
            claims: Some(admin.clone()),
            admin_claims: Some(admin.clone()),
            attendee_claims: None,
        };
        assert_eq!(admin_session.require_admin().unwrap().role, "admin");
        assert_eq!(
            admin_session.require_attendee().unwrap_err().0,
            StatusCode::FORBIDDEN
        );

        let attendee_session = AuthSession {
            claims: Some(attendee.clone()),
            admin_claims: None,
            attendee_claims: Some(attendee),
        };
        assert_eq!(
            attendee_session.require_admin().unwrap_err().0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            attendee_session.require_attendee().unwrap().role,
            Role::Attendee.as_str()
        );

        let no_session = AuthSession {
            claims: None,
            admin_claims: None,
            attendee_claims: None,
        };
        assert_eq!(
            no_session.require_admin().unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            no_session.require_attendee().unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );

        let attendee_from_claims_only = AuthSession {
            claims: Some(claims(Role::Attendee, &config)),
            admin_claims: None,
            attendee_claims: None,
        };
        assert_eq!(
            attendee_from_claims_only.require_attendee().unwrap().role,
            "attendee"
        );
    }

    #[test]
    fn cookie_builders_apply_expected_attributes() {
        let config = test_config();
        let cookie = build_session_cookie(&config, "token".to_string(), Duration::from_secs(60));
        assert_eq!(cookie.name(), "oc_session");
        assert_eq!(cookie.path(), Some("/"));
        assert!(cookie.http_only().unwrap_or(false));

        let attendee = build_attendee_session_cookie(
            &config,
            "attendee-token".to_string(),
            Duration::from_secs(60),
        );
        assert_eq!(attendee.name(), "oc_attendee_session");
        assert!(attendee.http_only().unwrap_or(false));

        let csrf = build_csrf_cookie(&config, "csrf".to_string(), Duration::from_secs(60));
        assert_eq!(csrf.name(), "oc_csrf");
        assert!(!csrf.http_only().unwrap_or(true));

        let cleared = clear_cookie("oc_session");
        assert_eq!(cleared.name(), "oc_session");
    }

    #[test]
    fn verify_token_supports_secret_rotation() {
        let mut config = test_config();
        config.secrets = vec!["new".to_string(), "old".to_string()];

        let old_only = AuthConfig {
            secrets: vec!["old".to_string()],
            ..config.clone()
        };
        let token = old_only
            .issue_token(&claims(Role::Admin, &old_only))
            .expect("token with old secret");

        let decoded = config
            .verify_token(&token)
            .expect("decoded with rotated secrets");
        assert_eq!(decoded.role, "admin");
    }

    async fn make_state_with_auth(auth: AuthConfig) -> Arc<AppState> {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("sqlite");

        let mut state = AppState::new(
            pool,
            DbKind::Sqlite,
            "admin".to_string(),
            "pw".to_string(),
            false,
        );
        state.auth = auth;
        Arc::new(state)
    }

    #[tokio::test]
    async fn from_request_parts_extracts_admin_and_attendee_claims() {
        let config = test_config();
        let state = make_state_with_auth(config.clone()).await;

        let admin_token = config
            .issue_token(&claims(Role::Admin, &config))
            .expect("admin token");
        let attendee_token = config
            .issue_token(&claims(Role::Attendee, &config))
            .expect("attendee token");

        let cookie_header = format!(
            "{}={}; {}={}",
            config.cookie_name, admin_token, config.attendee_cookie_name, attendee_token
        );
        let request = Request::builder()
            .uri("/api/x")
            .header("cookie", cookie_header)
            .body(Body::empty())
            .unwrap();
        let (mut parts, _) = request.into_parts();

        let session = AuthSession::from_request_parts(&mut parts, &state)
            .await
            .expect("extract session");
        assert!(session.admin_claims.is_some());
        assert!(session.attendee_claims.is_some());
        assert!(session.claims.is_some());
    }

    #[tokio::test]
    async fn from_request_parts_handles_invalid_or_missing_tokens() {
        let config = test_config();
        let state = make_state_with_auth(config.clone()).await;

        let invalid_request = Request::builder()
            .uri("/api/x")
            .header("cookie", format!("{}=bad-token", config.cookie_name))
            .body(Body::empty())
            .unwrap();
        let (mut invalid_parts, _) = invalid_request.into_parts();
        let invalid = AuthSession::from_request_parts(&mut invalid_parts, &state)
            .await
            .expect("extract invalid");
        assert!(invalid.claims.is_none());

        let no_cookie_request = Request::builder()
            .uri("/api/x")
            .body(Body::empty())
            .unwrap();
        let (mut no_cookie_parts, _) = no_cookie_request.into_parts();
        let none = AuthSession::from_request_parts(&mut no_cookie_parts, &state)
            .await
            .expect("extract no cookie");
        assert!(none.claims.is_none());

        let invalid_attendee_request = Request::builder()
            .uri("/api/x")
            .header(
                "cookie",
                format!("{}=bad-token", config.attendee_cookie_name),
            )
            .body(Body::empty())
            .unwrap();
        let (mut invalid_attendee_parts, _) = invalid_attendee_request.into_parts();
        let invalid_attendee = AuthSession::from_request_parts(&mut invalid_attendee_parts, &state)
            .await
            .expect("extract invalid attendee");
        assert!(invalid_attendee.claims.is_none());
    }
}
