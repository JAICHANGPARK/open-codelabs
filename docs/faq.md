# FAQ (자주 묻는 질문)

Open Codelabs 사용 중 자주 묻는 질문과 답변입니다.

## 설치 및 실행

### Q: Docker 없이 실행할 수 있나요?

**A:** 네, 가능합니다. Rust와 Bun을 설치하고 로컬에서 실행할 수 있습니다.

```bash
# Backend
cd backend
cargo run

# Frontend
cd frontend
bun install && bun run dev
```

자세한 내용은 [로컬 개발 환경](self-hosting/local-development.md)을 참조하세요.

### Q: M1/M2 Mac에서 Docker 빌드가 느려요

**A:** Apple Silicon에서는 Rosetta 2를 통한 에뮬레이션으로 인해 느릴 수 있습니다.

해결 방법:
1. Docker Desktop 설정에서 "Use Rosetta for x86/amd64 emulation" 활성화
2. 네이티브 빌드 사용:

```yaml
# docker-compose.yml
services:
  backend:
    platform: linux/arm64  # 추가
```

### Q: 포트 8080과 5173이 이미 사용 중입니다

**A:** `docker-compose.yml`에서 포트를 변경하세요:

```yaml
services:
  backend:
    ports:
      - "3080:8080"  # 호스트:컨테이너

  frontend:
    ports:
      - "3000:5173"
```

또는 기존 프로세스 종료:

```bash
# 포트 사용 확인
lsof -i :8080
lsof -i :5173

# 프로세스 종료
kill -9 <PID>
```

### Q: Windows에서 실행이 안 됩니다

**A:** WSL2를 사용하는 것을 권장합니다:

1. WSL2 설치
2. Ubuntu 설치
3. Docker Desktop for Windows 설치 및 WSL2 통합 활성화
4. WSL2 Ubuntu에서 프로젝트 클론 및 실행

## 사용법

### Q: 관리자 비밀번호를 잊어버렸어요

**A:** 환경 변수를 변경하고 재시작하세요:

```bash
# Docker
docker compose down
# docker-compose.yml에서 ADMIN_PW 변경
docker compose up

# 로컬
# backend/.env에서 ADMIN_PW 변경
cargo run
```

### Q: 참가자 이름이 중복되면 어떻게 하나요?

**A:** 같은 Codelab 내에서 이름은 중복될 수 없습니다. 참가자에게 고유한 이름을 사용하도록 안내하세요:

- "홍길동_1", "홍길동_2"
- "홍길동", "길동홍"
- 이메일 사용: "hong@example.com"

### Q: Step을 삭제할 수 있나요?

**A:** 현재는 직접 Step 삭제 기능이 없습니다. Step 목록을 다시 작성하여 저장하면 기존 Step이 대체됩니다.

또는 데이터베이스에서 직접 삭제:

```bash
sqlite3 backend/data/sqlite.db
sqlite> DELETE FROM steps WHERE id = 'step_id';
```

### Q: Markdown에서 이미지가 표시되지 않아요

**A:** 이미지 경로를 확인하세요:

1. 관리자 페이지에서 이미지 업로드
2. 자동 생성된 URL 복사
3. Markdown에 삽입:

```markdown
![설명](http://localhost:8080/assets/images/xxx.png)
```

외부 이미지 URL도 사용 가능합니다:

```markdown
![설명](https://example.com/image.png)
```

### Q: 채팅 메시지가 사라집니다

**A:** WebSocket 연결이 끊어지면 실시간 메시지가 손실될 수 있습니다. 하지만 모든 메시지는 데이터베이스에 저장됩니다.

페이지를 새로고침하면 기존 채팅 기록을 불러옵니다.

## 공개 배포

### Q: ngrok 무료 플랜으로 충분한가요?

**A:** 소규모 워크샵(~40명)이면 충분합니다.

ngrok 무료 플랜:
- 연결 수: 40/분
- 대역폭: 무제한
- 터널 수: 1개

더 많은 참가자가 있다면:
- ngrok Pro 플랜 사용
- Cloudflare Tunnel 사용
- 자체 서버에 배포

### Q: ngrok URL이 매번 바뀝니다

**A:** 무료 플랜에서는 세션마다 새로운 URL이 생성됩니다.

고정 URL이 필요하면:
1. ngrok 유료 플랜 (Reserved Domain)
2. Cloudflare Tunnel
3. 자체 도메인에 배포

### Q: 참가자가 "ngrok 경고 페이지"를 봅니다

**A:** ngrok 무료 플랜은 처음 접속 시 경고 페이지를 표시합니다.

"Visit Site" 버튼을 클릭하면 접속됩니다.

경고 페이지를 제거하려면:
- ngrok 유료 플랜 사용
- 다른 터널링 서비스 (Cloudflare Tunnel)

## 성능 및 확장

### Q: 최대 몇 명까지 지원하나요?

**A:** 테스트 결과:

- **동시 사용자**: 100명 (안정적)
- **최대 테스트**: 200명 (CPU 사용률 증가)

제약 요소:
- SQLite 동시 쓰기
- 단일 서버 WebSocket

더 많은 사용자:
- PostgreSQL로 마이그레이션
- Redis로 WebSocket 메시지 브로커
- 로드 밸런서 추가

### Q: 메모리 사용량이 높습니다

**A:** 확인 사항:

```bash
# 컨테이너 리소스 확인
docker stats

# 로그 크기 확인
docker compose logs backend | wc -l
```

최적화:
1. 로그 레벨 낮추기: `RUST_LOG=info`
2. 리소스 제한 설정:

```yaml
services:
  backend:
    deploy:
      resources:
        limits:
          memory: 512M
```

### Q: 데이터베이스 파일이 계속 커집니다

**A:** 정기적으로 VACUUM 실행:

```bash
sqlite3 backend/data/sqlite.db "VACUUM;"
```

또는 오래된 데이터 삭제:

```sql
-- 30일 이상 된 Codelab 삭제
DELETE FROM codelabs WHERE created_at < datetime('now', '-30 days');
```

## 문제 해결

### Q: "Database is locked" 에러가 발생합니다

**A:** SQLite는 동시 쓰기를 지원하지 않습니다.

해결 방법:
1. `?mode=rwc` 확인: `DATABASE_URL=sqlite:data/sqlite.db?mode=rwc`
2. 연결 풀 크기 줄이기:

```rust
let pool = SqlitePoolOptions::new()
    .max_connections(1)  // 기본값 5
    .connect(&database_url)
    .await?;
```

3. PostgreSQL로 마이그레이션 고려

### Q: WebSocket 연결이 자꾸 끊어집니다

**A:** 원인:
- 방화벽/프록시
- 네트워크 불안정
- 서버 재시작

디버깅:

```javascript
// Frontend에서 재연결 로직 확인
const ws = new WebSocket(wsUrl);

ws.onclose = (event) => {
    console.log('WebSocket closed:', event.code, event.reason);

    // 재연결
    setTimeout(() => reconnect(), 3000);
};
```

### Q: 이미지 업로드가 실패합니다

**A:** 확인 사항:

1. 파일 크기: 최대 10MB (기본값)
2. 파일 형식: PNG, JPG, GIF, WebP
3. 디렉토리 권한:

```bash
# Docker
docker compose exec backend ls -la /app/static/assets/images

# 권한 문제시
docker compose exec backend chmod 755 /app/static/assets
```

### Q: Frontend에서 API 호출이 실패합니다

**A:** CORS 문제일 수 있습니다.

Backend 로그 확인:

```bash
docker compose logs backend | grep CORS
```

`main.rs`에서 CORS 설정 확인:

```rust
.layer(tower_http::cors::CorsLayer::permissive())
```

## 기타

### Q: 다국어 지원이 되나요?

**A:** 현재는 한국어만 지원합니다. 하지만 i18n 구조는 준비되어 있습니다.

기여 방법:
1. `frontend/src/lib/i18n/` 에 언어 파일 추가
2. Pull Request 제출

### Q: 모바일에서 사용할 수 있나요?

**A:** 네, 반응형 디자인으로 모바일과 태블릿을 지원합니다.

테스트된 환경:
- iOS Safari
- Android Chrome
- iPad

### Q: Codelab을 PDF로 내보낼 수 있나요?

**A:** 현재는 직접 지원하지 않지만:

1. Export로 ZIP 다운로드
2. Markdown을 Pandoc으로 변환:

```bash
pandoc step_1.md -o step_1.pdf
```

또는 브라우저 인쇄 기능 사용 (Ctrl/Cmd+P)

### Q: 프로젝트에 기여하고 싶어요

**A:** 환영합니다! [기여 가이드](contributing/guide.md)를 참조하세요.

간단한 기여 방법:
- 버그 리포트: [GitHub Issues](https://github.com/JAICHANGPARK/open-codelabs/issues)
- 기능 제안: [Discussions](https://github.com/JAICHANGPARK/open-codelabs/discussions)
- Pull Request: [기여 워크플로우](contributing/workflow.md)

### Q: 상업적으로 사용할 수 있나요?

**A:** 네, MIT 라이선스로 자유롭게 사용, 수정, 배포할 수 있습니다.

단, 다음을 포함해야 합니다:
- 원본 저작권 고지
- MIT 라이선스 텍스트

자세한 내용은 [라이선스](license.md)를 참조하세요.

## 추가 도움

위에서 답을 찾지 못하셨나요?

- [GitHub Issues](https://github.com/JAICHANGPARK/open-codelabs/issues) - 버그 리포트
- [GitHub Discussions](https://github.com/JAICHANGPARK/open-codelabs/discussions) - 질문 및 토론
- [문서](index.md) - 전체 문서 탐색
