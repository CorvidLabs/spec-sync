import { existsSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import type { SpecSyncConfig, ResolvedConfig } from './types.js';

const DEFAULT_REQUIRED_SECTIONS = [
  'Purpose',
  'Public API',
  'Invariants',
  'Behavioral Examples',
  'Error Cases',
  'Dependencies',
  'Change Log',
];

const DEFAULTS: ResolvedConfig = {
  specsDir: 'specs',
  sourceDirs: ['src'],
  schemaDir: undefined,
  schemaPattern: 'CREATE (?:VIRTUAL )?TABLE(?:\\s+IF NOT EXISTS)?\\s+(\\w+)',
  requiredSections: DEFAULT_REQUIRED_SECTIONS,
  excludeDirs: ['__tests__'],
  excludePatterns: ['**/__tests__/**', '**/*.test.ts', '**/*.spec.ts'],
};

/** Load specsync.json from the project root, merging with defaults */
export function loadConfig(root: string): ResolvedConfig {
  const configPath = join(root, 'specsync.json');

  if (!existsSync(configPath)) {
    return { ...DEFAULTS };
  }

  const raw = readFileSync(configPath, 'utf-8');
  const userConfig: Partial<SpecSyncConfig> = JSON.parse(raw);

  return {
    specsDir: userConfig.specsDir ?? DEFAULTS.specsDir,
    sourceDirs: userConfig.sourceDirs ?? DEFAULTS.sourceDirs,
    schemaDir: userConfig.schemaDir ?? DEFAULTS.schemaDir,
    schemaPattern: userConfig.schemaPattern ?? DEFAULTS.schemaPattern,
    requiredSections: userConfig.requiredSections ?? DEFAULTS.requiredSections,
    excludeDirs: userConfig.excludeDirs ?? DEFAULTS.excludeDirs,
    excludePatterns: userConfig.excludePatterns ?? DEFAULTS.excludePatterns,
  };
}
