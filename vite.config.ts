import vue from '@vitejs/plugin-vue';
import VueJsx from '@vitejs/plugin-vue-jsx';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, URL } from 'node:url';
import AutoImport from 'unplugin-auto-import/vite';
import ViteFonts from 'unplugin-fonts/vite';
import { defineConfig } from 'vite';
import vueDevTools from 'vite-plugin-vue-devtools';
import vuetify from 'vite-plugin-vuetify';

const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [
    vue(),
    VueJsx(),
    vuetify(),
    vueDevTools(),
    ViteFonts({
      fontsource: {
        families: [
          {
            name: 'Roboto',
            weights: [400, 500, 700],
            styles: ['normal'],
          },
        ],
      },
    }),
    AutoImport({
      imports: [
        'vue',
        'vue-router',
        {
          'vue-i18n': ['useI18n', 't'],
          'vue-sonner': ['toast'],
          '@tanstack/vue-query': ['useMutation', 'useQuery', 'useQueryClient'],
        },
        {
          '@/lib/service': ['instance'],
          '@/lib/router': ['router'],
        },
        {
          '@tauri-apps/plugin-log': ['info'],
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
    chunkSizeWarningLimit: 4000,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
      },
      output: {
        manualChunks(id) {
          if (id.includes('monaco-editor')) return 'monaco-editor';
        },
      },
    },
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/target/**', '**/src-tauri/**'],
    },
  },
});
