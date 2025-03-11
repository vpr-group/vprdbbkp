FROM rust:1.76-slim as builder

WORKDIR /usr/src/app

# Install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Cache dependencies
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -f target/release/deps/db_backup_cli*

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Create runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates postgresql-client mysql-client gzip && \
    rm -rf /var/lib/apt/lists/*

# Copy our build
COPY --from=builder /usr/src/app/target/release/db-backup-cli /usr/local/bin/

# Set entrypoint
ENTRYPOINT ["db-backup-cli"]