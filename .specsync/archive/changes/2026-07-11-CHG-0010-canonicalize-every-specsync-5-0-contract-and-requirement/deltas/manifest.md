## ADDED

### REQUIREMENT REQ-manifest-001

Manifest discovery SHALL identify supported project modules and source roots deterministically without claiming unsupported workspace expansion.

Acceptance Criteria
- Seven manifest types supported: Cargo.toml, Package.swift, build.gradle.kts, package.json, pubspec.yaml, go.mod, pyproject.toml
- Parsers are tried in fixed order with results merged (first wins on name conflicts)
- Cargo workspace members are parsed recursively with source paths prefixed by member directory
- package.json workspaces support both array and object forms with glob expansion
- Go module name uses the last path segment of the module path
- Python tries `[project]` before `[tool.poetry]` in pyproject.toml
- Gradle multi-module detection via `include()` in settings.gradle
- All TOML/YAML parsing is zero-dependency (regex and string-based)
- `ManifestDiscovery::default()` returns empty collections (safe fallback)
- Unparseable manifests are silently skipped (no errors, try next format)
