use std::time::Instant;

use duckdb::{Connection, Result};

const QUERY: &str = r#"
SELECT CAST(dt AS VARCHAR), CAST(expiration AS VARCHAR), COUNT(*) AS cnt
FROM 'data/options_data/symbol=AMZN/*.parquet'
GROUP BY dt, expiration
ORDER BY dt DESC, expiration
LIMIT 1000;
"#;

fn main() -> Result<()> {
    let connection = Connection::open_in_memory()?;
    let start = Instant::now();

    let mut statement = connection.prepare(QUERY)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>>>()?;

    let elapsed = start.elapsed();

    println!("dt\texpiration\tcnt");
    for (dt, expiration, count) in rows {
        println!("{dt}\t{expiration}\t{count}");
    }

    println!("\nElapsed time: {:.3} ms", elapsed.as_secs_f64() * 1000.0);
    Ok(())
}
