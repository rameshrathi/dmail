# Local Development and Deployment Guide

## 1. Prerequisites

- **Rust toolchain:** 1.75+ (or latest stable)
- **Docker & Docker Compose:** v2.20+
- **PostgreSQL Client (optional):** `psql` for database inspection

---

## 2. Local Infrastructure Architecture

The local development environment uses Docker Compose to provide the backing services required by DMN:

- **PostgreSQL 16:** Backs the Identity Service, key records, and delivery tracking.
- **MinIO:** Provides S3-compatible local object storage for encrypted blobs and attachments.
- **Stalwart Mail Server:** Provides standard SMTP submission, server-to-server relay, and IMAP/JMAP mailbox retrieval.

---

## 3. Starting the Local Environment

Copy `.env.example` to `.env`:
```bash
cp .env.example .env
```

Start the Docker Compose stack:
```bash
docker compose -f deploy/compose/docker-compose.yml up -d
```

Check health status:
```bash
docker compose -f deploy/compose/docker-compose.yml ps
```

Stop and clean up:
```bash
docker compose -f deploy/compose/docker-compose.yml down -v
```

---

## 4. Building and Testing the Rust Workspace

```bash
# Check code formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --workspace --all-targets -- -D warnings

# Execute test suite
cargo test --workspace
```
