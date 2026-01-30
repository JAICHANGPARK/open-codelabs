# Security Guide

This document provides practical guidance for running Open Codelabs safely in a self-hosted environment.

## Quick checklist

- Change `ADMIN_PW` to a strong password and rotate it regularly.
- Set `AUTH_SECRETS` to separate signing keys and allow rotation.
- Enable `COOKIE_SECURE=true` when using HTTPS.
- Use `TRUST_PROXY=true` only when running behind a proxy.
- Restrict `CORS_ALLOWED_ORIGINS` to real frontend domains.
- Tune `RATE_LIMIT_*` values for your environment.
- Minimize permissions for upload paths and database files.

## Auth, sessions, and CSRF

- Admin and attendee auth uses **session-cookie JWTs**.
- On login or attendee registration, a session cookie (`oc_session`) and CSRF cookie (`oc_csrf`) are issued.
- With a session, `POST/PUT/DELETE` requests must include the `oc_csrf` value in the `X-CSRF-Token` header.
- With HTTPS and `COOKIE_SECURE=true`, cookies use the `__Host-` prefix.

Related environment variables:

```
AUTH_SECRETS=secret1,secret2
ADMIN_SESSION_TTL_SECONDS=28800
ATTENDEE_SESSION_TTL_SECONDS=43200
COOKIE_SECURE=true
COOKIE_SAMESITE=lax
```

## Network and HTTPS

- Production must be protected with HTTPS.
- Set `TRUST_PROXY=true` only when running behind a reverse proxy (Nginx/Traefik/Caddy).
- Ensure the proxy forwards `x-forwarded-proto` correctly.

Related environment variables:

```
TRUST_PROXY=true
CORS_ALLOWED_ORIGINS=https://codelabs.example.com
```

## Security headers

Default CSP/HSTS headers are included in code and can be overridden via environment variables.

```
CSP_HEADER=default-src 'self'; ...
HSTS_HEADER=max-age=63072000; includeSubDomains; preload
```

## Rate limiting

IP-based rate limits are applied. Defaults:

- General requests: 120 per minute
- Login: 20 per 5 minutes
- AI requests: 30 per minute
- Uploads: 20 per minute

Adjust if needed:

```
RATE_LIMIT_GENERAL_PER_MINUTE=120
RATE_LIMIT_LOGIN_PER_5_MIN=20
RATE_LIMIT_AI_PER_MINUTE=30
RATE_LIMIT_UPLOAD_PER_MINUTE=20
```

## Upload security

Uploads have size limits:

- Image uploads: 5MB (WebP conversion)
- Material uploads: 10MB
- Submissions: 5MB per file, 10MB total

Files are stored under `/static/uploads`. In production:

- Minimize permissions on upload directories.
- Add external storage or virus scanning if needed.

## Secret management

- Never commit `.env` files.
- Use CI/CD secret storage (GitHub Actions Secrets, etc.).
- Rotate production secrets regularly.

## Gemini API key handling

- Keys stored in the admin UI must be **encrypted with `ADMIN_PW`**.
- The backend rejects plaintext keys.
- `GEMINI_API_KEY` is used as the server default key.

## Audit logs

Important events are stored in the `audit_logs` table.

- Login, settings changes, uploads, AI requests, and more
- In production, review logs and define a retention policy

## Data protection and backups

- Restrict permissions on SQLite database files.
- Use Docker volumes to avoid data loss.
- Perform regular backups and verify restore procedures.

## Public exposure notes

- For public access, use `./run-public.sh --ngrok|--bore|--cloudflare`.
- Add Basic Auth if you need extra protection.

## Related docs

- [Environment variables](environment.md)
- [Public deployment](public-deployment.md)
- [API Reference](../specification/api-reference.md)
