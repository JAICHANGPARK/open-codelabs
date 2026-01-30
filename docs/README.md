# Open Codelabs 문서

이 디렉토리는 MkDocs로 작성된 Open Codelabs 프로젝트 문서입니다.

## MkDocs 설치 및 실행

### 1. Python 및 MkDocs 설치

```bash
# Python 3.8+ 필요
python --version

# pip로 MkDocs와 Material 테마 설치
pip install mkdocs mkdocs-material

# 추가 플러그인 설치
pip install mkdocs-git-revision-date-localized-plugin
```

또는 requirements.txt 사용:

```bash
pip install -r requirements.txt
```

### 2. 로컬에서 문서 미리보기

```bash
# 프로젝트 루트에서
mkdocs serve

# 브라우저에서 접속
# http://localhost:8000
```

자동으로 변경사항을 감지하고 새로고침됩니다.

### 3. 문서 빌드

```bash
# 정적 HTML로 빌드
mkdocs build

# 결과는 site/ 디렉토리에 생성됨
```

### 4. GitHub Pages에 배포

```bash
# gh-pages 브랜치에 자동 배포
mkdocs gh-deploy
```

## 문서 구조

```
docs/
├── index.md                    # 홈페이지
├── getting-started/            # 시작하기
│   ├── quickstart.md          # 빠른 시작
│   ├── installation.md        # 설치 가이드
│   └── first-codelab.md       # 첫 Codelab 만들기
├── self-hosting/               # Self-Hosting
│   ├── docker.md              # Docker 배포
│   ├── local-development.md   # 로컬 개발
│   ├── public-deployment.md   # 공개 배포
│   ├── supabase.md            # Supabase
│   └── environment.md         # 환경 변수
├── specification/              # 프로젝트 명세
│   ├── overview.md            # 개요
│   ├── features.md            # 기능 명세
│   ├── database-schema.md     # DB 스키마
│   └── api-reference.md       # API 레퍼런스
├── architecture/               # 아키텍처
│   ├── system-architecture.md # 시스템 아키텍처
│   ├── backend.md             # Backend 구조
│   ├── frontend.md            # Frontend 구조
│   └── websocket.md           # WebSocket
├── code-guide/                 # 코드 가이드
│   ├── backend-examples.md    # Backend 예제
│   ├── frontend-examples.md   # Frontend 예제
│   ├── data-models.md         # 데이터 모델
│   └── api-usage.md           # API 사용법
├── contributing/               # 기여하기
│   ├── guide.md               # 기여 가이드
│   ├── workflow.md            # 개발 워크플로우
│   └── code-style.md          # 코드 스타일
├── faq.md                      # FAQ
└── license.md                  # 라이선스
```

## 문서 작성 가이드

### Markdown 기본

```markdown
# 제목 1
## 제목 2
### 제목 3

**굵게**
*기울임*

- 리스트
- 항목

1. 순서
2. 리스트

[링크](url)
![이미지](url)
```

### 코드 블록

````markdown
```python
def hello():
    print("Hello, World!")
```
````

### Admonitions

```markdown
!!! note "참고"
    이것은 참고 사항입니다.

!!! tip "팁"
    유용한 팁입니다.

!!! warning "주의"
    주의가 필요합니다.

!!! danger "위험"
    위험한 작업입니다.
```

### 탭

```markdown
=== "Tab 1"
    Tab 1 내용

=== "Tab 2"
    Tab 2 내용
```

### 표

```markdown
| 컬럼 1 | 컬럼 2 |
|--------|--------|
| 값 1   | 값 2   |
```

## 설정

### mkdocs.yml

프로젝트 루트의 `mkdocs.yml` 파일에서 설정:

```yaml
site_name: 프로젝트 이름
theme:
  name: material
  palette:
    primary: indigo
nav:
  - 홈: index.md
  - 시작하기:
      - 빠른 시작: getting-started/quickstart.md
```

## 로컬 개발

### Live Reload

```bash
mkdocs serve --dev-addr 0.0.0.0:8000
```

### 빌드 확인

```bash
# 빌드만
mkdocs build --strict

# --strict: 경고를 에러로 처리
```

## 문서 기여

1. 새 파일 추가 시 `mkdocs.yml`의 `nav`에 추가
2. 마크다운 파일 작성
3. `mkdocs serve`로 미리보기
4. Pull Request 생성

## 문제 해결

### MkDocs를 찾을 수 없음

```bash
# PATH 확인
which mkdocs

# 재설치
pip install --upgrade mkdocs mkdocs-material
```

### 한글 인코딩 문제

파일을 UTF-8로 저장하세요.

### 플러그인 오류

```bash
# 모든 의존성 재설치
pip install --upgrade -r requirements.txt
```

## 참고 자료

- [MkDocs 공식 문서](https://www.mkdocs.org/)
- [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/)
- [Markdown Guide](https://www.markdownguide.org/)
