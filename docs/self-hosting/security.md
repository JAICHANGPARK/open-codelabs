# 보안 가이드

이 문서는 Self-hosting 환경에서 Open Codelabs를 안전하게 운영하기 위한 실전 지침입니다.

## 빠른 체크리스트

- `ADMIN_PW`를 강력한 비밀번호로 변경하고 정기적으로 교체합니다.
- `AUTH_SECRETS`를 설정해 토큰 서명을 분리하고 회전 가능하게 합니다.
- HTTPS 환경에서 `COOKIE_SECURE=true`로 설정합니다.
- 프록시 뒤에서 운영 시에만 `TRUST_PROXY=true`를 사용합니다.
- `CORS_ALLOWED_ORIGINS`에 실제 프론트엔드 도메인만 허용합니다.
- `RATE_LIMIT_*` 값을 운영 환경에 맞게 조정합니다.
- 업로드 경로/DB 파일에 대한 권한을 최소화합니다.

## 인증, 세션, CSRF

- 관리자/참가자 인증은 **세션 쿠키 기반 JWT**로 동작합니다.
- 로그인 또는 참가자 등록 시 세션 쿠키(`oc_session`)와 CSRF 쿠키(`oc_csrf`)가 발급됩니다.
- **세션이 있는 상태에서** `POST/PUT/DELETE` 요청은 `X-CSRF-Token` 헤더에 `oc_csrf` 값을 전달해야 합니다.
- HTTPS에서 `COOKIE_SECURE=true`를 설정하면 쿠키는 `__Host-` 접두사를 사용합니다.

관련 환경 변수:

```
AUTH_SECRETS=secret1,secret2
ADMIN_SESSION_TTL_SECONDS=28800
ATTENDEE_SESSION_TTL_SECONDS=43200
COOKIE_SECURE=true
COOKIE_SAMESITE=lax
```

## 네트워크와 HTTPS

- 운영 환경은 반드시 HTTPS로 보호합니다.
- 리버스 프록시(Nginx/Traefik/Caddy 등) 뒤에서 운영할 경우에만 `TRUST_PROXY=true`를 설정합니다.
- 프록시가 `x-forwarded-proto`를 정확히 전달하도록 설정하세요.

관련 환경 변수:

```
TRUST_PROXY=true
CORS_ALLOWED_ORIGINS=https://codelabs.example.com
```

## 보안 헤더

기본 CSP/HSTS는 코드에 포함되어 있으며 필요 시 환경 변수로 덮어쓸 수 있습니다.

```
CSP_HEADER=default-src 'self'; ...
HSTS_HEADER=max-age=63072000; includeSubDomains; preload
```

## 레이트 리밋

IP 기준 요청 제한이 적용됩니다. 기본값은 아래와 같습니다.

- 일반 요청: 분당 120회
- 로그인: 5분당 20회
- AI 요청: 분당 30회
- 업로드: 분당 20회

필요 시 조정:

```
RATE_LIMIT_GENERAL_PER_MINUTE=120
RATE_LIMIT_LOGIN_PER_5_MIN=20
RATE_LIMIT_AI_PER_MINUTE=30
RATE_LIMIT_UPLOAD_PER_MINUTE=20
```

## 업로드 보안

업로드는 파일 크기 제한이 적용됩니다.

- 이미지 업로드: 5MB (WebP 변환)
- 자료 업로드: 10MB
- 과제 제출: 파일당 5MB, 총 10MB

업로드 경로는 `/static/uploads` 하위에 저장됩니다. 운영 환경에서는:

- 업로드 디렉토리 권한을 최소화합니다.
- 필요 시 외부 스토리지나 바이러스 스캔 파이프라인을 추가합니다.

## 비밀 관리

- `.env` 파일은 절대 커밋하지 않습니다.
- CI/CD에서는 시크릿 저장소(GitHub Actions Secrets 등)를 사용합니다.
- 운영 비밀은 주기적으로 회전합니다.

## Gemini API 키 관리

- 관리자 UI에서 저장하는 키는 **ADMIN_PW로 암호화**되어야 합니다.
- 백엔드는 평문 키를 거부합니다.
- `GEMINI_API_KEY` 환경 변수는 서버 기본 키로 사용됩니다.

## 감사 로그

`audit_logs` 테이블에 주요 이벤트가 기록됩니다.

- 로그인/설정 변경/업로드/AI 요청 등
- 운영 환경에서는 정기적으로 로그를 점검하고 보관 정책을 정의하세요.

## 데이터 보호 및 백업

- SQLite 사용 시 DB 파일 권한을 제한합니다.
- Docker 볼륨을 사용해 데이터 유실을 방지합니다.
- 정기 백업을 수행하고 복구 절차를 검증합니다.

## 공개 배포 시 주의

- 외부 공개 시 `./run-public.sh --ngrok|--bore|--cloudflare` 사용을 권장합니다.
- 필요 시 Basic Auth를 추가해 공개 URL 보호를 강화합니다.

## 관련 문서

- [환경 변수 설정](environment.md)
- [공개 배포](public-deployment.md)
- [API 레퍼런스](../specification/api-reference.md)
