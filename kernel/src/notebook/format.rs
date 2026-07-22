use crate::notebook::types::*;
use anyhow::{Context, Result};
use std::path::Path;

/// Load a notebook from a `.onb` Protobuf file.
pub fn load_from_file(path: &Path) -> Result<Notebook> {
    let data = std::fs::read(path).with_context(|| {
        format!("Failed to read notebook file: {}", path.display())
    })?;

    let proto: crate::proto::Notebook = prost::Message::decode(data.as_slice())?;
    notebook_from_proto(proto)
}

/// Save a notebook to a `.onb` Protobuf file.
pub fn save_to_file(notebook: &Notebook, path: &Path) -> Result<()> {
    let proto = notebook_to_proto(notebook);
    let data = prost::Message::encode_to_vec(&proto);
    std::fs::write(path, &data).with_context(|| {
        format!("Failed to write notebook file: {}", path.display())
    })?;
    Ok(())
}

pub fn notebook_from_proto(proto: crate::proto::Notebook) -> Result<Notebook> {
    let meta = proto.metadata.ok_or_else(|| anyhow::anyhow!("Missing notebook metadata"))?;

    let cells: Result<Vec<Cell>> = proto.cells.into_iter().map(cell_from_proto).collect();

    let dag = proto.dag.map(dag_from_proto).unwrap_or_default();

    Ok(Notebook {
        format_version: proto.format_version,
        metadata: NotebookMetadata {
            title: meta.title,
            description: meta.description,
            created_by: if meta.created_by.is_empty() { None } else { Some(meta.created_by) },
            created_at_unix_ms: meta.created_at_unix_ms,
            last_modified_at_unix_ms: meta.last_modified_at_unix_ms,
            language: meta.language,
            tags: meta.tags,
        },
        cells: cells?,
        dag,
        crdt_snapshot: if proto.crdt_snapshot.is_empty() {
            None
        } else {
            Some(proto.crdt_snapshot)
        },
    })
}

fn notebook_to_proto(notebook: &Notebook) -> crate::proto::Notebook {
    crate::proto::Notebook {
        format_version: notebook.format_version.clone(),
        metadata: Some(crate::proto::NotebookMetadata {
            title: notebook.metadata.title.clone(),
            description: notebook.metadata.description.clone(),
            created_by: notebook.metadata.created_by.clone().unwrap_or_default(),
            created_at_unix_ms: notebook.metadata.created_at_unix_ms,
            last_modified_at_unix_ms: notebook.metadata.last_modified_at_unix_ms,
            language: notebook.metadata.language.clone(),
            custom: std::collections::HashMap::new(),
            kernel_version: env!("CARGO_PKG_VERSION").to_string(),
            tags: notebook.metadata.tags.clone(),
        }),
        cells: notebook.cells.iter().map(cell_to_proto).collect(),
        crdt_snapshot: notebook.crdt_snapshot.clone().unwrap_or_default(),
        dag: Some(dag_to_proto(&notebook.dag)),
    }
}

fn cell_from_proto(proto: crate::proto::Cell) -> Result<Cell> {
    let id = uuid::Uuid::parse_str(&proto.id)
        .map_err(|e| anyhow::anyhow!("Invalid cell UUID '{}': {}", proto.id, e))?;

    let kind = match proto.kind {
        k if k == crate::proto::CellKind::Python as i32 => CellKind::Python,
        k if k == crate::proto::CellKind::Sql as i32 => CellKind::Sql,
        k if k == crate::proto::CellKind::R as i32 => CellKind::R,
        k if k == crate::proto::CellKind::Markdown as i32 => CellKind::Markdown,
        k if k == crate::proto::CellKind::Raw as i32 => CellKind::Raw,
        _ => CellKind::Python,
    };

    let status = match proto.status {
        k if k == crate::proto::ExecutionStatus::Success as i32 => ExecutionStatus::Success,
        k if k == crate::proto::ExecutionStatus::Error as i32 => ExecutionStatus::Error,
        k if k == crate::proto::ExecutionStatus::Running as i32 => ExecutionStatus::Running,
        k if k == crate::proto::ExecutionStatus::Queued as i32 => ExecutionStatus::Queued,
        k if k == crate::proto::ExecutionStatus::Cancelled as i32 => ExecutionStatus::Cancelled,
        _ => ExecutionStatus::Idle,
    };

    let output = proto.output.map(output_from_proto);

    Ok(Cell {
        id,
        kind,
        source: proto.source,
        output,
        execution_count: proto.execution_count,
        status,
        collapsed: match proto.collapsed.as_str() {
            "collapsed" => CellCollapseState::Collapsed,
            "hidden" => CellCollapseState::Hidden,
            _ => CellCollapseState::Expanded,
        },
        metadata: proto.metadata,
    })
}

fn cell_to_proto(cell: &Cell) -> crate::proto::Cell {
    use crate::proto::CellKind as P;
    let kind = match cell.kind {
        CellKind::Python => P::Python as i32,
        CellKind::Sql => P::Sql as i32,
        CellKind::R => P::R as i32,
        CellKind::Markdown => P::Markdown as i32,
        CellKind::Raw => P::Raw as i32,
    };

    use crate::proto::ExecutionStatus as E;
    let status = match cell.status {
        ExecutionStatus::Idle => E::Idle as i32,
        ExecutionStatus::Running => E::Running as i32,
        ExecutionStatus::Success => E::Success as i32,
        ExecutionStatus::Error => E::Error as i32,
        ExecutionStatus::Queued => E::Queued as i32,
        ExecutionStatus::Cancelled => E::Cancelled as i32,
    };

    let collapsed = match cell.collapsed {
        CellCollapseState::Collapsed => "collapsed".to_string(),
        CellCollapseState::Hidden => "hidden".to_string(),
        CellCollapseState::Expanded => "expanded".to_string(),
    };

    crate::proto::Cell {
        id: cell.id.to_string(),
        kind,
        source: cell.source.clone(),
        output: cell.output.as_ref().map(output_to_proto),
        execution_count: cell.execution_count,
        status,
        last_executed_at_unix_ms: 0,
        metadata: cell.metadata.clone(),
        collapsed,
    }
}

fn output_from_proto(proto: crate::proto::CellOutput) -> CellOutput {
    CellOutput {
        items: proto
            .items
            .into_iter()
            .map(|item| OutputItem {
                mime_type: item.mime_type,
                data: item.data,
                text: if item.text.is_empty() { None } else { Some(item.text) },
                render_priority: item.render_priority,
            })
            .collect(),
        error: if proto.error_traceback.is_empty() {
            None
        } else {
            Some(proto.error_traceback)
        },
        duration_ms: proto.duration_ms as u64,
    }
}

fn output_to_proto(output: &CellOutput) -> crate::proto::CellOutput {
    crate::proto::CellOutput {
        items: output
            .items
            .iter()
            .map(|item| crate::proto::OutputItem {
                mime_type: item.mime_type.clone(),
                data: item.data.clone(),
                text: item.text.clone().unwrap_or_default(),
                render_priority: item.render_priority,
                metadata: std::collections::HashMap::new(),
            })
            .collect(),
        error_traceback: output.error.clone().unwrap_or_default(),
        duration_ms: output.duration_ms as i64,
    }
}

fn dag_from_proto(proto: crate::proto::Dag) -> DAG {
    DAG {
        edges: proto
            .edges
            .into_iter()
            .map(|e| {
                let from = uuid::Uuid::parse_str(&e.from_cell_id).unwrap_or_else(|_| uuid::Uuid::nil());
                let to = uuid::Uuid::parse_str(&e.to_cell_id).unwrap_or_else(|_| uuid::Uuid::nil());
                DAGEdge {
                    from_cell_id: from,
                    to_cell_id: to,
                    variables: e.variables,
                }
            })
            .collect(),
    }
}

fn dag_to_proto(dag: &DAG) -> crate::proto::Dag {
    crate::proto::Dag {
        edges: dag
            .edges
            .iter()
            .map(|e| crate::proto::DagEdge {
                from_cell_id: e.from_cell_id.to_string(),
                to_cell_id: e.to_cell_id.to_string(),
                variables: e.variables.clone(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_empty_notebook() {
        let nb = Notebook::new("test notebook");
        let bytes = prost::Message::encode_to_vec(&notebook_to_proto(&nb));
        let decoded = notebook_from_proto(
            prost::Message::decode(bytes.as_slice()).unwrap(),
        ).unwrap();

        assert_eq!(nb.metadata.title, decoded.metadata.title);
        assert_eq!(nb.cells.len(), decoded.cells.len());
        assert_eq!(nb.format_version, decoded.format_version);
    }

    #[test]
    fn test_roundtrip_with_cells() {
        let mut nb = Notebook::new("test");
        nb.add_cell(CellKind::Python, "x = 42");
        nb.add_cell(CellKind::Sql, "SELECT 1");
        nb.add_cell(CellKind::Markdown, "# Hello");

        let bytes = prost::Message::encode_to_vec(&notebook_to_proto(&nb));
        let decoded = notebook_from_proto(
            prost::Message::decode(bytes.as_slice()).unwrap(),
        ).unwrap();

        assert_eq!(nb.cells.len(), decoded.cells.len());
        assert_eq!(nb.cells[0].source, decoded.cells[0].source);
        assert_eq!(nb.cells[1].kind, decoded.cells[1].kind);
    }

    #[test]
    fn test_file_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.onb");

        let mut nb = Notebook::new("file test");
        nb.add_cell(CellKind::Python, "print('hello')");

        save_to_file(&nb, &path).unwrap();
        let loaded = load_from_file(&path).unwrap();

        assert_eq!(loaded.metadata.title, "file test");
        assert_eq!(loaded.cells.len(), 1);
    }
}
