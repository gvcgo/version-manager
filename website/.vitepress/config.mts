import { defineConfig } from 'vitepress'

export default defineConfig({
  base: '/version-manager/',
  title: 'vmr',
  head: [['link', { rel: 'icon', type: 'image/png', href: '/version-manager/brand/mark.png' }]],
  locales: {
    root: {
      label: 'English',
      lang: 'en-US',
      description: 'Cross-platform SDK version manager',
      themeConfig: {
        nav: [
          { text: 'Guide', link: '/starts/quickstart' },
          { text: 'Commands', link: '/reference/commands' },
          { text: 'Plugins', link: '/plugins/' },
        ],
        sidebar: [
          {
            text: 'Getting Started',
            items: [
              { text: 'Quick Start', link: '/starts/quickstart' },
              { text: 'Core Concepts', link: '/starts/concepts' },
            ],
          },
          {
            text: 'Guides',
            items: [
              { text: 'Tutorial', link: '/guides/tutorial' },
              { text: 'FAQ', link: '/guides/faq' },
            ],
          },
          {
            text: 'Reference',
            items: [
              { text: 'Commands', link: '/reference/commands' },
              { text: 'Plugins', link: '/plugins/' },
            ],
          },
        ],
      },
    },
    zh: {
      label: '简体中文',
      lang: 'zh-CN',
      description: '跨平台 SDK 版本管理器',
      themeConfig: {
        nav: [
          { text: '指南', link: '/zh/starts/quickstart' },
          { text: '命令', link: '/zh/reference/commands' },
          { text: '插件', link: '/zh/plugins/' },
        ],
        sidebar: [
          {
            text: '开始使用',
            items: [
              { text: '快速开始', link: '/zh/starts/quickstart' },
              { text: '核心概念', link: '/zh/starts/concepts' },
            ],
          },
          {
            text: '指南',
            items: [
              { text: '教程', link: '/zh/guides/tutorial' },
              { text: '常见问题', link: '/zh/guides/faq' },
            ],
          },
          {
            text: '参考',
            items: [
              { text: '命令', link: '/zh/reference/commands' },
              { text: '插件', link: '/zh/plugins/' },
            ],
          },
        ],
        outline: { label: '本页目录' },
        docFooter: { prev: '上一页', next: '下一页' },
        lastUpdated: { text: '最后更新' },
        returnToTopLabel: '返回顶部',
        sidebarMenuLabel: '菜单',
      },
    },
  },
  themeConfig: {
    logo: '/brand/mark.png',
    socialLinks: [{ icon: 'github', link: 'https://github.com/gvcgo/version-manager' }],
    search: {
      provider: 'local',
      options: {
        locales: {
          zh: {
            translations: {
              button: { buttonText: '搜索文档', buttonAriaLabel: '搜索文档' },
              modal: {
                noResultsText: '无法找到相关结果',
                resetButtonTitle: '清除查询条件',
                footer: { selectText: '选择', navigateText: '切换', closeText: '关闭' },
              },
            },
          },
        },
      },
    },
  },
})
