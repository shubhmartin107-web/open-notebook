import { describe, it, expect } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import App from './App';

describe('App', () => {
  it('renders the title input', async () => {
    window.history.pushState({}, '', '/test-notebook');
    render(<App />);
    await waitFor(() => {
      expect(screen.getByPlaceholderText('Notebook title...')).toBeDefined();
    });
  });

  it('renders add cell buttons', async () => {
    window.history.pushState({}, '', '/test-notebook');
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText('+ Python')).toBeDefined();
      expect(screen.getByText('+ SQL')).toBeDefined();
    });
  });
});
