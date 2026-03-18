import { readFileSync, existsSync, readdirSync, statSync } from 'node:fs';
import { join, resolve, relative } from 'node:path';
import type { ValidationResult, CoverageReport, ResolvedConfig } from './types.js';
import { parseFrontmatter, getSpecSymbols, getMissingSections } from './parser.js';
import { getExportedSymbols } from './exports.js';

// ─── Schema Table Discovery ────────────────────────────────────────────

export function getSchemaTableNames(root: string, config: ResolvedConfig): Set<string> {
  const tables = new Set<string>();
  if (!config.schemaDir) return tables;

  const schemaDir = resolve(root, config.schemaDir);
  if (!existsSync(schemaDir)) return tables;

  const regex = new RegExp(config.schemaPattern, 'g');
  for (const entry of readdirSync(schemaDir)) {
    if (!entry.endsWith('.ts') && !entry.endsWith('.sql')) continue;
    const content = readFileSync(join(schemaDir, entry), 'utf-8');
    let match: RegExpExecArray | null;
    while ((match = regex.exec(content)) !== null) {
      tables.add(match[1]);
    }
  }
  return tables;
}

// ─── File Discovery ────────────────────────────────────────────────────

export function findSpecFiles(dir: string): string[] {
  const results: string[] = [];
  if (!existsSync(dir)) return results;

  for (const entry of readdirSync(dir)) {
    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) {
      results.push(...findSpecFiles(fullPath));
    } else if (entry.endsWith('.spec.md')) {
      results.push(fullPath);
    }
  }
  return results;
}

function findSourceFiles(dir: string, excludeDirs: Set<string>): string[] {
  const results: string[] = [];
  if (!existsSync(dir)) return results;

  for (const entry of readdirSync(dir)) {
    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) {
      if (!excludeDirs.has(entry)) {
        results.push(...findSourceFiles(fullPath, excludeDirs));
      }
    } else if (entry.endsWith('.ts') && !entry.endsWith('.d.ts')) {
      results.push(fullPath);
    }
  }
  return results;
}

// ─── Single Spec Validation ────────────────────────────────────────────

export function validateSpec(
  specPath: string,
  root: string,
  schemaTables: Set<string>,
  config: ResolvedConfig,
): ValidationResult {
  const result: ValidationResult = {
    specPath: relative(root, specPath),
    errors: [],
    warnings: [],
  };

  const content = readFileSync(specPath, 'utf-8').replace(/\r\n/g, '\n');
  const parsed = parseFrontmatter(content);

  if (!parsed) {
    result.errors.push('Missing or malformed YAML frontmatter (expected --- delimiters)');
    return result;
  }

  const { frontmatter: fm, body } = parsed;

  // ─── Level 1: Structural ──────────────────────────────────────────

  if (!fm.module) result.errors.push('Frontmatter missing required field: module');
  if (!fm.version) result.errors.push('Frontmatter missing required field: version');
  if (!fm.status) result.errors.push('Frontmatter missing required field: status');
  if (!fm.files || !Array.isArray(fm.files) || fm.files.length === 0) {
    result.errors.push('Frontmatter missing required field: files (must be a non-empty list)');
  }

  // Check files exist
  if (fm.files && Array.isArray(fm.files)) {
    for (const file of fm.files) {
      const fullPath = join(root, file);
      if (!existsSync(fullPath)) {
        result.errors.push(`Source file not found: ${file}`);
      }
    }
  }

  // Check db_tables exist in schema
  if (fm.db_tables && Array.isArray(fm.db_tables)) {
    for (const table of fm.db_tables) {
      if (schemaTables.size > 0 && !schemaTables.has(table)) {
        result.errors.push(`DB table not found in schema: ${table}`);
      }
    }
  }

  // Required markdown sections
  const missingSections = getMissingSections(body, config.requiredSections);
  for (const section of missingSections) {
    result.errors.push(`Missing required section: ## ${section}`);
  }

  // ─── Level 2: API Surface ────────────────────────────────────────

  if (fm.files && Array.isArray(fm.files)) {
    const rawExports: string[] = [];
    for (const file of fm.files) {
      const fullPath = join(root, file);
      rawExports.push(...getExportedSymbols(fullPath));
    }
    const allExports = [...new Set(rawExports)];
    const specSymbols = getSpecSymbols(body);
    const specSet = new Set(specSymbols);
    const exportSet = new Set(allExports);

    // Spec documents something that doesn't exist = ERROR
    for (const sym of specSymbols) {
      if (!exportSet.has(sym)) {
        result.errors.push(`Spec documents '${sym}' but no matching export found in source`);
      }
    }

    // Code exports something not in spec = WARNING
    for (const sym of allExports) {
      if (!specSet.has(sym)) {
        result.warnings.push(`Export '${sym}' not in spec (undocumented)`);
      }
    }

    const documented = specSymbols.filter((s) => exportSet.has(s)).length;
    if (allExports.length > 0) {
      const summary = `${documented}/${allExports.length} exports documented`;
      if (documented < allExports.length) {
        result.warnings.unshift(summary);
      } else {
        result.exportSummary = summary;
      }
    }
  }

  // ─── Level 3: Dependencies ───────────────────────────────────────

  if (fm.depends_on && Array.isArray(fm.depends_on)) {
    for (const dep of fm.depends_on) {
      const fullPath = join(root, dep);
      if (!existsSync(fullPath)) {
        result.errors.push(`Dependency spec not found: ${dep}`);
      }
    }
  }

  // Check Consumed By section references
  const consumedByMatch = body.match(/### Consumed By\s*\n([\s\S]*?)(?=\n## |\n### |$)/);
  if (consumedByMatch) {
    const section = consumedByMatch[1];
    const fileRefRegex = /\|\s*`([^`]+\.ts)`\s*\|/g;
    let match: RegExpExecArray | null;
    while ((match = fileRefRegex.exec(section)) !== null) {
      const filePath = join(root, match[1]);
      if (!existsSync(filePath)) {
        result.warnings.push(`Consumed By references missing file: ${match[1]}`);
      }
    }
  }

  return result;
}

// ─── Coverage ──────────────────────────────────────────────────────────

function isExcludedFile(filePath: string, config: ResolvedConfig): boolean {
  for (const pattern of config.excludePatterns) {
    // Simple pattern matching for common cases
    if (pattern.includes('__tests__') && filePath.includes('__tests__')) return true;
    if (pattern.endsWith('.test.ts') && filePath.endsWith('.test.ts')) return true;
    if (pattern.endsWith('.spec.ts') && filePath.endsWith('.spec.ts')) return true;
  }
  return false;
}

function collectSpeccedFiles(specFiles: string[]): Set<string> {
  const speccedFiles = new Set<string>();
  for (const specFile of specFiles) {
    const content = readFileSync(specFile, 'utf-8').replace(/\r\n/g, '\n');
    const parsed = parseFrontmatter(content);
    if (!parsed) continue;
    const { frontmatter: fm } = parsed;
    if (fm.files && Array.isArray(fm.files)) {
      for (const f of fm.files) {
        speccedFiles.add(f);
      }
    }
  }
  return speccedFiles;
}

function getModuleDirs(dir: string, excludeDirs: Set<string>): string[] {
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((entry) => {
      const fullPath = join(dir, entry);
      return statSync(fullPath).isDirectory() && !excludeDirs.has(entry);
    })
    .sort();
}

export function computeCoverage(
  root: string,
  specFiles: string[],
  config: ResolvedConfig,
): CoverageReport {
  const speccedFiles = collectSpeccedFiles(specFiles);
  const excludeDirSet = new Set(config.excludeDirs);

  // Collect all source files across all configured source dirs
  const allSourceFiles: string[] = [];
  for (const srcDir of config.sourceDirs) {
    const fullDir = join(root, srcDir);
    const files = findSourceFiles(fullDir, excludeDirSet)
      .map((f) => relative(root, f).replace(/\\/g, '/'))
      .filter((f) => !isExcludedFile(f, config));
    allSourceFiles.push(...files);
  }

  const unspeccedFiles = allSourceFiles.filter((f) => !speccedFiles.has(f)).sort();

  // Module coverage: check which source dirs have matching spec dirs
  const specsDir = resolve(root, config.specsDir);
  const specModules = new Set(getModuleDirs(specsDir, new Set()));
  const unspeccedModules: string[] = [];

  for (const srcDir of config.sourceDirs) {
    const fullDir = join(root, srcDir);
    const modules = getModuleDirs(fullDir, excludeDirSet);
    for (const mod of modules) {
      if (!specModules.has(mod)) {
        unspeccedModules.push(mod);
      }
    }
  }

  const speccedFileCount = allSourceFiles.length - unspeccedFiles.length;
  const coveragePercent =
    allSourceFiles.length > 0 ? Math.round((speccedFileCount / allSourceFiles.length) * 100) : 100;

  return {
    totalSourceFiles: allSourceFiles.length,
    speccedFileCount,
    unspeccedFiles,
    unspeccedModules,
    coveragePercent,
  };
}
