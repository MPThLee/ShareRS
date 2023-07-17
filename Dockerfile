# Rust, Stable, With bookworm
FROM lukemathwalker/cargo-chef:latest-rust-1-bookworm AS chef

# Install build-time dependencies
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates build-essential git \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

WORKDIR src

# Create recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Let's build
FROM chef AS builder
COPY --from=planner /src/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
ARG SQLX_OFFLINE=true
RUN cargo build --release --bin app

# Runtime
FROM debian:bookworm-slim AS runtime

# Update Ca Certificates and Tini
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates tini \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

WORKDIR app
COPY --from=builder /src/migrations/* /app/migrations
COPY --from=builder /src/target/release/app /app/app

# Tini for safety
ENTRYPOINT ["/usr/bin/tini", "--"]

# Run app
CMD /app/app