---
change: CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec
artifact: context
---

# Context

Accepted verification currently stores one aggregate delivery-input digest. When a later canonical change legitimately modifies part of that input set, the aggregate proves that something changed but cannot prove which individual paths remain exact. The current successor helpers compensate with record-level affected-path and affected-spec unions. That permits two unsafe outcomes: a non-final successor can mask stale evidence, and one successor can cover a path while a different successor independently names the module (the path/module cross-product problem).

CHG43 replaces that inference with signed per-input evidence, definition-bound explicit predecessor edges, signed semantic succession bindings, and one shared recursive validity decision. Every newly accepted input is represented independently, including topology and ownership derived from the accepted snapshot. Exact inputs remain valid directly. Every changed path/owner obligation is valid only through a terminal semantic successor whose approved edge and closing evidence bind the predecessor ID, path, owner module, old entry digest, and new entry digest to an actual semantic delta and Git-tree transition. ID order, timestamps, and scope overlap are filters only; they are never succession proof. The successor must itself have valid current closing evidence, so validity can recurse without accepting cycles or partially complete lifecycle states.

Legacy accepted records remain byte-compatible when their aggregate is still exact. A stale legacy record receives no synthetic trust: SpecSync must find a unique trusted historical commit containing that accepted record, reconstruct the historical per-input set from that Git tree, reproduce the signed legacy aggregate exactly, and prove any legacy successor through the exact before-and-after tree transition that applied its semantic delta. Missing, ambiguous, uncommitted, or non-reproducible evidence fails closed and requires audited reopen.

Archived successors remain part of the immutable evidence graph. Moving an accepted successor into the archive therefore cannot silently make a predecessor invalid. Check, status, reopen, and archive must consume the same validator and produce the same conclusion.

The final Git evidence implementation uses one bounded subprocess surface and returns immutable per-candidate topology and payload snapshots. Git repository detection is the only command allowed to select non-Git behavior; after positive detection, command, index, attribute, and parsing failures remain fatal. The runner drains stdout and stderr concurrently into pre-capped buffers, kills on overflow, and always reaps the child. Ambient repository/worktree/common-directory and injected-config variables are removed, repository-local configuration remains authoritative, and an explicitly supplied `GIT_INDEX_FILE` is resolved against the governed root.

Evidence inspection fingerprints only the effective index and the actual split-index dependency reported by Git. It scopes stage, visibility, fsmonitor, dirty-state, and transforming-attribute checks to exact digest candidates, captures their topology and bytes, then repeats both the scoped capture and index fingerprint. A changed generation or candidate snapshot receives one bounded retry and then fails closed. Callers consume the returned snapshot; archived definition artifacts retain canonical digest labels while using their actual dated location as the evidence candidate.

Repository detection must itself be evidence-bearing. Only Git's ordinary outside-a-work-tree result in a directory without repository/worktree markers may select the strict non-Git walker. A nonzero detection result accompanied by Git metadata or operational, configuration, permission, ownership, corruption, or dubious-repository diagnostics is inconclusive and fails closed with bounded stderr instead of silently weakening evidence rules.

Candidate discovery and candidate capture form one evidence generation. Both Git and non-Git paths require matching pre/post inventories and matching immutable payload/topology snapshots inside the same bounded retry; a governed tracked or untracked addition, removal, or replacement during either phase retries once and then fails closed.

Bounded Git execution includes both byte and time bounds. A deadline sized for the maximum supported inventory prevents silent helpers or hooks from blocking forever, while a child/process guard owns termination, reaping, and pipe-thread cleanup on every return path. Tests use a shorter injected deadline without weakening the production bound.

Git command arguments never receive governed paths as raw pathspecs. Exact candidates use top-level literal pathspecs, while directory discovery scopes use an explicit top-level literal-prefix form with intentional recursion. Filenames beginning with a colon or containing glob metacharacters therefore cannot omit their own evidence or select unrelated paths.

Index stage/object inspection is scoped to the same exact literal candidates in bounded batches. Caps aggregate across those batches; every returned record is validated, each tracked candidate has exactly one stage-zero entry, and relevant unresolved stages fail closed without reading or being denied by unrelated index contents.

Definition input enumeration is equally strict: a missing delta directory is distinct from an unreadable or mutating directory, every entry and portable UTF-8 name is validated within bounds, and the resulting inventory participates in the same immutable capture generation. Enumeration errors can never silently omit an approved artifact.

Volatile project trees are pruned during streaming discovery, before directory descent and before path/count/byte accounting. This prevents generated `target`, dependency, and active lifecycle workspaces from denying unrelated project evidence, while an exact path explicitly governed by the delivery contract remains evidence-relevant even when it resides beneath an otherwise volatile prefix.

Each digest operation selects repository mode once and carries an authenticated repository/worktree identity through discovery and capture. The identity and mode are revalidated with the final inventory and snapshot, so mutation of `.git`, linked-worktree metadata, the effective Git directory, or worktree association cannot mix Git discovery with non-Git capture or evidence from different repositories.

Effective and shared Git index dependencies are fingerprinted as bounded regular files. Metadata establishes type, identity, and cumulative size before streaming bytes into the digest; post-read metadata must match. Symlinks, non-regular files, oversized dependencies, truncation, replacement, and growth fail closed without an unbounded allocation.

Attribute inspection authenticates its response shape as well as its values. NUL output must contain exactly one `(path, attribute, value)` record for each requested path crossed with `filter`, `working-tree-encoding`, and `ident`; malformed termination, empty fields, wrong or unrequested names, duplicates, omissions, and extras fail closed before any canonical index substitution.

The correction ledger is a signed definition input, not an untyped virtual default. When present, `corrections.json` must be captured as regular-file immutable evidence before parsing; symlink, gitlink, missing/non-file, and external-target topology is rejected, and generated canonical content retains the captured regular-file mode.

Archive snapshot discovery is scoped literally to the dated archive subtree and participates in the same repository identity, inventory, and capture generation. Unrelated repository size or nonportable names cannot block the narrow archive snapshot.

Controlled checkout semantics use an exact allowlist: normalized effective `core.autocrlf`, `core.eol`, `core.symlinks`, and `core.filemode` values are resolved in the governed root and replayed explicitly only for diff/status classification that must interpret an existing checkout consistently. Ambient repository redirection, object stores, executable paths, config injection, hooks, filters, fsmonitor, and all non-allowlisted configuration remain scrubbed or fail closed.
