import { useState, useEffect } from 'react';

interface NotebookSummary {
  id: string;
  title: string;
  cells: any[];
}

export default function Dashboard() {
  const [notebooks, setNotebooks] = useState<NotebookSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [serverAvailable, setServerAvailable] = useState(true);
  const [creating, setCreating] = useState(false);

  const fetchNotebooks = () => {
    setLoading(true);
    fetch('/api/notebooks')
      .then(res => {
        if (!res.ok) throw new Error('Server unavailable');
        return res.json();
      })
      .then((data: NotebookSummary[]) => {
        setNotebooks(data);
        setServerAvailable(true);
      })
      .catch(() => {
        setServerAvailable(false);
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    fetchNotebooks();
  }, []);

  const createNotebook = async () => {
    setCreating(true);
    try {
      const res = await fetch('/api/notebooks', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title: 'Untitled Notebook' }),
      });
      if (!res.ok) throw new Error('Failed');
      const nb = await res.json();
      window.location.href = `/${nb.id}`;
    } catch {
      const id = crypto.randomUUID();
      window.location.href = `/${id}`;
    }
    setCreating(false);
  };

  const createLocalNotebook = () => {
    const id = crypto.randomUUID();
    window.location.href = `/${id}`;
  };

  if (loading) {
    return (
      <div style={s.container}>
        <p style={{ color: '#6b7280', fontSize: 14 }}>Loading notebooks...</p>
      </div>
    );
  }

  return (
    <div style={s.container}>
      <header style={s.header}>
        <h1 style={s.h1}>Open Notebook</h1>
        <div style={s.headerBtns}>
          {!serverAvailable && <span style={s.offlineBadge}>Offline</span>}
          <button onClick={createNotebook} disabled={creating} style={s.newBtn}>
            {creating ? 'Creating...' : '+ New Notebook'}
          </button>
        </div>
      </header>

      <p style={s.subtitle}>
        {serverAvailable
          ? `${notebooks.length} notebook${notebooks.length !== 1 ? 's' : ''}`
          : 'Server unavailable — working offline'}
      </p>

      {!serverAvailable && (
        <div style={s.offlineCard}>
          <p style={{ margin: '0 0 12px', fontSize: 13, color: '#374151' }}>
            The server is not reachable. You can create a local-only notebook.
          </p>
          <button onClick={createLocalNotebook} style={s.newBtn}>
            + Create Local Notebook
          </button>
        </div>
      )}

      <div style={s.grid}>
        {notebooks.map(nb => (
          <a key={nb.id} href={`/${nb.id}`} style={s.card}>
            <div style={s.cardTitle}>{nb.title || 'Untitled'}</div>
            <div style={s.cardMeta}>
              {nb.cells?.length ?? 0} cell{(nb.cells?.length ?? 0) !== 1 ? 's' : ''}
            </div>
          </a>
        ))}
        {notebooks.length === 0 && serverAvailable && (
          <div style={s.empty}>No notebooks yet. Create one to get started.</div>
        )}
      </div>
    </div>
  );
}

const s: Record<string, React.CSSProperties> = {
  container: {
    maxWidth: 800,
    margin: '0 auto',
    padding: 32,
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 8,
    gap: 12,
    flexWrap: 'wrap',
  },
  h1: {
    margin: 0,
    fontSize: 24,
    fontWeight: 700,
    color: '#111827',
  },
  headerBtns: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
  },
  offlineBadge: {
    padding: '3px 10px',
    borderRadius: 12,
    background: '#fef3c7',
    color: '#92400e',
    fontSize: 11,
    fontWeight: 600,
  },
  newBtn: {
    padding: '8px 18px',
    borderRadius: 6,
    border: '1px solid #3b82f6',
    background: '#3b82f6',
    color: '#fff',
    fontSize: 13,
    fontWeight: 600,
    cursor: 'pointer',
    transition: 'background 0.1s',
  },
  subtitle: {
    fontSize: 13,
    color: '#6b7280',
    marginBottom: 24,
  },
  offlineCard: {
    padding: 16,
    background: '#fefce8',
    border: '1px solid #fde68a',
    borderRadius: 8,
    marginBottom: 24,
  },
  grid: {
    display: 'flex',
    flexDirection: 'column',
    gap: 8,
  },
  card: {
    display: 'block',
    padding: '14px 18px',
    background: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: 8,
    textDecoration: 'none',
    transition: 'border-color 0.1s, box-shadow 0.1s',
  },
  cardTitle: {
    fontSize: 15,
    fontWeight: 600,
    color: '#111827',
    marginBottom: 4,
  },
  cardMeta: {
    fontSize: 12,
    color: '#9ca3af',
  },
  empty: {
    textAlign: 'center',
    color: '#9ca3af',
    fontSize: 14,
    padding: 48,
  },
};
