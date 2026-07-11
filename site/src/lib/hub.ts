/**
 * Canonical CorvidLabs hub URLs that supersede the retired standalone site.
 *
 *  - Marketing (incl. the languages gallery and blog) -> HUB_MARKETING
 *  - Docs and examples                                 -> HUB_DOCS
 */
export const HUB_MARKETING = 'https://corvidlabs.xyz/spec-sync/'
export const HUB_DOCS = 'https://corvidlabs.xyz/spec-sync/docs/'

/** Return the canonical hub URL for one documentation slug. */
export function hubDocsUrl(slug: string): string {
  const normalized = slug.replace(/^\/+|\/+$/g, '')
  return new URL(`${normalized}/`, HUB_DOCS).toString()
}
