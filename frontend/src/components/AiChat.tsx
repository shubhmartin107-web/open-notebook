import { useState, useRef, useEffect } from 'react';

export interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
}

interface Props {
  notebookId: string;
  cellIndex: number | null;
  onInsertCode: (code: string) => void;
}

function extractCode(text: string): string {
  const blocks: string[] = [];
  const regex = /```(?:\w+)?\n?([\s\S]*?)```/g;
  let match;
  while ((match = regex.exec(text)) !== null) {
    blocks.push(match[1].trim());
  }
  return blocks.length > 0 ? blocks.join('\n\n') : text;
}

export default function AiChat({ notebookId, cellIndex, onInsertCode }: Props) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (listRef.current) {
      listRef.current.scrollTop = listRef.current.scrollHeight;
    }
  }, [messages, loading]);

  const sendMessage = async () => {
    const prompt = input.trim();
    if (!prompt || loading) return;
    setInput('');
    setError(null);

    setMessages(prev => [...prev, { role: 'user', content: prompt, timestamp: Date.now() }]);
    setLoading(true);

    try {
      const res = await fetch(
        `/api/notebooks/${encodeURIComponent(notebookId)}/execute`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            cmd: 'ai_generate',
            notebook: {},
            cell_index: cellIndex ?? 0,
            prompt,
          }),
        },
      );

      if (!res.ok) {
        const text = await res.text().catch(() => '');
        throw new Error(`Request failed (${res.status}): ${text || res.statusText}`);
      }

      const data = await res.json();
      const content = data.text ?? data.content ?? data.result ?? JSON.stringify(data);

      setMessages(prev => [
        ...prev,
        { role: 'assistant', content, timestamp: Date.now() },
      ]);
    } catch (e: any) {
      setError(e.message ?? String(e));
    } finally {
      setLoading(false);
      inputRef.current?.focus();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  const disabled = loading || cellIndex === null;

  return (
    <div style={s.container}>
      <div style={s.header}>
        <span>AI Chat</span>
        <span style={s.cellBadge}>
          {cellIndex !== null ? `Cell ${cellIndex + 1}` : 'No cell'}
        </span>
      </div>

      <div ref={listRef} style={s.messageList}>
        {messages.length === 0 && !loading && (
          <div style={s.empty}>
            Ask the AI to generate code or answer questions about your notebook.
          </div>
        )}

        {messages.map((msg, i) => (
          <div key={i} style={msg.role === 'user' ? s.userMsg : s.assistantMsg}>
            <div style={s.msgLabel}>
              {msg.role === 'user' ? 'You' : 'AI'}
            </div>
            <div style={s.msgContent}>{msg.content}</div>
            {msg.role === 'assistant' && (
              <button
                style={s.codeBtn}
                onClick={() => onInsertCode(extractCode(msg.content))}
              >
                Generate Code
              </button>
            )}
          </div>
        ))}

        {loading && (
          <div style={s.assistantMsg}>
            <div style={s.msgLabel}>AI</div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '8px 0' }}>
              <span className="spinner" />
              <span style={{ fontSize: 12, color: '#6b7280' }}>Generating...</span>
            </div>
          </div>
        )}

        {error && <div style={s.errorMsg}>{error}</div>}
      </div>

      <div style={s.inputArea}>
        <input
          ref={inputRef}
          style={s.input}
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={cellIndex !== null ? 'Ask AI something...' : 'Select a cell first'}
          disabled={disabled}
        />
        <button
          style={s.sendBtn}
          onClick={sendMessage}
          disabled={disabled || !input.trim()}
        >
          Send
        </button>
      </div>
    </div>
  );
}

const s: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    fontFamily:
      '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
    fontSize: 13,
    color: '#1f2937',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '12px 16px',
    borderBottom: '1px solid #e5e7eb',
    fontWeight: 700,
    fontSize: 14,
    background: '#f9fafb',
    flexShrink: 0,
  },
  cellBadge: {
    fontSize: 11,
    fontWeight: 600,
    color: '#6b7280',
    background: '#f3f4f6',
    padding: '2px 8px',
    borderRadius: 4,
  },
  messageList: {
    flex: 1,
    overflowY: 'auto',
    padding: '12px 16px',
    display: 'flex',
    flexDirection: 'column',
    gap: 12,
  },
  empty: {
    textAlign: 'center',
    color: '#9ca3af',
    fontSize: 12,
    padding: '32px 16px',
  },
  userMsg: {
    alignSelf: 'flex-end',
    maxWidth: '85%',
    background: '#eff6ff',
    border: '1px solid #bfdbfe',
    borderRadius: '8px 8px 4px 8px',
    padding: '8px 12px',
  },
  assistantMsg: {
    alignSelf: 'flex-start',
    maxWidth: '90%',
    background: '#f9fafb',
    border: '1px solid #e5e7eb',
    borderRadius: '8px 8px 8px 4px',
    padding: '8px 12px',
  },
  msgLabel: {
    fontSize: 10,
    fontWeight: 700,
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
    color: '#6b7280',
    marginBottom: 4,
  },
  msgContent: {
    fontSize: 13,
    lineHeight: 1.5,
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
  },
  codeBtn: {
    marginTop: 8,
    padding: '4px 10px',
    fontSize: 11,
    fontWeight: 600,
    background: '#f0fdf4',
    border: '1px solid #bbf7d0',
    borderRadius: 4,
    color: '#166534',
    cursor: 'pointer',
    display: 'block',
  },
  errorMsg: {
    padding: '8px 12px',
    background: '#fef2f2',
    border: '1px solid #fecaca',
    borderRadius: 6,
    color: '#b91c1c',
    fontSize: 12,
  },
  inputArea: {
    display: 'flex',
    gap: 8,
    padding: '12px 16px',
    borderTop: '1px solid #e5e7eb',
    background: '#fff',
    flexShrink: 0,
  },
  input: {
    flex: 1,
    padding: '8px 12px',
    borderRadius: 6,
    border: '1px solid #d1d5db',
    fontSize: 13,
    fontFamily: 'inherit',
    outline: 'none',
  },
  sendBtn: {
    padding: '8px 16px',
    borderRadius: 6,
    border: '1px solid #d1d5db',
    background: '#fff',
    fontSize: 13,
    fontWeight: 600,
    cursor: 'pointer',
    whiteSpace: 'nowrap',
  },
};
