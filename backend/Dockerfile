FROM rust:1-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.toml
COPY backend/Cargo.toml backend/Cargo.toml
COPY backend/src backend/src
COPY backend/migrations backend/migrations

RUN cargo build --release -p signal-backend

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/signal-backend /usr/local/bin/signal-backend
COPY backend/migrations /app/migrations

ENV SIGNAL_BACKEND_HOST=0.0.0.0
ENV SIGNAL_BACKEND_PORT=8080

EXPOSE 8080

CMD ["signal-backend"]
