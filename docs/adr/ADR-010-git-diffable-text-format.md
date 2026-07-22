# ADR-010: Git-Diffable Text Representation

**Status**: Accepted  
**Date**: 2026-07-20  
**Deciders**: OpenNotebook team  

## Context

The `.onb.md` text format must:
- Be human-readable in diffs (changed lines should clearly show what changed in which cell)
- Round-trip losslessly through `.onb` → `.onb.md` → `.onb` (canonical → text → canonical)
- Support code review workflows (GitHub PRs, GitLab MRs)
- Be machine-generated (primary authoring is via the UI, not manual text editing)

## Decision

The `.onb.md` format is defined as follows:

```
┌─────────────────────────────────────┐
│ ┌─────────────────────────────────┐ │
│ │ Notebook header (metadata)      │ │
│ │ "# Notebook Title"              │ │
│ │ "```onb-meta"                   │ │
│ │ "format_version: onb/v1"        │ │
│ │ "created: 2026-07-20Z"          │ │
│ │ "```"                           │ │
│ └─────────────────────────────────┘ │
│                                     │
│ ┌─────────────────────────────────┐ │
│ │ Per-cell sections               │ │
│ │ "## Cell: <id> [<kind>]"       │ │
│ │ "```<language>"                 │ │
│ │ <source code>                    │ │
│ │ "```"                           │ │
│ │ "*Output:* <summary>"           │ │
│ │ "```onb-output"                  │ │
│ │ <serialized output>              │ │
│ │ "```"                            │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

### Cell Header Format

Each cell starts with a level-2 heading:

```
## Cell: abc123 [python]
## Cell: def456 [sql]
## Cell: ghi789 [markdown]
## Cell: jkl012 [r]
```

### Cell Source

Standard fenced code block with language identifier:

```python
import pandas as pd
df = pd.read_csv("data.csv")
```

### Cell Output (Optional)

Cell output is a Markdown italic line followed by a `onb-output` code block:

```
*Output: table 5 rows × 3 cols*

```onb-output
{
  "mime_type": "application/vnd.dataframe+json",
  "data": {
    "columns": ["name", "age", "city"],
    "rows": [
      ["Alice", 30, "NYC"],
      ["Bob", 25, "SF"],
      ["Carol", 35, "CHI"],
      ...
    ]
  }
}
```

### DAG Metadata (Optional, for documentation)

```
*Upstream: abc123, def456*
```

### Round-Trip Rules

1. `.onb` is the canonical format. `.onb.md` is always generated from `.onb`.
2. On import, if only `.onb.md` exists (e.g., user manually created a diff during code review), regenerate `.onb` from `.onb.md`.
3. Cell source is authoritative from the code block. DAG edges, CRDT snapshots, and binary outputs are from `onb-meta` / `onb-output` blocks.
4. Round-trip validation: `load(serialize(notebook)) == notebook` (tested in CI).

### Consequences

- Positive: Diffs show each cell change independently — reviewers see exactly which cell was modified.
- Positive: GitHub renders Markdown natively with syntax-highlighted code blocks.
- Positive: Round-trip lossless means no "file format drift" over time.
- Negative: Users may try to manually edit `.onb.md` and break the format (mitigated by validation and clear "generated file" header).
- Negative: Large outputs (big DataFrames, binary plots) bloat the Markdown file. Mitigation: large outputs stored as attachment files (`.onb/attachments/uuid.png`) with references in the Markdown.
