---
change: CHG-0138-release-profile-test-barriers-must-not-fail-the-suite-they-cannot-synchronise
artifact: testing
---

# Testing

## Both profiles, and the arithmetic that proves the scope

    cargo test --release   rc=0   2276 unit · 368 integration · 6 ignored
    cargo test (debug)     rc=0   2276 unit · 374 integration · 0 ignored
    cargo fmt --check      rc=0
    cargo clippy -- -D warnings   rc=0

`368 + 6 = 374` reconciles exactly. Six ignored on macOS rather than seven, because the two
`#[cfg(windows)]` tests are not compiled here at all: five originally-failing, plus the one
converted from `#[cfg(all(unix, debug_assertions))]` — which previously was not even type-checked
in release.

That arithmetic is the check that the gating hit precisely the intended set and nothing else. A
count that did not reconcile would mean a test was silenced by accident.

`cargo clippy --all-targets` is rc=101 on this branch and on unmodified `main`, with finding sets
proven identical by stashing the change and re-running. Pre-existing debt, filed as #608.

## No assertion was weakened

Every test keeps every assertion it had. Only the profile in which it runs changed.
`cfg_attr(not(debug_assertions), ignore)` was chosen over `cfg` deliberately: under `cfg` the test
would not be compiled in release, so it could rot silently and would vanish from the run output
rather than being listed as ignored. Visible-and-ignored is strictly better than absent.

## The shipped binary is unchanged

    git diff --name-only <base>..HEAD -- src/   ->   0 files

The diff is `CHANGELOG.md`, `tests/integration/commands.rs`, `tests/integration/mcp.rs`.

## The guards fire in release — measured, not argued

A release binary was driven under a live symlink race with no barrier at all, on a source tree
widened to open the window:

    DETECTED attempt=1 exit=1
    SpecSync discovery is inconclusive: Coverage project root … changed during retained traversal

Independent adversarial verification went further: coverage-root retarget refused at every delay
from 60–450 ms; generate post-coverage retarget refused with zero writes into the attacker tree;
MCP startup identity flip refused 160/400 times with the exact guard text; MCP read-root
replacement refused at every delay 50–850 ms with victim bytes intact; and a 120-run randomised
fuzz produced zero suspicious outcomes. A debug binary from the same commit behaved identically —
release is not laxer.

`strings` on the release binary: every guard message present; `SPECSYNC_TEST` and `BARRIER` absent.
Guards linked in, rendezvous erased.

## Release coverage that is lost, stated plainly

Three guards have no release-runnable test after this change:

    verify_coverage_project_root        src/validator.rs:1692
    verify_public_path                  src/commands/generate.rs:491   (no unit coverage in ANY profile)
    revalidate_before_success           src/mcp.rs:5088

`open_server_root_capability` is NOT among them — it keeps release coverage through
`mcp::tests::server_root_capability_rejects_a_root_replaced_before_canonicalization`
(`src/mcp.rs:6311`), a test the original write-up missed.

An earlier draft claimed `retained_coverage_snapshot_rejects_post_discovery_symlink_replacement`
as substitute coverage for `verify_coverage_project_root`. It is not: that test asserts
`"symlink or reparse point"`, the source-directory symlink refusal, a different guard. The claim
is removed rather than softened, and the gap is filed as #614.

## Why no pipeline caught this

    .github/workflows/ci.yml:275   cargo test --verbose   (debug)
    fledge.toml:5                  cargo test             (debug)

Nothing runs `cargo test --release`. All seven tests still run on every CI run and every RC
qualification, so this change removes coverage from **zero** pipelines — but it also means nothing
caught the original breakage and nothing would catch its return.
