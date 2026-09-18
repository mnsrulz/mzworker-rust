use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use sqlparser::ast::{Statement, ObjectName};
use tokio::task;
use tracing::{debug, warn, error};

use crate::options::{ExecuteQueryRequest, ExecuteQueryResponse, Row, Value};

const DEFAULT_LIMIT: i32 = 1000;
const MAX_LIMIT: i32 = 10000;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub enum QueryError {
    SqlSyntax(String),
    NotSelect,
    PathNotAllowed(String),
    Timeout,
    ExecutionError(String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::SqlSyntax(e) => write!(f, "SQL syntax error: {}", e),
            QueryError::NotSelect => write!(f, "Only SELECT queries are allowed"),
            QueryError::PathNotAllowed(p) => write!(f, "Path not allowed: {}", p),
            QueryError::Timeout => write!(f, "Query timeout"),
            QueryError::ExecutionError(e) => write!(f, "Query execution failed: {}", e),
        }
    }
}

impl std::error::Error for QueryError {}

pub fn validate_query(sql: &str, data_dir: &Path) -> Result<(), QueryError> {
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, sql)
        .map_err(|e| QueryError::SqlSyntax(e.to_string()))?;

    if ast.len() != 1 {
        return Err(QueryError::SqlSyntax("Expected exactly one statement".to_string()));
    }

    match &ast[0] {
        Statement::Query(_) => {}
        _ => return Err(QueryError::NotSelect),
    }

    let sql_lower = sql.to_lowercase();
    let forbidden_patterns = ["/etc/", "/proc/", "/sys/", "/dev/"];
    for pattern in &forbidden_patterns {
        if sql_lower.contains(pattern) {
            return Err(QueryError::PathNotAllowed(pattern.to_string()));
        }
    }

    Ok(())
}

pub fn enforce_limit(request_limit: i32) -> i32 {
    if request_limit <= 0 {
        DEFAULT_LIMIT
    } else {
        request_limit.min(MAX_LIMIT)
    }
}

pub async fn execute_query(
    request: ExecuteQueryRequest,
    data_dir: PathBuf,
) -> Result<ExecuteQueryResponse, QueryError> {
    let sql = request.query.clone();
    let limit = enforce_limit(request.limit);

    validate_query(&sql, &data_dir)?;

    let sql_with_limit = if sql.to_lowercase().contains("limit") {
        sql
    } else {
        format!("{} LIMIT {}", sql, limit)
    };

    let data_dir_str = data_dir.to_string_lossy().to_string();

    debug!(query = %sql_with_limit, "Executing query");

    let result = task::spawn_blocking(move || {
        let connection = duckdb::Connection::open_in_memory()
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        let mut stmt = connection.prepare(&sql_with_limit)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        let column_count = stmt.column_count() as usize;
        let mut columns: Vec<String> = (0..column_count)
            .map(|i| stmt.column_name(i as u32).unwrap_or_default().to_string())
            .collect();

        let mut rows: Vec<Row> = Vec::new();

        let rows_iter = stmt.query_map([], |row| {
            let mut values = Vec::new();
            for i in 0..column_count {
                let value = if let Ok(v) = row.get::<_, String>(i) {
                    Value { value: Some(crate::options::value::Value::StringVal(v)) }
                } else if let Ok(v) = row.get::<_, f64>(i) {
                    Value { value: Some(crate::options::value::Value::DoubleVal(v)) }
                } else if let Ok(v) = row.get::<_, i64>(i) {
                    Value { value: Some(crate::options::value::Value::IntVal(v)) }
                } else if let Ok(v) = row.get::<_, bool>(i) {
                    Value { value: Some(crate::options::value::Value::BoolVal(v)) }
                } else {
                    Value { value: None }
                };
                values.push(value);
            }
            Ok(Row { values })
        })
        .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        for row_result in rows_iter {
            match row_result {
                Ok(row) => rows.push(row),
                Err(e) => return Err(QueryError::ExecutionError(e.to_string())),
            }
        }

        Ok(ExecuteQueryResponse { columns, rows })
    })
    .await
    .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_query_select() {
        let data_dir = PathBuf::from("data/options_data");
        let result = validate_query("SELECT * FROM 'data/options_data/symbol=AMZN/*.parquet'", &data_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_query_insert_rejected() {
        let data_dir = PathBuf::from("data/options_data");
        let result = validate_query("INSERT INTO table VALUES (1)", &data_dir);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::NotSelect));
    }

    #[test]
    fn test_validate_query_drop_rejected() {
        let data_dir = PathBuf::from("data/options_data");
        let result = validate_query("DROP TABLE users", &data_dir);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::NotSelect));
    }

    #[test]
    fn test_validate_query_forbidden_path() {
        let data_dir = PathBuf::from("data/options_data");
        let result = validate_query("SELECT * FROM '/etc/passwd'", &data_dir);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::PathNotAllowed(_)));
    }

    #[test]
    fn test_validate_query_syntax_error() {
        let data_dir = PathBuf::from("data/options_data");
        let result = validate_query("SELECT FROM WHERE", &data_dir);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::SqlSyntax(_)));
    }

    #[test]
    fn test_enforce_limit_default() {
        assert_eq!(enforce_limit(0), 1000);
        assert_eq!(enforce_limit(-1), 1000);
    }

    #[test]
    fn test_enforce_limit_custom() {
        assert_eq!(enforce_limit(100), 100);
    }

    #[test]
    fn test_enforce_limit_max_cap() {
        assert_eq!(enforce_limit(50000), 10000);
    }
}

#[cfg(test)]
mod cte_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_query_cte() {
        let data_dir = PathBuf::from("data/options_data");
        let sql = r#"
            WITH amzn_options AS (
                SELECT * FROM 'data/options_data/symbol=AMZN/*.parquet'
                WHERE expiration = '2025-01-17'
            )
            SELECT strike, bid, ask FROM amzn_options WHERE strike > 100
        "#;
        let result = validate_query(sql, &data_dir);
        assert!(result.is_ok(), "CTE should be accepted: {:?}", result.err());
    }
}
