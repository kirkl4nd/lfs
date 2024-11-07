# Build stage
FROM rust:1.82 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
WORKDIR /app

# Install SSL certificates and SQLite
RUN apt-get update && \
    apt-get install -y ca-certificates libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/backend /app/backend

# Create directories for volumes
RUN mkdir -p /app/data/storage /app/data/db

# Set environment variables with defaults
ENV STORAGE_TYPE=local
ENV STORAGE_PATH=/app/data/storage
ENV DATABASE_TYPE=sqlite
ENV DATABASE_PATH=/app/data/db/database.db

EXPOSE 8080

CMD ["./backend"]
