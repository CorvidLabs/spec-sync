---
change: a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its
artifact: docs
---

# Docs

## Canonical specs

| Spec | Version | What changed |
|---|---|---|
| `manifest` | 18 → 19 | Invariant 14 now states that `includeBuild` is decided by its argument, not its token, and why a token-only guard cannot tell an in-repo composite build from an escape. Two Error Cases rows: one literal confined path parses and contributes no module; escaping or non-literal arguments fail closed naming the argument. |
| `validator` | 36 → 37 | Invariant 13 now scopes propagation to the case where the source list came from discovery, and states that a stated `source_dirs` is not overruled — the error becomes a notice, and the notice is not optional because module attribution degraded with it. Error Cases split into the two verdicts. |
| `types` | 12 → 13 | `SpecSyncConfig.source_dirs_set` and `CoverageReport.manifest_notices`. |
| `config` | 21 → 22 | Invariant 3 records that WHICH of stated/inferred happened is now kept, and why it cannot be recovered later. |
| `output` | 8 → 9 | Manifest notices render beside the coverage figures in text, markdown, and JSON. |
| `cmd_check` | 24 → 25 | `check --format json` carries `manifest_notices`. |

Companion requirements corrected rather than appended to: `specs/manifest/requirements.md` stated
"Unsupported invoked inclusion APIs such as `includeFlat` and `includeBuild` fail closed" in two
places. That is now the wrong contract, so both lines were rewritten instead of being left to
contradict the spec. `specs/validator/requirements.md` REQ-validator-008 gains the precedence as an
acceptance criterion beside the unchanged propagation one.

The lesson is folded into `specs/manifest/context.md` and `specs/validator/context.md` so the next
session scoping either module meets it at proposal time.

## User-facing surface

New output, on a run that previously could not complete at all:

```
⚠ Manifest discovery was skipped (Cannot parse Gradle settings manifest settings.gradle.kts: …);
  coverage used the configured source_dirs, so modules declared only by that manifest are not reported
```

- `coverage` / `check` text: one `⚠` line after the coverage figures, beside the missing-files and
  skipped-links notes.
- Markdown: `- **Manifest discovery degraded:** …` in the same block.
- JSON: `manifest_notices` in `coverage_json` and in the `check --format json` payload, next to
  `skipped_links`.

It does **not** gate. Unlike `skipped_links`, a manifest notice cannot inflate a percentage, and
gating on it would leave the reported project able to run and still unable to gate CI.

No CLI flags, config keys, or file formats change. `config.source_dirs_set` is runtime-only and
never serialized, so no config file gains or loses a field.

## Contract that changed for existing behaviour

A project that STATES `source_dirs` and has an unsafe or unreadable Gradle manifest previously got
`inconclusive: true` and exit 1 from `check`, `coverage`, `generate`, `report`, and `score`. It now
gets a completed run with the failure reported as a notice. Nothing the refusal protected changed:
no byte outside the project root is read or disclosed, nothing is generated out of the rejected
discovery, and the outside tree is untouched — all asserted directly rather than inferred from the
exit code. A project that does NOT state `source_dirs` is unaffected: still inconclusive, still
exit 1.

## Not documented as support

An accepted `includeBuild` is parsed and ignored. Modules and source directories are NOT discovered
from inside the included build — its settings belong to a separate build. The specs say
"contributes no module" explicitly so this is not mistaken for composite-build support.
