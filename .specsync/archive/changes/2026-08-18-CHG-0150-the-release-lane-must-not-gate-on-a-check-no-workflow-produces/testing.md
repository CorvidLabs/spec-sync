# Testing

## What can and cannot be verified here

This change is GitHub Actions configuration. It cannot be executed locally, and the honest
statement of its verification limit is that **the lane itself is still unexercised** — that is
the defect being fixed, and the `dry_run` path added here is what finally makes exercising it
possible. Everything below is static analysis and isolated simulation of the changed logic.

## Producer/consumer check — the discriminating test

Every check-run name `release.yml` selects on must have a producer somewhere under `.github/`.
Run against the file at `origin/main` and the file in this change, in the same tree:

    ════ BEFORE (origin/main 8fad38d4) ════
      waits on: specsync, SpecSync archive binding, SpecSync release candidate
      PRODUCER FOUND for 'specsync': trust.yml ci.yml …
      NO PRODUCER for 'SpecSync archive binding' — the wait can never be satisfied
      PRODUCER FOUND for 'SpecSync release candidate': trust.yml
      verdict: FAIL   rc=1

    ════ AFTER (this change) ════
      waits on: specsync, SpecSync release candidate
      PRODUCER FOUND for 'specsync': trust.yml ci.yml …
      PRODUCER FOUND for 'SpecSync release candidate': trust.yml
      verdict: PASS   rc=0

The two surviving names are the control. The check is not passing because it stopped looking:
`release.yml` still waits on two check runs, both of which resolve to a real producer in
`trust.yml`. Only the third had none.

## Dry-run mode — isolated simulation

The `resolve` mode branch, extracted verbatim and driven directly:

    PASS  RC tag push                          -> qualify
    PASS  dispatch, dry_run unset              -> promote
    PASS  dispatch, dry_run=false (control)    -> promote
    PASS  dispatch, dry_run=true               -> dry-run
    PASS  dispatch, dry_run=TRUE fails closed  -> REJECTED
    PASS  dispatch, dry_run=yes fails closed   -> REJECTED
    PASS  branch push still unsupported        -> UNSUPPORTED
    PASS  final tag push still unsupported     -> UNSUPPORTED

The two `REJECTED` rows exist because the first draft did not have them: it used
`[[ "$X" == "true" ]]` with an `else` to `promote`, so `dry_run=TRUE` — reachable via
`gh workflow run -f dry_run=TRUE` — would have silently run a **real promotion** for someone
who asked for a dry run. The simulation caught it and the branch now fails closed.

## Job graph, parsed from the file

    qualify   resolve validate qualify record-qualification
    promote   resolve validate authorize-release promote build release
    dry-run   resolve validate

`qualify` and `promote` are unchanged. `dry-run` reaches nothing that creates a tag or
publishes an artifact: every other job is guarded on `qualify` or `promote`, and `release`
depends on jobs that skip.

## Static checks

- `yaml.safe_load` parses the file; 8 jobs, unchanged names and dependency edges
- `actionlint .github/workflows/release.yml` — clean
- no `needs.validate.outputs` reference anywhere (the deleted step had no `id` and the job
  exposes no outputs, so nothing downstream can dangle)
- no remaining reference to `archive binding`, `archive-binding` or `specsync-archive`

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| n/a — `--no-spec-change` | `release.yml` is CI configuration under `.github/` with no owning spec module (precedent CHG-0014). Evidence is the producer/consumer discrimination above, which fails on the current `main` file and passes on this one while keeping two real producer/consumer pairs as controls, plus the eight-case mode simulation including two fail-closed rows |

## Known gap, deliberately not closed here

The producer/consumer check is run by hand, not in CI. Making it permanent means adding
`.github/scripts/`, which is outside this change's declared `affected_paths`, and delivery
scope cannot be widened after the interview (#542). Filed separately rather than smuggled in —
a small live illustration of that issue's cost.
