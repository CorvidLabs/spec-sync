---
title: "Adversarial SDD Proof"
description: "Separate knowledge preservation from deterministic API and schema drift detection."
section: "Comparisons"
order: 5
---

An SDD tool can preserve excellent knowledge without deterministically proving that code still matches it. This matrix separates those jobs.

**Legend:** **Block** = deterministic core check can fail automation; **Agent** = an agent, extension, or configured workflow can inspect it; **Record** = the artifact preserves the knowledge but does not itself parse implementation drift; **—** = not a core capability.

## Knowledge preservation

| Adversarial question | SpecSync 5.1 | Spec Kit core | OpenSpec core |
|---|---|---|---|
| Why was the behavior requested? | **Record** — requirements/change artifacts | **Record** — `spec.md` | **Record** — proposal and delta requirements |
| What architecture was chosen and why? | **Record** — context/design | **Record** — plan/research/data model | **Record** — design/proposal |
| What work remains? | **Record** — tasks | **Record** — tasks | **Record** — tasks |
| What is current truth after several changes? | **Record + Block** — canonical requirements/module contract | **Record** under the team's chosen persistence model | **Record** — canonical domain specs |
| Can the exact approved change be audited later? | **Record** — digests, verification, archive | **Record** — feature/workflow history according to team convention | **Record** — archived change folder |
| Does editing tested inputs invalidate closing evidence? | **Block** | Agent/workflow dependent | Agent/workflow dependent |

## Implementation drift

| Mutation introduced after the spec is written | SpecSync 5.1 | Spec Kit core | OpenSpec core |
|---|---|---|---|
| Add an undocumented public export | **Block** in strict mode | **Agent** / extension | **Agent** |
| Delete or rename a documented export | **Block** | **Agent** / extension | **Agent** |
| Delete a source file named by the contract | **Block** | **Agent** / custom workflow | **Agent** |
| Remove a documented database table or column | **Block** | **Agent** / extension | **Agent** |
| Change a database column type without updating the spec | **Block** in strict mode | **Agent** / extension | **Agent** |
| Break a declared cross-module spec dependency | **Block** | **Agent** / extension | **Agent** |
| Leave a requirement without required test/API evidence | **Block** at acceptance | Analyze/checklist/workflow dependent | Validate/verify workflow dependent |
| Make tests fail | **Block** when configured for verification/CI | Configurable workflow command | Agent/tool execution |
| Corrupt or stale approval/verification evidence | **Block** | Workflow implementation dependent | Workflow implementation dependent |

## Reproduce the SpecSync side

The repository ships executable examples rather than screenshots:

```bash
cargo build --release
SPECSYNC_BIN="$PWD/target/release/specsync" ./examples/sdd-lifecycle/run.sh
SPECSYNC_BIN="$PWD/target/release/specsync" ./examples/sdd-concurrent-changes/run.sh
SPECSYNC_BIN="$PWD/target/release/specsync" ./examples/sdd-five-epics/run.sh
```

The Rust test suite also injects phantom exports, undocumented exports, missing files, schema drift, stale evidence, malformed deltas, conflicting changes, unsafe paths, and cross-platform Git states. CI runs the same deterministic binary on Linux, macOS, Windows, and a packaged GitHub Action consumer.

## Fair interpretation

This is not a claim that Spec Kit or OpenSpec cannot find drift. A capable coding agent, extension, or custom workflow can find many of these mutations. The narrower claim is testable: their core artifact workflows do not ship SpecSync's language-aware export and database-schema parser as a deterministic blocking contract checker. Conversely, SpecSync does not replace Spec Kit's orchestration ecosystem or OpenSpec's lightweight proposal experience.

Sources: [Spec Kit workflow](https://github.github.com/spec-kit/quickstart.html), [Spec Kit persistence models](https://github.github.com/spec-kit/concepts/spec-persistence.html), and [OpenSpec artifact model](https://github.com/Fission-AI/OpenSpec/blob/main/docs/getting-started.md). Verified 2026-07-11.
