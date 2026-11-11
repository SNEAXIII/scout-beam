# Build stage: compiles against the committed `.sqlx` cache, never a database.
FROM rust:1.96-slim-bookworm AS builder

WORKDIR /app

ENV SQLX_OFFLINE=true

# Dependencies first, from the manifests alone: the stub sources only give cargo something to compile.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin \
    && echo 'fn main() {}' > src/main.rs \
    && echo 'fn main() {}' > src/bin/seed_user.rs \
    && : > src/lib.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY .sqlx ./.sqlx
COPY migrations ./migrations
COPY src ./src

# Cargo decides what to rebuild from mtimes, and the stub artifacts above still look current.
RUN touch src/main.rs src/lib.rs \
    && cargo build --release --locked --bin scout-beam

# Runtime stage: the binary carries its migrations (`sqlx::migrate!`).
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/scout-beam /usr/local/bin/scout-beam

# Numeric, so the account stays unprivileged whatever the host makes of names.
RUN useradd --uid 10001 --user-group --no-create-home --shell /usr/sbin/nologin scout
USER 10001:10001

# Required at run time: DATABASE_URL, JWT_SECRET and ENV (`prod`; `dev` mounts the dev login).
EXPOSE 8002
CMD ["scout-beam"]
