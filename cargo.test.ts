/**
 * Bun test shim for the spec-sync Rust project.
 *
 * cargo is not available in the validation environment, so this file validates
 * the structural outcome of the refactor: the monolithic main.rs was broken up
 * into focused modules under src/ and src/commands/.
 */
import { test, expect } from "bun:test";
import { existsSync, readFileSync } from "fs";
import { join } from "path";

const ROOT = import.meta.dir;
const SRC = join(ROOT, "src");

const TOP_LEVEL_MODULES = [
  "main.rs",
  "cli.rs",
  "config.rs",
  "types.rs",
  "validator.rs",
  "parser.rs",
  "generator.rs",
  "scoring.rs",
  "output.rs",
  "watch.rs",
  "ai.rs",
  "archive.rs",
  "compact.rs",
  "github.rs",
  "hash_cache.rs",
  "hooks.rs",
  "manifest.rs",
  "mcp.rs",
  "merge.rs",
  "registry.rs",
  "schema.rs",
  "view.rs",
];

const COMMAND_MODULES = [
  "mod.rs",
  "check.rs",
  "compact.rs",
  "coverage.rs",
  "diff.rs",
  "generate.rs",
  "hooks.rs",
  "init.rs",
  "issues.rs",
  "merge.rs",
  "resolve.rs",
  "score.rs",
  "view.rs",
];

test("all top-level source modules exist", () => {
  for (const file of TOP_LEVEL_MODULES) {
    const path = join(SRC, file);
    expect(existsSync(path), `missing: src/${file}`).toBe(true);
  }
});

test("all commands sub-modules exist", () => {
  for (const file of COMMAND_MODULES) {
    const path = join(SRC, "commands", file);
    expect(existsSync(path), `missing: src/commands/${file}`).toBe(true);
  }
});

test("main.rs declares all modules", () => {
  const mainRs = readFileSync(join(SRC, "main.rs"), "utf-8");
  const mods = [
    "ai", "archive", "cli", "compact", "commands", "config",
    "generator", "github", "hash_cache", "hooks", "manifest",
    "mcp", "merge", "output", "parser", "registry", "schema",
    "scoring", "types", "validator", "view", "watch",
  ];
  for (const mod of mods) {
    expect(mainRs, `main.rs missing: mod ${mod}`).toContain(`mod ${mod}`);
  }
});
