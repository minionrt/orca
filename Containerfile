FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin teamprojekt-agents

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
# Install necessary libraries and Docker
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libssl3 \
        ca-certificates \
        curl \
        gnupg \
        lsb-release
WORKDIR /app
COPY --from=builder /app/target/release/teamprojekt-agents /usr/local/bin
ENTRYPOINT ["sh", "-c", "/usr/local/bin/teamprojekt-agents"]
