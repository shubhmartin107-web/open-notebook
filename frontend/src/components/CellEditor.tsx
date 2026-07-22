import { memo, useRef, useEffect } from 'react';
import { EditorView, basicSetup } from 'codemirror';
import { python } from '@codemirror/lang-python';
import { sql } from '@codemirror/lang-sql';
import { markdown } from '@codemirror/lang-markdown';
import { oneDark } from '@codemirror/theme-one-dark';
import { Transaction } from '@codemirror/state';
import type { CellKind } from '../types';

interface Props {
  initialSource: string;
  kind: CellKind;
  onChange: (source: string) => void;
}

const langExtensions: Record<CellKind, () => ReturnType<typeof python>> = {
  python,
  sql,
  markdown,
  raw: () => [] as any,
};

const CellEditor = memo(function CellEditor({ initialSource, kind, onChange }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const initialRef = useRef(initialSource);

  useEffect(() => {
    if (!containerRef.current) return;

    const langExt = langExtensions[kind]?.() ?? [];
    const view = new EditorView({
      doc: initialRef.current,
      extensions: [
        basicSetup,
        oneDark,
        langExt,
      ],
      parent: containerRef.current,
      dispatch: (tr: Transaction) => {
        view.update([tr]);
        if (tr.docChanged) {
          onChangeRef.current(view.state.doc.toString());
        }
      },
    });
    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, [kind]);

  return (
    <div
      ref={containerRef}
      style={{ borderTop: '1px solid #eee', minHeight: 36 }}
    />
  );
});

export default CellEditor;
