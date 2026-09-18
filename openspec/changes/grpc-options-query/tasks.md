## 1. Project Setup

- [x] 1.1 Add tonic, prost, tokio, tonic-build, sqlparser, and tracing dependencies to Cargo.toml
- [x] 1.2 Create `proto/` directory and `proto/options.proto` service definition
- [x] 1.3 Create `build.rs` to compile proto files with tonic-build

## 2. Proto Definition

- [x] 2.1 Define `ExecuteQueryRequest` message (query: string, limit: int32)
- [x] 2.2 Define `ExecuteQueryResponse` message (columns: repeated string, rows: repeated Row)
- [x] 2.3 Define `Row` message (values: repeated Value)
- [x] 2.4 Define `Value` message with oneof (string, double, int64, bool, null)
- [x] 2.5 Define `OptionsQueryService` with `ExecuteQuery` RPC

## 3. Query Module

- [x] 3.1 Create `src/query.rs` with SQL validation using sqlparser
- [x] 3.2 Implement SELECT-only check (reject non-SELECT statements)
- [x] 3.3 Implement path restriction validation (data directory only)
- [x] 3.4 Implement DuckDB query execution with timeout
- [x] 3.5 Implement result mapping from DuckDB rows to Response format

## 4. Server Module

- [x] 4.1 Create `src/server.rs` with OptionsQueryService implementation
- [x] 4.2 Implement `ExecuteQuery` RPC handler calling query module
- [x] 4.3 Add health check service implementation
- [x] 4.4 Configure server builder with listen address

## 5. Main Entry Point

- [x] 5.1 Refactor `src/main.rs` to parse CLI args (--listen-addr, --data-dir, --query-timeout)
- [x] 5.2 Initialize tracing subscriber with INFO level
- [x] 5.3 Validate data directory exists at startup (fail fast with error log if missing)
- [x] 5.4 Initialize tokio runtime and start gRPC server with INFO log
- [x] 5.5 Add graceful shutdown signal handling (SIGINT, SIGTERM) with INFO log

## 6. Testing

- [x] 6.1 Add unit tests for SQL validation (SELECT-only, path check)
- [ ] 6.2 Add integration test for gRPC server start and health check
- [ ] 6.3 Add test for data directory validation (missing dir fails)
- [ ] 6.4 Manual test with grpcurl or similar client
