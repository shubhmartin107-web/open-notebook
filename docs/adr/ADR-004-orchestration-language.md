# ADR-004: Orchestration Server Language

**Status**: Accepted  
**Date**: 2026-07-20  
**Deciders**: OpenNotebook team  

## Context

The orchestration server handles:
- HTTP endpoints for CRUD operations on notebooks/cells
- WebSocket relay for CRDT collaboration ops
- Session lifecycle management (start/stop kernels per notebook)
- File system operations (save, load, list notebooks)
- Authentication (future: OAuth, SSO, API keys)
- User management (future: multi-tenant)

## Options Considered

| Option | Pros | Cons |
|---|---|---|
| **Go** | Excellent concurrency, fast compile, small binary, great HTTP/WS stdlib, single-binary deploy | No generics (pre-1.18 baggage), GC pauses at scale |
| **Rust** | Same language as kernel, maximum performance | Slower compile, more complex async, heavier cognitive load for server logic |
| **Node.js** | Shared lang with frontend, event-driven | Single-threaded, heavy memory, npm dependency risk |
| **Python** | Large ecosystem | GIL, poor concurrency, complex deployment |

## Decision

Use **Go** for the orchestration server.

### Rationale

- Go's goroutines and channels are ideal for WebSocket fan-out (relaying CRDT ops to N connected clients).
- Go's standard library provides everything needed: `net/http`, `net/websocket` (via gorilla/websocket or nhooyr.io/websocket), `encoding/json`, `os/filepath`.
- Single static binary deploy is critical for cross-platform distribution.
- Clean separation of concerns: Go is the "traffic cop", Rust is the "compute engine". Each language does what it does best.
- Go 1.22+ has enhanced routing in stdlib, reducing dependencies.

### Architecture

```
Go Orchestration Server
├── HTTP Router (net/http mux)
│   ├── GET /api/v1/notebooks       → list notebooks
│   ├── POST /api/v1/notebooks      → create notebook
│   ├── GET /api/v1/notebooks/:id   → get notebook (.onb)
│   ├── PUT /api/v1/notebooks/:id   → save notebook
│   ├── DELETE /api/v1/notebooks/:id → delete notebook
│   ├── POST /api/v1/notebooks/:id/execute → execute all/selected cells
│   ├── GET /api/v1/notebooks/:id/dag → get DAG visualization
│   └── GET /api/v1/kernels/status  → kernel health
├── WebSocket Handler (per-notebook)
│   └── Loro CRDT op relay (fan-out)
├── Session Manager
│   └── Per-notebook Rust kernel process lifecycle
├── File Store
│   └── Read/write .onb files to local FS or S3-compatible
└── MCP Gateway (optional)
    └── Route MCP tool calls to kernel
```

### Consequences

- Positive: Clean architectural boundaries — Go is replaceable, kernel is reusable.
- Positive: Go's fast compile and static binaries make CI/CD simple.
- Positive: Goroutine-per-WebSocket model handles 10K+ concurrent connections easily.
- Negative: Need to maintain and version the Go↔Rust IPC protocol (currently gRPC, fallback to stdio JSON).
- Negative: Two languages in the project means two build systems (Go modules + Cargo).
