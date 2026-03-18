#!/usr/bin/env node

/**
 * SpecSync — Bidirectional spec-to-code validation
 *
 * Commands:
 *   check      Validate all specs (default)
 *   coverage   Show file and module coverage report
 *   generate   Scaffold specs for unspecced modules
 *   init       Create a specsync.json config file
 *
 * Flags:
 *   --strict              Treat warnings as errors
 *   --require-coverage N  Fail if file coverage < N%
 *   --root <path>         Project root (default: cwd)
 */

import { resolve, relative } from 'node:path';
import { existsSync, writeFileSync } from 'node:fs';
import { loadConfig } from './config.js';
import { findSpecFiles, validateSpec, getSchemaTableNames, computeCoverage } from './validator.js';
import { generateSpecsForUnspeccedModules } from './generator.js';

const args = process.argv.slice(2);
const command = args.find((a) => !a.startsWith('-')) ?? 'check';

// Parse flags
const strict = args.includes('--strict');
const rootIdx = args.indexOf('--root');
const root = rootIdx !== -1 && args[rootIdx + 1] ? resolve(args[rootIdx + 1]) : process.cwd();

let requiredCoverage: number | null = null;
const rcIdx = args.indexOf('--require-coverage');
if (rcIdx !== -1 && args[rcIdx + 1]) {
  requiredCoverage = parseInt(args[rcIdx + 1], 10);
  if (isNaN(requiredCoverage) || requiredCoverage < 0 || requiredCoverage > 100) {
    console.error('--require-coverage must be a number between 0 and 100');
    process.exit(1);
  }
}

// ─── Init Command ──────────────────────────────────────────────────────

if (command === 'init') {
  const configPath = resolve(root, 'specsync.json');
  if (existsSync(configPath)) {
    console.log('specsync.json already exists');
    process.exit(0);
  }

  const defaultConfig = {
    specsDir: 'specs',
    sourceDirs: ['src'],
    requiredSections: [
      'Purpose',
      'Public API',
      'Invariants',
      'Behavioral Examples',
      'Error Cases',
      'Dependencies',
      'Change Log',
    ],
    excludeDirs: ['__tests__'],
    excludePatterns: ['**/__tests__/**', '**/*.test.ts', '**/*.spec.ts'],
  };

  writeFileSync(configPath, JSON.stringify(defaultConfig, null, 2) + '\n');
  console.log('Created specsync.json');
  process.exit(0);
}

// ─── Help Command ──────────────────────────────────────────────────────

if (command === 'help' || args.includes('--help') || args.includes('-h')) {
  console.log(`
SpecSync — Bidirectional spec-to-code validation

Usage: specsync [command] [flags]

Commands:
  check       Validate all specs against source code (default)
  coverage    Show file and module coverage report
  generate    Scaffold spec files for unspecced modules
  init        Create a specsync.json config file
  help        Show this help message

Flags:
  --strict              Treat warnings as errors
  --require-coverage N  Fail if file coverage percent < N
  --root <path>         Project root directory (default: cwd)
`);
  process.exit(0);
}

// ─── Load Config & Discover ────────────────────────────────────────────

const config = loadConfig(root);
const specsDir = resolve(root, config.specsDir);
const specFiles = findSpecFiles(specsDir);

if (specFiles.length === 0) {
  console.log(`No spec files found in ${config.specsDir}/`);
  process.exit(0);
}

// Skip template file
const realSpecs = specFiles.filter((f) => !f.endsWith('_template.spec.md'));

if (realSpecs.length === 0) {
  console.log(`No spec files found in ${config.specsDir}/ (excluding template)`);
  process.exit(0);
}

const schemaTables = getSchemaTableNames(root, config);

// ─── Check Command ─────────────────────────────────────────────────────

if (command === 'check' || command === 'coverage' || command === 'generate') {
  let totalErrors = 0;
  let totalWarnings = 0;
  let passed = 0;

  for (const specFile of realSpecs) {
    const result = validateSpec(specFile, root, schemaTables, config);

    console.log(`\n${result.specPath}`);

    // Frontmatter check
    const hasFmErrors = result.errors.some(
      (e) => e.startsWith('Frontmatter') || e.startsWith('Missing or malformed'),
    );
    console.log(`  ${hasFmErrors ? '\u2717' : '\u2713'} Frontmatter valid`);

    // File existence
    const fileErrors = result.errors.filter((e) => e.startsWith('Source file'));
    const hasFilesField = !result.errors.some((e) => e.includes('files (must be'));
    if (fileErrors.length === 0 && hasFilesField) {
      console.log(`  \u2713 All source files exist`);
    } else {
      for (const e of fileErrors) console.log(`  \u2717 ${e}`);
    }

    // DB table check
    const tableErrors = result.errors.filter((e) => e.startsWith('DB table'));
    if (tableErrors.length > 0) {
      for (const e of tableErrors) console.log(`  \u2717 ${e}`);
    } else if (schemaTables.size > 0) {
      console.log(`  \u2713 All DB tables exist in schema`);
    }

    // Section check
    const sectionErrors = result.errors.filter((e) => e.startsWith('Missing required section'));
    if (sectionErrors.length > 0) {
      for (const e of sectionErrors) console.log(`  \u2717 ${e}`);
    } else {
      console.log(`  \u2713 All required sections present`);
    }

    // API surface
    const apiExportLine = result.warnings.find((w) => w.match(/^\d+\/\d+ exports documented$/));
    if (apiExportLine) {
      console.log(`  \u2713 ${apiExportLine}`);
    } else if (result.exportSummary) {
      console.log(`  \u2713 ${result.exportSummary}`);
    }

    const specDescribesNonexistent = result.errors.filter((e) => e.startsWith('Spec documents'));
    for (const e of specDescribesNonexistent) console.log(`  \u2717 ${e}`);

    const undocumented = result.warnings.filter((w) => w.startsWith("Export '"));
    for (const w of undocumented) console.log(`  \u26A0 ${w}`);

    // Dependency check
    const depErrors = result.errors.filter((e) => e.startsWith('Dependency spec'));
    if (depErrors.length > 0) {
      for (const e of depErrors) console.log(`  \u2717 ${e}`);
    } else {
      console.log(`  \u2713 All dependency specs exist`);
    }

    // Consumed-by warnings
    const consumedByWarnings = result.warnings.filter((w) => w.startsWith('Consumed By'));
    for (const w of consumedByWarnings) console.log(`  \u26A0 ${w}`);

    totalErrors += result.errors.length;
    totalWarnings += result.warnings.length;
    if (result.errors.length === 0) passed++;
  }

  // ─── Coverage ──────────────────────────────────────────────────────

  const coverage = computeCoverage(root, realSpecs, config);
  let coverageWarnings = 0;

  if (command === 'coverage' || command === 'generate') {
    console.log('\n--- Coverage Report ------------------------------------------------');

    if (coverage.unspeccedModules.length > 0) {
      console.log(`\n  Modules without specs (${coverage.unspeccedModules.length}):`);
      for (const mod of coverage.unspeccedModules) {
        console.log(`    \u26A0 ${mod}/`);
        coverageWarnings++;
      }
    } else {
      console.log('\n  \u2713 All source modules have spec directories');
    }

    if (coverage.unspeccedFiles.length > 0) {
      console.log(`\n  Files not in any spec (${coverage.unspeccedFiles.length}):`);
      for (const file of coverage.unspeccedFiles) {
        console.log(`    \u26A0 ${file}`);
        coverageWarnings++;
      }
    } else {
      console.log('  \u2713 All source files referenced by specs');
    }
  }

  if (command === 'generate') {
    console.log('\n--- Generating Specs -----------------------------------------------');
    const generated = generateSpecsForUnspeccedModules(root, coverage, config);
    if (generated === 0 && coverage.unspeccedModules.length === 0) {
      console.log('  \u2713 No specs to generate — full module coverage');
    } else if (generated > 0) {
      console.log(`\n  Generated ${generated} spec file(s) — edit them to fill in details`);
    }
  }

  // Summary
  const total = realSpecs.length;
  const failed = total - passed;
  const allWarnings = totalWarnings + coverageWarnings;
  console.log(
    `\n${total} specs checked: ${passed} passed, ${allWarnings} warning(s), ${failed} failed`,
  );
  console.log(
    `File coverage: ${coverage.speccedFileCount}/${coverage.totalSourceFiles} (${coverage.coveragePercent}%)`,
  );

  if (totalErrors > 0) {
    process.exit(1);
  }

  if (strict && allWarnings > 0) {
    console.log(`\n--strict mode: ${allWarnings} warning(s) treated as errors`);
    process.exit(1);
  }

  if (requiredCoverage !== null && coverage.coveragePercent < requiredCoverage) {
    console.log(
      `\n--require-coverage ${requiredCoverage}%: actual coverage is ${coverage.coveragePercent}% (${coverage.unspeccedFiles.length} file(s) missing specs)`,
    );
    if (coverage.unspeccedFiles.length > 0) {
      for (const f of coverage.unspeccedFiles) {
        console.log(`  \u2717 ${f}`);
      }
    }
    process.exit(1);
  }
}
