FROM rust:1.84-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# 依存関係のみを先行してコンパイルする
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release

COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/backend ./backend

CMD ["./backend"]