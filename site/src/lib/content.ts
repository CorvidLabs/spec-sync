export function contentSlug(id: string): string {
  return id.replace(/\.(?:md|mdx)$/, '')
}
