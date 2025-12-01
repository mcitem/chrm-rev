import { resolve } from 'node:path'
import { AntdvNextResolver } from '@antdv-next/auto-import-resolver'
import vue from '@vitejs/plugin-vue'
import VueJsx from '@vitejs/plugin-vue-jsx'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [
    vue(),
    VueJsx(),
    Components({
      dts: 'src/lib/components.d.ts',
      resolvers: [AntdvNextResolver()],
    }),
    AutoImport({
      imports: [
        'vue',
        'vue-router',
        {
          '@tanstack/vue-query': ['useMutation', 'useQuery', 'useQueryClient'],
        },
        {
          '@/lib/service': ['instance'],
          '@/lib/router': ['router'],
        },
      ],
      dts: 'src/lib/auto-imports.d.ts',
      eslintrc: {
        enabled: true,
        filepath: 'src/assets/auto-imports.json',
      },
    }),
  ],
  build: {
    target: ['chrome109', 'edge109'],
    chunkSizeWarningLimit: 4300,
    rollupOptions: {
      checks: { pluginTimings: false },
      input: {
        main: resolve('index.html'),
      },
      output: {
        manualChunks(id) {
          if (id.includes('monaco-editor'))
            return 'monaco-editor'
        },
      },
    },
  },
  resolve: {
    alias: {
      '@': resolve('./src'),
      '@root': resolve('.'),
    },
  },
  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/target/**', '**/src-tauri/**'],
    },
    proxy: {
      '/docs': {
        target: 'http://localhost:5173',
      },
    },
  },
})
