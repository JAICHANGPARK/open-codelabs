# 벤치마크 가이드

이 문서는 일반 사용자용이 아니라 Open Codelabs 메인테이너와 성능 검증 작업자를 위한 참고 문서입니다.

표준 설치 흐름은 여전히 아래 한 줄이면 충분합니다.

```bash
cargo install --path backend --bin oc
```

일반 사용자 설치와 기본 GitHub Release 아카이브는 `oc` 하나만 중심으로 제공합니다. `oc bench`는 메인테이너용 고급 명령이라, 보통은 소스 체크아웃 안에서 사용하는 것을 전제로 합니다.

## 권장 사용 방식

```bash
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs/backend

cargo run --bin oc -- bench local -- --help
cargo run --bin oc -- bench ops -- --help
cargo run --bin oc -- bench ws -- --help
```

`oc bench`는 아래 순서로 benchmark runner를 찾습니다.

1. `oc`와 같은 디렉터리에 있는 companion binary
2. 현재 디렉터리가 Open Codelabs 소스 체크아웃이면 `cargo run --release --bin ...`

표준 설치에서는 보통 2번 경로가 사용됩니다.

## 지원 대상

| 명령 | 의미 |
| --- | --- |
| `oc bench local -- <options...>` | attendee/help/submission 중심 API benchmark |
| `oc bench ops -- <options...>` | upload/backup/workspace 중심 운영 benchmark |
| `oc bench ws -- <options...>` | WebSocket benchmark |

옵션은 `--` 뒤에 그대로 넘깁니다.

예:

```bash
oc bench local -- --attendees 50 --read-requests 400
oc bench ops -- --profile paper --output bench-results/ops.json
oc bench ws -- --users 50,100,200 --duration-secs 60
```

## 직접 실행 가능한 내부 바이너리

필요하면 기존 benchmark 바이너리를 직접 실행해도 됩니다.

```bash
cargo run --release --bin local_bench -- --help
cargo run --release --bin ops_bench -- --help
cargo run --release --bin ws_bench -- --help
```

## 관련 문서

- 로컬 성능 벤치 참고: [BENCHMARK_LOCAL.md](/Users/jaichang/Documents/GitHub/open-codelabs/backend/BENCHMARK_LOCAL.md)
- 매트릭스 실행 스크립트: [run_matrix.sh](/Users/jaichang/Documents/GitHub/open-codelabs/backend/bench/run_matrix.sh)
- 전체 로컬 벤치 스크립트: [run_all_local.sh](/Users/jaichang/Documents/GitHub/open-codelabs/backend/bench/run_all_local.sh)
