/** YAML frontmatter parsed from a spec file */
export interface Frontmatter {
  module?: string;
  version?: string;
  status?: string;
  files?: string[];
  db_tables?: string[];
  depends_on?: string[];
}

/** Result of validating a single spec */
export interface ValidationResult {
  specPath: string;
  errors: string[];
  warnings: string[];
  exportSummary?: string;
}

/** Coverage report for the project */
export interface CoverageReport {
  totalSourceFiles: number;
  speccedFileCount: number;
  unspeccedFiles: string[];
  unspeccedModules: string[];
  coveragePercent: number;
}

/** User-provided configuration */
export interface SpecSyncConfig {
  /** Directory containing spec files (default: "specs") */
  specsDir: string;

  /** Source directories to check coverage against (default: ["src"]) */
  sourceDirs: string[];

  /** Directory containing SQL schema files for db_tables validation (optional) */
  schemaDir?: string;

  /** Regex pattern for extracting table names from schema files */
  schemaPattern?: string;

  /** Required markdown sections in spec body (default: standard set) */
  requiredSections: string[];

  /** Directories excluded from coverage (default: ["__tests__"]) */
  excludeDirs: string[];

  /** File patterns excluded from coverage */
  excludePatterns: string[];
}

/** Resolved config with all defaults applied */
export type ResolvedConfig = Required<Pick<SpecSyncConfig, 'specsDir' | 'sourceDirs' | 'requiredSections' | 'excludeDirs' | 'excludePatterns'>> & {
  schemaDir?: string;
  schemaPattern: string;
};
