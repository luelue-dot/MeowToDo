import { resolve } from 'path'

import Vue from '@vitejs/plugin-vue'
import Unocss from 'unocss/vite'
import { defineConfig } from 'vite'
import dayjs from 'vite-plugin-dayjs'

const host = process.env.TAURI_DEV_HOST

export default defineConfig(async () => ({
  plugins: [
    Vue(),
    Unocss(),
    dayjs(),
  ],

  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },

  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
}))