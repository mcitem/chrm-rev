import process from 'node:process'
import { defineConfig } from 'vitepress'

const base = process.env.DOCS_BASE || '/docs/'
console.log(`docs base: ${base}`)

export default defineConfig({
  title: 'chrm rev',
  description: 'chrm rev docs',
  base,
  outDir: '../dist/docs',
  themeConfig: {
    sidebar: [
      { text: '简介', link: '/intro' },
      { text: '菜单', link: '/menu' },
      {
        text: '业务',
        items: [
          { text: '关于余额', link: '/balance' },
          { text: '业务流程', link: '/biz' },
        ],
      },
      {
        text: '维护',
        items: [
          { text: '数据目录', link: '/datadir' },
          { text: '启动流程说明', link: '/bootloader' },
          { text: '配置管理', link: '/configloader' },
          { text: '数据导入', link: '/dataloader' },
          { text: '备份、恢复备份', link: '/backup' },
          { text: '如何编辑已经导入的数据', link: '/edit' },
          { text: '使用DataGrip编辑数据', link: '/datagrip' },
        ],
      },
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/mcitem/chrm-rev' },
    ],
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2019-present Evan You',
    },
  },
  sitemap: {
    hostname: 'https://chrm-rev.mcitem.net',
  },
  head: [
    ['meta', { name: 'baidu-site-verification', content: 'codeva-T06X5lyw0q' }],
    ['meta', { name: '360-site-verification', content: '4738aa5736873967a39c21cdfb6b67c0' }],
  ],
})
