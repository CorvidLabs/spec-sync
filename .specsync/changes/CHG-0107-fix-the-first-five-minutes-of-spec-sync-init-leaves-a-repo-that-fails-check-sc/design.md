---
change: CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc
artifact: design
---

# Design

Three independent mechanisms, one shared property: none of them weakens a guard for
the general case. Each narrows an over-broad gate to the exact situation that made it
unsatisfiable, and each is revocable by the user's own subsequent edits.

## 1. Bootstrap record — `.specsync/bootstrap.json`

`specsync init` calls the new `change::record_bootstrap_paths(root)`, which writes a
record naming the protected SDD files this bootstrap created, the commit it was
created against, and a digest of each.

`bootstrap_exempt_paths` honors a recorded path only when **all four** hold:

1. it is a protected SDD path — a record can never exempt product source;
2. it is absent at the delivery comparison base, so a *modification* of an
   already-tracked policy file is never exempt, only its creation;
3. the recorded base commit is a real ancestor of `HEAD`; and
4. the file still hashes to the digest recorded when spec-sync wrote it.

Editing a bootstrapped file revokes its own exemption, and the normal change workflow
applies from that point on. Forging the record requires writing a digest that matches
a file that is simultaneously absent at the base and descended from a real commit.

**The digest pins the enforcement surface, not the bytes.** `bootstrap_digest` clears
`verification_commands` before hashing. `init` writes that list empty whenever it
cannot detect a test command and then instructs the author to populate it; pinning the
field would revoke the bootstrap for following the tool's own instructions. Every field
that decides whether the gate bites stays pinned. A policy that does not parse falls
back to a byte digest, so a malformed file cannot slip through the projection.

**Backward compatibility:** `bootstrap_records` also reads the original single-path
`bootstrap_policy` shape written by `change adopt`, so records from earlier versions
keep covering the policy they created.

**Companion fix:** `comparison_base_commit` reduces both forms `pull_request_diff_base`
can yield — a `<ref>...HEAD` range and a bare commit — to the merge base with `HEAD`,
replacing the `HEAD~1...HEAD` fallback that does not resolve in a one-commit repository.

Failure to write the record is a **warning, not an error**. A project without Git
evidence has no coverage gate to satisfy in the first place, so failing initialization
over it would trade a spurious gate for a spurious hard failure.

## 2. Authorship-keyed stub exemption

`validate_effective_contracts` exempts a stub-section warning only when **no active
change authored that section**. The section name is parsed from the validator's warning
text via `STUB_SECTION_WARNING_PREFIX`; classification cannot be reused because
`WarningCategory::classify` matches `requirements` first and never reaches the stub
case. That coupling is deliberate and documented at the constant, mirroring how
`SCAFFOLD_BOILERPLATE_PREFIXES` is coupled to the scaffold text it detects.

The exemption is routed through `IgnoreRules` rather than re-deriving ignore semantics,
so a project's ignore configuration is honored identically here and everywhere else.

The negative case is the load-bearing one: a section an active change authored and then
emptied stays fatal. `effective_contract_keeps_authored_emptied_section_fatal` pins it.

## 3. Directory mappings become a reported error with an actionable fix

A new `SourceSnapshot::Directory` variant is kept distinct from `Rejected` so the
snapshot path stops reporting a confined directory as an out-of-root security escape.

Ambient validation gains a `directory_mapping` branch, guarded by
`safe_project_relative && full_path.is_dir() && source_within_root(root, file)` so it
fires only for a confined directory and never shadows the escape diagnostics that must
take precedence.

The error carries a fix built by `directory_mapping_fix`, which expands the directory
using `generator::find_module_source_files` — the same expansion `generate` and
`scaffold` apply to a `[modules."x"] files` directory. The remedy therefore names
exactly what generation would have written. `expand_directory_mapping` filters
`config.exclude_dirs`, sorts, and dedupes; the list is truncated at
`DIRECTORY_MAPPING_FIX_LIMIT` (5) with an "and N more" tail.

Snapshot callers validate retained bytes only and must not enumerate the ambient
filesystem, so they receive the shape guidance without the file list.

**Deliberately not done:** silently expanding the directory during validation. That
would make a spec assert a Public API it never declared. The author opts in by applying
the fix.

## Public API added

| Symbol | Module | Why public |
|---|---|---|
| `change::record_bootstrap_paths` | `change` | Called by `commands::init` |
| `generator::find_module_source_files` | `generator` | Called by `validator` to build the fix |
