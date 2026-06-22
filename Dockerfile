FROM rust:latest AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations
COPY templates ./templates
COPY static ./static

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/tracking-app /app/tracking-app

COPY templates /app/templates
COPY static /app/static
COPY migrations /app/migrations

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
ENV DATABASE_URL=sqlite:/data/tracking.db

VOLUME /data

EXPOSE 8000

CMD ["/app/tracking-app"]