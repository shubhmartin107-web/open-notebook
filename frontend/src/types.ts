export type CellKind = 'python' | 'sql' | 'markdown' | 'raw';
export type ExecutionStatus = 'idle' | 'running' | 'success' | 'error';

export interface Cell {
  id: string;
  kind: CellKind;
  source: string;
  output: CellOutput | null;
  status: ExecutionStatus;
  executionCount: number;
}

export interface CellOutput {
  items: OutputItem[];
  errorTraceback?: string;
  durationMs: number;
}

export interface OutputItem {
  mimeType: string;
  data: number[];
  text?: string;
  renderPriority: number;
}

export interface Notebook {
  id: string;
  title: string;
  cells: Cell[];
}
