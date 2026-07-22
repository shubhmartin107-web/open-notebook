use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct Command {
    cmd: String,
    path: Option<String>,
    notebook: Option<crate::notebook::Notebook>,
    cell_ids: Option<Vec<String>>,
    cell_index: Option<usize>,
    prompt: Option<String>,
}

#[derive(Serialize, Default)]
struct Response {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notebook: Option<crate::notebook::Notebook>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outputs: Option<HashMap<String, crate::notebook::CellOutput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    edges: Option<Vec<crate::notebook::DAGEdge>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visual: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    markdown: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generated_code: Option<String>,
}

impl Response {
    fn ok() -> Self {
        Response { ok: true, ..Default::default() }
    }

    fn err(msg: &str) -> Self {
        Response {
            ok: false,
            error: Some(msg.to_string()),
            ..Default::default()
        }
    }
}

pub async fn serve(port: u16, _notebook_path: Option<PathBuf>) -> Result<()> {
    tracing::info!(
        "OpenNotebook Kernel v{} ready on port {}",
        env!("CARGO_PKG_VERSION"),
        port
    );

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        tracing::debug!("Connection from {}", peer);

        tokio::spawn(async move {
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(_) => break,
                };

                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let cmd: Command = match serde_json::from_str(trimmed) {
                    Ok(c) => c,
                    Err(e) => {
                        let resp = Response::err(&format!("Invalid JSON: {}", e));
                        let json = serde_json::to_string(&resp).unwrap_or_default();
                        let _ = writer.write_all(json.as_bytes()).await;
                        let _ = writer.write_all(b"\n").await;
                        continue;
                    }
                };

                if cmd.cmd == "shutdown" {
                    break;
                }

                let resp = match cmd.cmd.as_str() {
                    "ping" => Response {
                        ok: true,
                        version: Some(env!("CARGO_PKG_VERSION").to_string()),
                        ..Default::default()
                    },

                    "load" => {
                        let path = match cmd.path {
                            Some(p) => p,
                            None => {
                                let resp = Response::err("Missing 'path' field");
                                let json = serde_json::to_string(&resp).unwrap_or_default();
                                let _ = writer.write_all(json.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                                continue;
                            }
                        };
                        match crate::notebook::format::load_from_file(std::path::Path::new(&path)) {
                            Ok(nb) => Response {
                                ok: true,
                                notebook: Some(nb),
                                ..Default::default()
                            },
                            Err(e) => Response::err(&format!("{}", e)),
                        }
                    }

                    "save" => {
                        let path = match cmd.path {
                            Some(p) => p,
                            None => {
                                let resp = Response::err("Missing 'path' field");
                                let json = serde_json::to_string(&resp).unwrap_or_default();
                                let _ = writer.write_all(json.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                                continue;
                            }
                        };
                        let nb = match cmd.notebook {
                            Some(n) => n,
                            None => {
                                let resp = Response::err("Missing 'notebook' field");
                                let json = serde_json::to_string(&resp).unwrap_or_default();
                                let _ = writer.write_all(json.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                                continue;
                            }
                        };
                        match crate::notebook::format::save_to_file(&nb, std::path::Path::new(&path)) {
                            Ok(()) => Response::ok(),
                            Err(e) => Response::err(&format!("{}", e)),
                        }
                    }

                    "execute" => {
                        let mut nb = match cmd.notebook {
                            Some(n) => n,
                            None => {
                                let resp = Response::err("Missing 'notebook' field");
                                let json = serde_json::to_string(&resp).unwrap_or_default();
                                let _ = writer.write_all(json.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                                continue;
                            }
                        };
                        let cell_ids = cmd.cell_ids.unwrap_or_default();
                        match crate::execution::execute_notebook(&mut nb, &cell_ids) {
                            Ok(outputs) => {
                                let outputs_map: HashMap<String, crate::notebook::CellOutput> = outputs
                                    .into_iter()
                                    .map(|(k, v)| (k.to_string(), v))
                                    .collect();
                                Response {
                                    ok: true,
                                    outputs: Some(outputs_map),
                                    ..Default::default()
                                }
                            }
                            Err(e) => Response::err(&format!("{}", e)),
                        }
                    }

                    "dag" => {
                        let mut nb = match cmd.notebook {
                            Some(n) => n,
                            None => {
                                let resp = Response::err("Missing 'notebook' field");
                                let json = serde_json::to_string(&resp).unwrap_or_default();
                                let _ = writer.write_all(json.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                                continue;
                            }
                        };
                        match crate::dag::build_dag(&mut nb) {
                            Ok(()) => {
                                let visual = crate::dag::visualize(&nb).unwrap_or_default();
                                let edges = nb.dag.edges.clone();
                                Response {
                                    ok: true,
                                    edges: Some(edges),
                                    visual: Some(visual),
                                    ..Default::default()
                                }
                            }
                            Err(e) => Response::err(&format!("{}", e)),
                        }
                    }

                    "export_md" => {
                        let nb = match cmd.notebook {
                            Some(n) => n,
                            None => {
                                let resp = Response::err("Missing 'notebook' field");
                                let json = serde_json::to_string(&resp).unwrap_or_default();
                                let _ = writer.write_all(json.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                                continue;
                            }
                        };
                        Response {
                            ok: true,
                            markdown: Some(export_to_markdown(&nb)),
                            ..Default::default()
                        }
                    }

                    "ai_generate" => {
                        if !crate::ai::detect_ollama() {
                            Response::err("Ollama is not running at http://localhost:11434")
                        } else {
                            let model = crate::ai::default_model();
                            let context = cmd.notebook.as_ref().map(|nb| {
                                nb.cells
                                    .iter()
                                    .map(|c| c.source.clone())
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }).unwrap_or_default();
                            let user_prompt = cmd.prompt.unwrap_or_default();
                            let cell_kind = cmd.notebook.as_ref()
                                .and_then(|nb| cmd.cell_index.and_then(|i| nb.cells.get(i)))
                                .map(|c| c.kind.as_str())
                                .unwrap_or("python");
                            let system_prompt =
                                crate::ai::prompts::code_generation(&context, cell_kind);

                            let rt = match tokio::runtime::Runtime::new() {
                                Ok(rt) => rt,
                                Err(e) => {
                                    let resp = Response::err(&format!("Failed to create runtime: {}", e));
                                    let json = serde_json::to_string(&resp).unwrap_or_default();
                                    let _ = writer.write_all(json.as_bytes()).await;
                                    let _ = writer.write_all(b"\n").await;
                                    continue;
                                }
                            };
                            match rt.block_on(crate::ai::generate(
                                &model,
                                &system_prompt,
                                &user_prompt,
                            )) {
                                Ok(code) => Response {
                                    ok: true,
                                    generated_code: Some(code),
                                    ..Default::default()
                                },
                                Err(e) => {
                                    Response::err(&format!("AI generation failed: {}", e))
                                }
                            }
                        }
                    }

                    other => Response::err(&format!("Unknown command: {}", other)),
                };

                let json = serde_json::to_string(&resp).unwrap_or_default();
                let _ = writer.write_all(json.as_bytes()).await;
                let _ = writer.write_all(b"\n").await;
            }
        });
    }
}

pub fn serve_stdio() -> Result<()> {
    tracing::info!(
        "OpenNotebook Kernel v{} (stdio mode)",
        env!("CARGO_PKG_VERSION")
    );

    let stdin = std::io::stdin();
    let mut line = String::new();

    loop {
        line.clear();
        let n = stdin.read_line(&mut line)?;
        if n == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let cmd: Command = match serde_json::from_str(trimmed) {
            Ok(c) => c,
            Err(e) => {
                let resp = Response::err(&format!("Invalid JSON: {}", e));
                println!("{}", serde_json::to_string(&resp)?);
                continue;
            }
        };

        let resp = match cmd.cmd.as_str() {
            "shutdown" => break,

            "ping" => Response {
                ok: true,
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                ..Default::default()
            },

            "load" => {
                let path = match cmd.path {
                    Some(p) => p,
                    None => {
                        println!(
                            "{}",
                            serde_json::to_string(&Response::err("Missing 'path' field"))?
                        );
                        continue;
                    }
                };
                match crate::notebook::format::load_from_file(std::path::Path::new(&path)) {
                    Ok(nb) => Response {
                        ok: true,
                        notebook: Some(nb),
                        ..Default::default()
                    },
                    Err(e) => Response::err(&format!("{}", e)),
                }
            }

            "save" => {
                let path = match cmd.path {
                    Some(p) => p,
                    None => {
                        println!(
                            "{}",
                            serde_json::to_string(&Response::err("Missing 'path' field"))?
                        );
                        continue;
                    }
                };
                let nb = match cmd.notebook {
                    Some(n) => n,
                    None => {
                        println!(
                            "{}",
                            serde_json::to_string(&Response::err("Missing 'notebook' field"))?
                        );
                        continue;
                    }
                };
                match crate::notebook::format::save_to_file(&nb, std::path::Path::new(&path)) {
                    Ok(()) => Response::ok(),
                    Err(e) => Response::err(&format!("{}", e)),
                }
            }

            "execute" => {
                let mut nb = match cmd.notebook {
                    Some(n) => n,
                    None => {
                        println!(
                            "{}",
                            serde_json::to_string(&Response::err("Missing 'notebook' field"))?
                        );
                        continue;
                    }
                };
                let cell_ids = cmd.cell_ids.unwrap_or_default();
                match crate::execution::execute_notebook(&mut nb, &cell_ids) {
                    Ok(outputs) => {
                        let outputs_map: HashMap<String, crate::notebook::CellOutput> = outputs
                            .into_iter()
                            .map(|(k, v)| (k.to_string(), v))
                            .collect();
                        Response {
                            ok: true,
                            outputs: Some(outputs_map),
                            ..Default::default()
                        }
                    }
                    Err(e) => Response::err(&format!("{}", e)),
                }
            }

            "dag" => {
                let mut nb = match cmd.notebook {
                    Some(n) => n,
                    None => {
                        println!(
                            "{}",
                            serde_json::to_string(&Response::err("Missing 'notebook' field"))?
                        );
                        continue;
                    }
                };
                match crate::dag::build_dag(&mut nb) {
                    Ok(()) => {
                        let visual = crate::dag::visualize(&nb).unwrap_or_default();
                        let edges = nb.dag.edges.clone();
                        Response {
                            ok: true,
                            edges: Some(edges),
                            visual: Some(visual),
                            ..Default::default()
                        }
                    }
                    Err(e) => Response::err(&format!("{}", e)),
                }
            }

            "export_md" => {
                let nb = match cmd.notebook {
                    Some(n) => n,
                    None => {
                        println!(
                            "{}",
                            serde_json::to_string(&Response::err("Missing 'notebook' field"))?
                        );
                        continue;
                    }
                };
                Response {
                    ok: true,
                    markdown: Some(export_to_markdown(&nb)),
                    ..Default::default()
                }
            }

            "ai_generate" => {
                if !crate::ai::detect_ollama() {
                    Response::err("Ollama is not running at http://localhost:11434")
                } else {
                    let model = crate::ai::default_model();
                    let context = cmd.notebook.as_ref().map(|nb| {
                        nb.cells
                            .iter()
                            .map(|c| c.source.clone())
                            .collect::<Vec<_>>()
                            .join("\n")
                    }).unwrap_or_default();
                    let user_prompt = cmd.prompt.unwrap_or_default();
                    let cell_kind = cmd.notebook.as_ref()
                        .and_then(|nb| cmd.cell_index.and_then(|i| nb.cells.get(i)))
                        .map(|c| c.kind.as_str())
                        .unwrap_or("python");
                    let system_prompt =
                        crate::ai::prompts::code_generation(&context, cell_kind);

                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt,
                        Err(e) => {
                            println!(
                                "{}",
                                serde_json::to_string(&Response::err(
                                    &format!("Failed to create runtime: {}", e)
                                ))?
                            );
                            continue;
                        }
                    };
                    match rt.block_on(crate::ai::generate(
                        &model,
                        &system_prompt,
                        &user_prompt,
                    )) {
                        Ok(code) => Response {
                            ok: true,
                            generated_code: Some(code),
                            ..Default::default()
                        },
                        Err(e) => {
                            Response::err(&format!("AI generation failed: {}", e))
                        }
                    }
                }
            }

            other => Response::err(&format!("Unknown command: {}", other)),
        };

        println!("{}", serde_json::to_string(&resp)?);
    }

    Ok(())
}

pub fn export_to_markdown(nb: &crate::notebook::Notebook) -> String {
    let mut md = String::new();
    md.push_str(&format!("# {}\n\n", nb.metadata.title));
    if !nb.metadata.description.is_empty() {
        md.push_str(&nb.metadata.description);
        md.push_str("\n\n");
    }
    for cell in &nb.cells {
        match cell.kind {
            crate::notebook::CellKind::Markdown => {
                md.push_str(&cell.source);
                md.push_str("\n\n");
            }
            crate::notebook::CellKind::Python => {
                md.push_str("```python\n");
                md.push_str(&cell.source);
                if !cell.source.ends_with('\n') {
                    md.push('\n');
                }
                md.push_str("```\n\n");
            }
            crate::notebook::CellKind::Sql => {
                md.push_str("```sql\n");
                md.push_str(&cell.source);
                if !cell.source.ends_with('\n') {
                    md.push('\n');
                }
                md.push_str("```\n\n");
            }
            crate::notebook::CellKind::R => {
                md.push_str("```r\n");
                md.push_str(&cell.source);
                if !cell.source.ends_with('\n') {
                    md.push('\n');
                }
                md.push_str("```\n\n");
            }
            crate::notebook::CellKind::Raw => {
                md.push_str("```\n");
                md.push_str(&cell.source);
                if !cell.source.ends_with('\n') {
                    md.push('\n');
                }
                md.push_str("```\n\n");
            }
        }
    }
    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notebook::{CellKind, Notebook};

    #[test]
    fn test_export_markdown() {
        let mut nb = Notebook::new("Test Notebook");
        nb.metadata.description = "A description".to_string();
        nb.add_cell(CellKind::Markdown, "# Hello");
        nb.add_cell(CellKind::Python, "x = 1\nprint(x)");
        nb.add_cell(CellKind::Sql, "SELECT * FROM t");

        let md = export_to_markdown(&nb);
        assert!(md.contains("# Test Notebook"));
        assert!(md.contains("A description"));
        assert!(md.contains("# Hello"));
        assert!(md.contains("```python"));
        assert!(md.contains("x = 1\nprint(x)"));
        assert!(md.contains("```sql"));
        assert!(md.contains("SELECT * FROM t"));
    }
}
