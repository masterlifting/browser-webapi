FROM rust:1.72-slim AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty project
WORKDIR /app
COPY Cargo.toml ./

# Build dependencies (and cache them)
RUN mkdir src && echo "fn main() {println!(\"Dummy\");}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy the actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install Chrome dependencies and Chrome itself
RUN apt-get update && apt-get install -y \
    ca-certificates \
    fonts-liberation \
    libappindicator3-1 \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libc6 \
    libcairo2 \
    libcups2 \
    libdbus-1-3 \
    libexpat1 \
    libfontconfig1 \
    libgbm1 \
    libgcc1 \
    libglib2.0-0 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libpango-1.0-0 \
    libpangocairo-1.0-0 \
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
    && rm -rf /var/lib/apt/lists/*

# Install Google Chrome
RUN wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - \
    && echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list \
    && apt-get update && apt-get install -y google-chrome-stable \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN groupadd -g 1000 appuser && useradd -u 1000 -g appuser -s /bin/bash -m appuser

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/browser_automation_webapi /usr/local/bin/

# Set working directory
WORKDIR /home/appuser

# Copy the .env file
COPY --chown=appuser:appuser .env ./

# Switch to the non-root user
USER appuser

# Set environment variables
ENV RUST_LOG=info

# Expose the service port
EXPOSE 8080

# Run the service
CMD ["browser_automation_webapi"]
