---
spec: manifest.spec.md
---

## Tasks

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

- Gradle settings.gradle `include` directives not parsed (only build.gradle itself)
- No support for Bazel BUILD files or Meson build definitions
- Workspace glob expansion not implemented for Cargo

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
