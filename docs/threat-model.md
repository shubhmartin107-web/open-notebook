# OpenNotebook Threat Model

**Version**: 0.1.0  
**Date**: 2026-07-20  
**Scope**: OpenNotebook v1.0 MVP (single-user, local-first, no cloud).

## 1. Assets

| Asset | Description | Criticality |
|---|---|---|
| **Notebook source code** | Python/SQL/R code in `.onb` files | High |
| **Cell outputs** | DataFrames, plots, computed results | High |
| **User data** | Data loaded into notebooks (CSV, Parquet, DB connections) | High |
| **AI provider API key** | Optional user-configured API key for remote LLM | Medium |
| **Kernel process** | Running kernel with access to all notebook data | High |
| **CRDT state** | In-memory collaboration state | Low (MVP: single-user) |

## 2. Threat Actors

| Actor | Motivation | Capability |
|---|---|---|
| **Local malware** | Steal data/code from the machine | Can read filesystem, monitor processes |
| **Network attacker** | Intercept or modify network traffic | Can MITM localhost or LAN connections |
| **Malicious notebook** | Exfiltrate data via code execution | Runs Python/SQL in the kernel |
| **Malicious AI model** | Inject harmful code via copilot | Generates text that user may execute |
| **Supply chain attacker** | Compromise dependencies | Can inject malicious code via compromised crate/npm package |

## 3. Threat Scenarios & Mitigations

### T1: Malicious Notebook Code Execution

**Scenario**: User opens a `.onb` file that contains `os.system("rm -rf /")` in a Python cell.

**Risk**: High. The kernel executes arbitrary Python by design.

**Mitigations**:
- **Container sandbox (optional)**: Run kernel in OCI container (Docker/Podman) with read-only filesystem, no network, resource limits.
- **User warning on open**: Show "This notebook contains code. Do you trust the author?" dialog before enabling execution.
- **Restricted builtins**: Remove `os`, `subprocess`, `shutil` from Python builtins by default (like Marimo's restricted mode). User can re-enable via settings.
- **Output validation**: Limit output size to prevent disk-filling DOS.

### T2: Data Exfiltration via AI Copilot

**Scenario**: The AI copilot (local or remote) accidentally or intentionally sends cell data to a remote server.

**Risk**: Medium. By default, AI runs locally (no exfiltration possible). If user configures a remote provider, data is sent to that endpoint.

**Mitigations**:
- **Local-first default**: All AI requests go to localhost (Ollama/llama.cpp). No data leaves the machine.
- **Clear warning on remote config**: "You are about to send notebook data to [remote endpoint]. This includes cell contents and outputs."
- **Data minimization**: Send only cell source code and schema metadata — no raw data.
- **Telemetry disabled by default**: No usage tracking, no crash reporting without explicit opt-in.

### T3: CRDT Relay Data Interception

**Scenario**: In a collaborative session, a network attacker intercepts CRDT ops between user A and the relay server.

**Risk**: Low (MPV: single-user). Post-MVP, CRDT ops contain cell source code.

**Mitigations**:
- **TLS by default**: WebSocket over WSS (wss://).
- **Optional end-to-end encryption**: CRDT ops can be encrypted with user-managed keys before relay.
- **No persistence**: Relay server only holds CRDT state in memory during active session.

### T4: Dependency Supply Chain Attack

**Scenario**: A compromised dependency (crate, npm package, Go module) executes malicious code during `cargo build` or `npm install`.

**Risk**: Medium — affects all users who install or update.

**Mitigations**:
- **Dependency pinning**: `Cargo.lock`, `go.sum`, `package-lock.json` committed to repo.
- **Dependency auditing**: `cargo-audit`, `npm audit`, `govulncheck` in CI.
- **Reproducible builds**: Build determinism ensures any supply chain intrusion would produce different checksums.
- **CycloneDX SBOM**: Published with each release for downstream auditing.
- **Sigstore signing**: Binary signatures verify tamper-free distribution.

### T5: Memory/Resource Exhaustion

**Scenario**: A malicious or buggy cell allocates infinite memory or runs an infinite loop.

**Risk**: Medium — can freeze the kernel or system.

**Mitigations**:
- **Execution timeout**: Default 5-minute per-cell timeout, configurable.
- **Memory limit**: Optional memory cap per kernel (configurable via settings/container).
- **Kernel isolation**: Each notebook gets its own kernel process. A crash in one doesn't affect others.
- **Output size limit**: Truncate outputs >100MB.

### T6: Protobuf Deserialization Attack

**Scenario**: A crafted `.onb` file exploits a buffer overflow or infinite loop in the Protobuf parser.

**Risk**: Low (Rust's memory safety prevents buffer overflows; but algorithmic complexity attacks are possible).

**Mitigations**:
- **Rust Protobuf library** (`prost`): Memory-safe by construction.
- **Recursion depth limits**: Limit nested Protobuf message depth.
- **Message size limits**: Reject `.onb` files >1GB.
- **Fuzz testing**: `cargo-fuzz` on the `.onb` parser.

### T7: CRDT Snapshot Tampering

**Scenario**: Attacker modifies the CRDT snapshot embedded in `.onb` to inject false state.

**Risk**: Low (single-user MVP). Post-MVP: could cause collaboration state corruption.

**Mitigations**:
- **CRDT integrity**: Loro snapshots include internal checksums.
- **Snapshot validation**: Reject invalid CRDT snapshots on load (restore from clean state).

## 4. Security Controls per Layer

| Layer | Controls |
|---|---|
| **File system** | Sandboxed kernel FS access; read-only mode option |
| **Network** | TLS on all external connections; CORS policies |
| **Kernel** | Restricted Python builtins; execution timeout; memory limits |
| **AI copilot** | Local-only default; data minimization; user warning on remote |
| **Build** | Dependency audit; reproducible builds; Sigstore signing |
| **CI/CD** | SAST (Semgrep, cargo-audit); DAST (OWASP ZAP) |
| **Supply chain** | Lock files; SBOM; fuzzing of parsers |

## 5. Future Threat Considerations (Post-MVP)

- **Multi-tenant server**: Authentication, authorization, session isolation, rate limiting.
- **Shared notebook links**: Access control, expiring links, read-only vs editable.
- **Secrets management**: Encrypted storage for API keys, DB credentials.
- **Audit logging**: Who executed what cell when.
- **SQL injection via DuckDB**: Input validation on parameterized SQL cells.
