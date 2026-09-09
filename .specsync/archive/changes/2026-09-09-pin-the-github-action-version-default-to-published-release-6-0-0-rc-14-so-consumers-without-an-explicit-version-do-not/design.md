---
change: pin-the-github-action-version-default-to-published-release-6-0-0-rc-14-so-consumers-without-an-explicit-version-do-not
artifact: design
---

# Design

## Decision

Pin the Action `version` **default** (and the site inputs-table default) to `6.0.0-rc.14`.
Keep Cargo.toml at `6.0.0`. Teach `validate-release-version.py` a narrow candidate-window
exception for those two surfaces only.

## Alternatives rejected

| Option | Why not |
|---|---|
| Default `5.2.0` (latest stable) | Parent scope for #628 chose the latest published **6.0** candidate with assets |
| Default `latest` | Moves under consumers; description already discourages it for CI |
| Default `6.0.0-rc.15`..`rc.17` | Tags exist; Releases have **no** assets |
| Change Cargo.toml to `6.0.0-rc.14` | Binaries self-report `6.0.0`; package version is not the download pin |
| Add `[modules."github"] owns` for `action.yml` | Requires editing `.specsync/config.toml`, which cannot be owned and reopen of CHG-0082 rematerialized a stale Invariants delta onto tip |

## Validator shape

Introduce `PUBLISHED_ACTION_DEFAULT = "6.0.0-rc.14"`. When package version is `6.0.0` and no
stable release has been published yet, `action.yml` default and the site docs default must
equal `PUBLISHED_ACTION_DEFAULT`. All other surfaces (README pins, CI consumer with mirror,
Trust pin) keep requiring package version equality.

## Spec delta

Amend `REQ-github-002` so the "current default matches promoted stable package version" rule
applies after stable publication; before that, the default must be a published Release with
assets for the version under promotion.
