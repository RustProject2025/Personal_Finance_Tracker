FROM rust:slim AS builder

WORKDIR /app

RUN cargo install sqlx-cli --no-default-features --features postgres

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN mkdir -p migrations
COPY migrations/20251102021659_init.sql ./migrations/

RUN cargo build --release

FROM debian:testing-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/migrations ./migrations

ENV DATABASE_URL=postgresql://postgres:postgres@db:5432/finance_tracker
ENV RUST_LOG=info

EXPOSE 3000

CMD ["./backend"]

