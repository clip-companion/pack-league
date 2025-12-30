import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  build: {
    lib: {
      entry: path.resolve(__dirname, 'index.ts'),
      name: 'LeaguePack',
      fileName: () => 'frontend.js',
      formats: ['iife'],
    },
    rollupOptions: {
      // React is provided by the host app
      external: ['react', 'react-dom'],
      output: {
        globals: {
          react: 'React',
          'react-dom': 'ReactDOM',
        },
        // Export to global namespace for dynamic loading
        footer: `
if (typeof window !== 'undefined') {
  if (!window.__COMPANION_PACKS__) window.__COMPANION_PACKS__ = {};
  window.__COMPANION_PACKS__['league'] = LeaguePack.default;
}
        `,
      },
    },
    // Output to dist directory
    outDir: 'dist',
    // Generate a single file
    minify: true,
    sourcemap: false,
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, '.'),
    },
  },
});
