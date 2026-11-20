## ADDED

### REQUIREMENT REQ-change-169

Definition-approval validation SHALL compare every committed version of the change's `approvals.json` that `HEAD` reaches, both sides of every merge included. When two versions each hold an approval the other lacks, as when two clones approve the same change and their branches merge, the definition approval SHALL NOT count until one definition approval recorded after both is the one that counts. `change check`, `change review`, `change finalize`, `change ship` and `change audit` SHALL refuse it, and `change status` SHALL report the approval as not valid, naming both approvers, the two commits and `specsync change approve <id> --actor <name>`, which records that one fresh approval. A version that only extends another SHALL NOT count as a fork, and the `relayed_by` of an approval SHALL NOT be compared. An archived change SHALL NOT be judged again.

Acceptance Criteria
- Two clones approve the same change with different actors and each commits. After a merge that keeps one clone's ledger, or both, `change check` and `change audit --strict` refuse, and `change status` reports the approval as not valid, naming both approvers, both commits and the fresh approval.
- One `change approve` on the merged branch settles it, and the same commands pass.
- Approvals that only append are not a fork, and neither is a change sent back and approved again, or a relay removed or restored by `change repair-relays`.
- An archived change whose history holds a fork is not judged again.
- Through the binary, the reproduction above is refused, and one fresh approval lets `change check --commit` pass.
