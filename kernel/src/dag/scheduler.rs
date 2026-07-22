use crate::notebook::Notebook;
use crate::dag::graph;
use anyhow::Result;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Compute the execution order for cells in a notebook using Kahn's algorithm
/// (topological sort based on indegree).
///
/// Returns a list of cell IDs in execution order.
/// If `specific_cells` is non-empty, only runs those cells and their downstream dependencies.
pub fn compute_execution_order(
    notebook: &Notebook,
    specific_cells: &[Uuid],
) -> Result<Vec<Uuid>> {
    let adjacency = graph::build_adjacency_list(notebook);

    // Determine the set of cells to execute
    let cells_to_run: Vec<Uuid> = if specific_cells.is_empty() {
        notebook.cells.iter().map(|c| c.id).collect()
    } else {
        let mut to_run = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue: VecDeque<Uuid> = specific_cells.iter().cloned().collect();

        // BFS to find all downstream cells (transitive)
        while let Some(cell_id) = queue.pop_front() {
            if visited.contains(&cell_id) {
                continue;
            }
            visited.insert(cell_id);
            to_run.push(cell_id);

            if let Some(neighbors) = adjacency.get(&cell_id) {
                for d in neighbors {
                    if !visited.contains(d) {
                        queue.push_back(*d);
                    }
                }
            }
        }

        to_run
    };

    // Filter to only cells in the execution set
    let cells_set: std::collections::HashSet<Uuid> = cells_to_run.iter().cloned().collect();

    // Kahn's algorithm: topological sort
    let mut indeg: HashMap<Uuid, usize> = HashMap::new();
    for &cell_id in &cells_to_run {
        indeg.entry(cell_id).or_insert(0);
    }

    // Only count edges where both endpoints are in the execution set
    for edge in &notebook.dag.edges {
        if cells_set.contains(&edge.to_cell_id) && cells_set.contains(&edge.from_cell_id) {
            *indeg.entry(edge.to_cell_id).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<Uuid> = VecDeque::new();
    for (&cell_id, &degree) in &indeg {
        if degree == 0 && cells_set.contains(&cell_id) {
            queue.push_back(cell_id);
        }
    }

    let mut order = Vec::new();
    while let Some(cell_id) = queue.pop_front() {
        order.push(cell_id);

        // Decrease indegree of immediate downstream cells (direct children)
        if let Some(neighbors) = adjacency.get(&cell_id) {
            for d in neighbors {
                if cells_set.contains(d) {
                    if let Some(degree) = indeg.get_mut(d) {
                        if *degree > 0 {
                            *degree -= 1;
                        }
                        if *degree == 0 {
                            queue.push_back(*d);
                        }
                    }
                }
            }
        }
    }

    if order.len() != cells_to_run.len() {
        anyhow::bail!(
            "Topological sort incomplete: {} of {} cells ordered (possible cycle or disconnected cells)",
            order.len(),
            cells_to_run.len()
        );
    }

    Ok(order)
}

/// Generate a DAG execution plan that groups cells into layers.
/// Cells in the same layer can be executed in parallel (no dependencies between them).
pub fn compute_execution_layers(notebook: &Notebook) -> Result<Vec<Vec<Uuid>>> {
    let adjacency = graph::build_adjacency_list(notebook);
    let indegrees = graph::compute_indegrees(notebook);

    let mut indeg = indegrees.clone();
    let mut layers: Vec<Vec<Uuid>> = Vec::new();

    loop {
        // Find all cells with indegree 0 that haven't been scheduled
        let current_layer: Vec<Uuid> = indeg
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        if current_layer.is_empty() {
            break;
        }

        layers.push(current_layer.clone());

        // Remove these cells and update indegrees
        for &cell_id in &current_layer {
            indeg.remove(&cell_id);
            if let Some(neighbors) = adjacency.get(&cell_id) {
                for d in neighbors {
                    if let Some(deg) = indeg.get_mut(d) {
                        if *deg > 0 {
                            *deg -= 1;
                        }
                    }
                }
            }
        }
    }

    if !indeg.is_empty() {
        anyhow::bail!(
            "DAG contains cycle or unreachable cells: {} cells remain unscheduled",
            indeg.len()
        );
    }

    Ok(layers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notebook::{CellKind, Notebook};

    #[test]
    fn test_linear_order() {
        let mut nb = Notebook::new("test");
        let id0 = nb.add_cell(CellKind::Python, "x = 5");
        let id1 = nb.add_cell(CellKind::Python, "y = x + 1");
        let id2 = nb.add_cell(CellKind::Python, "z = y * 2");
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id0,
            to_cell_id: id1,
            variables: vec!["x".into()],
        });
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id1,
            to_cell_id: id2,
            variables: vec!["y".into()],
        });

        let order = compute_execution_order(&nb, &[]).unwrap();
        assert_eq!(order, vec![id0, id1, id2]);
    }

    #[test]
    fn test_diamond_order() {
        let mut nb = Notebook::new("test");
        let id0 = nb.add_cell(CellKind::Python, "data = load()");
        let id1 = nb.add_cell(CellKind::Python, "a = process_a(data)");
        let id2 = nb.add_cell(CellKind::Python, "b = process_b(data)");
        let id3 = nb.add_cell(CellKind::Python, "result = merge(a, b)");
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id0,
            to_cell_id: id1,
            variables: vec!["data".into()],
        });
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id0,
            to_cell_id: id2,
            variables: vec!["data".into()],
        });
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id1,
            to_cell_id: id3,
            variables: vec!["a".into()],
        });
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id2,
            to_cell_id: id3,
            variables: vec!["b".into()],
        });

        let order = compute_execution_order(&nb, &[]).unwrap();
        // id0 must be first, id3 must be last
        assert_eq!(order[0], id0);
        assert_eq!(order[order.len() - 1], id3);
        // id1 and id2 can be in either order
        assert!(order.contains(&id1));
        assert!(order.contains(&id2));
    }

    #[test]
    fn test_specific_cells_only_runs_downstream() {
        let mut nb = Notebook::new("test");
        let id0 = nb.add_cell(CellKind::Python, "x = 5");
        let id1 = nb.add_cell(CellKind::Python, "y = x + 1");
        let id2 = nb.add_cell(CellKind::Python, "z = y * 2");
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id0,
            to_cell_id: id1,
            variables: vec!["x".into()],
        });
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id1,
            to_cell_id: id2,
            variables: vec!["y".into()],
        });

        // Only run cell 1 — should also run cell 2 (downstream)
        let order = compute_execution_order(&nb, &[id1]).unwrap();
        assert!(order.contains(&id1));
        assert!(order.contains(&id2));
        assert!(!order.contains(&id0));
    }

    #[test]
    fn test_execution_layers_linear() {
        let mut nb = Notebook::new("test");
        let id0 = nb.add_cell(CellKind::Python, "x = 5");
        let id1 = nb.add_cell(CellKind::Python, "y = x + 1");
        let id2 = nb.add_cell(CellKind::Python, "z = y * 2");
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id0,
            to_cell_id: id1,
            variables: vec!["x".into()],
        });
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id1,
            to_cell_id: id2,
            variables: vec!["y".into()],
        });

        let layers = compute_execution_layers(&nb).unwrap();
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0], vec![id0]);
        assert_eq!(layers[1], vec![id1]);
        assert_eq!(layers[2], vec![id2]);
    }

    #[test]
    fn test_execution_layers_diamond() {
        let mut nb = Notebook::new("test");
        let id0 = nb.add_cell(CellKind::Python, "data = load()");
        let id1 = nb.add_cell(CellKind::Python, "a = process_a(data)");
        let id2 = nb.add_cell(CellKind::Python, "b = process_b(data)");
        let id3 = nb.add_cell(CellKind::Python, "result = merge(a, b)");
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id0,
            to_cell_id: id1,
            variables: vec!["data".into()],
        });
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id0,
            to_cell_id: id2,
            variables: vec!["data".into()],
        });
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id1,
            to_cell_id: id3,
            variables: vec!["a".into()],
        });
        nb.dag.edges.push(crate::notebook::DAGEdge {
            from_cell_id: id2,
            to_cell_id: id3,
            variables: vec!["b".into()],
        });

        let layers = compute_execution_layers(&nb).unwrap();
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0], vec![id0]); // Layer 0: root
                                          // Layer 1: id1 and id2 (parallel)
        assert_eq!(layers[1].len(), 2);
        assert!(layers[1].contains(&id1));
        assert!(layers[1].contains(&id2));
        assert_eq!(layers[2], vec![id3]); // Layer 2: merge
    }
}
