import { defineConfig } from 'astro/config'
import { unified } from '@astrojs/markdown-remark'
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
  // Preserve pre-v7 whitespace between inline elements (avoid "helloworld" gluing).
  compressHTML: true,
  legacy: {
    collectionsBackwardsCompat: true,
  },
  integrations: [mdx(), sitemap()],
  markdown: {
    // Keep the existing remark link rewrite; Astro 7 defaults to Sätteri which
    // does not use remarkPlugins without an explicit unified() processor.
    processor: unified({
      remarkPlugins: [rewriteMdLinks],
    }),
    shikiConfig: {
      // github-dark-high-contrast passes WCAG AA for all token colors
      // (#6A737D comment color in github-dark fails 3.05:1 on its #24292e bg)
      theme: 'github-dark-high-contrast',
    },
  },
})
