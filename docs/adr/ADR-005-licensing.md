# ADR-005: Licensing Strategy

**Status**: Accepted  
**Date**: 2026-07-20  
**Deciders**: OpenNotebook team  

## Context

OpenNotebook must be:
- Free and open-source forever
- Protected from proprietary SaaS clones
- Permissive enough for library reuse in other OSS projects
- Compatible with upstream dependencies (Rust, Go, PyO3, DuckDB, Loro, Ark)

## Decision

Adopt a **dual-license model**:

| Component | License | Rationale |
|---|---|---|
| **Kernel libraries** (rust kernel core, DAG engine, CRDT bindings) | **Apache 2.0** | Maximum permissiveness for library reuse; compatible with Rust ecosystem; allows embedding in other OSS |
| **Server + orchestration** (open-notebookd, Go server) | **AGPL 3.0** | Copyleft at the network level — anyone running a modified version as a service must release their changes. Protects against proprietary SaaS competition. |
| **Frontend** (TypeScript/React) | **Apache 2.0** | Permissive for UI reuse; consistent with kernel libraries |
| **CLI tools** (onb CLI) | **Apache 2.0** | Permissive for automation scripts |
| **Documentation** | **CC BY-SA 4.0** | Share-alike for docs |

### Why AGPL for the Server

The network interaction clause (AGPL §13) means that anyone who modifies OpenNotebook's server and runs it as a network service must distribute their modified source code to all users. This prevents:
- Proprietary forks being sold as hosted services
- Corporations taking the OSS and building competitive closed-source SaaS without contributing back

### Apache 2.0 Compatibility

All direct dependencies are Apache/MIT/BSD compatible:
- Rust ecosystem (Apache 2.0 / MIT)
- Go ecosystem (BSD)
- DuckDB (MIT)
- Loro (MIT)
- Ark (MIT)
- PyO3 (Apache 2.0)
- CodeMirror 6 (MIT)
- React (MIT)

### Consequences

- Positive: AGPL protects the server from proprietary SaaS without being as restrictive as GPL for library consumers.
- Positive: Apache 2.0 for libraries maximizes adoption in the OSS ecosystem.
- Positive: CC BY-SA for docs ensures documentation improvements flow back.
- Negative: AGPL may deter some corporate users who have blanket policies against AGPL software (mitigated by documentation and FAQ explaining the intent).
- Negative: Need clear LICENSE files per directory/component and a top-level explanation.
