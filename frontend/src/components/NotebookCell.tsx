import React from 'react';
import type { Cell, CellKind } from '../types';
import CellEditor from './CellEditor';
import CellOutput from './CellOutput';

interface Props {
  cell: Cell;
  onExecute: (id: string) => void;
  onSourceChange: (id: string, source: string) => void;
  onDelete: (id: string) => void;
  onMove: (id: string, direction: 'up' | 'down') => void;
  onKindChange: (id: string, kind: CellKind) => void;
}

const kindColors: Record<string, string> = {
  python: '#3776AB',
  sql: '#00758F',
  markdown: '#7C3AED',
  raw: '#6B7280',
};

const kindCycle: CellKind[] = ['python', 'sql', 'markdown', 'raw'];

const NotebookCell = React.memo(function NotebookCell({ cell, onExecute, onSourceChange, onDelete, onMove, onKindChange }: Props) {
  const handleKindClick = () => {
    const idx = kindCycle.indexOf(cell.kind);
    const next = kindCycle[(idx + 1) % kindCycle.length];
    onKindChange(cell.id, next);
  };
  return (
    <div style={s.cell}>
      <div style={s.header}>
        <div style={s.headerLeft}>
          <span
            style={{ ...s.badge, backgroundColor: kindColors[cell.kind] ?? '#6B7280', cursor: 'pointer' }}
            onClick={handleKindClick}
            title="Click to cycle cell kind"
          >
            {cell.kind.toUpperCase()}
          </span>
          {cell.executionCount > 0 && (
            <span style={s.count}>[{cell.executionCount}]</span>
          )}
        </div>
        <div style={{ display: 'flex', gap: 4 }}>
          <button
            onClick={() => onMove(cell.id, 'up')}
            style={s.moveBtn}
            title="Move up"
          >
            ▲
          </button>
          <button
            onClick={() => onMove(cell.id, 'down')}
            style={s.moveBtn}
            title="Move down"
          >
            ▼
          </button>
          <button
            onClick={() => onExecute(cell.id)}
            disabled={cell.status === 'running'}
            style={s.runBtn}
            title="Execute cell"
          >
            ▶
          </button>
          <button
            onClick={() => onDelete(cell.id)}
            style={s.delBtn}
            title="Delete cell"
          >
            ✕
          </button>
        </div>
      </div>
      <CellEditor
        initialSource={cell.source}
        kind={cell.kind}
        onChange={src => onSourceChange(cell.id, src)}
      />
      <CellOutput output={cell.output} isRunning={cell.status === 'running'} />
    </div>
  );
});

export default NotebookCell;

const s: Record<string, React.CSSProperties> = {
  cell: {
    border: '1px solid #e5e7eb',
    borderRadius: 8,
    background: '#fff',
    overflow: 'hidden',
    transition: 'box-shadow 0.15s',
    boxShadow: '0 1px 2px rgba(0,0,0,0.04)',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '4px 8px 4px 12px',
    background: '#f9fafb',
    borderBottom: '1px solid #f3f4f6',
  },
  headerLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
  },
  badge: {
    display: 'inline-block',
    padding: '1px 7px',
    borderRadius: 4,
    fontSize: 10,
    fontWeight: 700,
    color: '#fff',
    letterSpacing: '0.5px',
  },
  count: {
    fontSize: 11,
    color: '#9ca3af',
    fontFamily: 'monospace',
  },
  moveBtn: {
    width: 28,
    height: 26,
    borderRadius: 6,
    border: '1px solid #d1d5db',
    background: '#fff',
    cursor: 'pointer',
    fontSize: 11,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    lineHeight: 1,
    padding: 0,
    color: '#6b7280',
    transition: 'background 0.1s',
  },
  runBtn: {
    width: 28,
    height: 26,
    borderRadius: 6,
    border: '1px solid #d1d5db',
    background: '#fff',
    cursor: 'pointer',
    fontSize: 13,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    lineHeight: 1,
    padding: 0,
    transition: 'background 0.1s',
  },
  delBtn: {
    width: 28,
    height: 26,
    borderRadius: 6,
    border: '1px solid #fca5a5',
    background: '#fff',
    cursor: 'pointer',
    fontSize: 12,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    lineHeight: 1,
    padding: 0,
    color: '#dc2626',
    transition: 'background 0.1s',
  },
};
