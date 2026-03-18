import { existsSync, readFileSync, writeFileSync, mkdirSync, readdirSync, statSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';
import type { CoverageReport, ResolvedConfig } from './types.js';

function findSourceFilesInDir(dir: string): string[] {
  const results: string[] = [];
  if (!existsSync(dir)) return results;
  for (const entry of readdirSync(dir)) {
    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) {
      results.push(...findSourceFilesInDir(fullPath));
    } else if (entry.endsWith('.ts') && !entry.endsWith('.d.ts') && !entry.endsWith('.test.ts') && !entry.endsWith('.spec.ts') && !fullPath.includes('__tests__')) {
      results.push(fullPath);
    }
  }
  return results;
}

function generateSpec(moduleName: string, sourceFiles: string[], root: string, specsDir: string): string {
  const templatePath = join(specsDir, '_template.spec.md');
  let template = existsSync(templatePath)
    ? readFileSync(templatePath, 'utf-8')
    : '';

  if (!template) {
    template = `---
module: module-name
version: 1
status: draft
files: []
db_tables: []
depends_on: []
---

# Module Name

## Purpose

<!-- TODO: describe what this module does -->

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

### Exported Types

| Type | Description |
|------|-------------|

## Invariants

1. <!-- TODO -->

## Behavioral Examples

### Scenario: TODO

- **Given** precondition
- **When** action
- **Then** result

## Error Cases

| Condition | Behavior |
|-----------|----------|

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|

### Consumed By

| Module | What is used |
|--------|-------------|

## Change Log

| Date | Author | Change |
|------|--------|--------|
`;
  }

  const titleCase = moduleName
    .split('-')
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ');

  const filesYaml = sourceFiles.map((f) => `  - ${f}`).join('\n');

  let spec = template
    .replace(/^module:\s*.+$/m, `module: ${moduleName}`)
    .replace(/^status:\s*.+$/m, 'status: draft')
    .replace(/^version:\s*.+$/m, 'version: 1');

  spec = spec.replace(
    /^files:\n(?:\s+-\s+.+\n?)*/m,
    `files:\n${filesYaml}\n`,
  );

  spec = spec.replace(/^# .+$/m, `# ${titleCase}`);

  spec = spec.replace(
    /^db_tables:\n(?:\s+-\s+.+\n?)*/m,
    'db_tables: []\n',
  );

  return spec;
}

export function generateSpecsForUnspeccedModules(
  root: string,
  report: CoverageReport,
  config: ResolvedConfig,
): number {
  const specsDir = resolve(root, config.specsDir);
  let generated = 0;

  for (const moduleName of report.unspeccedModules) {
    const specDir = join(specsDir, moduleName);
    const specFile = join(specDir, `${moduleName}.spec.md`);

    // Find source files for this module across all source dirs
    const moduleFiles: string[] = [];
    for (const srcDir of config.sourceDirs) {
      const moduleDir = join(root, srcDir, moduleName);
      const files = findSourceFilesInDir(moduleDir)
        .map((f) => relative(root, f).replace(/\\/g, '/'));
      moduleFiles.push(...files);
    }

    if (moduleFiles.length === 0) continue;

    mkdirSync(specDir, { recursive: true });
    try {
      writeFileSync(specFile, generateSpec(moduleName, moduleFiles, root, specsDir), { flag: 'wx' });
    } catch (e: unknown) {
      if (e instanceof Error && 'code' in e && (e as NodeJS.ErrnoException).code === 'EEXIST') continue;
      throw e;
    }
    console.log(`  \u2713 Generated ${relative(root, specFile)} (${moduleFiles.length} files)`);
    generated++;
  }

  return generated;
}
