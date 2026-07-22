# ADR-008: R Language Support via Ark

**Status**: Accepted  
**Date**: 2026-07-20  
**Deciders**: OpenNotebook team  

## Context

OpenNotebook must support R for data analysis alongside Python and SQL. Options for R execution in Rust:
1. Build an `extendr` binding from scratch (Rust bindings to R's C API)
2. Use **Ark** — Posit's existing Rust-based R kernel (MIT, 4,600+ commits, production-grade)
3. Use R sessions via Jupyter kernel protocol (ZMQ-based)

## Options Considered

| Option | Pros | Cons |
|---|---|---|
| **Ark** | Production-grade, LSP+DAP built-in, maintained by Posit, handles R's complex C API, MIT license | Designed as a standalone process; embedding API may need work |
| **extendr** | Lightweight, easier to embed | Still requires reimplementing R type marshaling, R C API wrapper, error handling — months of work |
| **Jupyter Kernel** | Standard protocol, works with IRkernel | IPC overhead, no shared memory, ZMQ complexity, two processes |

## Decision

Use **Ark** for R language support.

### Integration Strategy

Ark is designed as a standalone Jupyter kernel, but its architecture supports embedding:

```
Rust Kernel
├── Ark binding (wraps ark::RMain, ark::ROptions)
│   ├── R cell → ark::execute_source(source)
│   ├── R display → ark::last_value_to_arrow() → shared memory
│   └── R LSP → ark::lsp_request(id, params)
└── Shared Arrow memory between R, Python, DuckDB
```

The Ark project at Posit has already done the hard work of:
- Binding R's C API safely in Rust
- Managing R's memory protection (PROTECT/UNPROTECT)
- Handling R's error/condition system (tryCatch)
- Providing LSP server for R
- Providing DAP (Debug Adapter Protocol) for R
- Converting R objects to/from Arrow

We contribute upstream to add an embedding API if needed, rather than duplicating this work.

### Scope for MVP

R cells are **post-MVP** (not in initial Phase C). The MVP ships with Python + SQL only. R support is added in Phase C+.

### Consequences

- Positive: Avoids months of reimplementing R C API bindings.
- Positive: Gets LSP and DAP for free — R users get autocomplete, hover docs, go-to-definition, and debugging.
- Positive: Posit's ongoing maintenance ensures compatibility with R 4.x updates.
- Negative: Ark is designed as a standalone binary — embedding requires extracting its library interface (contribute patches upstream).
- Negative: Ark bundles its own R installation — need to handle R discovery on user's system (system R vs bundled R).
- Negative: R + Python interop via Arrow is less mature than pandas ↔ DuckDB — will need careful testing of type conversions.
