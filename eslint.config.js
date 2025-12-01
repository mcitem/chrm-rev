import antfu from '@antfu/eslint-config'
import { FlatCompat } from '@eslint/eslintrc'

const compat = new FlatCompat()

export default antfu(
  {
    formatters: true,
    vue: {
      overrides: {
        'vue/block-order': [
          'error',
          {
            order: ['template', 'script', 'style'],
          },
        ],
      },
    },
    rules: {
      'no-new': 'off',
      'no-console': 'off',
    },
    ignores: [
      './dist/**/*',
      './target/**/*',
      './node_modules/**/*',
      './src/bindings/**/*',
      './dcos/.vitepress/dist/*',
      './dcos/.vitepress/cache/**/*',
      './src-tauri/Microsoft.WebView2.FixedVersionRuntime.109.0.1518.78.x86/**/*',
    ],
  },
  ...compat.config({
    extends: ['plugin:tailwindcss/recommended'],
    rules: {
      'tailwindcss/no-custom-classname': 'off',
    },
  }),
)
