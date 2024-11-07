# LFS (local file sharing)

A high-performance, modular file server built in Rust, designed for handling large files with streaming support. This project is currently a work in progress.

## Features

- 🚀 Optimized for large file transfers with streaming upload/download
- 🔌 Modular storage backend system (currently supports local storage)
- 📦 Modular database system (currently supports SQLite)
- 🐳 Docker ready
- 🔄 Async I/O throughout
- 📝 File metadata tracking
- 🌐 REST API
- ⚡ Built with Rust for maximum performance

## Quick Start with Docker

The easiest way to get started is using Docker Compose:

```bash
git clone <repository-url>
cd lfs
docker-compose up -d
```

The server will be available at `http://localhost:8080`

## Configuration

The following environment variables can be configured:

```env
STORAGE_TYPE=local
STORAGE_PATH=/app/data/storage
DATABASE_TYPE=sqlite
DATABASE_PATH=/app/data/db/database.db
```

## API Endpoints

- `GET /` - Web interface
- `POST /upload` - Upload a file
- `GET /entries` - List all entries
- `GET /entry/{uuid}` - Get entry metadata
- `GET /contents/{uuid}` - Download file
- `DELETE /entry/{uuid}` - Delete entry and file

## Architecture

### Storage Backend

The storage system is modular and implements the `Storage` trait, allowing for different storage backends. Currently supported:

- Local filesystem storage

### Database Backend

The database system is modular and implements the `Database` trait. Currently supported:

- SQLite

Future planned database backends:
- PostgreSQL

### Performance Optimizations

- Streaming file uploads and downloads to minimize memory usage
- Async I/O for all operations
- Connection pooling for database operations
- Efficient file handling with minimal copies

## Development

### Prerequisites

- Rust 1.82 or higher
- SQLite 3.x

### Local Development

1. Clone the repository
2. Create a `.env` file:
```env
STORAGE_TYPE=local
STORAGE_PATH=./storage
DATABASE_TYPE=sqlite
DATABASE_PATH=./database.db
```

3. Run the server:
```bash
cargo run
```

## This is a work in progress
Stay in school.