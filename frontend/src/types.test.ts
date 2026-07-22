import { describe, it, expect } from 'vitest';
import type { CellKind } from './types';

describe('types', () => {
  it('CellKind values match expected', () => {
    const kind: CellKind = 'python';
    expect(kind).toBe('python');
  });
});
