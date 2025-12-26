# GitHub Pages 배포 가이드

이 문서는 MkDocs 문서를 GitHub Pages에 자동 배포하는 방법을 설명합니다.

## 🚀 자동 배포 설정

### 1. GitHub Repository 설정

#### Step 1: GitHub Pages 활성화

1. GitHub 저장소로 이동
2. **Settings** 탭 클릭
3. 왼쪽 메뉴에서 **Pages** 클릭
4. **Source** 섹션에서:
   - Source: **GitHub Actions** 선택

![GitHub Pages Settings](https://docs.github.com/assets/cb-47267/mw-1440/images/help/pages/github-actions-source.webp)

#### Step 2: 워크플로우 파일 확인

이미 다음 파일들이 생성되어 있습니다:

```
.github/workflows/
├── docs.yml              # 자동 배포 워크플로우
└── docs-pr-preview.yml   # PR 빌드 체크
```

### 2. 저장소에 푸시

```bash
git add .
git commit -m "docs: setup MkDocs with GitHub Pages deployment"
git push origin main
```

### 3. 배포 확인

1. GitHub 저장소의 **Actions** 탭으로 이동
2. "Deploy MkDocs to GitHub Pages" 워크플로우 확인
3. 빌드가 완료되면 (약 1-2분):
   - ✅ 성공 표시 확인
   - **Settings > Pages**에서 URL 확인

배포된 URL:
```
https://<username>.github.io/<repository-name>/
```

예시:
```
https://yourusername.github.io/open-codelabs/
```

## 📋 워크플로우 설명

### docs.yml (자동 배포)

**트리거 조건**:
- `main` 브랜치에 푸시될 때
- `docs/`, `mkdocs.yml`, `requirements.txt` 파일이 변경될 때
- 수동 실행 (workflow_dispatch)

**동작**:
1. 저장소 체크아웃
2. Python 설치
3. 의존성 설치 (`requirements.txt`)
4. MkDocs 빌드 (`mkdocs build`)
5. GitHub Pages에 자동 배포

### docs-pr-preview.yml (PR 체크)

**트리거 조건**:
- Pull Request 생성 시
- 문서 관련 파일 변경 시

**동작**:
1. MkDocs 빌드 테스트
2. 빌드 성공 시 PR에 코멘트 추가
3. 빌드 실패 시 에러 표시

## 🔧 커스터마이징

### 커스텀 도메인 설정

1. `docs/` 디렉토리에 `CNAME` 파일 생성:

```bash
echo "docs.yourdomain.com" > docs/CNAME
```

2. `mkdocs.yml`에 site_url 추가:

```yaml
site_url: https://docs.yourdomain.com/
```

3. DNS 설정:
   - CNAME 레코드: `docs` → `<username>.github.io`

### 베이스 URL 변경

저장소 이름이 `open-codelabs`가 아니라면 `mkdocs.yml` 수정:

```yaml
site_url: https://<username>.github.io/<your-repo-name>/
```

## 🐛 문제 해결

### 배포가 실패합니다

**원인 1: GitHub Pages가 비활성화됨**

**해결**:
1. Settings > Pages
2. Source를 "GitHub Actions"로 설정

**원인 2: 권한 문제**

**해결**:
1. Settings > Actions > General
2. "Workflow permissions" 섹션
3. "Read and write permissions" 선택
4. "Allow GitHub Actions to create and approve pull requests" 체크

**원인 3: 빌드 에러**

**해결**:
```bash
# 로컬에서 빌드 테스트
mkdocs build --strict

# 에러 확인 후 수정
```

### 페이지가 404 에러를 표시합니다

**원인**: 베이스 URL이 잘못됨

**해결**:

`mkdocs.yml`에서 `site_url` 확인:

```yaml
# 올바른 형식
site_url: https://yourusername.github.io/open-codelabs/

# 또는 커스텀 도메인
site_url: https://docs.yourdomain.com/
```

### CSS/JS가 로드되지 않습니다

**원인**: 상대 경로 문제

**해결**:

`mkdocs.yml`에 추가:

```yaml
use_directory_urls: true
```

### 이미지가 표시되지 않습니다

**원인**: 이미지 경로 문제

**해결**:

문서에서 이미지는 다음과 같이 참조:

```markdown
![Image](../assets/image.png)

# 또는 절대 경로
![Image](/assets/image.png)
```

이미지는 `docs/assets/` 에 저장하세요.

## 📊 워크플로우 상태 배지

README.md에 배지 추가:

```markdown
[![Documentation](https://github.com/<username>/<repo>/actions/workflows/docs.yml/badge.svg)](https://github.com/<username>/<repo>/actions/workflows/docs.yml)
```

## 🔄 수동 배포

자동 배포 외에 수동으로도 배포 가능:

### 방법 1: GitHub Actions에서 수동 실행

1. **Actions** 탭
2. "Deploy MkDocs to GitHub Pages" 선택
3. "Run workflow" 버튼
4. "Run workflow" 확인

### 방법 2: 로컬에서 배포

```bash
# mkdocs-ghpages 플러그인 사용
pip install mkdocs-git-revision-date-localized-plugin

# gh-pages 브랜치에 배포
mkdocs gh-deploy --force

# 메시지와 함께 배포
mkdocs gh-deploy -m "docs: update documentation"
```

!!! warning "주의"
    로컬 배포 시 GitHub Actions와 충돌할 수 있습니다.
    가능하면 GitHub Actions를 사용하세요.

## 📈 배포 히스토리 확인

1. **Actions** 탭
2. "Deploy MkDocs to GitHub Pages" 워크플로우
3. 각 실행 내역 확인
4. 로그 및 아티팩트 다운로드 가능

## 🎯 모범 사례

### 1. 브랜치 전략

```bash
# 개발은 feature 브랜치에서
git checkout -b docs/update-installation-guide

# 문서 수정
vim docs/getting-started/installation.md

# 커밋
git commit -m "docs: update installation guide for M1 Mac"

# PR 생성
git push origin docs/update-installation-guide
```

### 2. 커밋 메시지

```bash
# 좋은 예
docs: add architecture documentation
docs: fix typo in API reference
docs: update deployment guide with GitHub Actions

# 나쁜 예
Update docs
Fix
문서 수정
```

### 3. PR 리뷰

- 문서 변경 시 PR 생성
- 자동 빌드 체크 통과 확인
- 리뷰어가 내용 검토
- 머지 후 자동 배포

## 🔐 보안

### Secrets 사용 (필요 시)

GitHub Secrets에 민감한 정보 저장:

1. Settings > Secrets and variables > Actions
2. "New repository secret"
3. 워크플로우에서 사용:

```yaml
- name: Deploy
  env:
    API_KEY: ${{ secrets.API_KEY }}
  run: mkdocs gh-deploy
```

## 📚 추가 리소스

- [GitHub Pages 공식 문서](https://docs.github.com/en/pages)
- [GitHub Actions 문서](https://docs.github.com/en/actions)
- [MkDocs 배포 가이드](https://www.mkdocs.org/user-guide/deploying-your-docs/)
- [Material for MkDocs 설정](https://squidfunk.github.io/mkdocs-material/setup/)

## ✅ 체크리스트

배포 전 확인:

- [ ] GitHub Pages가 "GitHub Actions"로 설정됨
- [ ] `mkdocs.yml`의 `site_url`이 올바름
- [ ] `requirements.txt`에 모든 의존성 포함
- [ ] 로컬에서 `mkdocs build --strict` 성공
- [ ] `.github/workflows/docs.yml` 파일 존재
- [ ] 변경사항을 `main` 브랜치에 푸시

배포 후 확인:

- [ ] Actions 탭에서 워크플로우 성공 확인
- [ ] GitHub Pages URL 접속 가능
- [ ] 모든 페이지가 정상 표시됨
- [ ] 네비게이션이 올바르게 동작
- [ ] 검색 기능 작동
- [ ] 모바일에서도 정상 표시

## 🎉 완료!

이제 문서가 자동으로 배포됩니다:

1. 문서 수정 후 `main` 브랜치에 푸시
2. GitHub Actions가 자동으로 빌드 및 배포
3. 1-2분 후 변경사항이 반영됨

문서 URL: `https://<username>.github.io/<repository>/`
