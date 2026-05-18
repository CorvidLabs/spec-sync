import { defineConfig } from 'astro/config'
import mdx from '@astrojs/mdx'
import sitemap from '@astrojs/sitemap'

export default defineConfig({
  site: 'https://corvidlabs.github.io',
  base: '/spec-sync/',
  trailingSlash: 'never',
  integrations: [mdx(), sitemap()],
})
