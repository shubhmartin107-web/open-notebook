use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

use crate::notebook::{Notebook, CellKind};

#[derive(Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Option<JsonValue>,
}

#[derive(Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

impl McpResponse {
    fn success(id: u64, result: JsonValue) -> Self {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: u64, code: i32, message: String) -> Self {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError {
                code,
                message,
                data: None,
            }),
        }
    }
}

#[derive(Serialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<JsonValue>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: JsonValue,
}

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Executor
    }

    pub fn execute_cell_by_index(&self, notebook: &mut Notebook, index: usize) -> Result<JsonValue> {
        let cell_id = notebook.cells.get(index)
            .map(|c| c.id)
            .ok_or_else(|| anyhow::anyhow!("Cell index {} out of bounds", index))?;

        let results = crate::execution::execute_notebook(notebook, &[cell_id.to_string()])?;

        let output = results.get(&cell_id)
            .ok_or_else(|| anyhow::anyhow!("No output returned for cell"))?;

        Ok(serde_json::to_value(output)?)
    }
}

pub async fn serve_mcp(
    notebook: Arc<Mutex<Notebook>>,
    executor: Arc<Mutex<Executor>>,
    port: u16,
) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await
        .with_context(|| format!("Failed to bind to {}", addr))?;

    tracing::info!("MCP server listening on {}", addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        tracing::debug!("MCP connection from {}", peer);

        let notebook = notebook.clone();
        let executor = executor.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, notebook, executor).await {
                tracing::error!("MCP connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    notebook: Arc<Mutex<Notebook>>,
    executor: Arc<Mutex<Executor>>,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: McpRequest = match serde_json::from_str(trimmed) {
            Ok(req) => req,
            Err(e) => {
                let resp = McpResponse::error(0, -32700, format!("Parse error: {}", e));
                let json = serde_json::to_string(&resp)?;
                writer.write_all(json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                continue;
            }
        };

        let id = request.id;
        let response = dispatch(&request.method, request.params, id, &notebook, &executor).await;

        let json = serde_json::to_string(&response)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    Ok(())
}

async fn dispatch(
    method: &str,
    params: Option<JsonValue>,
    id: u64,
    notebook: &Arc<Mutex<Notebook>>,
    executor: &Arc<Mutex<Executor>>,
) -> McpResponse {
    match method {
        "list_tools" => respond(id, handle_list_tools()),
        "read_cells" => respond(id, handle_read_cells(notebook).await),
        "write_cell" => respond(id, handle_write_cell(params, notebook).await),
        "execute_cell" => respond(id, handle_execute_cell(params, notebook, executor).await),
        "get_outputs" => respond(id, handle_get_outputs(params, notebook).await),
        _ => McpResponse::error(id, -32601, format!("Method not found: {}", method)),
    }
}

fn respond(id: u64, result: Result<JsonValue>) -> McpResponse {
    match result {
        Ok(val) => McpResponse::success(id, val),
        Err(e) => McpResponse::error(id, -32603, format!("Internal error: {}", e)),
    }
}

fn handle_list_tools() -> Result<JsonValue> {
    let tools = vec![
        ToolDefinition {
            name: "read_cells".to_string(),
            description: "Read all cells in the notebook with their source and outputs".to_string(),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        },
        ToolDefinition {
            name: "write_cell".to_string(),
            description: "Write source code to a cell at the given index".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "index": {"type": "integer", "description": "Cell index"},
                    "source": {"type": "string", "description": "Cell source code"},
                    "kind": {
                        "type": "string",
                        "enum": ["python", "sql", "markdown"],
                        "description": "Cell kind"
                    }
                },
                "required": ["index", "source", "kind"]
            }),
        },
        ToolDefinition {
            name: "execute_cell".to_string(),
            description: "Execute the cell at the given index".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "index": {"type": "integer", "description": "Cell index"}
                },
                "required": ["index"]
            }),
        },
        ToolDefinition {
            name: "get_outputs".to_string(),
            description: "Get the output of the cell at the given index".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "index": {"type": "integer", "description": "Cell index"}
                },
                "required": ["index"]
            }),
        },
    ];
    Ok(serde_json::to_value(tools)?)
}

async fn handle_read_cells(notebook: &Arc<Mutex<Notebook>>) -> Result<JsonValue> {
    let nb = notebook.lock().await;
    let cells: Vec<JsonValue> = nb.cells.iter().enumerate().map(|(i, cell)| {
        serde_json::json!({
            "index": i,
            "id": cell.id.to_string(),
            "kind": cell.kind.as_str(),
            "source": cell.source,
            "output": cell.output,
            "execution_count": cell.execution_count,
            "status": cell.status.as_str(),
        })
    }).collect();
    Ok(serde_json::to_value(cells)?)
}

async fn handle_write_cell(
    params: Option<JsonValue>,
    notebook: &Arc<Mutex<Notebook>>,
) -> Result<JsonValue> {
    let params = params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
    let index = params.get("index")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'index'"))? as usize;
    let source = params.get("source")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'source'"))?
        .to_string();
    let kind_str = params.get("kind")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'kind'"))?;

    let kind: CellKind = kind_str.parse()
        .map_err(|e: String| anyhow::anyhow!("Invalid cell kind '{}': {}", kind_str, e))?;

    let mut nb = notebook.lock().await;

    if index < nb.cells.len() {
        let cell = &mut nb.cells[index];
        cell.source = source;
        cell.kind = kind;
        cell.output = None;
        cell.execution_count = 0;
        cell.status = crate::notebook::ExecutionStatus::Idle;
    } else {
        nb.add_cell(kind, &source);
    }

    Ok(serde_json::json!({"ok": true}))
}

async fn handle_execute_cell(
    params: Option<JsonValue>,
    notebook: &Arc<Mutex<Notebook>>,
    executor: &Arc<Mutex<Executor>>,
) -> Result<JsonValue> {
    let params = params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
    let index = params.get("index")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'index'"))? as usize;

    let mut nb = notebook.lock().await;
    let exec = executor.lock().await;
    exec.execute_cell_by_index(&mut nb, index)
}

async fn handle_get_outputs(
    params: Option<JsonValue>,
    notebook: &Arc<Mutex<Notebook>>,
) -> Result<JsonValue> {
    let params = params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
    let index = params.get("index")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'index'"))? as usize;

    let nb = notebook.lock().await;
    let cell = nb.cells.get(index)
        .ok_or_else(|| anyhow::anyhow!("Cell index {} out of bounds", index))?;

    Ok(serde_json::to_value(&cell.output)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions() {
        let result = handle_list_tools().unwrap();
        let tools: Vec<ToolDefinition> = serde_json::from_value(result).unwrap();
        assert_eq!(tools.len(), 4);
    }

    #[test]
    fn test_tool_names() {
        let result = handle_list_tools().unwrap();
        let tools: Vec<ToolDefinition> = serde_json::from_value(result).unwrap();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"read_cells"));
        assert!(names.contains(&"write_cell"));
        assert!(names.contains(&"execute_cell"));
        assert!(names.contains(&"get_outputs"));
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
