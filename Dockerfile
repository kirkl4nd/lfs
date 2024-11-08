# Build stage
FROM rust:1.82-bullseye as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/lfs
COPY . .
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r appgroup && useradd -r -g appgroup appuser

# Create app directories and set ownership
WORKDIR /app
RUN mkdir -p /app/data/storage /app/data/db && \
    chown -R appuser:appgroup /app

# Copy only the binary from builder
COPY --from=builder /usr/src/lfs/target/release/lfs /app/

# Set permissions
RUN chmod +x /app/lfs && \
    chown appuser:appgroup /app/lfs

# Switch to non-root user
USER appuser

# Expose the port
EXPOSE 8080

# Run the binary
CMD ["/app/lfs"]
