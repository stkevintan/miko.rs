import { defineConfig, loadEnv } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  const port = parseInt(env.PORT || '8081')

  return {
    plugins: [
      svelte(),
      tailwindcss(),
    ],
    server: {
      port: port === 8081 ? 8082 : 8081,
      proxy: {
        '/api': {
          target: `http://localhost:${port}`,
          changeOrigin: true,
        },
        '/rest': {
          target: `http://localhost:${port}`,
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
  }
})
