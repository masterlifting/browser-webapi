# Stage 1: Build the Rust application
FROM rust:1.89-slim-bookworm AS builder

WORKDIR /app

# Copy only dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/browser_webapi*

# Copy the actual source code
COPY src ./src/

# Build the application with optimizations
RUN cargo build --release && \
    strip target/release/browser-webapi

# Stage 2: Runtime with Google Chrome and Xvfb
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# Keep the runtime image smaller:
# - use --no-install-recommends
# - avoid keeping APT package lists
# - drop docs/man pages/locales we don't need for a containerized service
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends ca-certificates wget gnupg; \
    \
    # Install Google Chrome (not Chromium) using a modern keyring (apt-key is deprecated).
    wget -qO- https://dl-ssl.google.com/linux/linux_signing_key.pub | gpg --dearmor > /usr/share/keyrings/google-linux-signing-keyring.gpg; \
    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/google-linux-signing-keyring.gpg] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list; \
    apt-get update; \
    \
    # Install Chrome + minimal runtime deps we explicitly rely on.
    # Note: Chrome will pull its required shared libs as dependencies.
    apt-get install -y --no-install-recommends \
      google-chrome-stable \
      xvfb \
      x11-utils \
      fonts-liberation \
      fonts-noto; \
    \
    # Ensure runtime packages are marked as manually installed so they don't get removed by autoremove.
    apt-mark manual google-chrome-stable xvfb x11-utils fonts-liberation fonts-noto; \
    \
    # Optional: prune Chrome UI locales to reduce size (keep en-US + it).
    if [ -d /opt/google/chrome/locales ]; then \
      find /opt/google/chrome/locales -type f ! -name 'en-US.pak' ! -name 'it.pak' -delete; \
    fi; \
    \
    # Remove installer tooling.
    # NOTE: Do not purge `wget` here: on Ubuntu 22.04, `google-chrome-stable` depends on it,
    # and purging `wget` would remove Chrome as well.
    apt-get purge -y gnupg dirmngr gnupg-l10n gnupg-utils gpg gpg-agent gpg-wks-client gpg-wks-server gpgconf gpgsm; \
    \
    # Drop docs/manpages/locales shipped by base packages.
    rm -rf /usr/share/doc/* /usr/share/man/* /usr/share/locale/*; \
    \
    rm -rf /var/lib/apt/lists/*

# Create non-root user for Chrome
RUN groupadd -r chrome && useradd -r -g chrome -G audio,video chrome \
    && mkdir -p /home/chrome/Downloads /app/chrome-profile \
    && chown -R chrome:chrome /home/chrome /app/chrome-profile

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/browser-webapi ./browser-webapi

# Set permissions
RUN chown chrome:chrome /app/browser-webapi

# Create startup script with Xvfb and logging
RUN cat > /app/start.sh <<'EOF'
#!/bin/bash
set -e

rm -f /tmp/.X99-lock

mkdir -p "$USER_DATA_DIR"
rm -f "$USER_DATA_DIR/SingletonLock"
rm -f "$USER_DATA_DIR/SingletonCookie"
rm -f "$USER_DATA_DIR/SingletonSocket"

Xvfb :99 -screen 0 1920x1080x24 -nolisten tcp -nolisten unix -ac &
XVFB_PID=$!

export DISPLAY=:99

sleep 2
xdpyinfo -display :99 > /dev/null 2>&1

exec ./browser-webapi
EOF
RUN chmod +x /app/start.sh

USER chrome

CMD ["/app/start.sh"]
