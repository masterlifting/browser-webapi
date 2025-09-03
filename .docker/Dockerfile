# Stage 1: Build the Rust application
FROM rust:1.89-slim-bookworm AS builder

# Install musl tools for cross-compilation to Alpine
RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-dev \
    musl-tools \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add x86_64-unknown-linux-musl

WORKDIR /app

# Copy only dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    rm -rf src target/x86_64-unknown-linux-musl/release/deps/browser_api*

# Copy the actual source code
COPY src ./src/

# Build the application with optimizations
RUN cargo build --release --target x86_64-unknown-linux-musl && \
    strip target/x86_64-unknown-linux-musl/release/browser_api

# Stage 2: Create the minimal runtime image
FROM zenika/alpine-chrome:124

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/browser_api ./browser_api

# Override the default ENTRYPOINT from the base image
ENTRYPOINT []
CMD ["./browser_api"]
