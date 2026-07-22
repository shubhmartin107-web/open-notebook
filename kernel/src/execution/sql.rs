use crate::notebook::{CellOutput, OutputItem};
use anyhow::Result;

#[cfg(feature = "sql")]
pub fn execute_sql_cell(source: &str) -> Result<CellOutput> {
    use arrow::array::Array;
    use duckdb::Connection;

    let start = std::time::Instant::now();
    let conn = Connection::open_in_memory()?;

    let mut stmt = conn.prepare(source)?;

    let arrow_iter = stmt.query_arrow([])?;
    let mut text_output = String::new();
    let mut row_count = 0;

    for batch in arrow_iter {
        let ncols = batch.num_columns();
        let nrows = batch.num_rows();

        for r in 0..nrows {
            let mut row_strs = Vec::new();
            for c in 0..ncols {
                let col = batch.column(c);
                let val = if let Some(string_arr) = col.as_any().downcast_ref::<arrow::array::StringArray>() {
                    if string_arr.is_null(r) {
                        "NULL".to_string()
                    } else {
                        string_arr.value(r).to_string()
                    }
                } else if let Some(int_arr) = col.as_any().downcast_ref::<arrow::array::Int64Array>() {
                    if int_arr.is_null(r) {
                        "NULL".to_string()
                    } else {
                        int_arr.value(r).to_string()
                    }
                } else if let Some(float_arr) = col.as_any().downcast_ref::<arrow::array::Float64Array>() {
                    if float_arr.is_null(r) {
                        "NULL".to_string()
                    } else {
                        float_arr.value(r).to_string()
                    }
                } else {
                    format!("<{:?}>", col.data_type())
                };
                row_strs.push(val);
            }
            text_output.push_str(&row_strs.join(" | "));
            text_output.push('\n');
            row_count += 1;
        }
    }

    let duration = start.elapsed().as_millis() as u64;
    let summary = format!("{} rows returned", row_count);

    Ok(CellOutput {
        items: vec![OutputItem {
            mime_type: "text/plain".to_string(),
            data: text_output.as_bytes().to_vec(),
            text: Some(if text_output.is_empty() {
                summary
            } else {
                text_output
            }),
            render_priority: 1,
        }],
        error: None,
        duration_ms: duration,
    })
}

#[cfg(not(feature = "sql"))]
pub fn execute_sql_cell(source: &str) -> Result<CellOutput> {
    let _ = source;
    Ok(CellOutput {
        items: vec![],
        error: Some("SQL execution not available (compiled without 'sql' feature)".to_string()),
        duration_ms: 0,
    })
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "sql")]
    #[test]
    fn test_sql_simple_execution() {
        let output = super::execute_sql_cell("SELECT 1 AS num, 'hello' AS text").unwrap();
        assert!(output.error.is_none());
    }
}
