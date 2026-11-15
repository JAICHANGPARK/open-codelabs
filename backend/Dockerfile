FROM rust:1-slim-bookworm AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake build-essential && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
# Create dummy lib/bin targets so Cargo can resolve every declared target while
# priming the dependency cache before the real source tree is copied in.
RUN mkdir -p src/bin && \
    touch src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/local_bench.rs && \
    echo "fn main() {}" > src/bin/oc.rs && \
    echo "fn main() {}" > src/bin/ops_bench.rs && \
    echo "fn main() {}" > src/bin/ws_bench.rs
RUN cargo build --release
RUN rm -rf src target/release/deps/libbackend* target/release/deps/backend* target/release/deps/oc* target/release/deps/local_bench* target/release/deps/ops_bench* target/release/deps/ws_bench*

COPY src ./src
COPY migrations ./migrations
COPY migrations-postgres ./migrations-postgres
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates git tar && rm -rf /var/lib/apt/lists/*
WORKDIR /app
RUN mkdir -p /app/data /app/static/uploads /app/workspaces && chmod -R 777 /app/static /app/data /app/workspaces
COPY --from=builder /app/target/release/backend /usr/local/bin/backend

EXPOSE 8080
CMD ["backend"]
