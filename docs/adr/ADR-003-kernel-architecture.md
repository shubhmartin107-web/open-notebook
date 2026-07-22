# ADR-003: Kernel Architecture

**Status**: Accepted  
**Date**: 2026-07-20  
**Deciders**: OpenNotebook team  

## Context

The kernel must:
- Execute cells within a reactive DAG (run only cells whose upstreams changed)
- Support polyglot execution (Python, SQL, R, Markdown)
- Share data across languages (Python ↔ SQL ↔ R via Arrow zero-copy)
- Be fast (sub-ms DAG overhead)
- Be embeddable as a library or runnable as a sidecar process
- Support content-addressed caching for deterministic re-execution

## Decision

Write the kernel in **Rust** with language-specific embedders:

### Architecture

```
┌──────────────────────────────────────────────┐
│                 Rust Kernel                    │
│  ┌────────────────────────────────────────┐  │
│  │ DAG Engine                              │  │
│  │  • syn crate: Python AST analysis       │  │
│  │  • DirectedGraph: defs/refs extraction  │  │
│  │  • Topological sort + cycle detection   │  │
│  │  • Single-assignment enforcement        │  │
│  └────────────────┬───────────────────────┘  │
│                   │ Task Graph                │
│  ┌────────────────┼───────────────────────┐  │
│  │  ┌────────┐  ┌┴───────┐  ┌─────────┐   │  │
│  │  │ PyO3   │  │ DuckDB │  │ Ark     │   │  │
│  │  │ (Py3)  │  │ (SQL)  │  │ (R)     │   │  │
│  │  └────────┘  └────────┘  └─────────┘   │  │
│  │           Content-Addressed Cache        │  │
│  │           Loro CRDT State                │  │
│  └─────────────────────────────────────────┘  │
└──────────────────────────────────────────────┘
```

### Language Runtimes

| Language | Runtime | Rationale |
|---|---|---|
| Python | **PyO3** (Rust bindings to CPython) | In-process, zero-copy Arrow, fastest possible IPC |
| SQL | **DuckDB** (in-process OLAP engine) | Embedded, zero external deps, Arrow-native |
| R | **Ark kernel** (Posit's Rust R runtime) | Production-grade, LSP/DAP built-in, MIT license |
| Markdown | **comrak** (Rust MD parser) | Fast, spec-compliant CommonMark |

### DAG Execution Model

1. **Static analysis phase**: Parse each cell's AST, extract `defs` (declared variables) and `refs` (referenced variables) using `syn` crate for Python, `tree-sitter` for R/SQL.
2. **Graph construction**: Build DirectedGraph where edges are `refs → defs` across cells. Reject on single-assignment violation (same variable defined in >1 cell).
3. **Execution scheduling**: Topological sort → execute in order. Skip cells whose upstream hashes match cached values.
4. **Incremental execution**: On cell edit, recompute only affected sub-DAG (downstream of changed cells).
5. **Cache**: Content-addressed key = hash(cell_source + upstream_cell_content_hashes + cell_arguments). Cache stored in memory + optional LMDB on disk.

### Consequences

- Positive: In-process execution eliminates Jupyter kernel IPC overhead (messages, serialization, ZMQ).
- Positive: Shared memory between PyO3 and DuckDB enables zero-copy dataframe transfers without serialization.
- Positive: Rust provides memory safety + performance for the critical execution path.
- Negative: In-process means a crash in one language runtime can take down the whole kernel (mitigated by separate kernel process per notebook).
- Negative: PyO3 requires linking against specific Python versions (need build matrix for 3.10-3.13).
- Negative: Ark binding is new territory — no existing Rust project has embedded Ark alongside PyO3.
