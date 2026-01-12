import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    svelte(),
    tailwindcss(),
  ],
  server: {
    port: 8081,
    proxy: {
      '/api': {
        target: 'http://localhost:3334',
        changeOrigin: true,
      },
      '/rest': {
        target: 'http://localhost:3334',
        changeOrigin: true,
      },
    }
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './web'),
    },
  },
  build: {
    outDir: './dist',
    emptyOutDir: true,
  },
})
