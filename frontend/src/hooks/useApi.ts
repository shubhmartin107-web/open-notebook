import { useCallback } from 'react';

const BASE = '/api/notebooks';

interface Notebook {
  id: string;
  title: string;
  cells: any[];
}

async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const res = await fetch(url, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  if (!res.ok) {
    const body = await res.text().catch(() => '');
    throw new Error(`API error ${res.status}: ${body}`);
  }
  return res.json();
}

export function useApi() {
  const listNotebooks = useCallback(async () =>
    request<Notebook[]>(BASE), []);

  const createNotebook = useCallback(async (title: string) =>
    request<Notebook>(BASE, {
      method: 'POST',
      body: JSON.stringify({ title }),
    }), []);

  const getNotebook = useCallback(async (id: string) =>
    request<Notebook>(`${BASE}/${id}`), []);

  const updateNotebook = useCallback(async (id: string, data: Partial<Notebook>) =>
    request<Notebook>(`${BASE}/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }), []);

  const deleteNotebook = useCallback(async (id: string) => {
    await fetch(`${BASE}/${id}`, { method: 'DELETE' });
  }, []);

  const addCell = useCallback(async (notebookId: string, kind: string, source: string) =>
    request<any>(`${BASE}/${notebookId}/cells`, {
      method: 'POST',
      body: JSON.stringify({ kind, source }),
    }), []);

  const updateCell = useCallback(async (notebookId: string, cellId: string, source: string) =>
    request<any>(`${BASE}/${notebookId}/cells/${cellId}`, {
      method: 'PUT',
      body: JSON.stringify({ source }),
    }), []);

  const deleteCell = useCallback(async (notebookId: string, cellId: string) => {
    await fetch(`${BASE}/${notebookId}/cells/${cellId}`, { method: 'DELETE' });
  }, []);

  const executeNotebook = useCallback(async (notebookId: string) =>
    request<Notebook>(`${BASE}/${notebookId}/execute`, {
      method: 'POST',
    }), []);

  const executeCellFull = useCallback(async (notebookId: string, cellId: string) =>
    request<Notebook>(`${BASE}/${notebookId}/cells/${cellId}/execute`, {
      method: 'POST',
    }), []);

  return {
    listNotebooks,
    createNotebook,
    getNotebook,
    updateNotebook,
    deleteNotebook,
    addCell,
    updateCell,
    deleteCell,
    executeNotebook,
    executeCellFull,
  };
}
