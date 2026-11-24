---
kind: evidence
id: 2026-11-24-the-change-read-module-3md-with-a-plain-read-written-before-6
aliases: []
claim: "The change read <module>.3md with a plain read, written before 6.4.2 put every project read behind the bounded reader. Red control at 30283539 with only the new test added: a_3md_over_the_read_bound_is_reported_and_not_read failed (the file was read in full, nothing reported) and no_project_file_is_read_outside_the_bounded_read failed naming src/validator/consumed_by_re.rs:2288. With read_project_string at the spec-file bound (a0013747) both pass, with the 128 tests of the check, check_scaling, project_reads, project_links and project_writes groups. A lesson file that is a link is refused earlier by coverage discovery under either read, shown on the CLI for check, score and report."
result: passed
evidence_kind: product-test
revision: cb679a13f1433afbfc0989666a234d05ae822f66
content: a309099a545c98c834a7d5a3bf4a0e3c684e607bb7325f90d97bed3d80c78d19
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two test threads
fingerprint: sha256:d70bbfdf8231ce7eeb1b3b4cdced4b4525ade895de89a80d375072f38a446f09
external: ext-e6caea
criteria: [THREE-1]
requirements: []
contracts: []
limitations: []
observed: 2026-11-24T20:49:23Z
actor:
  - agent: claude-code
    terminal: t14v
---
**The change read <module>.3md with a plain read, written before 6.4.2 put every project read behind the bounded reader. Red control at 30283539 with only the new test added: a_3md_over_the_read_bound_is_reported_and_not_read failed (the file was read in full, nothing reported) and no_project_file_is_read_outside_the_bounded_read failed naming src/validator/consumed_by_re.rs:2288. With read_project_string at the spec-file bound (a0013747) both pass, with the 128 tests of the check, check_scaling, project_reads, project_links and project_writes groups. A lesson file that is a link is refused earlier by coverage discovery under either read, shown on the CLI for check, score and report.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two test threads at revision cb679a13f143, observed 2026-11-24T20:49:23Z.
