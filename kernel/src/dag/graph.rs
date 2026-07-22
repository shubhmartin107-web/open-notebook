use crate::notebook::Notebook;
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Find variables that are defined in more than one cell (single-assignment violation).
pub fn find_single_assignment_violations(
    notebook: &Notebook,
) -> Result<HashMap<String, Vec<Uuid>>> {
    let mut def_map: HashMap<String, Vec<Uuid>> = HashMap::new();

    for cell in &notebook.cells {
        let defs = crate::dag::variable_table::extract_defs(&cell.source, &cell.kind)?;
        for def in defs {
            def_map.entry(def).or_default().push(cell.id);
        }
    }

    let violations: HashMap<String, Vec<Uuid>> = def_map
        .into_iter()
        .filter(|(_, cells)| cells.len() > 1)
        .collect();

    Ok(violations)
}

/// Check for cycles in the notebook's DAG.
/// Returns Err with cycle description if a cycle is found.
pub fn check_for_cycles(notebook: &Notebook) -> Result<()> {
    let graph = build_adjacency_list(notebook);
    let mut visited: HashSet<Uuid> = HashSet::new();
    let mut in_stack: HashSet<Uuid> = HashSet::new();
    let mut path: Vec<Uuid> = Vec::new();

    for &start in graph.keys() {
        if !visited.contains(&start) {
            if let Some(cycle) = dfs_cycle_check(&graph, start, &mut visited, &mut in_stack, &mut path) {
                let cycle_str: Vec<String> = cycle
                    .iter()
                    .map(|id| {
                        notebook
                            .get_cell(id)
                            .map(|c| format!("{} ({})", id, c.kind.as_str()))
                            .unwrap_or_else(|| id.to_string())
                    })
                    .collect();
                bail!("DAG cycle detected: {}", cycle_str.join(" → "));
            }
        }
    }

    Ok(())
}

fn dfs_cycle_check(
    graph: &HashMap<Uuid, Vec<Uuid>>,
    node: Uuid,
    visited: &mut HashSet<Uuid>,
    in_stack: &mut HashSet<Uuid>,
    path: &mut Vec<Uuid>,
) -> Option<Vec<Uuid>> {
    visited.insert(node);
    in_stack.insert(node);
    path.push(node);

    if let Some(neighbors) = graph.get(&node) {
        for &neighbor in neighbors {
            if !visited.contains(&neighbor) {
                if let Some(cycle) =
                    dfs_cycle_check(graph, neighbor, visited, in_stack, path)
                {
                    return Some(cycle);
                }
            } else if in_stack.contains(&neighbor) {
                // Found a cycle — extract it
                let cycle_start = path.iter().position(|&x| x == neighbor).unwrap();
                let mut cycle = path[cycle_start..].to_vec();
                cycle.push(neighbor); // Close the cycle
                return Some(cycle);
            }
        }
    }

    path.pop();
    in_stack.remove(&node);
    None
}

/// Build an adjacency list from the DAG edges: parent → children (downstream direction).
pub fn build_adjacency_list(notebook: &Notebook) -> HashMap<Uuid, Vec<Uuid>> {
    let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

    // Ensure all cells have entries (even leaves with no downstream)
    for cell in &notebook.cells {
        graph.entry(cell.id).or_default();
    }

    // Add edges (upstream → downstream)
    for edge in &notebook.dag.edges {
        graph
            .entry(edge.from_cell_id)
            .or_default()
            .push(edge.to_cell_id);
    }

    graph
}

/// Compute the reverse adjacency list: child → parents (upstream direction).
pub fn build_reverse_adjacency(notebook: &Notebook) -> HashMap<Uuid, Vec<Uuid>> {
    let mut reverse: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

    for cell in &notebook.cells {
        reverse.entry(cell.id).or_default();
    }

    for edge in &notebook.dag.edges {
        reverse
            .entry(edge.to_cell_id)
            .or_default()
            .push(edge.from_cell_id);
    }

    reverse
}

/// Get all downstream cell IDs (transitive closure) starting from a given cell.
pub fn get_downstream_cells(notebook: &Notebook, cell_id: &Uuid) -> Vec<Uuid> {
    let graph = build_adjacency_list(notebook);
    let mut downstream = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = vec![*cell_id];

    while let Some(node) = stack.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node);
        if node != *cell_id {
            downstream.push(node);
        }
        if let Some(neighbors) = graph.get(&node) {
            stack.extend(neighbors);
        }
    }

    downstream
}

/// Get the indegree (number of upstream dependencies) for each cell.
pub fn compute_indegrees(notebook: &Notebook) -> HashMap<Uuid, usize> {
    let mut indegrees: HashMap<Uuid, usize> = HashMap::new();

    for cell in &notebook.cells {
        indegrees.entry(cell.id).or_insert(0);
    }

    for edge in &notebook.dag.edges {
        *indegrees.entry(edge.to_cell_id).or_insert(0) += 1;
    }

    indegrees
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notebook::{CellKind, Notebook};

    fn make_notebook_with_edges(cell_sources: Vec<(&str, CellKind)>, edges: Vec<(usize, usize, Vec<String>)>) -> Notebook {
        let mut nb = Notebook::new("test");
        let mut ids = Vec::new();
        for (source, kind) in &cell_sources {
            let id = nb.add_cell(kind.clone(), source);
            ids.push(id);
        }
        for (from, to, vars) in &edges {
            nb.dag.edges.push(crate::notebook::DAGEdge {
                from_cell_id: ids[*from],
                to_cell_id: ids[*to],
                variables: vars.clone(),
            });
        }
        nb
    }

    #[test]
    fn test_no_cycle_linear() {
        let nb = make_notebook_with_edges(
            vec![
                ("x = 5", CellKind::Python),
                ("y = x + 1", CellKind::Python),
                ("z = y * 2", CellKind::Python),
            ],
            vec![
                (0, 1, vec!["x".into()]),
                (1, 2, vec!["y".into()]),
            ],
        );
        assert!(check_for_cycles(&nb).is_ok());
    }

    #[test]
    fn test_cycle_detected() {
        let nb = make_notebook_with_edges(
            vec![
                ("x = 1", CellKind::Python),
                ("y = x + 1", CellKind::Python),
            ],
            vec![
                (0, 1, vec!["x".into()]),
                (1, 0, vec!["y".into()]),
            ],
        );
        assert!(check_for_cycles(&nb).is_err());
    }

    #[test]
    fn test_no_cycle_diamond() {
        let nb = make_notebook_with_edges(
            vec![
                ("data = load()", CellKind::Python),
                ("cleaned = clean(data)", CellKind::Python),
                ("aggregated = agg(data)", CellKind::Python),
                ("result = merge(cleaned, aggregated)", CellKind::Python),
            ],
            vec![
                (1, 3, vec!["cleaned".into()]),
                (2, 3, vec!["aggregated".into()]),
            ],
        );
        assert!(check_for_cycles(&nb).is_ok());
    }

    #[test]
    fn test_downstream_basic() {
        let mut nb = Notebook::new("test");
        let id0 = nb.add_cell(CellKind::Python, "x = 5");
        let id1 = nb.add_cell(CellKind::Python, "y = x + 1");
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id0,
            to_cell_id: id1,
            variables: vec!["x".into()],
        });

        let down = get_downstream_cells(&nb, &id0);
        assert_eq!(down, vec![id1]);
    }

    #[test]
    fn test_single_assignment_ok() {
        let nb = make_notebook_with_edges(
            vec![
                ("x = 5", CellKind::Python),
                ("y = x + 1", CellKind::Python),
            ],
            vec![(0, 1, vec!["x".into()])],
        );
        let violations = find_single_assignment_violations(&nb).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_single_assignment_violated() {
        let nb = make_notebook_with_edges(
            vec![
                ("x = 5", CellKind::Python),
                ("x = 10", CellKind::Python),
            ],
            vec![],
        );
        let violations = find_single_assignment_violations(&nb).unwrap();
        assert!(violations.contains_key("x"));
        assert_eq!(violations["x"].len(), 2);
    }

    #[test]
    fn test_indegrees() {
        let nb = make_notebook_with_edges(
            vec![
                ("x = 5", CellKind::Python),
                ("y = x + 1", CellKind::Python),
                ("z = y * 2", CellKind::Python),
            ],
            vec![
                (0, 1, vec!["x".into()]),
                (1, 2, vec!["y".into()]),
            ],
        );
        let indeg = compute_indegrees(&nb);
        let ids: Vec<Uuid> = nb.cells.iter().map(|c| c.id).collect();
        assert_eq!(indeg[&ids[0]], 0);
        assert_eq!(indeg[&ids[1]], 1);
        assert_eq!(indeg[&ids[2]], 1);
    }
}
