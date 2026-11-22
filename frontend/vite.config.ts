import { fileURLToPath } from 'node:url'
import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig(({ mode }) => {
  const projectRoot = fileURLToPath(new URL('../', import.meta.url))
  const { HTTP_PORT = '8081' } = loadEnv(mode, projectRoot, 'HTTP_PORT')

  return {
    plugins: [react()],
    server: {
      proxy: {
        '/v1': {
          target: `http://127.0.0.1:${HTTP_PORT}`,
          changeOrigin: true,
        },
      },
    },
  }
})
