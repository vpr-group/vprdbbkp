# Use a lightweight base image with PostgreSQL client
FROM ubuntu:22.04

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV VPRS3BKP_VERSION=latest

# Install dependencies
RUN apt-get update && apt-get install -y \
    postgresql-client \
    mysql-client \
    curl \
    ca-certificates \
    gzip \
    && rm -rf /var/lib/apt/lists/*

# Install vprs3bkp
RUN curl -fsSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install.sh | bash

# Create a non-root user for security
RUN useradd -r -u 1001 -g root backup-user

# Create directories for backups
RUN mkdir -p /backups && chown backup-user:root /backups

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

# Create entrypoint script
COPY --chown=backup-user:root entrypoint.sh /usr/local/bin/entrypoint.sh

# Default command
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["backup"]