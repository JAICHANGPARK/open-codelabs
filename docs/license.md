# 라이선스

Open Codelabs는 Apache License 2.0 하에 배포됩니다.

## Apache License 2.0

```
Copyright 2025 JAICHANGPARK

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

## 요약

Apache License 2.0은 **관대한(permissive) 오픈소스 라이선스**로, 다음을 허용합니다:

### ✅ 허용되는 것

- **상업적 사용**: 상업적 목적으로 사용 가능
- **수정**: 소스 코드 수정 가능
- **배포**: 원본 또는 수정본 배포 가능
- **특허 사용**: 기여자의 특허 사용 가능
- **비공개 사용**: 공개 의무 없음

### 📋 조건

- **라이선스 및 저작권 고지**: 원본 라이선스와 저작권 고지 포함 필요
- **상태 변경 표시**: 파일 수정 시 명시 필요
- **동일한 라이선스 사용**: Apache 2.0 라이선스 유지 (원본 코드 부분)

### 🚫 제한

- **책임 제한**: 소프트웨어 사용으로 인한 책임 없음
- **보증 없음**: "있는 그대로" 제공
- **상표 사용 불가**: 프로젝트 이름/로고 사용 제한

## 상업적 사용

Open Codelabs를 상업적으로 사용하실 수 있습니다:

### 가능한 사용 사례

1. **워크샵/교육 서비스**
   - 유료 워크샵에서 사용
   - 기업 교육 프로그램에 통합
   - 온라인 강의 플랫폼에 활용

2. **SaaS 제공**
   - 클라우드 서비스로 제공
   - 구독 모델로 운영
   - 엔터프라이즈 버전 판매

3. **제품 통합**
   - 자사 제품에 통합
   - 커스터마이징하여 재판매
   - OEM 제품으로 사용

### 준수 사항

상업적 사용 시에도 다음을 준수해야 합니다:

1. **라이선스 고지**
   ```
   This product includes Open Codelabs software
   developed by JAICHANGPARK

   Copyright 2025 JAICHANGPARK

   Licensed under the Apache License, Version 2.0
   ```

2. **NOTICE 파일** (있는 경우)
   - 원본 NOTICE 파일 내용 포함
   - 제품 문서나 About 페이지에 명시

3. **수정 사항 표시**
   - 수정한 파일에 변경 내역 명시
   - 예: `// Modified by Company X on 2024-01-01`

## MIT License와의 차이

README에서는 MIT License를 언급했지만, 실제 LICENSE 파일은 Apache 2.0입니다.

### Apache 2.0의 장점

| 항목 | MIT | Apache 2.0 |
|------|-----|------------|
| 사용 허가 | ✅ | ✅ |
| 수정 허가 | ✅ | ✅ |
| 상업적 사용 | ✅ | ✅ |
| **특허 보호** | ❌ | ✅ |
| **변경 사항 명시** | ❌ | ✅ (명시적) |
| 간결함 | 매우 간단 | 상세함 |

**Apache 2.0이 더 나은 이유**:
- 특허 관련 명시적 허가
- 기여자 보호 강화
- 법적 명확성

## 의존성 라이선스

Open Codelabs가 사용하는 주요 라이브러리들의 라이선스:

### Backend (Rust)

| 라이브러리 | 라이선스 | 설명 |
|-----------|---------|------|
| Axum | MIT | 웹 프레임워크 |
| Tokio | MIT | 비동기 런타임 |
| SQLx | Apache-2.0 / MIT | 데이터베이스 |
| Serde | Apache-2.0 / MIT | 직렬화 |
| Tower | MIT | 미들웨어 |

### Frontend (TypeScript)

| 라이브러리 | 라이선스 | 설명 |
|-----------|---------|------|
| Svelte | MIT | UI 프레임워크 |
| SvelteKit | MIT | 풀스택 프레임워크 |
| Vite | MIT | 빌드 도구 |
| Tailwind CSS | MIT | CSS 프레임워크 |
| marked | MIT | Markdown 파서 |
| DOMPurify | Apache-2.0 / MPL-2.0 | XSS 방지 |

모든 의존성은 관대한 라이선스를 사용하므로 **상업적 사용에 문제 없습니다**.

## 기여 시 라이선스

프로젝트에 기여할 때:

### 기여 계약

코드를 기여하면:
- 자동으로 Apache License 2.0 하에 라이선스됨
- 기여자는 저작권을 유지함
- 프로젝트는 기여 코드를 Apache 2.0으로 사용 가능

### 다른 코드 포함

다른 프로젝트의 코드를 포함할 때:

1. **호환 라이선스 확인**
   - MIT, BSD, Apache 2.0: ✅ 가능
   - GPL, AGPL: ❌ 불가능 (copyleft)

2. **출처 명시**
   ```rust
   // This function is based on code from Project X
   // Copyright 2023 Original Author
   // Licensed under MIT License
   ```

3. **라이선스 파일 추가**
   - `licenses/PROJECT_NAME.txt`

## 자주 묻는 질문

### Q: 소스 코드를 공개해야 하나요?

**A:** 아니요. Apache 2.0은 소스 코드 공개를 요구하지 않습니다. 수정한 버전을 비공개로 유지할 수 있습니다.

### Q: 프로젝트 이름을 변경할 수 있나요?

**A:** 네, 수정된 버전에 새로운 이름을 사용할 수 있습니다. 단, 원본이 "Open Codelabs"임을 명시해야 합니다.

### Q: 유료로 판매할 수 있나요?

**A:** 네, 완전히 가능합니다. Apache 2.0은 상업적 사용을 명시적으로 허용합니다.

### Q: 특허 소송에서 보호받나요?

**A:** Apache 2.0은 기여자의 특허에 대한 명시적 허가를 포함합니다. 하지만 소송을 제기하면 특허 라이선스가 종료됩니다.

### Q: 라이선스 고지는 어디에 해야 하나요?

**A:** 다음 중 하나:
- 제품 문서
- README 파일
- About/Legal 페이지
- 소프트웨어 내 표시

## 전체 라이선스 텍스트

전체 Apache License 2.0 텍스트는 프로젝트 루트의 [LICENSE](https://github.com/JAICHANGPARK/open-codelabs/blob/main/LICENSE) 파일에서 확인할 수 있습니다.

## 추가 정보

- [Apache License 2.0 공식 텍스트](https://www.apache.org/licenses/LICENSE-2.0)
- [Apache License FAQ](https://www.apache.org/foundation/license-faq.html)
- [Choose a License](https://choosealicense.com/licenses/apache-2.0/)

## 문의

라이선스 관련 질문이 있으시면:

- [GitHub Issues](https://github.com/JAICHANGPARK/open-codelabs/issues)
- [이메일](mailto:team@example.com)
