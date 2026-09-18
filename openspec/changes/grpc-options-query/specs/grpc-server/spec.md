## ADDED Requirements

### Requirement: gRPC server lifecycle
The system SHALL run a tonic gRPC server that listens on a configurable address (default: `0.0.0.0:50051`).

#### Scenario: Server starts successfully
- **WHEN** the binary is executed with default arguments
- **THEN** the gRPC server starts listening on `0.0.0.0:50051`

#### Scenario: Custom listen address
- **WHEN** the binary is executed with `--listen-addr 127.0.0.1:8080`
- **THEN** the gRPC server starts listening on `127.0.0.1:8080`

### Requirement: Startup logging
The system SHALL log server startup information at INFO level including listen address and data directory path.

#### Scenario: Server logs startup
- **WHEN** the server starts successfully
- **THEN** the system logs `INFO Server starting on 0.0.0.0:50051`

#### Scenario: Server logs data directory
- **WHEN** the server starts successfully
- **THEN** the system logs `INFO Data directory: /path/to/data/options_data`

### Requirement: Data directory validation
The system SHALL validate that the data directory exists at startup and exit with an error if it does not.

#### Scenario: Data directory exists
- **WHEN** the binary is executed with `--data-dir /path/to/existing/dir`
- **THEN** the server starts successfully

#### Scenario: Data directory does not exist
- **WHEN** the binary is executed with `--data-dir /nonexistent/path`
- **THEN** the system logs `ERROR Data directory does not exist: /nonexistent/path` and exits with code 1

### Requirement: Graceful shutdown
The system SHALL handle SIGINT/SIGTERM signals and shut down the gRPC server gracefully, finishing in-flight requests.

#### Scenario: Signal handling
- **WHEN** the server receives a SIGINT signal
- **THEN** the system logs `INFO Shutting down...` and stops accepting new requests

### Requirement: Health check
The system SHALL implement gRPC health checking protocol (grpc.health.v1.Health).

#### Scenario: Health check returns serving
- **WHEN** a health check request is received
- **THEN** the server responds with SERVING status
