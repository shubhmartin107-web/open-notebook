use crate::dag::variable_table;
use crate::notebook::{CellOutput, Notebook};
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

/// Compute a content-addressed cache key for a cell.
///
/// The cache key is BLAKE3 hash of:
/// - cell source code
/// - upstream cell content hashes (recursive dependency tracking)
/// - cell kind
pub fn compute_cache_key(notebook: &Notebook, cell_id: &Uuid) -> Result<blake3::Hash> {
    let cell = notebook
        .get_cell(cell_id)
        .ok_or_else(|| anyhow::anyhow!("Cell not found: {}", cell_id))?;

    let mut hasher = blake3::Hasher::new();

    // Hash the cell source
    hasher.update(cell.source.as_bytes());

    // Hash the cell kind
    hasher.update(cell.kind.as_str().as_bytes());

    // Hash upstream cell content hashes
    let upstream_ids = find_upstream_cells(notebook, cell_id);
    for up_id in &upstream_ids {
        if let Some(up_cell) = notebook.get_cell(up_id) {
            if let Some(ref output) = up_cell.output {
                // Hash the upstream output
                hasher.update(&output.duration_ms.to_le_bytes());
                if let Some(err) = &output.error {
                    hasher.update(err.as_bytes());
                }
                for item in &output.items {
                    hasher.update(&item.data);
                }
            }
        }
    }

    Ok(hasher.finalize())
}

/// Find cells that are upstream dependencies (the cell references their variables).
fn find_upstream_cells(notebook: &Notebook, cell_id: &Uuid) -> Vec<Uuid> {
    let mut upstream = Vec::new();

    if let Some(cell) = notebook.get_cell(cell_id) {
        if let Ok(refs) = variable_table::extract_refs(&cell.source, &cell.kind) {
            for other in &notebook.cells {
                if other.id == *cell_id {
                    continue;
                }
                if let Ok(defs) = variable_table::extract_defs(&other.source, &other.kind) {
                    if refs.iter().any(|r| defs.contains(r)) {
                        upstream.push(other.id);
                    }
                }
            }
        }
    }

    upstream
}

// ── In-memory cache ──

use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref CACHE: Mutex<HashMap<blake3::Hash, CellOutput>> =
        Mutex::new(HashMap::new());
}

/// Try to get a cached output for a cell.
pub fn get_cached_output(notebook: &Notebook, cell_id: &Uuid) -> Result<Option<CellOutput>> {
    let key = compute_cache_key(notebook, cell_id)?;
    let cache = CACHE.lock().unwrap();
    Ok(cache.get(&key).cloned())
}

/// Store a cell output in the cache.
pub fn set_cached_output(
    notebook: &Notebook,
    cell_id: &Uuid,
    output: &CellOutput,
) -> Result<()> {
    let key = compute_cache_key(notebook, cell_id)?;
    let mut cache = CACHE.lock().unwrap();
    cache.insert(key, output.clone());
    Ok(())
}

/// Clear the entire cache.
pub fn clear_cache() {
    let mut cache = CACHE.lock().unwrap();
    cache.clear();
}

/// Get cache hit/miss statistics.
pub fn cache_stats() -> (usize, usize) {
    let cache = CACHE.lock().unwrap();
    (cache.len(), 0) // (entries, misses) — miss tracking TBD
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notebook::{CellKind, Notebook};

    #[test]
    fn test_same_source_same_key() {
        let mut nb = Notebook::new("test");
        let id = nb.add_cell(CellKind::Python, "x = 5");
        let key1 = compute_cache_key(&nb, &id).unwrap();
        let key2 = compute_cache_key(&nb, &id).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_source_different_key() {
        let mut nb = Notebook::new("test");
        let id1 = nb.add_cell(CellKind::Python, "x = 5");
        let id2 = nb.add_cell(CellKind::Python, "y = 10");

        let key1 = compute_cache_key(&nb, &id1).unwrap();
        let key2 = compute_cache_key(&nb, &id2).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_roundtrip() {
        let mut nb = Notebook::new("test");
        let id = nb.add_cell(CellKind::Python, "x = 5");

        let output = CellOutput {
            items: vec![],
            error: None,
            duration_ms: 42,
        };

        set_cached_output(&nb, &id, &output).unwrap();
        let cached = get_cached_output(&nb, &id).unwrap();

        assert!(cached.is_some());
        assert_eq!(cached.unwrap().duration_ms, 42);
    }
}
