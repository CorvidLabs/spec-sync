# CLI Module Tasks

## Open

- [ ] Add shell completion generation subcommand (`specsync completions bash/zsh/fish`)
- [ ] Add `--quiet` flag to suppress non-error output
- [ ] Add `--color never/always/auto` flag for explicit color control

## Done

- [x] Implement all 12 subcommands (check, coverage, generate, init, score, watch, mcp, add-spec, init-registry, resolve, hooks install/uninstall/status)
- [x] Add `--json` output mode for all reporting commands
- [x] Add `--strict` and `--require-coverage` global flags
- [x] Add `--root` flag for non-cwd project roots
- [x] Make `check` the default subcommand when none is specified
