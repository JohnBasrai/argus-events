# Multi-stage build for Argus Events server using cr8s base images
# Stage 1: Build environment
FROM ghcr.io/johnbasrai/cr8s/rust-dev:1.83.0-rev5 as builder

# Set working directory
WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo.toml changes)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Run comprehensive testing suite following cr8s patterns
RUN /bin/sh -c 'echo "👀 Lint checks..." >&2'
RUN cargo fmt --check
RUN cargo clippy --release --all-targets --all-features -- -D warnings

# Security audit - adjust ignore list as needed for argus-events
RUN cargo audit || cargo outdated || true

# Skip integration tests, only run unit tests (following cr8s pattern)
# Integration tests will be run externally against the built container
RUN cargo test --release --lib --bins -- --nocapture

# Build the actual application
# Touch main.rs to ensure it rebuilds
RUN cargo build --release

# Stage 2: Runtime environment
FROM ghcr.io/johnbasrai/cr8s/rust-runtime:0.1.3

# Copy the binary from builder stage
COPY --from=builder /app/target/release/argus-events /app/argus-events

# Make sure the binary is executable
RUN chmod +x /app/argus-events

# Expose the default port
EXPOSE 3000

# Set default environment variables
ENV ARGUS_ENDPOINT=0.0.0.0:3000
ENV ARGUS_REPOSITORY=memory
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/events || exit 1

# Run the binary
CMD ["./argus-events"]
