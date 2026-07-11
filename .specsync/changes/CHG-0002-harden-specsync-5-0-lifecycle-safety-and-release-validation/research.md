---
change: CHG-0002-harden-specsync-5-0-lifecycle-safety-and-release-validation
artifact: research
---

# Research

The release audit uses the Gemini and Codex inline findings on PR #335 as adversarial inputs. The most important distinction is between green happy-path CI and preservation/security properties that require negative tests. External proof must exercise installed artifacts from clean directories so workspace-only dependencies, absolute paths, shallow Git history, shell assumptions, and agent discovery failures are observable.

## Review Finding Disposition

| Thread | Disposition | Required proof |
|---|---|---|
| `3563034501` Markdown block boundary | Valid, release blocker | Last `###` before `##` modify/remove preservation tests |
| `3563034507` captured command output | Valid UX/reliability issue | Verification output is streamed and exit evidence retained |
| `3563034509` lexicographic base hash | Valid fallback bug | Multiple active changes select chronological/ancestral base |
| `3563034514` Windows backslash input | Partly valid | Normalize safe relative input while still rejecting traversal/prefix escapes |
| `3563034516` Windows `false` command | Environment-sensitive | Use a cross-platform failing fixture command |
| `3563051411` dirty working tree | Valid, release blocker | Post-verification source/test/config edits invalidate evidence |
| `3563051413` absolute approval paths | Fixed in prior commit | Cross-root digest regression remains green |
| `3563051416` archived path coverage | Valid lifecycle/docs mismatch | Archive timing is enforced or archived delivery scope remains provable |
| `3563051417` malformed policy | Valid, release blocker | Existing invalid policy is a hard check error |
| `3563051419` no-spec scoped module | Valid | Requirement collection returns no IDs without reading deltas |
| `3563051422` dependency ordering | Valid, release blocker | Reverse-ID dependency applies prerequisite first |
| `3563051424` prefix path scope | Valid | `src` never covers `src-old` or `src2.rs` |
| `3563051427` failed evidence | Valid | Local unified check rejects fresh but failed evidence |
| `3563051429` unavailable path diff | Valid, release blocker | Required coverage fails closed in shallow/initial invalid-base cases |
| `3563051431` effective contract gate | Valid, release blocker | Phantom API delta fails verify and accept |
| `3563051432` late ordering gates | Valid, release blocker | Late conflict/dependency mutation blocks acceptance |

No finding may be closed solely because the current CI is green. Each valid finding requires executable evidence at the layer where the invariant is enforced.
