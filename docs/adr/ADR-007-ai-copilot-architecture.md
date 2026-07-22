# ADR-007: AI Copilot Architecture

**Status**: Accepted  
**Date**: 2026-07-20  
**Deciders**: OpenNotebook team  

## Context

The AI copilot must:
- Work completely offline with local models
- Require zero API keys out of the box
- Auto-detect available local inference engines
- Generate code (Python, SQL, R), debug errors, explain cells, suggest completions
- Understand notebook context (cell content, outputs, DAG structure, variable scope)

## Design Decisions

### Decision 1: Abstract Provider Interface

The copilot uses a provider pattern:

```
AIProvider (trait)
├── OllamaProvider      Default, auto-detect localhost:11434
├── OpenaiCompatProvider  Any OpenAI-compatible endpoint (vLLM, TGI, LM Studio)
├── LlamaCppProvider     Direct llama.cpp server
└── NullProvider         No LLM available, graceful degradation
```

**Default behavior on first launch**: Probe `http://localhost:11434/api/tags`. If Ollama detected, auto-configure. If not, show a "Connect to local model" dialog with options (Ollama, vLLM, LM Studio, etc.).

### Decision 2: Default Model

**Qwen2.5-Coder-7B-Instruct** (via Ollama).

Rationale:
- Best code generation quality at 7B scale (HumanEval 86.5%, MBPP 80.2%)
- Apache 2.0 licensed
- Strong in Python, SQL, and R
- Runs on consumer GPUs (8GB VRAM) and Apple Silicon (M2+)

### Decision 3: Context Injection

The copilot automatically injects context into each prompt:

```
System: You are a data science assistant in the OpenNotebook notebook.
Current notebook language: Python (with DuckDB SQL and R cells available).

Context:
- Cell DAG (cells above and below, their sources, outputs)
- Variable scope (defined variables, types, shapes)
- Last error traceback (if debugging)
- Schema of DataFrames in scope (column names, types, row counts)

User: [their prompt or selected cell]
```

### Decision 4: Interaction Points

| Feature | Trigger | Input to LLM |
|---|---|---|
| **Code generation** | User types prompt in Copilot panel | Cell context + prompt → generated code inserted at cursor |
| **Inline completion** | User types in CodeMirror (debounced) | Current cell source + cursor position → completion suggestion |
| **Explain cell** | Right-click → "Explain" | Cell source + outputs → natural language explanation |
| **Debug error** | Cell execution error | Error traceback + cell source → fix suggestion |
| **Edit cell** | Select text → "Edit with AI" | Selected text + instruction → replacement |
| **Generate notebook** | New notebook → "Generate from prompt" | Task description → multi-cell notebook draft |

### Decision 5: Streaming

All LLM interactions stream tokens via Server-Sent Events (SSE). The frontend shows a streaming typewriter effect for generated code.

### Decision 6: No Data Exfiltration

By default, all AI requests go to localhost. If the user configures a remote endpoint, a clear warning is shown. No telemetry is sent.

### Consequences

- Positive: Zero-config setup — auto-detect local model, no API keys required.
- Positive: Qwen2.5-Coder is SOTA for 7B code models and Apache 2.0 licensed.
- Positive: Provider abstraction means users can plug in any model (local or cloud).
- Positive: Rich context injection means the model actually understands the notebook state.
- Negative: 7B models are less capable than 70B+ for complex multi-step reasoning (acceptable trade for local execution).
- Negative: LLM on CPU is slow — we recommend GPU for acceptable latency.
- Negative: Inline completion requires careful UX (debouncing, cancellation) to avoid frustration.
