import { useState, useCallback, useRef, useEffect } from 'react';
import type { Cell, CellKind, CellOutput } from './types';
import NotebookCell from './components/NotebookCell';
import AiPanel from './components/AiPanel';
import Dashboard from './pages/Dashboard';
import { useApi } from './hooks/useApi';
import './styles.css';

function createCell(kind: CellKind, id?: string): Cell {
  return {
    id: id ?? crypto.randomUUID(),
    kind,
    source: '',
    output: null,
    status: 'idle',
    executionCount: 0,
  };
}

function goOutputToCellOutput(goOutput: any): CellOutput | null {
  if (!goOutput) return null;
  const items: CellOutput['items'] = [];
  if (goOutput.stdout) {
    items.push({ mimeType: 'text/plain', data: [], text: goOutput.stdout, renderPriority: 0 });
  }
  if (goOutput.stderr) {
    items.push({ mimeType: 'text/plain', data: [], text: goOutput.stderr, renderPriority: 0 });
  }
  return { items, errorTraceback: goOutput.stderr || undefined, durationMs: 0 };
}

export default function App() {
  const [route, setRoute] = useState<'dashboard' | 'editor'>('dashboard');
  const [currentNotebookId, setCurrentNotebookId] = useState<string | null>(null);

  useEffect(() => {
    const pathId = window.location.pathname.split('/').filter(Boolean).pop();
    if (pathId) {
      setRoute('editor');
      setCurrentNotebookId(pathId);
    } else {
      setRoute('dashboard');
      setCurrentNotebookId(null);
    }
  }, []);

  if (route === 'dashboard') {
    return <Dashboard />;
  }

  return <NotebookEditor notebookId={currentNotebookId} />;
}

function NotebookEditor({ notebookId }: { notebookId: string | null }) {
  const [title, setTitle] = useState('Untitled Notebook');
  const [cells, setCells] = useState<Cell[]>([]);
  const [dagOpen, setDagOpen] = useState(false);
  const [activeCellIndex, setActiveCellIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const api = useApi();
  const cellsRef = useRef(cells);
  cellsRef.current = cells;
  const initDone = useRef(false);

  const historyRef = useRef<Cell[][]>([]);
  const historyIndexRef = useRef(-1);

  const pushHistory = useCallback((snapshot: Cell[]) => {
    const clone = JSON.parse(JSON.stringify(snapshot));
    let hist = historyRef.current;
    const idx = historyIndexRef.current;
    hist = hist.slice(0, idx + 1);
    hist.push(clone);
    if (hist.length > 50) hist.splice(0, hist.length - 50);
    historyRef.current = hist;
    historyIndexRef.current = hist.length - 1;
  }, []);

  const undo = useCallback(() => {
    if (historyIndexRef.current > 0) {
      historyIndexRef.current--;
      setCells(JSON.parse(JSON.stringify(historyRef.current[historyIndexRef.current])));
    }
  }, []);

  const redo = useCallback(() => {
    if (historyIndexRef.current < historyRef.current.length - 1) {
      historyIndexRef.current++;
      setCells(JSON.parse(JSON.stringify(historyRef.current[historyIndexRef.current])));
    }
  }, []);

  useEffect(() => {
    if (initDone.current) return;
    initDone.current = true;
    const pathId = notebookId;
    if (pathId && pathId !== 'default') {
      api.getNotebook(pathId).then(nb => {
        const mapped: Cell[] = nb.cells.map((c: any) => ({
          id: c.id,
          kind: c.kind === 'code' ? 'python' : (c.kind as CellKind),
          source: c.source || '',
          output: goOutputToCellOutput(c.output),
          status: 'idle',
          executionCount: 0,
        }));
        if (mapped.length === 0) mapped.push(createCell('python'));
        setCells(mapped);
        historyRef.current = [JSON.parse(JSON.stringify(mapped))];
        historyIndexRef.current = 0;
        setTitle(nb.title);
      }).catch(() => {
        api.createNotebook('Untitled').then(_nb => {
          const initial = [createCell('python')];
          setCells(initial);
          historyRef.current = [JSON.parse(JSON.stringify(initial))];
          historyIndexRef.current = 0;
        }).catch(() => {
          const initial = [createCell('python')];
          setCells(initial);
          historyRef.current = [JSON.parse(JSON.stringify(initial))];
          historyIndexRef.current = 0;
        });
      });
    } else {
      api.createNotebook('Untitled').then(_nb => {
        const initial = [createCell('python')];
        setCells(initial);
        historyRef.current = [JSON.parse(JSON.stringify(initial))];
        historyIndexRef.current = 0;
      }).catch(() => {
        const initial = [createCell('python')];
        setCells(initial);
        historyRef.current = [JSON.parse(JSON.stringify(initial))];
        historyIndexRef.current = 0;
      });
    }
  }, [api, notebookId]);

  const addCell = useCallback((kind: CellKind) => {
    setCells(prev => {
      pushHistory(prev);
      return [...prev, createCell(kind)];
    });
  }, [pushHistory]);

  const updateSource = useCallback((id: string, source: string) => {
    setCells(prev => {
      pushHistory(prev);
      return prev.map(c => (c.id === id ? { ...c, source } : c));
    });
  }, [pushHistory]);

  const executeCell = useCallback(async (id: string) => {
    const cell = cellsRef.current.find(c => c.id === id);
    if (!cell || cell.status === 'running') return;

    setCells(prev => prev.map(c => (c.id === id ? { ...c, status: 'running' as const } : c)));
    setError(null);

    try {
      if (notebookId) {
        const result = await api.executeCellFull(notebookId, id);
        const goOutput = (result as any)?.cells?.find((c: any) => c.id === id)?.output;
        setCells(prev =>
          prev.map(c =>
            c.id === id
              ? { ...c, output: goOutputToCellOutput(goOutput), status: 'success' as const, executionCount: c.executionCount + 1 }
              : c,
          ),
        );
      } else {
        await new Promise(r => setTimeout(r, 100));
        setCells(prev =>
          prev.map(c =>
            c.id === id
              ? {
                  ...c,
                  output: {
                    items: [{ mimeType: 'text/plain', data: [], text: `[${c.kind.toUpperCase()}] Executed:\n${c.source || '(empty)'}`, renderPriority: 0 }],
                    durationMs: 0,
                  },
                  status: 'success' as const,
                  executionCount: c.executionCount + 1,
                }
              : c,
          ),
        );
      }
    } catch (e: any) {
      setError(e.message ?? String(e));
      setCells(prev =>
        prev.map(c => (c.id === id ? { ...c, status: 'error' as const } : c)),
      );
    }
  }, [notebookId, api]);

  const executeAll = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      if (notebookId) {
        const result = await api.executeNotebook(notebookId);
        const nb = result as any;
        setCells(prev => prev.map(c => {
          const updated = nb.cells?.find((nc: any) => nc.id === c.id);
          return updated ? {
            ...c,
            output: goOutputToCellOutput(updated.output),
            status: updated.output?.stderr ? 'error' as const : 'success' as const,
            executionCount: c.executionCount + 1,
          } : c;
        }));
      } else {
        for (const cell of cellsRef.current) {
          await executeCell(cell.id);
        }
      }
    } catch (e: any) {
      setError(e.message ?? String(e));
    }
    setLoading(false);
  }, [executeCell, notebookId, api]);

  const removeCell = useCallback((id: string) => {
    setCells(prev => {
      pushHistory(prev);
      if (notebookId) api.deleteCell(notebookId, id).catch(() => {});
      return prev.filter(c => c.id !== id);
    });
  }, [notebookId, api, pushHistory]);

  const moveCell = useCallback((id: string, direction: 'up' | 'down') => {
    setCells(prev => {
      pushHistory(prev);
      const idx = prev.findIndex(c => c.id === id);
      if (idx < 0) return prev;
      if (direction === 'up' && idx === 0) return prev;
      if (direction === 'down' && idx === prev.length - 1) return prev;
      const newCells = [...prev];
      const swapIdx = direction === 'up' ? idx - 1 : idx + 1;
      [newCells[idx], newCells[swapIdx]] = [newCells[swapIdx], newCells[idx]];
      return newCells;
    });
  }, [pushHistory]);

  const changeCellKind = useCallback((id: string, kind: CellKind) => {
    setCells(prev => {
      pushHistory(prev);
      return prev.map(c => (c.id === id ? { ...c, kind } : c));
    });
  }, [pushHistory]);

  const handleInsertCode = useCallback((code: string) => {
    const cell = cellsRef.current[activeCellIndex];
    if (!cell) return;
    setCells(prev => {
      pushHistory(prev);
      return prev.map(c =>
        c.id === cell.id
          ? { ...c, source: c.source + (c.source ? '\n' : '') + code }
          : c,
      );
    });
  }, [activeCellIndex, pushHistory]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
        e.preventDefault();
        undo();
        return;
      }
      if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
        e.preventDefault();
        redo();
        return;
      }
      const cell = cellsRef.current[activeCellIndex];
      if (!cell) return;
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        executeCell(cell.id);
      } else if (e.shiftKey && e.key === 'Enter') {
        e.preventDefault();
        executeCell(cell.id);
        addCell('python');
      } else if (e.key === 'Escape') {
        (document.activeElement as HTMLElement)?.blur();
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [activeCellIndex, executeCell, addCell, undo, redo]);

  useEffect(() => {
    if (activeCellIndex >= cells.length && cells.length > 0) {
      setActiveCellIndex(cells.length - 1);
    }
  }, [cells.length, activeCellIndex]);

  const dagLines = cells.map((c, i) => {
    const firstLine = c.source.split('\n')[0] || '(empty)';
    return `  ${i + 1}. [${c.kind.toUpperCase()}] ${firstLine}`;
  });

  return (
    <>
      <div style={s.app}>
      <header style={s.header}>
        <div style={s.headerLeft}>
          <a href="/" style={s.homeLink}>◨</a>
          <input
            style={s.titleInput}
            value={title}
            onChange={e => setTitle(e.target.value)}
            placeholder="Notebook title..."
          />
        </div>
        <div style={s.headerBtns}>
          <button onClick={undo} style={s.btn} title="Undo (Ctrl+Z)">↩ Undo</button>
          <button onClick={redo} style={s.btn} title="Redo (Ctrl+Y)">↪ Redo</button>
          <button onClick={executeAll} disabled={loading} style={s.btn}>
            {loading ? '⏳ Running...' : (notebookId ? '▶ Run All (API)' : '▶ Run All')}
          </button>
          <button onClick={() => setDagOpen(o => !o)} style={s.btn}>
            {dagOpen ? '✕ Close DAG' : '◨ DAG'}
          </button>
        </div>
      </header>

      {error && <div style={s.errorBanner}>{error}</div>}

      <div style={s.body}>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={s.toolbar}>
            <button onClick={() => addCell('python')} style={s.addBtn}>+ Python</button>
            <button onClick={() => addCell('sql')} style={s.addBtn}>+ SQL</button>
            <button onClick={() => addCell('markdown')} style={s.addBtn}>+ Markdown</button>
            <button onClick={() => addCell('raw')} style={s.addBtn}>+ Raw</button>
          </div>

          <div style={s.cellList}>
            {cells.map((cell, i) => (
              <div
                key={cell.id}
                onClick={() => setActiveCellIndex(i)}
                style={{
                  cursor: 'pointer',
                  outline: i === activeCellIndex ? '2px solid #3b82f6' : 'none',
                  outlineOffset: 2,
                  borderRadius: 8,
                  transition: 'outline 0.15s',
                }}
              >
                <NotebookCell
                  cell={cell}
                  onExecute={executeCell}
                  onSourceChange={updateSource}
                  onDelete={removeCell}
                  onMove={moveCell}
                  onKindChange={changeCellKind}
                />
              </div>
            ))}
            {cells.length === 0 && (
              <div style={s.empty}>Add a cell to get started</div>
            )}
          </div>

          {loading && <div style={s.loadingBar}>⏳ Executing notebook...</div>}
        </div>

        {dagOpen && (
          <aside style={s.dagPanel}>
            <div style={s.dagTitle}>Execution DAG</div>
            <pre style={s.dagPre}>
              {dagLines.length > 0
                ? dagLines.join('\n')
                : '  (no cells)'}
            </pre>
          </aside>
        )}
      </div>
      </div>
      <AiPanel
        notebookId={notebookId || 'default'}
        cellIndex={activeCellIndex}
        onInsertCode={handleInsertCode}
      />
    </>
  );
}

const s: Record<string, React.CSSProperties> = {
  app: {
    maxWidth: 1024,
    margin: '0 auto',
    padding: 16,
    minHeight: '100vh',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 16,
    gap: 12,
    flexWrap: 'wrap',
    paddingBottom: 12,
    borderBottom: '1px solid #e5e7eb',
  },
  headerLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    flex: 1,
    minWidth: 200,
  },
  homeLink: {
    textDecoration: 'none',
    fontSize: 22,
    color: '#374151',
    fontWeight: 700,
    lineHeight: 1,
  },
  titleInput: {
    fontSize: 22,
    fontWeight: 700,
    border: 'none',
    background: 'transparent',
    flex: 1,
    minWidth: 200,
    color: '#111827',
    padding: '4px 0',
  },
  headerBtns: {
    display: 'flex',
    gap: 8,
    flexWrap: 'wrap',
  },
  btn: {
    padding: '7px 16px',
    borderRadius: 6,
    border: '1px solid #d1d5db',
    background: '#fff',
    fontSize: 13,
    fontWeight: 500,
    transition: 'background 0.1s, border-color 0.1s',
  },
  errorBanner: {
    padding: '10px 14px',
    background: '#fef2f2',
    border: '1px solid #fecaca',
    borderRadius: 6,
    color: '#b91c1c',
    fontSize: 13,
    marginBottom: 12,
  },
  body: {
    display: 'flex',
    gap: 16,
  },
  toolbar: {
    display: 'flex',
    gap: 8,
    marginBottom: 12,
    flexWrap: 'wrap',
  },
  addBtn: {
    padding: '6px 14px',
    borderRadius: 6,
    border: '1px dashed #d1d5db',
    background: '#fafafa',
    fontSize: 12,
    fontWeight: 600,
    color: '#374151',
    transition: 'background 0.1s, border-color 0.1s',
  },
  cellList: {
    display: 'flex',
    flexDirection: 'column',
    gap: 12,
  },
  empty: {
    textAlign: 'center',
    padding: 48,
    color: '#9ca3af',
    fontSize: 14,
  },
  loadingBar: {
    textAlign: 'center',
    padding: '10px 0',
    color: '#6b7280',
    fontSize: 13,
    fontWeight: 500,
  },
  dagPanel: {
    width: 280,
    flexShrink: 0,
    background: '#f9fafb',
    border: '1px solid #e5e7eb',
    borderRadius: 8,
    padding: 16,
    alignSelf: 'flex-start',
    position: 'sticky',
    top: 16,
  },
  dagTitle: {
    fontSize: 13,
    fontWeight: 700,
    color: '#374151',
    marginBottom: 8,
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
  },
  dagPre: {
    margin: 0,
    fontFamily: '"Fira Code", "Cascadia Code", "JetBrains Mono", monospace',
    fontSize: 12,
    lineHeight: 1.6,
    color: '#4b5563',
    whiteSpace: 'pre',
  },
};
