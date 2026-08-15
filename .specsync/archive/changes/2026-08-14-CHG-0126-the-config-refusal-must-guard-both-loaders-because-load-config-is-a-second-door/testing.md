---
change: CHG-0126-the-config-refusal-must-guard-both-loaders-because-load-config-is-a-second-door
artifact: testing
---

# Testing

The fixture is a config file that EXISTS and cannot be read — valid UTF-8
followed by raw `\xff\xfe\x00`. That shape matters: a TOML parse error does not
reproduce this, because `config.rs`'s scanner never reports one.

    BROKEN config                    VALID config control
      rules   exit 1                   rules   exit 0
      compact exit 1                   compact exit 0
      rehash  exit 1                   rehash  exit 0
      hashes.json NOT written

Both directions in one table. The right-hand column is what separates this from
a guard that refuses everything, and the `hashes.json` line is the one that
matters — before, `rehash` exited 0 and wrote a cache that later `check` runs
consult to skip specs.

Suite: fmt clean, clippy clean, 2242 unit + 367 integration, 0 failures.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-config-010 | All three previously-passing commands refuse, and the valid-config control is unchanged. The permissive loader is named for what it does, so a future caller's bypass is a word they typed rather than a guard they forgot |
| REQ-cmd-wizard-002 | `wizard` still runs over the broken-config fixture — the case where it is most needed |
| REQ-cmd-init-registry-003 | The registry initialiser likewise, and its now-unused `load_config` import is removed rather than left to rot |
