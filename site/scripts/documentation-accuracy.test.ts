import { describe, expect, test } from 'bun:test'
import { JSDOM } from 'jsdom'
import { marked } from 'marked'
import { resolve } from 'node:path'

// Mermaid's parser sanitizes labels through DOMPurify even without rendering.
const dom = new JSDOM('')
Object.defineProperty(globalThis, 'window', { value: dom.window, configurable: true })
Object.defineProperty(globalThis, 'document', { value: dom.window.document, configurable: true })
const { default: mermaid } = await import('mermaid')
mermaid.initialize({ startOnLoad: false })

const root = resolve(import.meta.dir, '../..')
const read = (path: string) => Bun.file(resolve(root, path)).text()
type Diagram = { file: string; line: number; body: string }

function diagramsIn(file: string, content: string): Diagram[] {
  const diagrams: Diagram[] = []
  let offset = 0
  marked.walkTokens(marked.lexer(content), token => {
    if (token.type !== 'code' || token.lang?.split(/\s+/)[0]?.toLowerCase() !== 'mermaid') return
    const index = content.indexOf(token.raw, offset)
    if (index < 0) throw new Error(`${file}: cannot locate Mermaid fence in source`)
    diagrams.push({ file, line: content.slice(0, index).split('\n').length, body: token.text })
    offset = index + token.raw.length
  })
  return diagrams
}

async function maintainedDiagrams(): Promise<Diagram[]> {
  const result = Bun.spawnSync(['git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'], { cwd: root })
  if (result.exitCode !== 0) throw new Error(result.stderr.toString())
  const paths = [...new Set(result.stdout.toString().split('\0'))].filter(path =>
    /\.(md|mdx|astro)$/.test(path) && (
      ['README.md', 'CONTRIBUTING.md'].includes(path) ||
      path.startsWith('docs/') || path.startsWith('site/') ||
      /^\.(agents|claude|codex|cursor|gemini)\/skills\//.test(path)
    ),
  ).sort()
  return (await Promise.all(paths.map(async file => diagramsIn(file, await read(file))))).flat()
}

function changeStates(source: string): string[] {
  const body = source.match(/pub enum ChangeState\s*\{([^}]+)\}/)?.[1]
  if (!body) throw new Error('cannot extract ChangeState')
  const stripped = body.replace(/\/\/[^\n]*/g, '')
  const states = [...stripped.matchAll(/\b([A-Z]\w*)\s*,/g)].map(match => match[1]!)
  if (!states.length || stripped.replace(/\b[A-Z]\w*\s*,/g, '').trim()) {
    throw new Error('unrecognized ChangeState definition')
  }
  if (new Set(states).size !== states.length) throw new Error('duplicate ChangeState variant')
  return states
}

function sendBackStates(source: string): string[] {
  const declaration = source.match(/const SEND_BACK_STATES:\s*\[ChangeState;\s*(\d+)\]\s*=\s*\[([^\]]+)\]/)
  if (!declaration) throw new Error('cannot extract SEND_BACK_STATES')
  const body = declaration[2]!.replace(/\/\/[^\n]*/g, '')
  const states = [...body.matchAll(/ChangeState::(\w+)\s*,/g)].map(match => match[1]!)
  if (states.length !== Number(declaration[1]) || body.replace(/ChangeState::\w+\s*,/g, '').trim()) {
    throw new Error('unrecognized SEND_BACK_STATES definition')
  }
  if (!/record\.state\s*=\s*ChangeState::Draft\s*;/.test(source)) {
    throw new Error('send-back destination changed; review the diagram contract')
  }
  return states
}

function assertLifecycle(body: string, states: string[], sendBack: string[]): void {
  for (const line of body.split('\n').map(line => line.trim())) {
    if (!line || line === 'stateDiagram-v2' || line.startsWith('%%')) continue
    if (!/^(\[\*\]|\w+)\s*-->\s*(\[\*\]|\w+)(?:\s*:\s*[^\n]+)?$/.test(line)) {
      throw new Error(`unrecognized lifecycle diagram statement: ${line}`)
    }
  }
  const edges = [...body.matchAll(/^\s*(\[\*\]|\w+)\s*-->\s*(\[\*\]|\w+)(?:\s*:\s*([^\n]+))?\s*$/gm)]
  if (!edges.length) throw new Error('lifecycle diagram contains no transitions')
  const nodes = new Set(edges.flatMap(edge => [edge[1]!, edge[2]!]).filter(node => node !== '[*]'))
  expect([...nodes].sort()).toEqual([...states].sort())
  const recovery = edges.filter(edge => /\bchange send-back\b/.test(edge[3] ?? ''))
  expect(recovery.map(edge => `${edge[1]}->${edge[2]}`).sort())
    .toEqual(sendBack.map(state => `${state}->Draft`).sort())
}

function assertArchiveCommitBeforePush(body: string): void {
  const commit = body.indexOf('chore(lifecycle): archive')
  const push = body.search(/^\s*opt change ship --push/m)
  if (commit < 0 || push < 0 || commit >= push) {
    throw new Error('native archive commit must precede the optional push block')
  }
}

const diagrams = await maintainedDiagrams()
const hld = await read('docs/HLD.md')
const states = changeStates(await read('src/change/sdd_version.rs'))
const sendBack = sendBackStates(await read('src/change/send_back.rs'))
const lifecycle = diagrams.filter(diagram => diagram.file === 'docs/HLD.md' && diagram.body.startsWith('stateDiagram-v2'))
const closeout = diagrams.filter(diagram => diagram.file === 'docs/HLD.md' && diagram.body.includes('change finalize id (or change ship id)'))

describe('maintained Mermaid syntax (REQ-github-027)', () => {
  test('discovers the README and HLD diagrams', () => {
    expect(diagrams.some(diagram => diagram.file === 'README.md')).toBe(true)
    expect(lifecycle).toHaveLength(1)
    expect(closeout).toHaveLength(1)
  })
  for (const diagram of diagrams) {
    test(`${diagram.file}:${diagram.line} parses`, async () => {
      expect(await mermaid.parse(diagram.body)).toHaveProperty('diagramType')
    })
  }
  test('rejects malformed syntax instead of reporting a parser pass', async () => {
    await expect(mermaid.parse('flowchart LR\nA[unterminated')).rejects.toThrow()
  })
  test('extracts metadata and tilde fences, excluding other languages', () => {
    const text = '~~~mermaid title="flow"\nflowchart LR\nA-->B\n~~~\n```rust\nnot mermaid\n```\n'
    expect(diagramsIn('fixture.md', text)).toEqual([{ file: 'fixture.md', line: 1, body: 'flowchart LR\nA-->B' }])
  })
})

describe('lifecycle meaning (REQ-github-027)', () => {
  test('matches ChangeState and every SEND_BACK_STATES transition', () => {
    expect(lifecycle).toHaveLength(1)
    assertLifecycle(lifecycle[0]!.body, states, sendBack)
  })
  const valid = [
    'stateDiagram-v2',
    ...states.map(state => `Draft --> ${state}: fixture`),
    ...sendBack.map(state => `${state} --> Draft: change send-back`),
  ].join('\n')
  test('accepts matching states and recovery edges', () => assertLifecycle(valid, states, sendBack))
  test('rejects a missing state', () => {
    const missing = valid.split('\n').filter(line => !line.includes('Archived')).join('\n')
    expect(() => assertLifecycle(missing, states, sendBack)).toThrow()
  })
  test('rejects a phantom state', () => {
    expect(() => assertLifecycle(`${valid}\nDraft --> Phantom: fixture`, states, sendBack)).toThrow()
    expect(() => assertLifecycle(`${valid}\nstate Phantom`, states, sendBack)).toThrow()
  })
  test('rejects a missing recovery edge', () => {
    const missing = valid.replace(`${sendBack[0]} --> Draft: change send-back`, '')
    expect(() => assertLifecycle(missing, states, sendBack)).toThrow()
  })
  test('rejects an incorrect recovery destination', () => {
    const changed = valid.replace(`${sendBack[0]} --> Draft: change send-back`, `${sendBack[0]} --> Archived: change send-back`)
    expect(() => assertLifecycle(changed, states, sendBack)).toThrow()
  })
  test('fails closed when the source definitions cannot be read', () => {
    expect(() => changeStates('pub enum Renamed { Draft, }')).toThrow()
    expect(() => changeStates('pub enum ChangeState { Draft(String), }')).toThrow()
    expect(() => sendBackStates('const SEND_BACK_STATES: [ChangeState; 2] = [ChangeState::Approved,];')).toThrow()
  })
})

describe('native closeout documentation', () => {
  test('describes explicit lifecycle staging, not git add -A', async () => {
    expect(await read('src/commands/change.rs')).toContain('fn stage_lifecycle_paths(')
    expect(hld.includes('stage_lifecycle_paths')).toBe(true)
    expect(hld.includes('git add -A')).toBe(false)
    expect(hld.includes('git_commit_all')).toBe(false)
  })
  test('shows the automatic archive commit before optional push', () => {
    expect(closeout).toHaveLength(1)
    assertArchiveCommitBeforePush(closeout[0]!.body)
  })
  test('rejects an archive commit incorrectly nested under push', () => {
    expect(() => assertArchiveCommitBeforePush('opt change ship --push\nCMD->>G: chore(lifecycle): archive id\nend')).toThrow()
    expect(() => assertArchiveCommitBeforePush('CMD->>G: chore(lifecycle): archive id\nopt change ship --push\nend')).not.toThrow()
  })
  test('does not schedule remote review on an open verifying change', async () => {
    expect(await read('.github/workflows/ci.yml')).toContain('reviews every archived pull')
    expect(hld.includes('opt a verifying change needs review')).toBe(false)
  })
})
