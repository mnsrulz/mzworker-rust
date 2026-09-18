## Why

The mzworker-rust project currently has a hardcoded DuckDB query that reads options data from parquet files, with results printed to stdout. To make this data accessible to other services, trading systems, and analysis tools, we need to expose it via a gRPC endpoint. This enables programmatic, low-latency access to options chain data.

## What Changes

- Add gRPC server using tonic with tokio runtime
- Define protobuf service with `ExecuteQuery` RPC accepting arbitrary SQL
- Implement query handler that validates and executes SQL against DuckDB
- Safety constraints: SELECT-only, data directory rooted, query timeout, result limit
- Add structured logging with tracing (startup, shutdown, queries, errors)
- Validate data directory exists at startup (fail fast if missing)
- Default data directory to `data/options_data` (matching existing query path)
- Data stored on SeaweedFS, mounted via rclone to each container
- Add command-line configuration for listen address, data dir, and query timeout

## Capabilities

### New Capabilities

- `grpc-server`: gRPC server setup, lifecycle management, logging, and configuration
- `options-query`: Arbitrary SQL query execution against options parquet data

### Modified Capabilities

_(none - this is a new feature)_

## Impact

- **Dependencies**: tonic, prost, tokio, prost-build, sqlparser-rs, tracing
- **New files**: proto/options.proto, src/server.rs, src/query.rs, build.rs
- **Modified files**: Cargo.toml, src/main.rs
- **Data**: Reads from SeaweedFS-mounted parquet files at `data/options_data/symbol=<TICKER>/*.parquet`
