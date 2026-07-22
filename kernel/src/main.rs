use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "onb-kernel", version, about = "OpenNotebook reactive kernel")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the kernel server (IPC with Go orchestration)
    Serve {
        /// Port for gRPC/stdio communication
        #[arg(long, default_value = "50051")]
        port: u16,

        /// Path to the notebook file (.onb)
        #[arg(long)]
        notebook: Option<PathBuf>,
    },

    /// Execute a notebook file and print results
    Execute {
        /// Path to .onb file
        notebook: PathBuf,

        /// Specific cell IDs to execute (omit for all)
        #[arg(long)]
        cells: Vec<String>,

        /// Print DAG visualization
        #[arg(long)]
        dag: bool,
    },

    /// Analyze a notebook's DAG and print info
    Dag {
        /// Path to .onb file
        notebook: PathBuf,
    },

    /// Export notebook to .onb.md text format
    Export {
        /// Path to .onb file
        notebook: PathBuf,

        /// Output path (defaults to <notebook>.onb.md)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Read a file (text, CSV, or JSON) and print its contents
    ReadFile {
        /// Path to the file
        path: PathBuf,

        /// File format (auto-detect from extension if not specified)
        #[arg(long, value_parser = ["auto", "text", "csv", "json"])]
        format: Option<String>,
    },

    /// Print kernel version info
    Version,

    /// Start MCP server for AI tool access
    Mcp {
        /// Port to listen on
        #[arg(long, default_value = "9876")]
        port: u16,
        /// Path to the notebook file (.onb)
        #[arg(long)]
        notebook: Option<PathBuf>,
    },

    /// AI code generation via Ollama
    Ai {
        /// AI action: generate, explain, debug
        action: String,
        /// User prompt (for generate)
        prompt: String,
        /// Cell kind (python/sql/markdown) for generation
        #[arg(long, default_value = "python")]
        kind: String,
        /// Notebook context file (.onb)
        #[arg(long)]
        notebook: Option<PathBuf>,
        /// Model to use
        #[arg(long)]
        model: Option<String>,
    },
}

fn main() -> Result<()> {
    let _ = color_eyre::install();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { port, notebook } => {
            tracing::info!("Starting kernel server on port {}", port);
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(onb_kernel::server::serve(port, notebook))?;
        }
        Commands::Execute {
            notebook,
            cells,
            dag,
        } => {
            let mut nb = if notebook.exists() {
                onb_kernel::notebook::format::load_from_file(&notebook)?
            } else {
                let mut nb = onb_kernel::notebook::Notebook::new("Untitled");
                nb.add_cell(onb_kernel::notebook::CellKind::Python, "x = 42\nprint(f'x = {x}')");
                nb.add_cell(onb_kernel::notebook::CellKind::Sql, "SELECT 1 AS result");
                nb
            };
            if dag {
                let visual = onb_kernel::dag::visualize(&nb)?;
                println!("{}", visual);
            }
            let results = onb_kernel::execution::execute_notebook(&mut nb, &cells)?;
            onb_kernel::notebook::format::save_to_file(&nb, &notebook)?;
            for (cell_id, output) in &results {
                println!("Cell {}: {}ms", cell_id, output.duration_ms);
                if let Some(err) = &output.error {
                    println!("  Error: {}", err);
                }
            }
        }
        Commands::Dag { notebook } => {
            let nb = onb_kernel::notebook::format::load_from_file(&notebook)?;
            let visual = onb_kernel::dag::visualize(&nb)?;
            println!("{}", visual);
        }
        Commands::Export { notebook, output } => {
            let nb = onb_kernel::notebook::format::load_from_file(&notebook)?;
            let md = onb_kernel::notebook::diff::export_to_markdown(&nb)?;
            let path = output.unwrap_or_else(|| {
                let mut p = notebook.clone();
                p.set_extension("onb.md");
                p
            });
            std::fs::write(&path, md)?;
            println!("Exported to {}", path.display());
        }
        Commands::ReadFile { path, format } => {
            let fmt = format
                .as_deref()
                .unwrap_or("auto")
                .to_lowercase();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let content = match (fmt.as_str(), ext) {
                ("csv", _) | (_, "csv") => onb_kernel::io::read_csv(&path)?,
                ("json", _) | (_, "json") => onb_kernel::io::read_json(&path)?,
                _ => onb_kernel::io::read_text(&path)?,
            };
            println!("{}", content);
        }
        Commands::Mcp { port, notebook } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let nb = if let Some(path) = notebook {
                    onb_kernel::notebook::format::load_from_file(&path)?
                } else {
                    onb_kernel::notebook::Notebook::new("MCP Notebook")
                };
                let notebook = Arc::new(Mutex::new(nb));
                let executor = Arc::new(Mutex::new(onb_kernel::mcp::Executor::new()));
                tracing::info!("MCP server listening on port {}", port);
                onb_kernel::mcp::serve_mcp(notebook, executor, port).await
            })?;
        }

        Commands::Ai {
            action,
            prompt,
            kind,
            notebook,
            model,
        } => {
            if !onb_kernel::ai::detect_ollama() {
                anyhow::bail!(
                    "Ollama is not running at http://localhost:11434.\n\
                     Start it with `ollama serve`"
                );
            }

            let model = model.unwrap_or_else(onb_kernel::ai::default_model);

            let context = if let Some(ref path) = notebook {
                let nb = onb_kernel::notebook::format::load_from_file(path)?;
                nb.cells
                    .iter()
                    .map(|c| c.source.clone())
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            };

            let system_prompt = match action.as_str() {
                "explain" => onb_kernel::ai::prompts::code_explanation(&prompt),
                "debug" => {
                    if notebook.is_some() && !context.is_empty() {
                        onb_kernel::ai::prompts::debug_error(&context, &prompt)
                    } else {
                        onb_kernel::ai::prompts::debug_error("", &prompt)
                    }
                }
                _ => onb_kernel::ai::prompts::code_generation(&context, &kind),
            };

            let result = tokio::runtime::Runtime::new()?
                .block_on(onb_kernel::ai::generate(
                    &model,
                    &system_prompt,
                    &prompt,
                ))?;

            println!("{}", result);
        }

        Commands::Version => {
            println!("OpenNotebook Kernel v{}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
