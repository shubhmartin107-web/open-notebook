use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CellKind {
    Python,
    Sql,
    R,
    Markdown,
    Raw,
}

impl CellKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CellKind::Python => "python",
            CellKind::Sql => "sql",
            CellKind::R => "r",
            CellKind::Markdown => "markdown",
            CellKind::Raw => "raw",
        }
    }
}

impl std::str::FromStr for CellKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "python" => Ok(CellKind::Python),
            "sql" => Ok(CellKind::Sql),
            "r" => Ok(CellKind::R),
            "markdown" => Ok(CellKind::Markdown),
            "raw" => Ok(CellKind::Raw),
            _ => Err(format!("unknown cell kind: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Idle,
    Running,
    Success,
    Error,
    Queued,
    Cancelled,
}

impl ExecutionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionStatus::Idle => "idle",
            ExecutionStatus::Running => "running",
            ExecutionStatus::Success => "success",
            ExecutionStatus::Error => "error",
            ExecutionStatus::Queued => "queued",
            ExecutionStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: Uuid,
    pub kind: CellKind,
    pub source: String,
    pub output: Option<CellOutput>,
    pub execution_count: i32,
    pub status: ExecutionStatus,
    pub collapsed: CellCollapseState,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CellCollapseState {
    #[default]
    Expanded,
    Collapsed,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellOutput {
    pub items: Vec<OutputItem>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputItem {
    pub mime_type: String,
    pub data: Vec<u8>,
    pub text: Option<String>,
    pub render_priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAGEdge {
    pub from_cell_id: Uuid,
    pub to_cell_id: Uuid,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAG {
    pub edges: Vec<DAGEdge>,
}

impl DAG {
    pub fn new() -> Self {
        DAG { edges: Vec::new() }
    }
}

impl Default for DAG {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookMetadata {
    pub title: String,
    pub description: String,
    pub created_by: Option<String>,
    pub created_at_unix_ms: i64,
    pub last_modified_at_unix_ms: i64,
    pub language: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub format_version: String,
    pub metadata: NotebookMetadata,
    pub cells: Vec<Cell>,
    pub dag: DAG,
    pub crdt_snapshot: Option<Vec<u8>>,
}

impl Notebook {
    pub fn new(title: &str) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Notebook {
            format_version: "onb/v1".to_string(),
            metadata: NotebookMetadata {
                title: title.to_string(),
                description: String::new(),
                created_by: None,
                created_at_unix_ms: now,
                last_modified_at_unix_ms: now,
                language: "python".to_string(),
                tags: Vec::new(),
            },
            cells: Vec::new(),
            dag: DAG::new(),
            crdt_snapshot: None,
        }
    }

    pub fn add_cell(&mut self, kind: CellKind, source: &str) -> Uuid {
        let id = Uuid::now_v7();
        let cell = Cell {
            id,
            kind,
            source: source.to_string(),
            output: None,
            execution_count: 0,
            status: ExecutionStatus::Idle,
            collapsed: CellCollapseState::Expanded,
            metadata: HashMap::new(),
        };
        self.cells.push(cell);
        id
    }

    pub fn get_cell(&self, id: &Uuid) -> Option<&Cell> {
        self.cells.iter().find(|c| c.id == *id)
    }

    pub fn get_cell_mut(&mut self, id: &Uuid) -> Option<&mut Cell> {
        self.cells.iter_mut().find(|c| c.id == *id)
    }

    pub fn remove_cell(&mut self, id: &Uuid) -> Option<Cell> {
        if let Some(pos) = self.cells.iter().position(|c| c.id == *id) {
            let cell = self.cells.remove(pos);
            self.dag.edges.retain(|e| {
                e.from_cell_id != *id && e.to_cell_id != *id
            });
            Some(cell)
        } else {
            None
        }
    }
}
