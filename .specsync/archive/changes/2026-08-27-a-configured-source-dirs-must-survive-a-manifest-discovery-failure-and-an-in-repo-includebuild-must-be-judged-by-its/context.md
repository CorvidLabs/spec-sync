---
change: a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its
artifact: context
---

# Context

## What led here

A 6.0 release blocker reported from the field (#723). A Gradle project with a valid in-repo
composite build — `includeBuild("vendor/podo-shared")` in `settings.gradle.kts` — cannot run
`check --strict` or `coverage` on **any** candidate from rc.1 through rc.7. `view` and `change new`
work. They have `source_dirs = ["app/src/main/java"]` explicitly configured and it does not rescue
them. v5.2.0 works and reports 24/29 files covered (82.76%).

That split is the worst possible one: the tool is usable for authoring and unusable for the thing CI
depends on, so a project can adopt it, get value, and only discover the gap when it tries to enforce
anything. And there is no v6 to pin — every candidate is affected, so it is not "upgrade later", it
is "6.0 cannot ship to this adopter".

Reproduced exactly before any change was made, against `target/release/specsync` built from `main`:

```
$ specsync coverage --root repro
Coverage inconclusive: Cannot parse Gradle settings manifest settings.gradle.kts:
  Unsupported Gradle workspace mutator includeBuild
COVERAGE EXIT: 1
CHECK EXIT: 1
```

## Two defects, and why the second one was fixed first

**A.** `src/manifest.rs` rejected `includeBuild` on the token prefix alone. The path was never read,
so an in-repo composite build and an escape out of the repository produced the identical hard `Err`.

**B.** `src/validator.rs` propagated a manifest-discovery failure with `?` even when the project had
explicitly configured `source_dirs`. Every other call site already degrades — `config.rs:66` uses
`unwrap_or_else`, `validator.rs:430` falls back to a scan — coverage did not.

B is the one that removes a class. Discovery exists to INFER what the user did not state; when the
user has stated it, a failure to infer must not veto it. Fixing only A would leave the next
unreadable manifest, in any ecosystem, able to override an explicit declaration all over again.

This is the same shape the release has fixed repeatedly — an input the tool could not interpret,
treated as a verdict about the project: #672 (an unparseable schema reported every table as
missing), #684 (a missing `schema_dir` gated a release), and the `bypass_actors` field a runner
cannot read. Here it is one layer out.

## Already ruled out

- **An escape hatch to disable manifest discovery.** There is none today. An opt-out is a worse
  answer than not needing one, and it would leave the class intact.
- **Skipping discovery entirely when `source_dirs` is configured.** That would silently change
  module attribution for every project that configures `source_dirs` AND has a valid manifest.
  Discovery still runs; only its FAILURE is reinterpreted.
- **Making the notice gate `--strict`.** It cannot inflate a percentage (unlike `skipped_links`,
  which shrinks the denominator), and gating on it would put the reported project back exactly where
  it started — able to run, unable to gate.
- **Reporting the degradation on stderr only.** #570 is the standing lesson: a CI job capturing
  stdout reads a clean pass while the warning goes where nobody looks.
- **Parsing the included build's own settings.** A composite build is a separate build; discovering
  its modules is a feature, not this fix. Accept-and-ignore is strictly better than aborting.

## Constraints worth knowing

- `is_gradle_include_start` already does not match `includeBuild`, so accepting one requires no
  change to the module loop — an accepted composite build is naturally skipped.
- Clippy is NOT in this project's `change check` verification commands (`fledge.toml` puts `lint` in
  the `verify`/`ci` lanes), so `change check` passes green while CI blocks the PR. Run
  `cargo clippy -- -D warnings` — bare, not `--all-targets` — by hand.

## Two things the first cut got wrong

Recorded while fresh, because both are the kind of thing the next change here will hit.

**The same conditional composite build got two different verdicts.** Judging `includeBuild` by its
argument left its POSITION unjudged, and the remainder check could not tell a trailing configuration
block (`{ dependencySubstitution … }`, which this parser does not model) from the closing brace of an
enclosing `if`. So `if (x) { includeBuild("vendor/s") }` was refused on one line and accepted across
three — only in the first case did the `}` land on the declaration's own line. A verdict that turns
on where the author pressed Enter is a bug whichever way it is settled. Settled by accepting both: a
composite build contributes no module whether or not its branch runs, so its position cannot change
what is discovered. That is deliberately ASYMMETRIC with `include`, which is still refused when
conditional — a conditional `include` does change the module set.

**Six integration tests encoded the old contract, and `change check` is what found them.** Every
`gradle_*_is_inconclusive_for_coverage_gating_commands` fixture calls `setup_minimal_project`, which
states `sourceDirs: ["src"]` — so they were all asserting the CONFIGURED case, which is exactly the
case this change stops treating as fatal. They were not wrong about safety; they were using the exit
code as a proxy for it. Each now clears `sourceDirs` so it still asserts the fail-closed contract on
an inferred source list, and a new test asserts the degraded half over the same unsafe shapes: no
outside byte read or disclosed, nothing generated from the rejected discovery, the outside tree
untouched. Asserting those directly rather than through exit status is what makes the pair stronger
than the original six.

`report` and `score` exit 1 in those fixtures for a reason that has nothing to do with the manifest —
they are not git repositories, so staleness is unmeasurable. The degradation test therefore judges
those two on the report they produced rather than on their status; only `check`, `coverage`, and
`generate` are asserted to exit 0.
