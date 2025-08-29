# Builder stage - optimized for caching
FROM rust:latest as builder

# Set environment variables for faster builds
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1

# Install build dependencies only (no Chrome needed - headless_chrome crate handles it)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && \
    # Clean up
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files for dependency caching (changes when deps change)
COPY Cargo.toml Cargo.lock ./

# Build dependencies only (cached until dependencies change)
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy source code (changes most frequently)
COPY src ./src

# Build the final application
RUN cargo build --release

# Runtime stage - optimized for caching
FROM debian:stable-slim

# Install runtime dependencies for headless_chrome
RUN apt-get update && apt-get install -y \
    ca-certificates \
    fonts-liberation \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libatspi2.0-0 \
    libc6 \
    libcairo2 \
    libcups2 \
    libdbus-1-3 \
    libdrm2 \
    libexpat1 \
    libfontconfig1 \
    libgbm1 \
    libgcc-s1 \
    libglib2.0-0 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libpango-1.0-0 \
    libstdc++6 \
    libx11-6 \
    libx11-xcb1 \
    libxcb1 \
    libxcomposite1 \
    libxcursor1 \
    libxdamage1 \
    libxext6 \
    libxfixes3 \
    libxi6 \
    libxrandr2 \
    libxrender1 \
    libxss1 \
    libxtst6 \
    lsb-release \
    wget \
    xdg-utils \
    && \
    # Install Chrome for headless_chrome compatibility
    wget -q https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb && \
    dpkg -i google-chrome-stable_current_amd64.deb || apt-get -fy install && \
    rm -f google-chrome-stable_current_amd64.deb && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Copy and setup application binary (changes when code changes)
COPY --from=builder /app/target/release/browser_api /app/browser_api
RUN chmod +x /app/browser_api

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Expose port
EXPOSE 8080

# Set working directory
WORKDIR /app

# Copy environment file (most likely to change - put last)
COPY .env /app/.env

# Run the application
CMD ["/app/browser_api"]