/// <reference types="vitest" />
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import environment from 'vite-plugin-environment';
import dotenv from 'dotenv';
import path from 'path';

export default defineConfig(({ mode }) => {
  console.log('mode = ', mode);

  const envFile = mode === 'sandbox' || mode === 'production' ? `.env.${mode}` : `.env`;
  dotenv.config({ path: path.resolve(__dirname, '..', envFile) });

  const domain = mode === 'sandbox' ? 'sandbox.icramp.xyz' : 'app.icramp.xyz';

  return {
    root: path.resolve(__dirname, 'src'),
    build: {
      outDir: path.resolve(__dirname, 'dist'),
      emptyOutDir: true,
    },
    optimizeDeps: {
      esbuildOptions: {
        define: {
          global: 'globalThis',
        },
      },
    },
    server: {
      proxy: {
        '/api': {
          target: 'http://127.0.0.1:4943',
          changeOrigin: true,
        },
      },
    },
    plugins: [
      react(),
      environment('all', { prefix: 'CANISTER_' }),
      environment('all', { prefix: 'DFX_' }),
      environment('all', { prefix: 'CONTRACT_' }),
      environment('all', { prefix: 'FRONTEND_' }),
      {
        name: 'copy-assets',
        generateBundle() {
          this.emitFile({
            type: 'asset',
            fileName: '.well-known/ic-domains',
            source: domain,
          });
          this.emitFile({
            type: 'asset',
            fileName: '.ic-assets.json',
            source: '[{"match": ".well-known", "ignore": false}]',
          });
        },
      },
    ],
    test: {
      environment: 'jsdom',
      setupFiles: path.resolve(__dirname, 'setupTests.ts'),
      cache: { dir: path.resolve(__dirname, 'node_modules/.vitest') },
    },
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: `@import "${path.resolve(
            __dirname,
            'src/index.css',
          )}";`,
        },
      },
    },
  };
});
