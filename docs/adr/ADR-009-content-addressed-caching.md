# ADR-009: Content-Addressed Caching for Reproducible Execution

**Status**: Accepted  
**Date**: 2026-07-20  
**Deciders**: OpenNotebook team  

## Context

Reactive notebooks need to:
- Avoid re-executing cells whose inputs haven't changed
- Provide deterministic, reproducible re-execution (same source → same outputs)
- Cache intermediate results across notebook sessions
- Invalidate cache when any upstream dependency changes
- Support cache key introspection (users should see why a cell re-executed)

## Decision

Implement a **content-addressed cache** in the Rust kernel.

### Cache Key Computation

For each cell, the cache key is:

```
cache_key = HASH(
    cell.source_code,              // The cell's source text
    upstream_cell_hashes,          // Cache keys of all upstream cells (content-addressed)
    cell_arguments,                // If cell is parameterized (future)
    kernel_version_hash,           // Hash of kernel binary version
    language_runtime_version       // Python/R version string
)
```

Where `HASH` is **BLAKE3** (fast, keyed, verified, streaming).

### Cache Lookup

```
Execution Request for Cell C
│
├── Compute cache_key from source + upstream hashes
│
├── Check in-memory cache (HashMap<BLAKE3Hash, CellOutput>)
│   └── Miss → Check on-disk cache (LMDB store at ~/.cache/open-notebook/)
│       └── Miss → Execute cell, store output in both caches
│
└── Hit  → Skip execution, return cached output
```

### Cache Invalidation

- **Automatic**: Cell source changes → cache key changes → cache miss → re-execute
- **Automatic**: Upstream cell output changes → upstream hash changes → cache key changes → cache miss
- **Manual**: User clicks "Clear cache" on a cell
- **Manual**: Kernel version changes → all caches invalidated (safe, conservative)
- **Scope**: Cache is per-notebook by default; optional global cache with warnings

### On-Disk Storage

Use **LMDB** (Lightning Memory-Mapped Database) for on-disk cache:
- Zero-copy reads (memory-mapped)
- ACID transactions
- No daemon process
- Survives kernel restarts
- Bounded size (LRU eviction, configurable max)

### Inspectability

The kernel exposes a `/cache` endpoint that shows:
```
Cell abc123: CACHE HIT (key: b3_abcdef..., cached 2026-07-20T14:30:00Z)
Cell def456: CACHE MISS (source changed at 2026-07-20T14:31:00Z)
Cell ghi789: CACHE MISS (upstream cell abc123 changed)
```

### Why Not mo.cache?

Marimo's `mo.cache` and `mo.persistent_cache` are Python-level decorators. They work but:
- Only capture function-level caching, not cell-level
- Cache key computation runs in Python (slow for large objects)
- Not persistent across kernel restarts
- No introspection API

Our Rust-level cache is faster (BLAKE3 in native code), coarser (cell-level), and more inspectable.

### Consequences

- Positive: Deterministic re-execution — same `.onb` + same data → same outputs, bit-for-bit.
- Positive: Major performance win for notebooks where only a few cells change between runs.
- Positive: LMDB is battle-tested (used in OpenLDAP, many Linux tools), zero-config, process-safe.
- Positive: BLAKE3 is faster than SHA-256 and supports keyed hashing and verified streaming.
- Negative: Cache size management is complex (LRU eviction, user-configurable limits).
- Negative: Serialization overhead for storing outputs to LMDB (Arrow-based serialization mitigates this).
- Negative: Stale caches may mask bugs if cache key doesn't capture all dependencies (conservative design: include kernel version, runtime version).
