# OpenNotebook

An AI-native, open-source reactive computational notebook and data IDE — like Marimo + Deepnote + Hex, but 100% free and local-first.

## Architecture

```
┌────────────────┐     ┌──────────────┐     ┌──────────────┐
│  React/TS      │────▶│  Go Server   │────▶│  Rust Kernel │
│  (CodeMirror 6)│     │  (REST API)  │     │  (DAG + Exec)│
└────────────────┘     └──────────────┘     └──────┬───────┘
                                                    │
                                           ┌────────▼────────┐
                                           │  Ollama (local)  │
                                           │  AI Copilot      │
                                           └─────────────────┘
```

- **Rust Kernel**: Reactive DAG engine, Python (PyO3) + SQL (DuckDB) execution, .onb protobuf format, AI copilot via Ollama, MCP server
- **Go Server**: REST API for notebook CRUD, cell execution, DAG analysis, kernel subprocess management
- **TypeScript Frontend**: React 19 + CodeMirror 6 notebook IDE with multi-language cells, rich output rendering, AI chat panel

## Quick Start

```bash
# 1. Build the kernel
cd kernel && cargo build --release

# 2. Start the Go server (requires kernel binary in PATH)
cd ../server && KERNEL_PATH=../kernel/target/release/onb-kernel go run ./cmd/server

# 3. Start the frontend
cd ../frontend && npm install && npm run dev
```

Open http://localhost:5173 in your browser.

## Requirements

- **Rust** 1.85+
- **Go** 1.24+
- **Node.js** 20+
- **Python** 3.10–3.13 (with `libpython3.12-dev` for PyO3)
- **Protobuf** compiler (`protoc`)
- **Ollama** (optional, for AI copilot)

## License

Dual-licensed: Apache 2.0 (core libraries) / AGPL-3.0 (network services). See LICENSE and ADR-005 for details.
