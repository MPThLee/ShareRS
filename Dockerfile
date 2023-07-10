FROM rust:1-bookworm AS chef 
RUN cargo install cargo-chef 
WORKDIR src

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /src/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
ARG SQLX_OFFLINE=true
RUN cargo build --release --bin app

FROM debian:bookworm-slim AS runtime

# Update Ca Certificates
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

WORKDIR app
COPY --from=builder /src/migrations/* /app/migrations
COPY --from=builder /src/target/release/app /app/app
CMD /app/app