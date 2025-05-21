# Base image for cargo-chef
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

# Planning phase to generate recipe.json
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build phase to cache dependencies and build the application
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - caching layer
RUN cargo chef cook --release --recipe-path recipe.json
# Build the application
COPY . .
RUN cargo build --release --bin teamprojekt-agents

# Runtime image: debian:bookworm-slim with Rust toolchain installed
FROM debian:bookworm-slim AS runtime

# Install necessary libraries, Rust toolchain, and dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libssl3 \
        ca-certificates \
        curl \
        build-essential \
        gcc \
        pkg-config && \
    # Installs Rust to the debian-runtime
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    export PATH=$PATH:/root/.cargo/bin && \
    rustup default stable && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Set up working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/teamprojekt-agents /usr/local/bin/teamprojekt-agents

# Copy source code for optional rebuilds
COPY . .

# Set PATH to include Rust binaries
ENV PATH="/root/.cargo/bin:$PATH"

# Set default entrypoint to the built binary
ENTRYPOINT ["/usr/local/bin/teamprojekt-agents"]