---
spec: config.spec.md
---

## Tasks

- [ ] Support config file extends/inheritance (`"extends": "./base-specsync.json"`)
- [ ] Add config validation with actionable error messages for invalid field values
- [ ] Support environment variable interpolation in config paths (e.g., `$HOME/specs`)

## Done

- [x] JSON config loading with field defaults
- [x] TOML config loading (zero-dependency parser)
- [x] Auto-detection of source directories by file extension
- [x] Manifest-aware source directory discovery
- [x] 46 hardcoded build/cache directory exclusions
- [x] Unknown key warnings for forward compatibility
- [x] Module definitions support in config

## Gaps

- TOML parsing handles only flat key-value pairs and simple arrays — nested tables would need extension
- No schema validation beyond type checking (e.g., invalid `status` values in modules aren't caught at config load time)

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
