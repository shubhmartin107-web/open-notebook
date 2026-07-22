# OpenNotebook

A reactive, polyglot computational notebook with DAG execution, local-first AI copilot, and real-time collaboration.

## Architecture

- **Rust kernel** (`kernel/`): DAG engine, code execution (Python/SQL/Markdown), `.onb` format, content-addressed cache, gRPC server
- **Go server** (`server/`): HTTP/WS API, session management, notebook CRUD, kernel IPC, CRDT relay
- **TypeScript frontend** (`frontend/`): React + CodeMirror 6 notebook IDE, rich output rendering, collaboration

## Quick Start

```bash
# Build kernel
cd kernel && cargo build --release

# Start Go server (spawns kernel automatically)
cd server && go run ./cmd/server

# Start frontend dev server
cd frontend && npm run dev
```

## Key Design Decisions

- **Format**: `.onb` (Protobuf binary) + `.onb.md` (git-diffable Markdown)
- **DAG**: Dataflow graph with single-assignment validation, cycle detection, topological sort
- **SQL**: Embedded DuckDB (in-process, Arrow-native)
- **Python**: PyO3 embedding (in-process)
- **Cache**: BLAKE3 content-addressed, invalidated on source/upstream change
- **AI**: Local models via Ollama, no data exfiltration
- **CRDT**: Loro library for real-time collaboration
- **Licensing**: Apache-2.0 (libraries), AGPL-3.0 (network services)

## Project Structure

```
open-notebook/
├── kernel/          # Rust reactive kernel
│   ├── src/
│   │   ├── dag/     # DAG engine (graph, scheduler, variable analysis)
│   │   ├── execution/ # Cell runners (Python, SQL, cache)
│   │   ├── notebook/  # Data types, Protobuf format, Markdown export
│   │   └── server/    # gRPC/stdio IPC server
│   └── Cargo.toml
├── server/          # Go orchestration server
│   ├── cmd/server/  # Entry point
│   ├── internal/    # Handlers, kernel client, notebook store
│   └── go.mod
├── frontend/        # TypeScript + React notebook IDE
│   ├── src/
│   │   ├── components/  # Cell editor, output renderer, notebook cell
│   │   ├── hooks/       # API client
│   │   └── types.ts
│   └── package.json
├── proto/           # Protobuf schema
├── docs/            # PRD, ADRs, API spec, threat model
└── AGENT.md         # This file
```

## Capabilities

| Capability | Status | Notes |
|---|---|---|
| DAG execution (Python) | ✅ | Cycle detection, single-assignment, auto-ordering |
| SQL cells (DuckDB) | ✅ | In-process, Arrow-native |
| `.onb` / `.onb.md` format | ✅ | Protobuf + Markdown |
| Cell types | ✅ | Python, SQL, Markdown, Raw |
| Content-addressed cache | ✅ | BLAKE3 hashing |
| CLI | ✅ | Execute, DAG viz, Export, Serve |
| gRPC/stdio server | ✅ | JSON protocol for Go IPC |
| Go HTTP API | ✅ | Notebook CRUD, cell CRUD, execution |
| React + CM6 frontend | ✅ | Multi-cell editor, output rendering |
| Rich output rendering | ✅ | DataFrame table, text, HTML, PNG |
| File I/O (CSV/Parquet) | ⏳ | Post-MVP |
| AI copilot (Ollama) | ⏳ | Post-MVP |
| CRDT collaboration | ⏳ | Post-MVP |
| R cells (Ark) | ⏳ | Post-MVP |
| MCP server | ⏳ | Post-MVP |
