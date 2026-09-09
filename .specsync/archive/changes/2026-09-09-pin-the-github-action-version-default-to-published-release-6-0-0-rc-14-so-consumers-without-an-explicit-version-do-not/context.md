---
change: pin-the-github-action-version-default-to-published-release-6-0-0-rc-14-so-consumers-without-an-explicit-version-do-not
artifact: context
---

# Context

Issue #628: `action.yml` defaults `version` to `6.0.0`, but no GitHub Release `v6.0.0` exists.
Any consumer that omits the optional `version:` input downloads a 404. Verified live:

- `gh release view v6.0.0` → release not found
- `gh release view v6.0.0-rc.14` → prerelease with the five Linux/macOS archives + sha256 sidecars
- Tags `v6.0.0-rc.15`..`rc.17` exist but have **no** Release assets — must not become the default

`validate-release-version.py` currently requires `action.yml` default and the site Action docs
default to equal `Cargo.toml`'s package version (`6.0.0`). That sync anticipated stable
publication and is exactly what made the broken default pass CI while consumers 404'd.

`REQ-github-002` says the composite Action's current default matches the promoted stable
package version. During the candidate window the package version is already `6.0.0` but has
not been published; the downloadable default must resolve to a Release that has assets.

Out of scope: changing README/`@v6.0.0` immutable pin *examples* (those document the intended
stable pin and stay synced to Cargo.toml), bumping Cargo.toml, publishing `v6.0.0`, or
defaulting to `rc.15+` / `latest` / `5.2.0`.

Succession note: delivery paths (`action.yml`, site docs, the validator script) stay
`@exact:delivery` without `[modules."github"] owns` — granting owns requires editing
`.specsync/config.toml`, which cannot itself be owned and whose reopen replayed a stale
Invariants delta. Spec companions under `specs/github/` are superseded from CHG-0082.
