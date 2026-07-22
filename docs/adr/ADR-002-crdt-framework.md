# ADR-002: CRDT Framework for Real-Time Collaboration

**Status**: Accepted  
**Date**: 2026-07-20  
**Deciders**: OpenNotebook team  

## Context

Real-time collaboration requires:
- Conflict-free merging of concurrent edits across users
- Cell-level operations: add, delete, reorder, edit content
- Awareness (cursor presence, selection highlights)
- Undo/redo per-user without breaking other users' state
- Time-travel debugging (restore notebook to any prior state)

## Options Considered

| Option | Pros | Cons |
|---|---|---|
| **Yjs** | Mature ecosystem, broad adoption, YATA algorithm | WASM-only (no native Rust), no MovableTree, larger bundle |
| **Loro** | Rust-native, MovableTree, RichText, time-travel, Swift/JS/Rust, smaller WASM | Newer ecosystem (v1.0 stable 2026), fewer adapters |
| **Automerge** | Well-researched, Rust bindings | Slower, no MovableTree, no time-travel |
| **Custom OT** | Full control | Extreme complexity, no existing tooling |

## Decision

Use **Loro CRDT** as the collaboration framework.

### Key Binding

The `loro-codemirror` v0.3.0 package provides a `LoroExtensions()` factory for CodeMirror 6 that handles:
- Document state sync (RichText per cell)
- Awareness/Selection sync
- Undo/redo stack per user
- Cursor position broadcasting

### Cell Structure

Use Loro's **MovableTree** container for cell ordering:

```
MovableTree<CellNode>
├── CellNode { id: "abc123", kind: "python", content: RichText }
├── CellNode { id: "def456", kind: "sql", content: RichText }
└── CellNode { id: "ghi789", kind: "markdown", content: RichText }
```

MovableTree handles:
- Drag-and-drop reorder without conflicts
- Insert/delete at any index
- Tree-structured hierarchies (future grouping/folding)

### Architecture

```
User A (Browser)                 User B (Browser)
      │                               │
      │ Loro CRDT ops                  │ Loro CRDT ops
      └───────────────┬───────────────┘
                      │
              Go WebSocket Relay
              (fan-out, no merge)
                      │
              Loro CRDT state
              (in-memory + periodic
               snapshot to .onb)
```

The Go server does NOT merge or transform ops — it simply relays. Loro handles all merge logic client-side.

### Consequences

- Positive: Loro's MovableTree is purpose-built for list reordering — cell drag-and-drop works across users without conflict.
- Positive: `loro-codemirror` is a production-ready binding (v0.3.0 with active maintenance).
- Positive: Rust-native Loro runs in both the kernel (native) and frontend (WASM), allowing kernel-side CRDT validation.
- Positive: Time-travel built-in — can snap a Loro snapshot at any point and restore.
- Negative: Loro's ecosystem is newer than Yjs (but v1.0 stable as of early 2026, FOSDEM 2026 talks confirm production readiness).
- Negative: Team must learn Loro API.
