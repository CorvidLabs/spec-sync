import { describe, expect, test } from 'bun:test'
import { contentSlug } from './content'

describe('contentSlug', () => {
  test('removes Markdown extensions from content-layer entry IDs', () => {
    expect(contentSlug('quickstart.md')).toBe('quickstart')
    expect(contentSlug('integrations/ai-agents.mdx')).toBe('integrations/ai-agents')
  })

  test('preserves extensionless IDs', () => {
    expect(contentSlug('quickstart')).toBe('quickstart')
  })
})
