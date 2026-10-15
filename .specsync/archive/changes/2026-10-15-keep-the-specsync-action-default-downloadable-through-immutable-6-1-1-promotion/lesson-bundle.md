# Lesson bundle — keep-the-specsync-action-default-downloadable-through-immutable-6-1-1-promotion

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep the SpecSync Action default downloadable through immutable 6.1.1 promotion
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/scripts/test-validate-release-candidate.py, .github/scripts/validate-release-version.py, CHANGELOG.md, README.md, action.yml, site/src/content/docs/integrations/github-action.md, specs/github/github.3md, specs/github/github.spec.md, specs/github/requirements.md, specs/github/tasks.md, specs/github/testing.md
- **Acceptance**: Keep the Action and inputs-table omitted default on qualified published6.1.0 while Cargo and explicit candidate CI/Trust remain6.1.1; enforce an independent reviewed published-default constant with real-validator regression controls; preserve candidate mirrors and all release identity guards; document the preceding stable fallback and immutablev6.1.1 default6.1.0 with explicitversion6.1.1 guidance; only a separately verified postpublication wrapper may advance main/floating after real platform asset and Action checks; unchanged frozen PhaseA patch and full local/native/hosted evidence precede archive and merge.

## Evidence

- Verification commit: `62cea5ae6a656a6ceb87a668275b265886257958`
- Base commit: `944ef631d698f3d45aa5ad657817a7cd3c11d2a2`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

The public Action default selects stable6.1.1 before stable assets exist. Live GitHub evidence identifies published6.1.0 with all five archive/checksum pairs and no6.1.1 Release. The old offline validator incorrectly rejects candidate6.1.1/default6.1.0 and accepts the premature default. The concrete eleven-file external PhaseA and conditional eight-file PhaseB were independently reviewed and accepted by Leif with I approve after the exact staged policy and immutable-tag caveat were presented. This package implements only PhaseA. Previously accepted archives remain intact. Real delivery evidence is not inferred from the published-default declaration.

## From the change's design.md

# Design

Pin the declared published default independently of the package candidate. Change the Action omitted default and documented default to6.1.0. Keep explicit CI/Trust6.1.1 localmirror inputs and all release/download/checksum/runtime guards byte-identical. Add twelve actual production-validator subprocess tests in the existing wired suite. Immutablev6.1.1 will retain default6.1.0; explicitversion6.1.1 selects its new binary. ConditionalPhaseB changes only the later wrapper/default declaration/current docs once its actual activation gates pass. No dynamic release selector or dependency is added.

## From the change's testing.md

# Testing

The original production validator failed two meaningful controls; receipt130308-CODEX-5e9c8b. Final external receipt131336-CODEX-efc001 passed the complete64-test existing+regression suite and actual version validator in each exact staged implementation. No asset or platform result is inferred.

## Requirement evidence

| Requirement | Required evidence |
|-------------|-------------------|
| REQ-github-002 | Actual validator accepts safe published6.1.0 for package6.1.1 and rejects premature6.1.1; arbitrary9.9.9/latest/missing/defaultdocsdrift fail; explicit CI/Trust candidate versions and mirrors and Cargo/lock agreement remain enforced. Independent final source review; full pinned applicable localverification and native scopedcheck; exact product/archive CI; actual immutableUbuntu/macOS qualification and stableasset/consumer tests remain later gates. |

Fresh local verification on the applied acceptedsource is still required before completion. It includes the complete64-test validator suite and productionversion validator; full Fledgeverify; strict coverage and current securityaudit; mandatoryprepush/Trust/Attest. The unchanged Rust runtime retains previous source evidence but is not substituted for the newly required applied-source checks. Existing manualMacwatchcontrol limitation remains candid; realLinuxnativewatch passes are preserved and the immutableMacrelease lane remains mandatory.

## Completed applied-source qualification

Applied-source full Fledge verify and both forced strict100% checks plus current offline audit passed. Actual Rust test result records: [('1', '0', '0'), ('1', '0', '0'), ('1', '0', '0'), ('1', '0', '0'), ('2582', '0', '0'), ('462', '0', '0')]. Full existing release-validator suite now includes64tests; all29CI-gate controls passed. Receipt134259-CODEX-0a143a and per-command qualification.json retain actual logs and exact frozen inputs. No actual future asset/platform result is claimed.

Leif genuinely accepted both prepared stages with I approve. Only prepublication PhaseA is applied. Native scopedcheck and later product/archive CI are separate actual gates; conditionalPhaseB stays external until genuine publication and platform conditions hold.

## Where these lessons go

- `specs/github/github.3md`
