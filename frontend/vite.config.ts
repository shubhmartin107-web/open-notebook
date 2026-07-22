import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    proxy: {
      '/api': 'http://localhost:8080',
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          codemirror: ['@codemirror/basic-setup', '@codemirror/lang-markdown', '@codemirror/lang-python', '@codemirror/lang-sql', '@codemirror/state', '@codemirror/theme-one-dark', '@codemirror/view', 'codemirror'],
          vega: ['vega-embed', 'vega-lite'],
        },
      },
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/setupTests.ts',
  },
});
