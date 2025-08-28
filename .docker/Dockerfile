FROM rust:1.85-slim as builder

# Add this for faster dependency compilation
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Build with fewer codegen units for faster compilation
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release

# Remove the dummy files
RUN rm -rf src/main.rs

# Copy the actual source code
COPY src ./src

# Touch main.rs to ensure it's newer than Cargo.toml
RUN touch src/main.rs

# Build the actual application
RUN cargo build --release

# Runtime stage - use latest Debian
FROM debian:bookworm-slim

# Minimal runtime dependencies (Debian)
RUN apt-get update && apt-get install -y --no-install-recommends \
    chromium \
    ca-certificates \
    fonts-liberation \
    libnss3 \
    libatk-bridge2.0-0 \
    libx11-6 \
    libxcomposite1 \
    libxdamage1 \
    libxrandr2 \
    curl \
 && rm -rf /var/lib/apt/lists/*


# Create app directory
RUN mkdir -p /app
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/browser_api /app/browser_api
COPY .docker/.env /app/.env

# Create user and set permissions
RUN useradd -r -s /bin/false -u 1001 browseruser && \
    chown -R browseruser:browseruser /app && \
    chmod +x /app/browser_api

# Switch to non-root user
USER browseruser

# Environment variables
ENV CHROME_BIN=/usr/bin/chromium
ENV CHROME_PATH=/usr/bin/chromium
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

EXPOSE 8080

# Updated healthcheck
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

CMD ["./browser_api"]