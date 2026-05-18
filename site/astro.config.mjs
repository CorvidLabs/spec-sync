import { defineConfig } from 'astro/config'
import mdx from '@astrojs/mdx'
import sitemap from '@astrojs/sitemap'

function rewriteMdLinks() {
  return (tree) => {
    const visit = (node) => {
      if (node.type === 'link' && typeof node.url === 'string') {
        node.url = node.url.replace(/\.md(#|$)/, '$1')
      }
      if (node.children) node.children.forEach(visit)
    }
    visit(tree)
  }
}

export default defineConfig({
  site: 'https://corvidlabs.github.io',
  base: '/spec-sync/',
  trailingSlash: 'never',
  integrations: [mdx(), sitemap()],
  markdown: {
    remarkPlugins: [rewriteMdLinks],
  },
})
