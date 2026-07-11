/**
 * Minimal RSS 2.0 feed for the spec-sync blog.
 *
 * Avoids pulling in @astrojs/rss as a new dependency — the feed shape is small
 * and we already escape the few characters that matter (XML doesn't allow
 * unescaped &, <, > in element text or CDATA-less descriptions).
 */
import type { APIContext } from 'astro'
import { getCollection } from 'astro:content'
import { contentSlug } from '../lib/content'

function xmlEscape(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;')
}

export async function GET(context: APIContext) {
  const site = context.site?.toString().replace(/\/$/, '') ?? 'https://corvidlabs.github.io'
  const base = (import.meta.env.BASE_URL ?? '/').replace(/\/$/, '')
  const posts = (await getCollection('blog', (p) => !p.data.draft)).sort(
    (a, b) => +b.data.date - +a.data.date,
  )

  const items = posts
    .map((p) => {
      const url = `${site}${base}/blog/${contentSlug(p.id)}`
      return `    <item>
      <title>${xmlEscape(p.data.title)}</title>
      <link>${url}</link>
      <guid isPermaLink="true">${url}</guid>
      <pubDate>${p.data.date.toUTCString()}</pubDate>
      <description>${xmlEscape(p.data.description)}</description>
      <author>noreply@corvidlabs.xyz (${xmlEscape(p.data.author)})</author>
    </item>`
    })
    .join('\n')

  const lastBuild = (posts[0]?.data.date ?? new Date()).toUTCString()
  const channelLink = `${site}${base}/blog`

  const body = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>spec-sync blog</title>
    <link>${channelLink}</link>
    <description>Release notes, language additions, and integration deep-dives for spec-sync.</description>
    <language>en-us</language>
    <lastBuildDate>${lastBuild}</lastBuildDate>
    <atom:link href="${site}${base}/rss.xml" rel="self" type="application/rss+xml" />
${items}
  </channel>
</rss>
`

  return new Response(body, {
    headers: { 'Content-Type': 'application/rss+xml; charset=utf-8' },
  })
}
