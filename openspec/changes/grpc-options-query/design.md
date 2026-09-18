## Context

The mzworker-rust project is a Rust binary that reads options data from Parquet files using DuckDB. Currently it runs a hardcoded SQL query and prints results to stdout. The data lives on SeaweedFS, mounted via rclone to each container at `data/options_data/symbol=<TICKER>/*.parquet` with columns including `dt`, `expiration`, `strike`, `call_put`, `bid`, `ask`, `volume`, `open_interest`, and `iv`.

We need to expose this data over gRPC so other services can query options chains programmatically. Clients will be implemented in Deno, Netlify, or Cloudflare Workers.

## Goals / Non-Goals

**Goals:**
- Provide gRPC service for SQL queries against options data
- Keep the architecture simple: single binary, embedded DuckDB, no external services
- Maintain low query latency (< 100ms for typical queries)
- Support horizontal scaling on k3s with multiple replicas
- Add structured logging for observability

**Non-Goals:**
- Real-time data ingestion or streaming
- Authentication/authorization (add later if needed)
- Data persistence or caching layer
- Support for multiple data formats beyond Parquet
- AMQP/async processing (gRPC only for now)

## Decisions

### 1. Use tonic + prost for gRPC

**Choice**: tonic (async gRPC) with prost (protobuf codegen)

**Rationale**: tonic is the de-facto Rust gRPC library, well-maintained, and integrates with tokio. prost handles protobuf encoding/decoding efficiently.

**Alternatives considered**:
- `grpc-rust`: Less mature, fewer features
- Custom TCP with manual protobuf: Too much boilerplate

### 2. Use DuckDB as the query engine

**Choice**: Keep embedded DuckDB for SQL queries over Parquet

**Rationale**: Already in use, excellent Parquet support, SQL is expressive for filtering/aggregation. No need to add another dependency.

**Alternatives considered**:
- `datafusion`: More Rust-native but would require rewriting queries
- `polars`: Great for DataFrame ops but less familiar SQL interface

### 3. Single-threaded query execution

**Choice**: Run DuckDB queries on a blocking thread pool (tokio::task::spawn_blocking)

**Rationale**: DuckDB is not Send/Sync, so we can't share connections across async tasks. Each request gets its own connection or we serialize access.

**Alternatives considered**:
- Connection pool: Adds complexity, DuckDB handles concurrency internally
- Multiple DuckDB instances: Memory overhead

### 4. Proto-first development

**Choice**: Define the .proto file first, generate Rust code via build.rs

**Rationale**: Clear API contract, type-safe generated code, easy to evolve.

### 5. Arbitrary SQL with safety constraints

**Choice**: Accept raw SQL from clients with validation

**Rationale**: Maximum flexibility for clients. Safety layer ensures:
- Only SELECT statements allowed (no INSERT, UPDATE, DELETE)
- Queries rooted in data directory only
- Query timeout (default 30s) to prevent long-running queries
- Result limit (default 1000, max 10000)

**Alternatives considered**:
- Predefined filter patterns: Less flexible, requires maintenance as new filters needed

### 6. Use tracing for structured logging

**Choice**: Use `tracing` crate for structured, contextual logging

**Rationale**: De-facto standard for Rust async logging. Integrates with tokio, supports structured fields, and works with `tracing-subscriber` for output formatting.

**Logs on startup:**
- `INFO` Server starting on `0.0.0.0:50051`
- `INFO` Data directory: `/path/to/data/options_data`
- `ERROR` Data directory does not exist, exiting

**Logs on query:**
- `DEBUG` Query received, execution time
- `WARN` Query timeout or validation failure
- `ERROR` Query execution error

**Alternatives considered**:
- `log`: Less structured, no contextual fields
- `slog`: Less ecosystem integration

### 7. Validate data directory at startup

**Choice**: Check that data directory exists before starting server, fail fast if missing

**Rationale**: Prevents confusing runtime errors when queries fail because the data path is wrong. Better to fail at startup with a clear error message.

**Default data directory**: `data/options_data` (matching existing query path in main.rs)

**CLI override**: `--data-dir /custom/path`

### 8. Client compatibility

**Deno:**
- Use `@grpc/grpc-js` or `connect-rpc` for gRPC
- Works natively with HTTP/2

**Cloudflare Workers:**
- Use `@grpc/grpc-js` or `connect-rpc` for gRPC
- Works via HTTP/2 support

**Netlify Functions/Edge Functions:**
- Use `@grpc/grpc-js` or `connect-rpc` for gRPC
- Works via HTTP/2 support

**Note:** AMQP is not supported in serverless platforms (CF Workers, Netlify) due to persistent TCP connection requirements. gRPC is the right choice for cross-platform compatibility.

### 9. Deployment on k3s with SeaweedFS

**Choice**: Deploy as Deployment with 3+ replicas, each mounting SeaweedFS via rclone

**Rationale**: Each pod has its own rclone mount, reads the same data. No shared PVC needed. SeaweedFS scales well for read-heavy workloads.

**Architecture**:
```
Pod 1 ──rclone──> SeaweedFS ──> parquet files
Pod 2 ──rclone──> SeaweedFS ──> parquet files
Pod 3 ──rclone──> SeaweedFS ──> parquet files
```

**Rclone considerations**:
- Enable `--vfs-cache-mode full` for repeated queries
- Monitor SeaweedFS volume server connection limits

## Risks / Trade-offs

- **[Risk] DuckDB contention under load** → Mitigation: Use spawn_blocking to avoid blocking async runtime; consider connection pooling if needed
- **[Risk] Large result sets** → Mitigation: Enforce max result limit of 10000 rows
- **[Risk] SQL injection** → Mitigation: Parse SQL to ensure SELECT-only, restrict table access to data directory
- **[Trade-off] No streaming** → Acceptable for now; most queries return finite result sets. Can add server streaming later.
- **[Trade-off] No auth** → Acceptable for internal services; add mTLS/API keys if exposed externally
- **[Trade-off] Rclone FUSE overhead** → ~10-20% I/O overhead vs local disk; acceptable for parquet columnar reads
