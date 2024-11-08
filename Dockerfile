# Build stage
FROM rust:1.82-alpine as builder

# Install minimal build dependencies
RUN apk add --no-cache musl-dev

# Create a new empty shell project
WORKDIR /usr/src/lfs
COPY . .

# Build for release
RUN cargo build --release

# Final stage
FROM alpine:3.19

# Create non-root user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

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
