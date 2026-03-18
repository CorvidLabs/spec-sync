import { describe, it, expect } from 'bun:test';
import { parseFrontmatter, getSpecSymbols, getMissingSections } from '../src/parser.js';

describe('parseFrontmatter', () => {
  it('parses valid frontmatter', () => {
    const content = `---
module: auth
version: 1
status: stable
files:
  - src/auth/service.ts
db_tables:
  - users
depends_on: []
---

# Auth

## Purpose

Authentication module.
`;
    const result = parseFrontmatter(content);
    expect(result).not.toBeNull();
    expect(result!.frontmatter.module).toBe('auth');
    expect(result!.frontmatter.version).toBe('1');
    expect(result!.frontmatter.status).toBe('stable');
    expect(result!.frontmatter.files).toEqual(['src/auth/service.ts']);
    expect(result!.frontmatter.db_tables).toEqual(['users']);
    expect(result!.frontmatter.depends_on).toEqual([]);
  });

  it('returns null for missing frontmatter', () => {
    expect(parseFrontmatter('# No frontmatter here')).toBeNull();
  });

  it('returns null for malformed frontmatter', () => {
    expect(parseFrontmatter('---\nmodule: test\n# missing closing ---')).toBeNull();
  });
});

describe('getSpecSymbols', () => {
  it('extracts symbols from Public API tables', () => {
    const body = `
## Purpose

Does stuff.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| \`createUser\` | \`(name: string)\` | \`User\` | Creates a user |
| \`deleteUser\` | \`(id: string)\` | \`void\` | Deletes a user |

### Exported Types

| Type | Description |
|------|-------------|
| \`User\` | A user object |

## Invariants
`;
    const symbols = getSpecSymbols(body);
    expect(symbols).toContain('createUser');
    expect(symbols).toContain('deleteUser');
    expect(symbols).toContain('User');
  });

  it('returns empty array when no Public API section', () => {
    expect(getSpecSymbols('## Purpose\n\nJust some text.')).toEqual([]);
  });

  it('skips class method tables', () => {
    const body = `
## Public API

### Exported Classes

| Class | Description |
|-------|-------------|
| \`MyService\` | Does things |

#### MyService Methods

| Method | Parameters | Returns | Description |
|--------|-----------|---------|-------------|
| \`doThing\` | \`()\` | \`void\` | Does a thing |

## Invariants
`;
    const symbols = getSpecSymbols(body);
    expect(symbols).toContain('MyService');
    expect(symbols).not.toContain('doThing');
  });
});

describe('getMissingSections', () => {
  it('finds missing sections', () => {
    const body = '## Purpose\n\nStuff.\n\n## Public API\n\nAPI here.';
    const missing = getMissingSections(body, ['Purpose', 'Public API', 'Invariants']);
    expect(missing).toEqual(['Invariants']);
  });

  it('returns empty when all present', () => {
    const body = '## Purpose\n\n## Public API\n\n## Invariants\n';
    const missing = getMissingSections(body, ['Purpose', 'Public API', 'Invariants']);
    expect(missing).toEqual([]);
  });
});
