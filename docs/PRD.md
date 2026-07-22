# OpenNotebook — Product Requirements Document

## 1. Product Vision

OpenNotebook is an AI-native, open-source reactive computational notebook and data IDE that rivals proprietary platforms like Deepnote, Hex, Google Colab, and Databricks Notebooks. It combines reactive DAG execution (like Marimo), real-time CRDT collaboration (like Deepnote), polyglot Python/SQL/R support, and a local-first AI copilot — all free, offline-capable, and built for a 10-year lifespan.

## 2. Target Users

| Persona | Needs | Key Features |
|---|---|---|
| **Data Scientist** | Interactive analysis, visualization, reproducibility | Reactive DAG, rich outputs, version control |
| **Data Engineer** | SQL + Python pipelines, large datasets | DuckDB SQL cells, Parquet/Iceberg support |
| **Analyst** | Shareable reports, no DevOps | One-click deploy, read-only sharing |
| **Educator** | Teach Python/R/SQL, grade notebooks | Markdown cells, deterministic re-execution |
| **Team** | Real-time pair programming | CRDT collab, presence cursors, chat |
| **AI/ML Researcher** | GPU-accelerated training, experiment tracking | GPU support, ONNX export, content-addr cache |

## 3. Feature Matrix

### MVP (Phase C — ~8-10 weeks)

| Feature | Priority | Description |
|---|---|---|
| Reactive DAG engine (Python only) | P0 | Topological sort, cycle detection, single-assignment, auto-execute on upstream change |
| DuckDB SQL cells | P0 | SQL cells that share scope with Python via DuckDB in-process |
| `.onb` file format | P0 | Protobuf binary + git-diffable Markdown representation |
| CodeMirror 6 editor | P0 | Syntax highlighting, autocomplete, bracket matching |
| Rich output rendering | P0 | DataFrame viewer, plots (vega-lite/altair), text, HTML, error display |
| Cell types: Python, SQL, Markdown | P0 | Three core cell types |
| Local file I/O | P0 | Read/write CSV, Parquet, JSON from filesystem |
| Single-user execution | P0 | Local kernel, no server needed for single-user mode |
| Content-addressed caching | P1 | Deterministic re-execution, cache invalidation on source change |
| AI copilot (local model) | P1 | Ollama auto-detect, streaming code gen, context injection |
| MCP server | P1 | Expose notebook read/write/exec via Model Context Protocol |
| AGENT.md + capability manifest | P1 | Autonomy-native metadata |
| Cross-platform (Linux/macOS/Windows) | P0 | CI builds for 6 targets (x86_64 + arm64) |

### Post-MVP (Phase C+ / Phase D)

| Feature | Priority | Description |
|---|---|---|
| R cells (Ark kernel) | P1 | R language support via Posit's Ark |
| Real-time CRDT collaboration | P1 | Multi-user editing via Loro + WebSocket relay |
| Rich chart builder | P2 | Visual chart builder (drag-drop config) |
| Environment variable manager | P2 | Secrets, API keys, config per notebook |
| Read-only sharing mode | P2 | Publish notebooks as read-only |
| GPU cell execution | P2 | `%%gpu` magic for CUDA/ROCm cells |
| SQL cell parameterization | P2 | Jinja-style `{{ var }}` in SQL cells |
| Vim/Emacs keybindings | P2 | CodeMirror 6 keymap plugins |
| Notebook scheduling | P3 | Cron-triggered re-execution |
| Package manager (pip/conda) | P3 | Declare dependencies per notebook |
| Dark mode + theming | P2 | Light/dark/high-contrast themes |
| i18n | P3 | Internationalized UI |
| Plugin system | P3 | Third-party cell types, renderers, tools |

## 4. Technical Requirements

### Performance (vs Deepnote benchmarks)

| Metric | Target | Measurement |
|---|---|---|
| Cold start (loaded kernel) | < 2s | From button click to first cell editable |
| DAG execution overhead | < 5ms per cell | Overhead of DAG scheduling beyond cell execution |
| 100-cell notebook load | < 1s | Parsing `.onb` + rendering cell deck |
| 10MB DataFrames (10 cols) | < 500ms | DuckDB query to rendered table |
| Collaboration latency | < 200ms p95 | CRDT op relay end-to-end |
| AI codegen latency | < 3s first token | 8B model on M2 Pro / RTX 3060 |

### Compatibility

- **OS**: Windows 10/11, Ubuntu 20.04/22.04/24.04, Fedora 38+, macOS 13+ (Intel + Apple Silicon)
- **Arch**: x86_64, arm64 (aarch64)
- **Python**: 3.10, 3.11, 3.12, 3.13
- **R**: 4.2+ (via Ark)
- **Browsers**: Chrome/Edge 110+, Firefox 115+, Safari 16+

### Security

- No network access required for local execution
- AI copilot runs locally (no data exfiltration)
- Sandboxed kernel execution (optional container mode)
- Sigstore signing for releases
- CycloneDX SBOM for dependency auditing
- SAST (Semgrep, cargo-audit) + DAST (OWASP ZAP) in CI

### 10-Year Durability

- **Formats**: Apache Arrow, Parquet, ONNX, Protobuf, Iceberg
- **Protocols**: gRPC, LSP, MCP, OpenTelemetry
- **Standards**: LLVM/MLIR for future compilation paths
- **Reproducible builds**: Deterministic, signed, verifiable

## 5. Success Metrics

| Metric | Target |
|---|---|
| GitHub stars | 10K in first 6 months |
| Monthly active users | 5K self-hosted + 2K cloud |
| Notebook compatibility | 90% of Marimo notebooks importable |
| Community contributions | 50+ contributors |
| CI pass rate | > 99% |
| Security vulns in prod | Zero critical/high |
| Time to first notebook | < 2 minutes from download |

## 6. Release Cadence

| Phase | Timeline | Deliverables |
|---|---|---|
| **Phase B** | Week 1-2 | PRD, ADRs, API spec, threat model, UI mocks |
| **Phase C (MVP)** | Week 3-10 | Kernel, file format, frontend, single-user execution |
| **Phase C+** | Week 11-16 | R cells, CRDT collab, AI copilot, MCP |
| **Phase D** | Week 17-20 | Benchmarks, fuzzing, CI hardening, security audit |
| **Phase E** | Week 21-24 | Docs, release scripts, Sigstore, sustainability |

## 7. Out of Scope (v1.0)

- Hosted cloud service (user self-hosts or third-party hosts)
- Mobile app
- VS Code / Jupyter extension (standalone app only)
- Managed notebook scheduler (cron job interface only)
- Auto-ML / Auto-Vis (future plugin)
