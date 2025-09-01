# Stage 1: Build the Rust application
FROM rust:latest AS builder

WORKDIR /app

# Copy Cargo.toml and Cargo.lock to leverage Docker's caching
COPY Cargo.toml Cargo.lock ./

# Build dependencies first to optimize caching
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

# Copy the entire source code
COPY . .

# Build the final release binary
RUN cargo build --release

# Stage 2: Create the minimal runtime image
FROM debian:bullseye-slim

# Create a non-privileged user
RUN useradd -ms /bin/bash appuser

# Switch to the non-privileged user
USER appuser

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/browser_api ./browser_api

# Expose the port if your application is a web server
EXPOSE 8080

# Command to run the application
CMD ["./browser_api"]