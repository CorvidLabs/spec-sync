---
change: CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement
artifact: requirements
---

# Requirements

## Contract accuracy

The accepted canonical contracts SHALL describe the current implementation rather than relying on symbol-name-only validation.

Acceptance Criteria
- All 35 currently undeclared source-import edges are represented in canonical `depends_on` frontmatter.
- The nine audited Public API rows use the current source signatures and preserve their current returns and descriptions.
- Canonical configuration and rule prose distinguish the current TOML layout from legacy JSON compatibility.

## Stable requirement identity

Every canonical requirements companion SHALL preserve its existing user stories, criteria, constraints, and exclusions while gaining stable normative identities where absent.

Acceptance Criteria
- The 44 legacy companions each gain at least one module-matching `REQ-...-NNN` block.
- Every added requirement contains a SHALL statement and explicit acceptance criteria.
- Existing requirement prose is retained; migration adds identity rather than replacing historical detail.

## Maturity and evidence

Canonical maturity and companion metadata SHALL reflect shipped behavior and verifiable evidence.

Acceptance Criteria
- `cmd_migrate` documents all eleven implementation steps and is promoted only after focused and full validation.
- `cmd_rules/context.md` gains its missing companion frontmatter during implementation.
- Open roadmap tasks remain visible and are not rewritten as completed work.

## Version-neutral canonical configuration

Canonical configuration generation SHALL use a version-neutral identity so a product release does not make newly generated project metadata immediately stale.

Acceptance Criteria
- `config_to_toml` emits `# spec-sync configuration` as its exact first line.
- The committed `.specsync/config.toml` uses the same exact first line.
- A focused regression test rejects a release-number-bearing canonical header.

## Task and signoff truth

Companion task and signoff sections SHALL distinguish completed evidence, partial coverage, deferred work, and human approval without inventing completion or approval.

Acceptance Criteria
- The six fully evidenced task rows are checked only with their concrete evidence retained.
- The four partially evidenced rows are split into a completed narrow claim and an open remainder.
- Other unchecked work is grouped under explicit `Post-5.0 Roadmap`, `Test Debt`, or `Manual` headings.
- Legacy pending signoff templates become a truthful informational note; no named or implied approval is fabricated.

## Dependency graph accuracy

Dependency validation SHALL analyze executable imports, resolve Rust module names to the spec that owns their source,
and keep the declared graph acyclic.

Acceptance Criteria
- Rust imports inside comments and ordinary, raw, byte, or raw-byte strings do not create dependency edges.
- A top-level Rust module such as `crate::cli` resolves through its owned source file rather than a coincidentally
  named spec module.
- Rehash discovers canonical specs without depending back on its parent command registry.
- Strict dependency validation reports zero undeclared imports, missing dependencies, and cycles.
