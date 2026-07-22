use duckdb::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    let mut stmt = conn.prepare("SELECT 1 AS num, 'hello' AS text")?;
    let col_count = stmt.column_count();
    println!("Column count: {}", col_count);
    
    let rows = stmt.query_map([], |row| {
        let v0: String = row.get::<_, String>(0).unwrap_or_default();
        let v1: String = row.get::<_, String>(1).unwrap_or_default();
        println!("Row: {} | {}", v0, v1);
        Ok((v0, v1))
    })?;
    
    for row in rows {
        let (a, b) = row?;
        println!("Got: {} and {}", a, b);
    }
    
    Ok(())
}
