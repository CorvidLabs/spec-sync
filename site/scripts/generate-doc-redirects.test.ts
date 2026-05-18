import { describe, test, expect } from 'bun:test'
import { redirectHtml, computeRedirects } from './generate-doc-redirects'

describe('redirectHtml', () => {
  test('emits meta-refresh + canonical', () => {
    const html = redirectHtml('/spec-sync/docs/quickstart')
    expect(html).toContain('<meta http-equiv="refresh" content="0; url=/spec-sync/docs/quickstart">')
    expect(html).toContain('canonical')
    expect(html).toContain('location.replace')
  })

  test('escapes target URL in JS via JSON.stringify', () => {
    const html = redirectHtml('/spec-sync/docs/cross-project-refs')
    expect(html).toContain('"/spec-sync/docs/cross-project-refs"')
  })
})

describe('computeRedirects', () => {
  test('maps top-level mdBook pages to new docs paths', () => {
    const mapped = computeRedirects(['quickstart.md', 'cli.md', 'spec-format.md'])
    expect(mapped['quickstart.html']).toBe('/spec-sync/docs/quickstart')
    expect(mapped['cli.html']).toBe('/spec-sync/docs/cli')
    expect(mapped['spec-format.html']).toBe('/spec-sync/docs/spec-format')
  })

  test('maps nested mdBook pages to new docs paths', () => {
    const mapped = computeRedirects(['integrations/github-action.md', 'integrations/vscode-extension.md'])
    expect(mapped['integrations/github-action.html']).toBe('/spec-sync/docs/integrations/github-action')
    expect(mapped['integrations/vscode-extension.html']).toBe('/spec-sync/docs/integrations/vscode-extension')
  })

  test('skips SUMMARY.md', () => {
    const mapped = computeRedirects(['SUMMARY.md', 'quickstart.md'])
    expect('SUMMARY.html' in mapped).toBe(false)
    expect(mapped['quickstart.html']).toBe('/spec-sync/docs/quickstart')
  })

  test('skips index.md', () => {
    const mapped = computeRedirects(['index.md', 'cli.md'])
    expect('index.html' in mapped).toBe(false)
    expect(mapped['cli.html']).toBe('/spec-sync/docs/cli')
  })
})
