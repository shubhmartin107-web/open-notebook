# ADR-006: SQL Execution Engine

**Status**: Accepted  
**Date**: 2026-07-20  
**Deciders**: OpenNotebook team  

## Context

Notebooks need SQL cells that:
- Query data from Python DataFrames, CSV files, Parquet, and databases
- Share results back to Python/R without serialization
- Run in-process (no external database daemon)
- Support standard SQL (SELECT, JOIN, GROUP BY, window functions, CTEs)
- Handle datasets larger than available RAM (out-of-core)

## Options Considered

| Option | Pros | Cons |
|---|---|---|
| **DuckDB** | In-process, Arrow-native, zero external deps, out-of-core, fast, Python/R/Java bindings, WASM-compatible | No built-in distributed query |
| **SQLite** | Ubiquitous, simple | Single-threaded, no columnar/vectorized execution, poor OLAP performance |
| **DataFusion** | Rust-native, Arrow-native | Younger ecosystem, fewer SQL features, no Python-internal table sharing |
| **MariaDB/Postgres** | Full SQL, row-level security | External daemon, not in-process, serialization overhead |

## Decision

Use **DuckDB** as the embedded SQL engine.

### Integration

```
Python DataFrame ──zero-copy Arrow──▶ DuckDB ──zero-copy Arrow──▶ Python DataFrame
                                          │
                                    DuckDB SQL cells
                                    ─────────────────────
                                    SELECT * FROM df
                                    WHERE revenue > 1000
                                    GROUP BY region
                                    ─────────────────────
                                    └── Result set → Rich output (table/chart)
```

### Cell Type Semantics

```python
# Python cell: produce a DataFrame
df = pd.read_csv("sales.csv")

# SQL cell: query it using DuckDB — result is a new DataFrame
df_result = """SELECT region, SUM(revenue) as total
               FROM df
               GROUP BY region
               ORDER BY total DESC"""
# (result automatically becomes available as "df_result" in Python scope)
```

### Key Features

1. **Arrow zero-copy**: DuckDB and Python both operate on Arrow arrays. No serialization needed.
2. **Out-of-core**: DuckDB spills to disk when data exceeds RAM — no memory errors on 100GB datasets.
3. **Multiple file formats**: DuckDB reads CSV, Parquet, JSON, Excel directly with `read_csv_auto()`, `read_parquet()`, etc.
4. **Extensions**: Loadable extensions for HTTPFS (S3/GCS), Postgres/MySQL/SQLite scanners, full-text search, spatial (GIS).
5. **WASM-compatible**: DuckDB compiles to WASM — relevant for future browser-only mode.

### Consequences

- Positive: In-process, zero-dependency SQL engine proven in Marimo, SQLRooms, and Hex.
- Positive: DuckDB's Arrow-native design enables zero-copy sharing between Python and SQL cells — critical for performance.
- Positive: DuckDB is embedded in the Rust kernel (via `duckdb` crate), no external daemon.
- Negative: DuckDB's SQL dialect has minor differences from PostgreSQL (e.g., implicit casting, date functions) — document incompatibilities in user docs.
- Negative: Some advanced SQL features (MERGE, recursive CTEs) were added recently — verify version requirements.
