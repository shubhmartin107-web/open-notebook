import { memo, useEffect, useRef } from 'react';
import type { CellOutput as CellOutputType } from '../types';

interface Props {
  output: CellOutputType | null;
  isRunning: boolean;
}

function bytesToBase64(data: number[]): string {
  const bytes = new Uint8Array(data);
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

function VegaChart({ spec }: { spec: any }) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!ref.current) return;
    let canceled = false;
    import('vega-embed').then((mod) => {
      if (!canceled && ref.current) {
        mod.default(ref.current, spec, { actions: false });
      }
    });
    return () => { canceled = true; };
  }, [spec]);

  return <div ref={ref} />;
}

function OutputItemRenderer({ item }: { item: NonNullable<CellOutputType['items'][0]> }) {
  const text = item.text ?? '';

  if (item.mimeType === 'text/plain') {
    return <pre style={s.pre}>{text}</pre>;
  }

  if (item.mimeType === 'text/html') {
    return <div dangerouslySetInnerHTML={{ __html: text }} />;
  }

  if (item.mimeType === 'image/png') {
    const base64 = bytesToBase64(item.data);
    return (
      <img
        src={`data:image/png;base64,${base64}`}
        alt="cell output"
        style={{ maxWidth: '100%' }}
      />
    );
  }

  if (item.mimeType === 'application/vnd.vegalite+json') {
    try {
      const spec = JSON.parse(text);
      return <VegaChart spec={spec} />;
    } catch {
      return <pre style={s.pre}>{text}</pre>;
    }
  }

  if (item.mimeType === 'application/vnd.dataframe+json') {
    try {
      const df = JSON.parse(text);
      if (Array.isArray(df) && df.length > 0) {
        const headers = Object.keys(df[0]);
        return (
          <table style={s.table}>
            <thead>
              <tr>
                {headers.map(h => <th key={h} style={s.th}>{h}</th>)}
              </tr>
            </thead>
            <tbody>
              {df.map((row: any, i: number) => (
                <tr key={i}>
                  {headers.map(h => (
                    <td key={h} style={s.td}>{String(row[h] ?? '')}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        );
      }
    } catch {}
    return <pre style={s.pre}>{text}</pre>;
  }

  return (
    <div>
      <span style={s.mimeBadge}>{item.mimeType}</span>
      <pre style={s.pre}>
        {text.slice(0, 200)}
        {text.length > 200 ? '...' : ''}
      </pre>
    </div>
  );
}

const CellOutput = memo(function CellOutput({ output, isRunning }: Props) {
  if (isRunning) {
    return (
      <div style={s.spinnerContainer}>
        <div className="spinner" />
        <span style={{ fontSize: 12, color: '#888' }}>Running...</span>
      </div>
    );
  }

  if (!output) return null;

  return (
    <div style={s.container}>
      {output.errorTraceback && (
        <pre style={s.error}>{output.errorTraceback}</pre>
      )}
      {output.items.map((item, i) => (
        <OutputItemRenderer key={i} item={item} />
      ))}
      <div style={s.duration}>{output.durationMs}ms</div>
    </div>
  );
});

export default CellOutput;

const s: Record<string, React.CSSProperties> = {
  container: {
    padding: '8px 12px',
    borderTop: '1px solid #eee',
    background: '#fafafa',
    fontSize: 13,
    lineHeight: 1.5,
  },
  pre: {
    margin: 0,
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
    fontFamily: '"Fira Code", "Cascadia Code", "JetBrains Mono", monospace',
    fontSize: 12,
  },
  error: {
    margin: 0,
    padding: 8,
    background: '#fef2f2',
    border: '1px solid #fecaca',
    borderRadius: 4,
    color: '#b91c1c',
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
    fontFamily: '"Fira Code", "Cascadia Code", "JetBrains Mono", monospace',
    fontSize: 12,
  },
  mimeBadge: {
    display: 'inline-block',
    padding: '1px 6px',
    background: '#e0e7ff',
    borderRadius: 4,
    fontSize: 10,
    fontWeight: 600,
    color: '#4338ca',
    marginBottom: 4,
  },
  table: {
    borderCollapse: 'collapse',
    width: '100%',
    fontSize: 12,
    fontFamily: 'monospace',
  },
  th: {
    border: '1px solid #d1d5db',
    padding: '4px 8px',
    background: '#f3f4f6',
    fontWeight: 600,
    textAlign: 'left',
  },
  td: {
    border: '1px solid #d1d5db',
    padding: '4px 8px',
  },
  spinnerContainer: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    padding: '8px 12px',
    borderTop: '1px solid #eee',
  },
  duration: {
    marginTop: 4,
    fontSize: 11,
    color: '#9ca3af',
    textAlign: 'right',
  },
};
