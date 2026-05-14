# -----------------------------------------------------------------------------
# Builder — compile release binary (needs librdkafka headers for `rdkafka`)
# -----------------------------------------------------------------------------
FROM rust:1-bookworm AS builder

ENV DEBIAN_FRONTEND=noninteractive

# Default Debian image pulls `bookworm-updates`, which often flakes with 5xx/EOF
# behind Docker/corporate networks. `main` + `security` are enough for librdkafka-dev.
RUN set -eux; \
    find /etc/apt/sources.list.d -maxdepth 1 -name '*.sources' -exec rm -f {} +; \
    printf '%s\n' \
        'deb http://deb.debian.org/debian bookworm main' \
        'deb http://deb.debian.org/debian-security bookworm-security main' \
        > /etc/apt/sources.list; \
    printf '%s\n' \
        'Acquire::Retries "10";' \
        'Acquire::http::Timeout "120";' \
        'Acquire::https::Timeout "120";' \
        > /etc/apt/apt.conf.d/99retry; \
    ok=0; \
    for attempt in 1 2 3 4 5 6 7 8; do \
        apt-get update && { ok=1; break; }; \
        echo "apt-get update failed (attempt $attempt), retrying..."; \
        sleep $((attempt * 2)); \
    done; \
    test "$ok" = 1; \
    apt-get install -y --no-install-recommends librdkafka-dev pkg-config; \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --locked --release

# -----------------------------------------------------------------------------
# Runtime — minimal image + shared libs for Kafka consumer mode
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

ENV DEBIAN_FRONTEND=noninteractive

RUN set -eux; \
    find /etc/apt/sources.list.d -maxdepth 1 -name '*.sources' -exec rm -f {} +; \
    printf '%s\n' \
        'deb http://deb.debian.org/debian bookworm main' \
        'deb http://deb.debian.org/debian-security bookworm-security main' \
        > /etc/apt/sources.list; \
    printf '%s\n' \
        'Acquire::Retries "10";' \
        'Acquire::http::Timeout "120";' \
        'Acquire::https::Timeout "120";' \
        > /etc/apt/apt.conf.d/99retry; \
    ok=0; \
    for attempt in 1 2 3 4 5 6 7 8; do \
        apt-get update && { ok=1; break; }; \
        echo "apt-get update failed (attempt $attempt), retrying..."; \
        sleep $((attempt * 2)); \
    done; \
    test "$ok" = 1; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        librdkafka1 \
        libssl3; \
    rm -rf /var/lib/apt/lists/*; \
    useradd --system --home /nonexistent --shell /usr/sbin/nologin --uid 10001 appuser

COPY --from=builder /app/target/release/auth-service /usr/local/bin/auth-service

USER appuser
EXPOSE 3000

ENV HOST=0.0.0.0
ENV PORT=3000

ENTRYPOINT ["/usr/local/bin/auth-service"]

# Build:   docker build -t auth-service .
# Run:    docker run --rm -p 3000:3000 --env-file .env auth-service
# (Build must succeed before `docker run`. If `docker pull`/`docker build` hits
#  `unexpected EOF` talking to Docker Hub, that is a separate network/DNS issue.)
# (Set APP_MODE, DATABASE_*, JWT_SECRET, optional REDIS_URL / Kafka / cron in `.env`.)
