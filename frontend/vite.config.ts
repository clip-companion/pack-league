import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { companionPack } from '@companion/pack-protocol/vite';
import { fileURLToPath, URL } from 'node:url';

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
      '@': fileURLToPath(new URL('.', import.meta.url)),
    },
  },
});
