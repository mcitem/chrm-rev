import { createAntdPlugin } from '@antdv-next/tailwind'

/** @type {import('tailwindcss').Config} */

module.exports = {
  darkMode: 'selector',
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx,vue}'],
  theme: {
    extend: {
      fontFamily: {
        mc10: ['Minecraft10', 'sans-serif'],
      },
    },
  },
  plugins: [
    createAntdPlugin({
      antPrefix: 'ant',
    }),
  ],
}
