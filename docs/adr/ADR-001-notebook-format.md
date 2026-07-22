# ADR-001: Notebook Storage Format

**Status**: Accepted  
**Date**: 2026-07-20  
**Deciders**: OpenNotebook team  

## Context

The notebook format must support:
- Lossless round-trip (save → load → save → identical bytes)
- Git-diffable code review (changed lines must be human-readable)
- Rich metadata (cell outputs, execution counts, timestamps, CRDT state)
- Polyglot cells (Python, SQL, R, Markdown)
- CRDT merge state for multi-user collaboration
- Forward compatibility for 10+ years

## Decision

Use a **dual-format representation**:

1. **Binary canonical format**: Protobuf (`.onb`) — single source of truth, schema-enforced, deterministic serialization.
2. **Git-diffable text format**: Markdown-based (`.onb.md`) — generated from binary, human-readable diffs, designed for code review.

### Protobuf Schema (`.onb`)

```protobuf
message Notebook {
  string format_version = 1;       // "onb/v1"
  NotebookMetadata metadata = 2;
  repeated Cell cells = 3;
  bytes crdt_snapshot = 4;         // Loro CRDT snapshot for merge state
}

message Cell {
  string id = 1;                    // UUIDv7
  CellKind kind = 2;
  string source = 3;                // Cell source code
  CellOutput outputs = 4;
  int32 execution_count = 5;
  double last_executed_timestamp = 6;
  map<string, string> tags = 7;     // User-defined metadata
  repeated string upstream_cells = 8;  // DAG edges (deterministic order)
}
```

### Markdown Text Format (`.onb.md`)

The text format is designed for human readability in diffs:

````markdown
# My Notebook

```onb-meta
format_version: onb/v1
created: 2026-07-20T12:00:00Z
```

## Cell: abc123 [python]
```python
import pandas as pd
df = pd.read_csv("data.csv")
df.head()
```
*Output: 5 rows rendered as table*

## Cell: def456 [sql]
```sql
SELECT COUNT(*) FROM df
```
*Output: 42*

## Cell: ghi789 [markdown]
```markdown
# Results Summary
The dataset contains **42** records.
```
````

### Rationale

- **Protobuf over JSON**: Strongly typed, schema-evolution safe, smaller, faster, deterministic. Unlike JSON, Protobuf field order doesn't change serialized output, enabling byte-level reproducibility.
- **Protobuf over flatbuffers/capnproto**: Widest ecosystem support (Rust, Go, TS, Python), mature tooling, Protobuf-es for frontend.
- **Dual format**: Protobuf is the canonical store; Markdown is generated for git. This avoids the mess of YAML/JSON notebooks that can't express binary CRDT state.
- **`.onb.md` structure**: Each cell is a fenced code block with language tag; outputs are Markdown comments. This keeps diffs scoped to individual cells.

### Consequences

- Positive: Human-readable diffs, reviewer can see exactly what changed in each cell.
- Positive: CRDT state stored in binary snapshot alongside human-readable content.
- Positive: Protobuf's backward-compatible field evolution supports 10-year durability.
- Negative: Dual-format means write logic for both (but the diff format is generated, so canonical source is always `.onb`).
- Negative: Markdown representation loses CRDT state (fine for git review — CRDT only matters at runtime).
- Risk: Users may manually edit `.onb.md` and expect it to load — we should validate this (like how Markdown can round-trip).
