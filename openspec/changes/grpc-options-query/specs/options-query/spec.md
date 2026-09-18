## ADDED Requirements

### Requirement: Execute arbitrary SQL query
The system SHALL provide an `ExecuteQuery` RPC that accepts a raw SQL string and executes it against DuckDB over parquet files.

#### Scenario: Query returns results
- **WHEN** a client sends `ExecuteQuery` with `query: "SELECT * FROM 'data/options_data/symbol=AMZN/*.parquet' LIMIT 10"`
- **THEN** the server executes the query and returns column names and rows

#### Scenario: Query with syntax error
- **WHEN** a client sends `ExecuteQuery` with `query: "SELECT FROM WHERE"`
- **THEN** the server returns an error with SQL syntax error details

### Requirement: SELECT-only validation
The system SHALL reject any SQL statement that is not a SELECT query.

#### Scenario: Reject INSERT statement
- **WHEN** a client sends `ExecuteQuery` with `query: "INSERT INTO ..."`
- **THEN** the server returns an error indicating only SELECT queries are allowed

#### Scenario: Reject DROP statement
- **WHEN** a client sends `ExecuteQuery` with `query: "DROP TABLE ..."`
- **THEN** the server returns an error indicating only SELECT queries are allowed

### Requirement: Data directory restriction
The system SHALL ensure queries can only access data within the configured data directory.

#### Scenario: Query accesses allowed path
- **WHEN** a client sends `ExecuteQuery` with `query: "SELECT * FROM 'data/options_data/symbol=AMZN/*.parquet'"`
- **THEN** the query executes successfully

#### Scenario: Query accesses forbidden path
- **WHEN** a client sends `ExecuteQuery` with `query: "SELECT * FROM '/etc/passwd'"`
- **THEN** the server returns an error indicating path not allowed

### Requirement: Query timeout
The system SHALL enforce a query timeout (default: 30 seconds).

#### Scenario: Query completes within timeout
- **WHEN** a client sends a query that completes in 5 seconds
- **THEN** the server returns the results

#### Scenario: Query exceeds timeout
- **WHEN** a client sends a query that runs for 35 seconds
- **THEN** the server returns a timeout error

### Requirement: Result limit
The system SHALL support a `limit` parameter to cap the number of returned records (default: 1000, max: 10000).

#### Scenario: Limit results
- **WHEN** a client sends `ExecuteQuery` with `limit: 100`
- **THEN** the server returns at most 100 rows

#### Scenario: Exceed max limit
- **WHEN** a client sends `ExecuteQuery` with `limit: 50000`
- **THEN** the server enforces the max limit of 10000 rows

### Requirement: Response format
The system SHALL return query results with column names and row data.

#### Scenario: Response contains columns and rows
- **WHEN** a client receives a query response
- **THEN** the response contains `columns` (list of column names) and `rows` (list of row arrays)
