---
change: drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows
artifact: research
---

# Research

## Every place the shipped Windows executable is asserted

The surface is smaller than it looks. Six files actually produce or gate the asset, five
more describe it to a reader. There is no standalone install script in the repository.

| File | What it does |
|---|---|
| `.github/workflows/release.yml` `build` | Builds and packages `specsync-windows-x86_64.exe` |
| `.github/workflows/rc-assets.yml` `build` | Same, for pre-release tags |
| `.github/scripts/validate-release-candidate.py` | `EXPECTED_ARTIFACT_ARCHIVES` — the exact-set gate |
| `.github/scripts/test-validate-release-candidate.py` | One test names the artifact literally |
| `action.yml` | Routes a Windows runner to the `.exe.zip` asset |
| `release.yml` `Create release` | `artifacts/**/*.zip`, under `fail_on_unmatched_files: true` |
| `README.md`, `github-action.md`, `quickstart.md`, `adversarial-proof.md` | Reader-facing claims |

Two coupled failures would have shipped if the matrix entries alone had been removed:

- `require_exact_entries` in the validator fails on a **missing** artifact directory, so the
  release would refuse to promote with "artifact directory has the wrong entries (missing
  specsync-windows-x86_64.exe)".
- `Create release` sets `fail_on_unmatched_files: true` and lists `artifacts/**/*.zip`. Once
  no job produces a zip, that glob matches nothing and the release step fails outright.

Neither is reachable from the two matrix entries named in the request; both are in scope here.

## Verified as content-correctness, not platform support — kept

Confirmed present and untouched: `src/parser.rs` CRLF handling (`parse_frontmatter`,
`strip_frontmatter`); `src/merge.rs` `crlf_count` line-ending preservation; `src/hooks.rs`
dominant-line-ending preservation; `src/validator.rs` CRLF normalization before validation;
`src/commands/mod.rs` `validate_module_name` / `is_reserved_module_name`; `src/change.rs`
`MAX_SLUG_BYTES` (justified by Windows `MAX_PATH` 260) and CRLF-tolerant canonical-JSON
comparison; the three `portable_output_path` / `slash_normalized_relative_path` helpers whose
`#[cfg(not(windows))]` arms are what preserve legal Unix backslash filenames; roughly 250
`#[cfg(windows)]` sites across `mcp.rs`, `manifest.rs`, `validator.rs`, `commands/issues.rs`;
and the CRLF and junction fixtures in `tests/integration/`.

`tests/integration/commands.rs:1915` asserts in a comment that its `#[cfg(windows)]` test
"does run on Windows during RC qualification". That statement stays true only because the
`qualify` lane is being kept — a second reason not to touch it.

## linux-aarch64: recommendation is to KEEP, and the evidence differs in kind

`linux-aarch64` has 0 downloads, one fewer than Windows, so the raw number invites the same
conclusion. It should not get it. The Windows case rests on three legs and aarch64 has only one.

1. **No demonstrated defect.** Windows shipped a binary that failed on every spec in a CRLF
   checkout for weeks. There is no equivalent for aarch64, and there is a structural reason
   why: `aarch64-unknown-linux-gnu` shares 100% of its `cfg` surface with
   `x86_64-unknown-linux-gnu`, which has 446 downloads and is the most heavily exercised
   target in the matrix. Windows was a genuinely different code path — different path
   semantics, different line endings, `#[cfg(windows)]` blocks with no Linux twin. aarch64
   Linux differs only in instruction set.
2. **`action.yml` already routes to it.** The arch detection maps `aarch64|arm64` to
   `specsync-linux-aarch64`, so every arm64 Linux runner using the packaged action — including
   GitHub's arm64 hosted runners — resolves to that asset. Removing it breaks the action on
   those runners immediately, with no fallback. Nothing equivalent is lost by removing Windows,
   because that path is being replaced with an explicit refusal.
3. **The user base is ARM-heavy.** `macos-aarch64` at 462 is the single most-downloaded asset.
   Those developers' Linux containers and CI images are arm64. A 0 on a five-week-old asset
   that nothing advertises is weak evidence of no demand rather than strong evidence of none.

Cost of keeping is one cross-compile step on an existing ubuntu runner
(`gcc-aarch64-linux-gnu` plus `cargo build --target`). No new runner OS, no new packaging
path, no second shell — which is precisely what the Windows entry did cost.

The real defect it shares with Windows is that the cross-compiled aarch64 binary is never
executed: `Verify packaged checksum` hashes the archive and never runs what is inside it. If
that is the concern, the proportionate fix is to make it verifiable — run the smoke check
under QEMU or on an arm64 runner — not to remove it. Recommend keeping it and raising
verification separately. `linux-musl` (2 downloads) is the static fallback for old-glibc
distros and is likewise cheap; it is not a candidate either.

This is a recommendation only. The owner approved Windows and nothing else, and no aarch64 or
musl entry is touched by this change.
