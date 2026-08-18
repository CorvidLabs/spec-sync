---
change: CHG-0147-an-explicit-enforcement-policy-must-survive-migrate
artifact: context
---

# Context

`config_to_toml` omitted the enforcement key whenever the value was `Warn`:

    match config.enforcement {
        EnforcementMode::Warn => {}                  // default, omit
        EnforcementMode::EnforceNew => { ... }
        EnforcementMode::Strict     => { ... }
    }

`Warn` has not been the default since the default moved to `Strict`
(`#[default]` sits on `Strict` in `src/types.rs`). The comment is a fossil.

So `specsync migrate` dropped an explicit `enforcement = "warn"`, and the next
load applied the current default instead. Measured on one tree, with a spec
citing a file that does not exist:

    before migrate   check rc=0     (warn: reported, not gated)
    after migrate    check rc=1     (strict: gating)
    enforcement lines in the migrated config: 0

Identical findings, identical output, opposite exit codes.

## Why it is hard to debug from the symptom

The config did not GAIN a `strict` line. It LOST a `warn` line. Someone whose
pipeline turned red after upgrading looks for something that was added, and
there is nothing to find.

It also voids the mitigation the CHANGELOG offers for the warn → strict default
change: the config-file form of that mitigation is the exact value `migrate`
deletes.

## The general defect

Omitting a value because it equals the default is safe only while the default
never moves. It moved. And the omission is undetectable afterwards, because an
absent key and a key holding the default are byte-identical on disk — nothing
downstream can tell "unset" from "deliberately set to this".

Same shape as the rest of this release: an absence read as agreement.
