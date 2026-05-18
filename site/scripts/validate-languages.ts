/**
 * validate-languages.ts
 *
 * Schema validator for site/src/data/languages.json and
 * the per-language files in site/src/data/languages/{slug}.json.
 *
 * Run as a prebuild step:
 *   bun scripts/validate-languages.ts
 *
 * Also exports `validateRegistry` for unit tests.
 */

import { existsSync, readdirSync } from "node:fs";
import { resolve, join, basename, dirname } from "node:path";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type Family = "native" | "managed" | "dynamic" | "markup";
export type DetectionStyle = "ast" | "regex" | "hybrid";

export interface RegistryEntry {
  slug: string;
  name: string;
  family: Family;
  detection_style: DetectionStyle;
  extensions: string[];
  test_patterns: string[];
  exports_detected: string[];
  description: string;
  since_version: string;
}

export type ValidationResult =
  | { ok: true }
  | { ok: false; errors: string[] };

export interface ValidateOptions {
  /** When true, check that per-language files exist on disk and match the registry. */
  checkDisk?: boolean;
  /** Base directory for locating data files. Defaults to the directory of this script's data folder. */
  dataDir?: string;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const VALID_FAMILIES = new Set<string>(["native", "managed", "dynamic", "markup"]);
const VALID_DETECTION_STYLES = new Set<string>(["ast", "regex", "hybrid"]);
const REQUIRED_KEYS: (keyof RegistryEntry)[] = [
  "slug",
  "name",
  "family",
  "detection_style",
  "extensions",
  "test_patterns",
  "exports_detected",
  "description",
  "since_version",
];

// ---------------------------------------------------------------------------
// Core validator
// ---------------------------------------------------------------------------

/**
 * Validate a registry JSON value.
 *
 * @param json     - The parsed (or unknown) registry value.
 * @param options  - Optional behaviour overrides.
 */
export function validateRegistry(
  json: unknown,
  options: ValidateOptions = {},
): ValidationResult {
  const errors: string[] = [];

  // 1. Top-level must be an array
  if (!Array.isArray(json)) {
    return {
      ok: false,
      errors: ["Registry must be a JSON array of language entries."],
    };
  }

  if (json === null) {
    return { ok: false, errors: ["Registry must not be null."] };
  }

  const entries = json as unknown[];

  // 2. Per-entry validation
  const registrySlugs = new Set<string>();

  for (let i = 0; i < entries.length; i++) {
    const entry = entries[i];
    const label = `Entry[${i}]`;

    if (typeof entry !== "object" || entry === null || Array.isArray(entry)) {
      errors.push(`${label}: must be an object.`);
      continue;
    }

    const e = entry as Record<string, unknown>;

    // 2a. Required keys present and non-empty
    for (const key of REQUIRED_KEYS) {
      if (!(key in e)) {
        errors.push(`${label}: missing required field "${key}".`);
      } else if (e[key] === null || e[key] === undefined || e[key] === "") {
        errors.push(`${label}: field "${key}" must not be null/empty.`);
      }
    }

    // After checking presence, validate types/values
    const slug = typeof e.slug === "string" ? e.slug : null;
    if (slug) {
      registrySlugs.add(slug);
    }

    // 2b. Enum: family
    if ("family" in e && !VALID_FAMILIES.has(e.family as string)) {
      errors.push(
        `${label} (slug: ${e.slug}): invalid family "${e.family}". Must be one of: ${[...VALID_FAMILIES].join(", ")}.`,
      );
    }

    // 2c. Enum: detection_style
    if ("detection_style" in e && !VALID_DETECTION_STYLES.has(e.detection_style as string)) {
      errors.push(
        `${label} (slug: ${e.slug}): invalid detection_style "${e.detection_style}". Must be one of: ${[...VALID_DETECTION_STYLES].join(", ")}.`,
      );
    }

    // 2d. extensions must be a non-empty array
    if ("extensions" in e) {
      if (!Array.isArray(e.extensions)) {
        errors.push(`${label} (slug: ${e.slug}): "extensions" must be an array.`);
      } else if ((e.extensions as unknown[]).length === 0) {
        errors.push(`${label} (slug: ${e.slug}): "extensions" array must not be empty.`);
      }
    }

    // 2e. exports_detected must be an array
    if ("exports_detected" in e && !Array.isArray(e.exports_detected)) {
      errors.push(`${label} (slug: ${e.slug}): "exports_detected" must be an array.`);
    }

    // 2f. test_patterns must be an array
    if ("test_patterns" in e && !Array.isArray(e.test_patterns)) {
      errors.push(`${label} (slug: ${e.slug}): "test_patterns" must be an array.`);
    }
  }

  // 3. Disk cross-checks (optional, enabled by default when running as a script)
  const shouldCheckDisk = options.checkDisk !== undefined ? options.checkDisk : false;

  if (shouldCheckDisk) {
    // Resolve the per-language data directory
    const scriptDir = dirname(new URL(import.meta.url).pathname);
    const dataDir =
      options.dataDir ?? resolve(scriptDir, "../src/data/languages");

    // 3a. Every registry slug must have a corresponding {slug}.json file
    for (const slug of registrySlugs) {
      const filePath = join(dataDir, `${slug}.json`);
      if (!existsSync(filePath)) {
        errors.push(
          `Registry entry "${slug}" has no matching per-language file at ${filePath}.`,
        );
      }
    }

    // 3b. Every {slug}.json file must have a corresponding registry entry
    if (existsSync(dataDir)) {
      const diskFiles = readdirSync(dataDir).filter((f) => f.endsWith(".json"));
      for (const file of diskFiles) {
        const slugOnDisk = basename(file, ".json");
        if (!registrySlugs.has(slugOnDisk)) {
          errors.push(
            `Dangling per-language file "${file}" has no corresponding registry entry.`,
          );
        }
      }
    }
  }

  if (errors.length > 0) {
    return { ok: false, errors };
  }

  return { ok: true };
}

// ---------------------------------------------------------------------------
// CLI entry point
// ---------------------------------------------------------------------------

async function main() {
  // Resolve paths relative to the site/ directory
  const scriptDir = dirname(new URL(import.meta.url).pathname);
  const siteDir = resolve(scriptDir, "..");
  const registryPath = resolve(siteDir, "src/data/languages.json");
  const langDir = resolve(siteDir, "src/data/languages");

  if (!existsSync(registryPath)) {
    console.error(`validate-languages: registry file not found at ${registryPath}`);
    process.exit(1);
  }

  let registry: unknown;
  try {
    registry = await Bun.file(registryPath).json();
  } catch (err) {
    console.error(`validate-languages: failed to parse ${registryPath}: ${err}`);
    process.exit(1);
  }

  const result = validateRegistry(registry, {
    checkDisk: true,
    dataDir: langDir,
  });

  if (!result.ok) {
    console.error("validate-languages: registry validation FAILED");
    for (const error of result.errors) {
      console.error(`  ✗ ${error}`);
    }
    process.exit(1);
  }

  console.log(`validate-languages: OK — ${(registry as unknown[]).length} language entries validated`);
}

// Run when executed directly (not when imported by tests)
const isMain = import.meta.path === Bun.main;
if (isMain) {
  main().catch((err) => {
    console.error(err);
    process.exit(1);
  });
}
