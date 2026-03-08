# Benchmarking Guide

This page is for Open Codelabs maintainers and performance work, not for normal end users.

The normal install flow is still just:

```bash
cargo install --path backend --bin oc
```

Standard installs and the default GitHub Release archives are centered on the single `oc` binary. `oc bench` is an advanced maintainer command, so the common workflow is to run it from a source checkout.

## Recommended workflow

```bash
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs/backend

cargo run --bin oc -- bench local -- --help
cargo run --bin oc -- bench ops -- --help
cargo run --bin oc -- bench ws -- --help
```

`oc bench` resolves its runner in this order:

1. companion benchmark binaries located next to `oc`
2. `cargo run --release --bin ...` when the current directory is an Open Codelabs source checkout

In the standard maintainer workflow, the second path is usually what you use.

## Supported targets

| Command | Meaning |
| --- | --- |
| `oc bench local -- <options...>` | attendee/help/submission-heavy API benchmark |
| `oc bench ops -- <options...>` | upload/backup/workspace operations benchmark |
| `oc bench ws -- <options...>` | WebSocket benchmark |

Everything after `--` is forwarded unchanged.

Examples:

```bash
oc bench local -- --attendees 50 --read-requests 400
oc bench ops -- --profile paper --output bench-results/ops.json
oc bench ws -- --users 50,100,200 --duration-secs 60
```

## Direct benchmark binaries

If needed, you can still run the original binaries directly.

```bash
cargo run --release --bin local_bench -- --help
cargo run --release --bin ops_bench -- --help
cargo run --release --bin ws_bench -- --help
```

## Related references

- Local benchmark notes: [BENCHMARK_LOCAL.md](/Users/jaichang/Documents/GitHub/open-codelabs/backend/BENCHMARK_LOCAL.md)
- Matrix script: [run_matrix.sh](/Users/jaichang/Documents/GitHub/open-codelabs/backend/bench/run_matrix.sh)
- Full local benchmark script: [run_all_local.sh](/Users/jaichang/Documents/GitHub/open-codelabs/backend/bench/run_all_local.sh)
