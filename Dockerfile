# Use a lightweight base image with PostgreSQL client
FROM ubuntu:22.04

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV VPRS3BKP_VERSION=latest

# Install dependencies including sudo for the install script
RUN apt-get update && apt-get install -y \
    postgresql-client \
    mysql-client \
    curl \
    ca-certificates \
    gzip \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# Install vprs3bkp and verify installation
RUN curl -fsSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install-cli.sh | bash && \
    echo "=== Checking installation ===" && \
    ls -la /usr/local/bin/ && \
    echo "=== Finding vprs3bkp binaries ===" && \
    find /usr -name "*vprs3bkp*" -o -name "*cli*" 2>/dev/null && \
    echo "=== Testing binary ===" && \
    if [ -f /usr/local/bin/vprs3bkp ]; then \
    /usr/local/bin/vprs3bkp --version && echo "✅ vprs3bkp working"; \
    elif [ -f /usr/local/bin/cli ]; then \
    /usr/local/bin/cli --version && echo "✅ cli working"; \
    ln -s /usr/local/bin/cli /usr/local/bin/vprs3bkp && echo "✅ Created symlink"; \
    else \
    echo "❌ No binary found" && exit 1; \
    fi

# Create a non-root user for security
RUN useradd -r -u 1001 -g root backup-user -m -d /home/backup-user

# Create directories for backups and cache
RUN mkdir -p /backups /backups/.cache /home/backup-user/.cache && \
    chown -R backup-user:root /backups /home/backup-user && \
    chmod -R 755 /home/backup-user /backups

# Copy and make entrypoint script executable
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh && chown backup-user:root /usr/local/bin/entrypoint.sh

# Set working directory
WORKDIR /backups

# Switch to non-root user
USER backup-user

# Default environment variables (can be overridden)
ENV DATABASE_TYPE=postgresql
ENV DATABASE=""
ENV HOST=localhost
ENV PORT=5432
ENV USERNAME=""
ENV PASSWORD=""
ENV STORAGE_TYPE=s3
ENV LOCATION=""
ENV BUCKET=""
ENV REGION=us-east-1
ENV ENDPOINT=""
ENV ACCESS_KEY=""
ENV SECRET_KEY=""
ENV BACKUP_NAME=""
ENV FORMAT=custom
ENV COMPRESS=true
ENV VERBOSE=false
# Set cache directory to a writable location
ENV XDG_CACHE_HOME=/backups/.cache
ENV HOME=/home/backup-user

# Default command
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["backup"]