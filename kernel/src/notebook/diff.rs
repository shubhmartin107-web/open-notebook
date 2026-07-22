use crate::notebook::types::*;
use anyhow::Result;

/// Export a notebook to the git-diffable `.onb.md` Markdown format.
pub fn export_to_markdown(notebook: &Notebook) -> Result<String> {
    let mut md = String::new();

    // Title
    md.push_str(&format!("# {}\n\n", notebook.metadata.title));

    // Metadata block
    md.push_str("```onb-meta\n");
    md.push_str(&format!("format_version: {}\n", notebook.format_version));
    md.push_str(&format!(
        "created: {}\n",
        format_timestamp(notebook.metadata.created_at_unix_ms)
    ));
    md.push_str(&format!(
        "last_modified: {}\n",
        format_timestamp(notebook.metadata.last_modified_at_unix_ms)
    ));
    if let Some(creator) = &notebook.metadata.created_by {
        md.push_str(&format!("created_by: {}\n", creator));
    }
    md.push_str(&format!("language: {}\n", notebook.metadata.language));
    if !notebook.metadata.tags.is_empty() {
        md.push_str(&format!("tags: {}\n", notebook.metadata.tags.join(", ")));
    }
    md.push_str("```\n\n");

    // Cells
    for cell in &notebook.cells {
        md.push_str(&format!(
            "## Cell: {} [{}]\n\n",
            cell.id, cell.kind.as_str()
        ));

        let lang_tag = match cell.kind {
            CellKind::Python => "python",
            CellKind::Sql => "sql",
            CellKind::R => "r",
            CellKind::Markdown => "markdown",
            CellKind::Raw => "text",
        };

        let source = &cell.source;

        md.push_str(&format!("```{}\n{}\n```\n\n", lang_tag, source));

        // Output
        if let Some(ref output) = cell.output {
            if let Some(ref error) = output.error {
                md.push_str("*Error:*\n\n");
                md.push_str(&format!("```\n{}\n```\n\n", error));
            } else if !output.items.is_empty() {
                let summaries: Vec<String> = output
                    .items
                    .iter()
                    .map(|item| {
                        let mime = &item.mime_type;
                        let preview = item
                            .text
                            .as_deref()
                            .unwrap_or("")
                            .chars()
                            .take(80)
                            .collect::<String>();
                        format!("[{}] {}", mime, preview)
                    })
                    .collect();

                if !summaries.is_empty() {
                    md.push_str(&format!("*Output:* {}\n\n", summaries.join("; ")));
                }
            }

            if output.duration_ms > 0 {
                md.push_str(&format!("*Duration:* {}ms\n\n", output.duration_ms));
            }
        }

        // DAG info
        let upstream: Vec<&str> = notebook
            .dag
            .edges
            .iter()
            .filter(|e| e.to_cell_id == cell.id)
            .map(|_| "")
            .collect();
        if !upstream.is_empty() {
            let var_names: Vec<&str> = notebook
                .dag
                .edges
                .iter()
                .filter(|e| e.to_cell_id == cell.id)
                .flat_map(|e| e.variables.iter().map(|s| s.as_str()))
                .collect();
            if !var_names.is_empty() {
                md.push_str(&format!(
                    "*Depends on:* {}\n\n",
                    var_names.join(", ")
                ));
            }
        }
    }

    Ok(md)
}

fn format_timestamp(unix_ms: i64) -> String {
    let secs = unix_ms / 1000;
    let nanos = ((unix_ms % 1000) * 1_000_000) as u32;
    match chrono::DateTime::from_timestamp(secs, nanos) {
        Some(dt) => dt.to_rfc3339(),
        None => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_empty_notebook() {
        let nb = Notebook::new("Empty Notebook");
        let md = export_to_markdown(&nb).unwrap();
        assert!(md.contains("# Empty Notebook"));
        assert!(md.contains("```onb-meta"));
        assert!(md.contains("format_version: onb/v1"));
    }

    #[test]
    fn test_export_with_cells() {
        let mut nb = Notebook::new("Test Export");
        nb.add_cell(CellKind::Python, "x = 42\nprint(x)");
        nb.add_cell(CellKind::Markdown, "# Hello **World**");
        let md = export_to_markdown(&nb).unwrap();

        assert!(md.contains("[python]"));
        assert!(md.contains("```python"));
        assert!(md.contains("x = 42"));
        assert!(md.contains("[markdown]"));
        assert!(md.contains("Hello **World**"));
    }

    #[test]
    fn test_export_with_output() {
        let mut nb = Notebook::new("Output Test");
        let id = nb.add_cell(CellKind::Python, "print('hello')");
        if let Some(cell) = nb.get_cell_mut(&id) {
            cell.output = Some(CellOutput {
                items: vec![OutputItem {
                    mime_type: "text/plain".to_string(),
                    data: b"hello\n".to_vec(),
                    text: Some("hello".to_string()),
                    render_priority: 0,
                }],
                error: None,
                duration_ms: 5,
            });
        }
        let md = export_to_markdown(&nb).unwrap();
        assert!(md.contains("*Output:*"));
        assert!(md.contains("*Duration:* 5ms"));
    }
}
