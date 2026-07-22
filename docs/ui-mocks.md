# OpenNotebook UI Design

## 1. Layout

```
┌─────────────────────────────────────────────────────────────┐
│  OpenNotebook — sales_analysis.onb                          │
│  ┌────────────────────────────────────────┐ ┌─────────────┐ │
│  │                                        │ │  Copilot     │ │
│  │  [Explorer] [Python 3.12] [▶ Run All] │ │  ┌─────────┐ │ │
│  │                                        │ │  │ Ask AI  │ │ │
│  │  ┌──────────────────────────────────┐  │ │  │ anything │ │ │
│  │  │ Cell 1 [python] ◉ abc123         │  │ │  └─────────┘ │ │
│  │  │ ┌──────────────────────────────┐ │  │ │             │ │
│  │  │ │ import pandas as pd          │ │  │ │ ▸ Generate  │ │
│  │  │ │ df = pd.read_csv("data.csv") │ │  │ │ ▸ Explain   │ │
│  │  │ └──────────────────────────────┘ │  │ │ ▸ Fix error │ │
│  │  │ [▶] [■] [⏳] Output: 5 rows    │  │ │ │ ▸ Chart     │ │
│  │  └──────────────────────────────────┘  │ │             │ │
│  │                                        │ │  ─────────  │ │
│  │  ┌──────────────────────────────────┐  │ │  Model:     │ │
│  │  │ Cell 2 [sql] ◉ def456            │  │ │  qwen2.5-   │ │
│  │  │ ┌──────────────────────────────┐ │  │ │  coder:7b  │ │
│  │  │ │ SELECT region, SUM(revenue)  │ │  │ │  ● Online   │ │
│  │  │ │ FROM df                      │ │  │ └─────────────┘ │
│  │  │ │ GROUP BY region              │ │  │                 │
│  │  │ └──────────────────────────────┘ │  │                 │
│  │  │ [▶] Output: table 3 rows × 2...│  │                 │
│  │  └──────────────────────────────────┘  │                 │
│  │                                        │                 │
│  │  ┌──────────────────────────────────┐  │                 │
│  │  │ Cell 3 [markdown] ◉ ghi789       │  │                 │
│  │  │ ┌──────────────────────────────┐ │  │                 │
│  │  │ │ # Results                    │ │  │                 │
│  │  │ │ The top region is **West**   │ │  │                 │
│  │  │ └──────────────────────────────┘ │  │                 │
│  │  └──────────────────────────────────┘  │                 │
│  │                                        │                 │
│  │  [+ Add Cell] [Python ▼]               │                 │
│  └────────────────────────────────────────┘ └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 2. Cell Types

### Python Cell
```
┌─ Cell 1 [python] ───────────────────── ◉ abc123 ──┐
│ ┌────────────────────────────────────────────────┐ │
│ │ import pandas as pd                            │ │
│ │ df = pd.read_csv("data.csv")                   │ │
│ └────────────────────────────────────────────────┘ │
│ [▶ Run] [■ Stop] [⏳ Clear Cache] [⋮ Menu]       │
│ Output: DataFrame (1,000 rows × 5 cols)            │
│ ┌────┬───────┬──────┬──────┬──────┐               │
│ │ #  │ name  │ age  │ city │ sal  │               │
│ ├────┼───────┼──────┼──────┼──────┤               │
│ │ 0  │ Alice │ 30   │ NYC  │ 95K  │               │
│ │ 1  │ Bob   │ 25   │ SF   │ 85K  │               │
│ │ 2  │ Carol │ 35   │ CHI  │ 110K │               │
│ │ ...│ ...   │ ...  │ ...  │ ...  │               │
│ └────┴───────┴──────┴──────┴──────┘               │
└────────────────────────────────────────────────────┘
```

### SQL Cell
```
┌─ Cell 2 [sql] ─────────────────────── ◉ def456 ──┐
│ ┌────────────────────────────────────────────────┐ │
│ │ SELECT region, SUM(revenue) as total_revenue   │ │
│ │ FROM df                                        │ │
│ │ GROUP BY region                                │ │
│ │ ORDER BY total_revenue DESC                    │ │
│ └────────────────────────────────────────────────┘ │
│ [▶ Run] Output: table 3 rows × 2 cols              │
│ ┌────────┬───────────────┐                        │
│ │ region │ total_revenue │                        │
│ ├────────┼───────────────┤                        │
│ │ West   │ 1,250,000     │                        │
│ │ East   │ 980,000       │                        │
│ │ South  │ 720,000       │                        │
│ └────────┴───────────────┘                        │
└────────────────────────────────────────────────────┘
```

### Markdown Cell
```
┌─ Cell 3 [markdown] ────────────────── ◉ ghi789 ──┐
│ ┌────────────────────────────────────────────────┐ │
│ │ # Regional Sales Analysis                      │ │
│ │                                                │ │
│ │ The **West** region leads with **$1.25M**      │ │
│ │ in revenue, followed by East ($980K) and       │ │
│ │ South ($720K).                                 │ │
│ │                                                │ │
│ │ ```                                            │ │
│ │ West:  42% of total revenue                    │ │
│ │ East:  33%                                     │ │
│ │ South: 25%                                     │ │
│ │ ```                                            │ │
│ └────────────────────────────────────────────────┘ │
│ [⋮ Menu] (no run button — rendered in place)       │
└────────────────────────────────────────────────────┘
```

### R Cell (Post-MVP)
```
┌─ Cell 4 [r] ───────────────────────── ◉ jkl012 ──┐
│ ┌────────────────────────────────────────────────┐ │
│ │ library(ggplot2)                               │ │
│ │ p <- ggplot(df, aes(x=region, y=revenue)) +    │ │
│ │      geom_col()                                │ │
│ │ print(p)                                       │ │
│ └────────────────────────────────────────────────┘ │
│ [▶ Run] Output: plot (PNG/SVG)                    │
│ ┌────────────────────────────────────────────────┐ │
│ │             ██                                  │ │
│ │   ████████████████                              │ │
│ │   ██  ██  ██  ████████                         │ │
│ │ West East South                                │ │
│ └────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────┘
```

## 3. Toolbar

The top toolbar contains:

| Icon/Button | Action |
|---|---|
| ☰ | Menu: File, Edit, View, Run, Help |
| 📁 | Open notebook |
| 💾 | Save (Ctrl+S) |
| ⏪ | Undo (Ctrl+Z) |
| ⏩ | Redo (Ctrl+Shift+Z) |
| ▶ Run All | Execute all cells in DAG order |
| ▶ Run Selected | Execute selected cells |
| ■ Stop All | Halt execution |
| 🔄 Clear Cache | Clear content-addressed cache |
| 🌐 | Collaboration status (MVP: offline) |
| 🤖 | AI Copilot toggle (show/hide panel) |
| ⚙️ | Settings |

## 4. Cell Toolbar (per-cell actions)

| Icon | Action |
|---|---|
| ▶ | Execute cell (and downstream) |
| ■ | Stop execution |
| ⏳ | Clear cache for this cell |
| ➕ | Insert cell above |
| ➖ | Delete cell |
| ⬆ ⬇ | Drag to reorder |
| ⋮ | More: Cut, Copy, Paste, Duplicate, Move to... |
| 🔒 | Collapse cell (show only output header) |

## 5. Collaboration Indicators (Post-MVP)

```
┌─ Cell 1 [python] ───── ● Alice editing ── ◉ abc123 ──┐
│ ┌────────────────────────────────────────────────────┐ │
│ │ import pandas as pd                                │ │
│ │ df = pd.read_───▌ Bob's cursor ────csv("data.csv") │ │
│ └────────────────────────────────────────────────────┘ │
│ [▶]  ● Alice seeing this output  ● Bob running cell 2 │
└────────────────────────────────────────────────────────┘
```

- **Cursor colors**: Each collaborator gets a unique color (hash of user ID).
- **Selection highlights**: Transparent overlay in the user's color.
- **Cell status**: "Alice editing", "Bob running", "Charlie viewing".
- **Presence list**: Top-right shows avatars/initials of connected users.
- **Conflict resolution**: Loro MovableTree handles reorder conflicts transparently.

## 6. AI Copilot Panel

```
┌─ AI Copilot ────────────────────────────────────────┐
│ [Model: qwen2.5-coder:7b] [● Online] [⚙️]         │
│                                                      │
│ ┌──────────────────────────────────────────────────┐ │
│ │ > Write a SQL query to find the top 5 regions    │ │
│ │   by revenue growth month-over-month              │ │
│ │                                                   │ │
│ │ ┌──────────────────────────────────────────────┐ │ │
│ │ │ SELECT region,                               │ │ │
│ │ │   revenue - LAG(revenue) OVER (              │ │ │
│ │ │     PARTITION BY region ORDER BY month       │ │ │
│ │ │   ) AS growth                                │ │ │
│ │ │ FROM monthly_revenue                         │ │ │
│ │ │ ORDER BY growth DESC                         │ │ │
│ │ │ LIMIT 5                                      │ │ │
│ │ └──────────────────────────────────────────────┘ │ │
│ │ [Insert at cursor] [Replace selected] [Copy]     │ │
│ │                                                   │ │
│ │ > What does this error mean?                      │ │
│ │                                                   │ │
│ │ ● The error says "KeyError: 'region'". This       │ │
│ │   means the column 'region' doesn't exist in      │ │
│ │   your DataFrame. Use df.columns to list...       │ │
│ └──────────────────────────────────────────────────┘ │
│ ┌──────────────────────────────────────────────────┐ │
│ │ [Ask anything about your notebook...        ]    │ │
│ └──────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

## 7. Interaction States

### Loading
```
┌─ Cell 1 [python] ───────────────── ◉ abc123 ──┐
│ Loading kernel...                               │
│ ████████░░░░░░░░░░░░  (kernel start progress)   │
└────────────────────────────────────────────────┘
```

### Running
```
┌─ Cell 1 [python] ──── ● Running ── ◉ abc123 ──┐
│ ⏳ K worker threads active                       │
│ ┌─────────────────────────────────────────────┐ │
│ │ progress bar ████████░░░░                    │ │
│ └─────────────────────────────────────────────┘ │
└────────────────────────────────────────────────┘
```

### Error
```
┌─ Cell 1 [python] ──── ● Error ── ◉ abc123 ──┐
│ ❌ NameError: name 'df' is not defined         │
│                                                │
│ Did you mean: df_result?                       │
│ [Show traceback] [Fix with AI] [Dismiss]      │
└───────────────────────────────────────────────┘
```

### DAG Visualization
```
┌─ DAG View ────────────────────────────────────┐
│                                                │
│   [Cell 1: python]                             │
│         │                                      │
│         ▼                                      │
│   [Cell 2: sql] ──▶ [Cell 4: python]          │
│         │                                      │
│         ▼                                      │
│   [Cell 3: markdown]                           │
│                                                │
│   ● = will execute   ○ = cached   ✕ = error   │
└────────────────────────────────────────────────┘
```

## 8. Responsive Breakpoints

| Breakpoint | Layout |
|---|---|
| > 1200px | Full: sidebar + cell deck + copilot panel |
| 800–1200px | Cell deck + copilot panel (collapsible sidebar) |
| < 800px | Cell deck only (copilot in overlay) |
| Print | All cells expanded, outputs rendered, no toolbars |
