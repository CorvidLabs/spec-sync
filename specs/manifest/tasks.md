---
spec: manifest.spec.md
---

## Tasks

## Post-5.0 Roadmap

- [ ] Add CMakeLists.txt support for C/C++ projects
- [ ] Add .csproj/.sln support for C# projects
- [ ] Handle Cargo workspace `members` with glob patterns (e.g., `"crates/*"`)
- [ ] Extract dependency information from manifests for cross-module `depends_on` pre-population

## Done

- [x] Cargo.toml parser (packages, binaries, workspace members)
- [x] Package.swift parser (targets with balanced paren extraction)
- [x] build.gradle/build.gradle.kts parser (modules, Android detection)
- [x] package.json parser (workspaces, monorepo support)
- [x] pubspec.yaml parser (single-entry lib/)
- [x] go.mod parser (module name + standard dirs)
- [x] pyproject.toml parser (project and poetry support)

## Gaps

- No support for Bazel BUILD files or Meson build definitions
- Workspace glob expansion not implemented for Cargo `members` (literal paths only; `package.json` workspace globs like `packages/*` are supported, Cargo's are not)
- Dependency extraction is only wired up for Cargo (`[dependencies]`) and Swift target `dependencies:`; other manifests leave `dependencies` empty

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
