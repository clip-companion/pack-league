import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { companionPack } from '@companion/pack-runtime/vite';
import path from 'path';

export default defineConfig({
  plugins: [
    react(),
    companionPack({
      packId: 'league',
      packName: 'LeaguePack',
    }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, '.'),
    },
  },
});
