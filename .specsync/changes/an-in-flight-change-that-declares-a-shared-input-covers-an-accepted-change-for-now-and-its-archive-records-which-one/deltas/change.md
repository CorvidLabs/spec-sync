## ADDED

### REQUIREMENT REQ-change-167

A changed delivery input of an `accepted` change SHALL count as covered for now when no accepted or archived successor covers it and an `approved`, `implementing` or `verifying` change created after the accepted one declares the input's owning module and a path scope that covers the input. An accepted change whose changed inputs are all covered, at least one of them only for now, SHALL read `provisionally-covered` as its terminal evidence, with a reason that names each such input and the change covering it. `change audit` SHALL report it as a warning naming both changes and never as an error, so `change audit --strict` exits 0 on it, and `specsync doctor` SHALL NOT list it as drifted. A changed input that nothing covers SHALL stay stale and stay an audit error. A provisionally covered change SHALL NOT cover another change's input and SHALL NOT stand as a retirement replacement, and `change reopen` SHALL still accept it. `change archive` SHALL accept it, the archive preflight of any other change SHALL pass over it, and its archive SHALL hold `coverage.json`: a schema version, the change id, and one `{path, change}` entry for each input covered for now. The archive's own evidence SHALL NOT depend on `coverage.json` or on what becomes of the covering change. `change status` and `change show` on such an archive SHALL print each recorded entry with the covering change's present state. When that change is no longer approved, implementing, verifying, accepted or archived, because it was sent back or closed, they and `change audit --archives` SHALL say, as a warning, that the input changed after acceptance and no accepted change stands behind it, and SHALL name `specsync change reopen <id> --actor <name> --reason <text>` for the archived change.

Acceptance Criteria
- An accepted change, a later approved change that declares the same module and path, and a committed edit of that file: the accepted change reads `provisionally-covered` naming the later change; `change audit` and `change audit --strict` exit 0 with one warning naming both; `specsync doctor` lists no drift for it.
- The same edit with no later change declaring the file: the accepted change reads `stale` and `change audit` exits 1, as before.
- A later change that is still a draft, or that declares the path without the owning module, does not cover.
- `change archive` on the provisionally covered change succeeds and writes `coverage.json` naming the file and the covering change; another change's finalize is not refused because of it.
- A provisionally covered change is not accepted as another change's covering successor or as a retirement replacement, and `change reopen` accepts it.
- When the covering change is accepted, the earlier change reads `successor-covered`.
- After the archive, with the covering change sent back to draft: the archived change still reads `authenticated-history`; `change status` on it prints the recorded entry and the warning; `change audit --archives` prints the warning and exits 0. With the covering change accepted or archived instead, there is no warning.
- An archive with no `coverage.json` reads exactly as before.

### REQUIREMENT REQ-change-168

The two archive refusals that share one stale delivery input SHALL each name the other side and say that it is blocked too. When `change archive` refuses an accepted change because a delivery input changed after acceptance and no accepted or archived successor covers it, the refusal SHALL add one sentence naming each later change whose declared scope covers that input and saying that it cannot be archived either while this change is stale. When archiving a change is refused because the archive preflight finds another accepted change stale, the refusal SHALL add one sentence saying that the named change cannot be archived either until it is reopened or a covering change is accepted, so the two wait on each other. Neither sentence SHALL replace the `change reopen` command the refusal already names.

Acceptance Criteria
- Two accepted changes share a delivery input, the later one edited it and does not cover it: `change archive` on the earlier one names the later one and says it is blocked too; archiving the later one names the earlier one and says it is blocked too; both still name `specsync change reopen` for the earlier one.
- Running the named `change reopen` on the earlier change lets the later one archive.
- With no later change whose scope covers the input, the first refusal reads as it does today.
