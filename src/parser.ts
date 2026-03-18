import type { Frontmatter } from './types.js';

/**
 * Parse YAML frontmatter from a spec file.
 * Zero-dependency: uses regex, no YAML parser needed.
 */
export function parseFrontmatter(content: string): { frontmatter: Frontmatter; body: string } | null {
  const match = content.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);
  if (!match) return null;

  const yamlBlock = match[1];
  const body = match[2];
  const fm: Frontmatter = {};

  let currentKey: string | null = null;
  let currentList: string[] | null = null;

  for (const line of yamlBlock.split('\n')) {
    // List item: "  - value"
    const listMatch = line.match(/^\s+-\s+(.+)$/);
    if (listMatch && currentKey && currentList) {
      currentList.push(listMatch[1].trim());
      continue;
    }

    // Key-value: "key: value" or "key:"
    const kvMatch = line.match(/^(\w[\w_]*)\s*:\s*(.*)$/);
    if (kvMatch) {
      if (currentKey && currentList) {
        (fm as Record<string, unknown>)[currentKey] = currentList;
      }

      const key = kvMatch[1];
      const value = kvMatch[2].trim();

      if (value === '' || value === '[]') {
        currentKey = key;
        currentList = [];
      } else {
        if (currentKey && currentList) {
          (fm as Record<string, unknown>)[currentKey] = currentList;
        }
        (fm as Record<string, unknown>)[key] = value;
        currentKey = null;
        currentList = null;
      }
      continue;
    }

    if (line.trim() === '' || line.trim().startsWith('#')) {
      if (currentKey && currentList) {
        (fm as Record<string, unknown>)[currentKey] = currentList;
        currentKey = null;
        currentList = null;
      }
    }
  }

  if (currentKey && currentList) {
    (fm as Record<string, unknown>)[currentKey] = currentList;
  }

  return { frontmatter: fm, body };
}

/**
 * Extract symbol names from the spec's Public API section.
 * Only extracts the FIRST backtick-quoted word in each table row.
 * Skips class method sub-tables (not top-level exports).
 */
export function getSpecSymbols(body: string): string[] {
  const symbols: string[] = [];

  const publicApiMatch = body.match(/## Public API\s*\n([\s\S]*?)(?=\n## (?!.*Public API))/);
  if (!publicApiMatch) return symbols;

  const apiSection = publicApiMatch[1];
  const subSections = apiSection.split(/(?=^### )/m);

  for (const sub of subSections) {
    const headerMatch = sub.match(/^### (.+)/);
    if (!headerMatch) continue;
    const header = headerMatch[1].trim();

    // Skip class method/constructor tables
    if (/Methods$/.test(header)) continue;
    if (/Constructor$/.test(header)) continue;

    const lines = sub.split('\n');
    let inMethodSubSection = false;

    for (const line of lines) {
      if (/^####\s+.*(?:Methods|Constructor|Properties)/.test(line)) {
        inMethodSubSection = true;
        continue;
      }
      if (/^###\s+/.test(line) && !line.startsWith('### ')) {
        inMethodSubSection = false;
      }
      if (inMethodSubSection) continue;

      const rowMatch = line.match(/^\|\s*`(\w+)`/);
      if (rowMatch) {
        symbols.push(rowMatch[1]);
      }
    }
  }

  return [...new Set(symbols)];
}

/** Check which required sections are missing from the spec body */
export function getMissingSections(body: string, requiredSections: string[]): string[] {
  const missing: string[] = [];
  for (const section of requiredSections) {
    const pattern = new RegExp(`^## ${section.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`, 'm');
    if (!pattern.test(body)) {
      missing.push(section);
    }
  }
  return missing;
}
