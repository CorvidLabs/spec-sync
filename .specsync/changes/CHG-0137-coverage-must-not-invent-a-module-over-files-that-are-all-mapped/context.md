---
change: CHG-0137-coverage-must-not-invent-a-module-over-files-that-are-all-mapped
artifact: context
---

# Context

`coverage` reported modules that do not exist, in the same report as 100% file and LOC coverage.

Reproduced on **this repository**, against `origin/main`:

    modules: ['specsync', 'change_tests']    files_covered: 106 / 106

`specsync` comes from `Cargo.toml`'s package name; `change_tests` from `src/change_tests.rs`,
a file created by CHG-0133 (#589) earlier in this same release. The refactor manufactured a
phantom module and nothing noticed.

In a throwaway fixture with five language-specific specs each mapping exactly one source file,
`coverage` printed `Modules without specs (1): strutil/` — a trailing-slash directory that does
not exist — beside `5 specs checked: 5 passed` and `File coverage: 5/5 (100%)`, at exit 0.

## The mechanism

Four blocks in `compute_coverage_checked` assembled `unspecced_modules`, and all four asked the
same question: **is there a spec *directory* bearing this name?** None consulted `specced_files`,
which is computed about thirty lines above them.

So a name derived from a path became a claim about coverage, on evidence that was never about
coverage at all. The mirror image of this release's defect class: not a value dropped for want of
input, but a value invented where there was none.

## Sibling sites

The bug report named the flat-file-stem site. There were four, and the one that produced the
phantom on this repository was a different one:

| derivation | source of the name |
|---|---|
| configured modules | `config.modules.keys()`, from `specsync.json` |
| **manifest modules** | **Cargo `[package]`, Swift target, Gradle project — this is the `specsync` phantom** |
| source subdirectories | directory listing |
| flat-file stems | `file_stem()` — the reported site |

Fixing only the reported site would have left the phantom on our own repository intact. That
pattern — a fix landing where the report points while a parallel implementation survives — has
now happened eight times in this release.

## Ruled out

Suppressing a candidate whenever it owns no discovered file. That is the opposite error: owning
nothing measurable is an absence of input, and reading it as coverage is the very failure this
release exists to close. Such a module stays reported.
