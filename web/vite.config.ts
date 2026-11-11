import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vite'

const api = process.env.MINDBLOW_API ?? 'http://127.0.0.1:7341'

export default defineConfig({
  plugins: [svelte()],
  server: { port: 5341, proxy: { '/api': { target: api, ws: true } } },
  build: { target: 'es2022', cssMinify: true },
})
