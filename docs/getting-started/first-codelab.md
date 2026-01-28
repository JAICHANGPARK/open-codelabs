# 첫 번째 Codelab 만들기

이 가이드에서는 Open Codelabs의 모든 기능을 활용하여 완성도 높은 Codelab을 만드는 방법을 배웁니다.

## Codelab이란?

Codelab은 단계별 학습 가이드입니다. 각 Codelab은:

- **여러 개의 Step**으로 구성
- **Markdown** 기반 콘텐츠
- **코드 예제**와 이미지 포함 가능
- 참가자가 **자신의 속도**로 진행

## 1단계: Codelab 생성

### 관리자 대시보드 접속

1. [http://localhost:5173/login](http://localhost:5173/login)에서 로그인
2. 관리자 대시보드로 이동

### 새 Codelab 만들기

"새 Codelab 만들기" 버튼을 클릭하고 정보를 입력:

```
제목: Rust로 REST API 만들기
설명: Axum 프레임워크를 사용하여 RESTful API 서버를 만들어봅시다
작성자: 홍길동
```

!!! tip "좋은 제목 작성하기"
    - 명확하고 구체적으로
    - 학습 결과가 드러나도록
    - 20-50자 정도로 간결하게

## 2단계: Step 구성하기

### Step 구조 설계

좋은 Codelab은 논리적으로 구성된 Step들로 이루어져 있습니다:

1. **개요** - 무엇을 배울지 소개
2. **환경 설정** - 필요한 도구 설치
3. **핵심 개념** - 이론 설명
4. **실습** - 단계별 코딩
5. **테스트** - 작동 확인
6. **마무리** - 요약 및 다음 단계

### Step 1: 개요

```markdown
# Rust REST API 개요

## 무엇을 만들까요?

이 Codelab에서는 Axum 프레임워크를 사용하여 간단한 TODO REST API를 만듭니다.

## 학습 내용

- Axum 프레임워크 기초
- RESTful API 설계 원칙
- SQLite 데이터베이스 연동
- 에러 핸들링

## 준비물

- Rust 1.75 이상
- 코드 에디터 (VS Code 권장)
- 터미널
- Postman 또는 curl

## 예상 소요 시간

약 45분
```

### Step 2: 환경 설정

```markdown
# 환경 설정

## 프로젝트 생성

새로운 Rust 프로젝트를 만듭니다:

\`\`\`bash
cargo new todo-api
cd todo-api
\`\`\`

## 의존성 추가

`Cargo.toml`을 열고 다음 의존성을 추가합니다:

\`\`\`toml
[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite"] }
\`\`\`

## 프로젝트 구조

다음과 같은 구조를 만듭니다:

\`\`\`
todo-api/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── models.rs
│   └── handlers.rs
└── .env
\`\`\`

!!! info "축하합니다!"
    환경 설정이 완료되었습니다. 다음 단계로 넘어가세요.
```

### Step 3: 데이터 모델 작성

```markdown
# 데이터 모델 작성

## Todo 구조체 정의

`src/models.rs` 파일을 생성하고 다음 코드를 작성합니다:

\`\`\`rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
}
\`\`\`

## 코드 설명

### Todo 구조체

- `id`: 할 일의 고유 식별자
- `title`: 할 일 제목
- `completed`: 완료 여부

### Serde 속성

- `#[derive(Serialize, Deserialize)]`: JSON 직렬화/역직렬화 지원
- `#[derive(Clone)]`: 복사 가능하게 만듦

!!! warning "주의"
    `CreateTodo`에는 `id`와 `completed`가 없습니다.
    이는 서버에서 자동으로 생성됩니다.
```

## 3단계: Markdown 고급 기능

### 코드 하이라이팅

다양한 언어의 코드를 하이라이팅할 수 있습니다:

````markdown
\`\`\`rust
fn main() {
    println!("Hello, world!");
}
\`\`\`

\`\`\`javascript
console.log("Hello, world!");
\`\`\`

\`\`\`python
print("Hello, world!")
\`\`\`
````

### Admonitions (알림 박스)

중요한 정보를 강조할 수 있습니다:

```markdown
!!! note "참고"
    이것은 일반적인 정보입니다.

!!! tip "팁"
    유용한 조언을 제공합니다.

!!! warning "주의"
    주의가 필요한 내용입니다.

!!! danger "위험"
    심각한 경고입니다.

!!! success "성공"
    성공 메시지입니다.

!!! info "정보"
    추가 정보입니다.
```

### 표 만들기

```markdown
| 메서드 | 경로 | 설명 |
|--------|------|------|
| GET | /todos | 모든 할 일 조회 |
| GET | /todos/:id | 특정 할 일 조회 |
| POST | /todos | 새 할 일 생성 |
| PUT | /todos/:id | 할 일 수정 |
| DELETE | /todos/:id | 할 일 삭제 |
```

### 링크와 이미지

```markdown
<!-- 외부 링크 -->
[Rust 공식 문서](https://www.rust-lang.org/)

<!-- 이미지 -->
![Architecture Diagram](https://example.com/architecture.png)

<!-- 이미지 업로드 -->
관리자 페이지에서 이미지를 업로드하면 자동으로 URL이 생성됩니다.
```

### 리스트

```markdown
<!-- 순서 없는 리스트 -->
- 항목 1
- 항목 2
  - 하위 항목 2.1
  - 하위 항목 2.2
- 항목 3

<!-- 순서 있는 리스트 -->
1. 첫 번째 단계
2. 두 번째 단계
3. 세 번째 단계

<!-- 체크리스트 -->
- [ ] 해야 할 일
- [x] 완료된 일
- [ ] 남은 일
```

## 4단계: 이미지 추가하기

### 이미지 업로드

1. Step 편집 화면에서 "이미지 업로드" 버튼 클릭
2. 이미지 파일 선택 (PNG, JPG, GIF)
3. 업로드 완료 후 자동으로 생성된 URL 복사
4. Markdown에 삽입:

```markdown
![설명 텍스트](업로드된_이미지_URL)
```

### 이미지 크기 조절

HTML을 사용하여 이미지 크기를 조절할 수 있습니다:

```html
<img src="이미지_URL" alt="설명" width="400">
<img src="이미지_URL" alt="설명" style="width: 50%;">
```

## 5단계: 참가자 관리

### 참가 코드 생성

Codelab을 만들 때 자동으로 참가 코드가 생성됩니다. 참가자는 이 코드를 입력하여 등록합니다.

### 참가자 모니터링

관리자 대시보드에서:

- 실시간으로 참가자 목록 확인
- 각 참가자의 진행 상황 추적
- 도움 요청 관리

### 도움 요청 처리

참가자가 도움을 요청하면:

1. 알림이 표시됩니다
2. 어떤 Step에서 막혔는지 확인
3. 1:1 DM으로 도움 제공
4. 문제 해결 후 "해결됨" 표시

## 6단계: 실시간 채팅 활용

### 전체 채팅

모든 참가자와 소통:

```
[Facilitator]: 5분 후 다음 섹션으로 넘어갑니다!
```

### 1:1 DM

특정 참가자와 개인 대화:

```
참가자 이름 옆의 "DM" 버튼 클릭
→ 개인 메시지 전송
```

## 7단계: Export & Import

### Codelab 내보내기

1. 관리자 대시보드에서 Codelab 선택
2. "Export" 버튼 클릭
3. ZIP 파일 다운로드

ZIP 파일 구조:

```
codelab_xxx.zip
├── codelab.json        # 메타데이터
├── step_1.md           # Step 1 내용
├── step_2.md           # Step 2 내용
└── step_3.md           # Step 3 내용
```

### Codelab 가져오기

1. "Import" 버튼 클릭
2. ZIP 파일 선택
3. 자동으로 Codelab과 모든 Step이 생성됨

!!! tip "버전 관리"
    Codelab을 Export하여 Git으로 버전 관리할 수 있습니다!

## 8단계: 피드백 수집

참가자가 Codelab을 완료하면 자동으로 피드백 양식이 표시됩니다:

- **난이도**: 1-5점
- **만족도**: 1-5점
- **의견**: 자유 형식

관리자는 대시보드에서 모든 피드백을 확인할 수 있습니다.

## 베스트 프랙티스

### 📝 콘텐츠 작성

1. **한 Step에 한 가지 개념**만 다루기
2. **코드는 작게, 설명은 자세히**
3. **실습 위주**로 구성
4. **검증 방법** 제공하기

### 🎯 Step 길이

- 이상적인 Step 길이: **5-10분**
- 너무 긴 Step은 분리하기
- 각 Step의 끝에 **요약** 제공

### 🖼️ 시각 자료

- 복잡한 개념은 **다이어그램**으로 설명
- 실행 결과는 **스크린샷**으로 보여주기
- 아키텍처는 **도식**으로 표현

### ⚠️ 에러 처리

- 흔히 발생하는 **에러 사례** 포함
- **해결 방법** 명확히 제시
- **문제 해결 팁** 박스 추가

## 예제 템플릿

완전한 Step 템플릿:

```markdown
# [Step 제목]

## 목표

이 Step에서는 [학습 목표]를 배웁니다.

## 이론

[개념 설명]

## 실습

### 1. [하위 단계 제목]

[설명]

\`\`\`[언어]
[코드]
\`\`\`

### 2. [하위 단계 제목]

[설명]

\`\`\`[언어]
[코드]
\`\`\`

## 실행하기

\`\`\`bash
[실행 명령]
\`\`\`

예상 출력:

\`\`\`
[출력 결과]
\`\`\`

## 검증

다음을 확인하세요:

- [ ] [체크 항목 1]
- [ ] [체크 항목 2]
- [ ] [체크 항목 3]

!!! success "성공!"
    잘 하셨습니다! 다음 Step으로 넘어가세요.

## 문제 해결

### 에러: [에러 메시지]

**원인**: [에러 원인]

**해결**: [해결 방법]

## 다음 단계

다음 Step에서는 [다음 내용 미리보기]를 다룹니다.
```

## 다음 단계

첫 번째 Codelab 작성 방법을 배웠습니다! 이제:

- [공개 배포하기](../self-hosting/public-deployment.md) - ngrok/bore/cloudflare로 외부 공개
- [API 활용하기](../specification/api-reference.md) - 자동화 및 통합
- [아키텍처 이해하기](../architecture/system-architecture.md) - 시스템 구조 파악

## 도움이 필요하신가요?

- [FAQ](../faq.md) - 자주 묻는 질문
- [GitHub Issues](https://github.com/JAICHANGPARK/open-codelabs/issues) - 버그 리포트
- [Contributing](../contributing/guide.md) - 기여하기
