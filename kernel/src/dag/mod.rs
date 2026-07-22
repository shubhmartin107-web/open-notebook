pub mod variable_table;
pub mod graph;
pub mod scheduler;

use crate::notebook::{Notebook, DAGEdge};
use anyhow::Result;

pub fn build_dag(notebook: &mut Notebook) -> Result<()> {
    let mut edges = Vec::new();

    for (i, cell) in notebook.cells.iter().enumerate() {
        let _defs = variable_table::extract_defs(&cell.source, &cell.kind)?;
        let refs = variable_table::extract_refs(&cell.source, &cell.kind)?;

        for j in 0..i {
            let upstream = &notebook.cells[j];
            let upstream_defs = variable_table::extract_defs(&upstream.source, &upstream.kind)?;

            let shared: Vec<String> = refs
                .iter()
                .filter(|r| upstream_defs.iter().any(|d| d == *r))
                .cloned()
                .collect();

            if !shared.is_empty() {
                edges.push(DAGEdge {
                    from_cell_id: upstream.id,
                    to_cell_id: cell.id,
                    variables: shared,
                });
            }
        }
    }

    notebook.dag.edges = edges;

    // Validate: no single-assignment violations
    let defined_in_cells = crate::dag::graph::find_single_assignment_violations(notebook)?;
    if !defined_in_cells.is_empty() {
        let msg: Vec<String> = defined_in_cells
            .iter()
            .map(|(var, cells)| format!("'{}' defined in cells: {:?}", var, cells))
            .collect();
        anyhow::bail!("Single-assignment violation: {}", msg.join("; "));
    }

    // Validate: no cycles
    if let Err(msg) = crate::dag::graph::check_for_cycles(notebook) {
        anyhow::bail!("Cycle detected: {}", msg);
    }

    Ok(())
}

pub fn visualize(notebook: &Notebook) -> Result<String> {
    use std::collections::HashMap;

    let cmap: HashMap<uuid::Uuid, usize> = notebook
        .cells
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id, i))
        .collect();

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("# DAG: {}", notebook.metadata.title));
    lines.push(format!("Cells: {}", notebook.cells.len()));
    lines.push(format!("Edges: {}", notebook.dag.edges.len()));
    lines.push(String::new());

    for (i, cell) in notebook.cells.iter().enumerate() {
        let kind = cell.kind.as_str();
        let preview = cell
            .source
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(60)
            .collect::<String>();
        lines.push(format!(" {} [{}] {}", i, kind, preview));
    }

    lines.push(String::new());
    lines.push("Edges:".to_string());
    for edge in &notebook.dag.edges {
        let from_idx = cmap.get(&edge.from_cell_id).copied().unwrap_or(999);
        let to_idx = cmap.get(&edge.to_cell_id).copied().unwrap_or(999);
        lines.push(format!(
            "  Cell {} → Cell {}  vars: {:?}",
            from_idx, to_idx, edge.variables
        ));
    }

    Ok(lines.join("\n"))
}
