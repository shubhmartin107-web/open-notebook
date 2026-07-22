import { useState } from 'react';
import AiChat from './AiChat';

interface Props {
  notebookId: string;
  cellIndex: number | null;
  onInsertCode: (code: string) => void;
}

export default function AiPanel({ notebookId, cellIndex, onInsertCode }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <>
      <style>{`
        @keyframes ai-slide-in {
          from { transform: translateX(100%); }
          to { transform: translateX(0); }
        }
      `}</style>

      <button
        style={{
          ...s.toggleBtn,
          right: open ? 360 : 0,
          borderRight: open ? '1px solid #e5e7eb' : 'none',
          borderRadius: open ? '6px 0 0 6px' : 0,
          background: open ? '#fff' : '#f9fafb',
        }}
        onClick={() => setOpen(o => !o)}
        title={open ? 'Close AI panel' : 'Open AI panel'}
      >
        {open ? '▶' : '◀'}
      </button>

      {open && (
        <aside style={s.panel}>
          <AiChat
            notebookId={notebookId}
            cellIndex={cellIndex}
            onInsertCode={onInsertCode}
          />
        </aside>
      )}
    </>
  );
}

const s: Record<string, React.CSSProperties> = {
  toggleBtn: {
    position: 'fixed',
    top: '50%',
    transform: 'translateY(-50%)',
    zIndex: 100,
    width: 28,
    height: 52,
    border: '1px solid #d1d5db',
    cursor: 'pointer',
    fontSize: 13,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    lineHeight: 1,
    padding: 0,
    color: '#374151',
    transition: 'right 0.2s ease, background 0.15s, border-radius 0.2s',
    boxShadow: '-2px 0 6px rgba(0,0,0,0.06)',
  },
  panel: {
    position: 'fixed',
    top: 0,
    right: 0,
    width: 360,
    height: '100vh',
    background: '#fff',
    borderLeft: '1px solid #e5e7eb',
    zIndex: 99,
    display: 'flex',
    flexDirection: 'column',
    boxShadow: '-4px 0 16px rgba(0,0,0,0.08)',
    animation: 'ai-slide-in 0.2s ease-out',
  },
};
