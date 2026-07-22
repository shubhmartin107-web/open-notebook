pub mod python;
pub mod sql;
#[cfg(feature = "cache")]
pub mod cache;

use crate::dag;
use crate::notebook::{CellKind, CellOutput, ExecutionStatus, Notebook, OutputItem};
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

/// Execute a notebook (or specific cells) and return outputs keyed by cell ID.
pub fn execute_notebook(
    notebook: &mut Notebook,
    specific_cell_ids: &[String],
) -> Result<HashMap<Uuid, CellOutput>> {
    // 1. Build the DAG
    dag::build_dag(notebook)?;

    // 2. Compute execution order
    let cell_uuids: Vec<Uuid> = if specific_cell_ids.is_empty() {
        Vec::new()
    } else {
        specific_cell_ids
            .iter()
            .map(|s| Uuid::parse_str(s))
            .collect::<Result<Vec<_>, _>>()?
    };

    let order = dag::scheduler::compute_execution_order(notebook, &cell_uuids)?;

    // 3. Execute cells in order
    let mut outputs: HashMap<Uuid, CellOutput> = HashMap::new();

    for &cell_id in &order {
        let cell = notebook
            .get_cell(&cell_id)
            .ok_or_else(|| anyhow::anyhow!("Cell not found: {}", cell_id))?;

        let source = cell.source.clone();
        let kind = cell.kind.clone();

        #[cfg(feature = "cache")]
        {
            if let Some(cached) = cache::get_cached_output(notebook, &cell_id)? {
                if let Some(cell_mut) = notebook.get_cell_mut(&cell_id) {
                    cell_mut.output = Some(cached.clone());
                    cell_mut.status = ExecutionStatus::Success;
                }
                outputs.insert(cell_id, cached);
                continue;
            }
        }

        // Execute
        let output = match kind {
            CellKind::Python => python::execute_python_cell(&source)?,
            CellKind::Sql => sql::execute_sql_cell(&source)?,
            CellKind::Markdown => CellOutput {
                items: vec![OutputItem {
                    mime_type: "text/markdown".to_string(),
                    data: source.as_bytes().to_vec(),
                    text: Some(source),
                    render_priority: 0,
                }],
                error: None,
                duration_ms: 0,
            },
            CellKind::Raw => CellOutput {
                items: vec![],
                error: None,
                duration_ms: 0,
            },
            CellKind::R => {
                // Post-MVP: Ark kernel
                CellOutput {
                    items: vec![],
                    error: Some("R execution not yet implemented (post-MVP)".to_string()),
                    duration_ms: 0,
                }
            }
        };

        // Store in cache
        #[cfg(feature = "cache")]
        cache::set_cached_output(notebook, &cell_id, &output)?;

        // Update cell
        if let Some(cell_mut) = notebook.get_cell_mut(&cell_id) {
            cell_mut.output = Some(output.clone());
            cell_mut.execution_count += 1;
            cell_mut.status = if output.error.is_some() {
                ExecutionStatus::Error
            } else {
                ExecutionStatus::Success
            };
        }

        outputs.insert(cell_id, output);
    }

    // Update last modified timestamp
    notebook.metadata.last_modified_at_unix_ms = chrono::Utc::now().timestamp_millis();

    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notebook::Notebook;
    use crate::notebook::format;

    #[cfg(all(feature = "python", feature = "sql"))]
    #[test]
    fn test_end_to_end_python_then_sql() {
        let mut nb = Notebook::new("e2e-test");
        let py_id = nb.add_cell(CellKind::Python, "x = 42\ny = x * 2");
        let sql_id = nb.add_cell(CellKind::Sql, "SELECT 1 AS a, 'hello' AS b");
        let md_id = nb.add_cell(CellKind::Markdown, "# Hello");

        let outputs = execute_notebook(&mut nb, &[]).unwrap();

        // Python cell
        let py_out = outputs.get(&py_id).unwrap();
        assert!(py_out.error.is_none(), "Python cell errored: {:?}", py_out.error);
        // duration_ms may be 0 on very fast systems
        let _ = py_out.duration_ms;

        // SQL cell
        let sql_out = outputs.get(&sql_id).unwrap();
        assert!(sql_out.error.is_none(), "SQL cell errored: {:?}", sql_out.error);

        // Markdown cell
        let md_out = outputs.get(&md_id).unwrap();
        assert!(md_out.error.is_none());
        assert_eq!(md_out.items[0].mime_type, "text/markdown");

        // Cell statuses should be updated
        assert_eq!(nb.get_cell(&py_id).unwrap().status, ExecutionStatus::Success);
        assert_eq!(nb.get_cell(&sql_id).unwrap().status, ExecutionStatus::Success);
        assert_eq!(nb.get_cell(&md_id).unwrap().status, ExecutionStatus::Success);

        // Execution counts should be 1
        assert_eq!(nb.get_cell(&py_id).unwrap().execution_count, 1);
    }

    #[cfg(feature = "python")]
    #[test]
    fn test_end_to_end_python_dag_execution() {
        let mut nb = Notebook::new("dag-test");
        // Cell 0: defines `data`
        let id0 = nb.add_cell(CellKind::Python, "data = [1, 2, 3, 4, 5]");
        // Cell 1: depends on `data`
        let id1 = nb.add_cell(CellKind::Python, "total = sum(data)");
        // Cell 2: depends on `data`
        let id2 = nb.add_cell(CellKind::Python, "count = len(data)");

        let outputs = execute_notebook(&mut nb, &[]).unwrap();

        // All cells should succeed
        for id in &[id0, id1, id2] {
            let out = outputs.get(id).unwrap();
            assert!(out.error.is_none(), "Cell {:?} errored: {:?}", id, out.error);
        }

        // DAG should have 2 edges (id0 → id1, id0 → id2)
        assert_eq!(nb.dag.edges.len(), 2);
        assert!(nb.dag.edges.iter().any(|e| e.from_cell_id == id0 && e.to_cell_id == id1));
        assert!(nb.dag.edges.iter().any(|e| e.from_cell_id == id0 && e.to_cell_id == id2));
    }

    #[cfg(all(feature = "python", feature = "sql"))]
    #[test]
    fn test_file_roundtrip_and_execute() {
        let mut nb = Notebook::new("roundtrip");
        nb.add_cell(CellKind::Python, "result = 42");
        nb.add_cell(CellKind::Sql, "SELECT 1 AS num");

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.onb");

        // Save to file
        format::save_to_file(&nb, &path).unwrap();

        // Load from file
        let mut loaded = format::load_from_file(&path).unwrap();
        assert_eq!(loaded.metadata.title, "roundtrip");
        assert_eq!(loaded.cells.len(), 2);

        // Execute
        let outputs = execute_notebook(&mut loaded, &[]).unwrap();
        assert_eq!(outputs.len(), 2);

        // Save again (with outputs)
        format::save_to_file(&loaded, &path).unwrap();

        // Reload and verify outputs are preserved
        let reloaded = format::load_from_file(&path).unwrap();
        for cell in &reloaded.cells {
            assert!(cell.output.is_some(), "Cell {} has no output", cell.id);
            assert_eq!(cell.status, ExecutionStatus::Success);
        }
    }
}
