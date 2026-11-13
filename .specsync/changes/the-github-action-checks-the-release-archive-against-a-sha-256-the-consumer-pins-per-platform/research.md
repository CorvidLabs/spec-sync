---
change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
artifact: research
---

# Research

- `action.yml` names the archive `specsync-${OS}-${ARCH}.tar.gz`, with OS `linux` or `macos` and ARCH
  `x86_64` or `aarch64`. Those four `<os>-<arch>` names are the platform keys.
- Release archives differ per platform, so one digest fits only a single runner type. That is why
  each platform gets its own line.
- `tests/integration/config.rs` forbids `eval` of inputs in `action.yml`. The input reaches the
  script through `env`, never interpolated into the shell.
