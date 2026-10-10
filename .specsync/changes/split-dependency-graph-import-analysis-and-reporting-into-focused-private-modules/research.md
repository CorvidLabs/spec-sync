---
change: split-dependency-graph-import-analysis-and-reporting-into-focused-private-modules
artifact: research
---

# Research

The baseline is main 815690d, with 2416 lines in src/deps.rs and 43 unit tests. The main baseline unit log records every deps test passing, so a duplicate pristine compilation is unnecessary. The original source will be preserved outside the repository for exact comparison.

ImportDialect is the single analysis authority. Rust masking handles nested comments, byte/raw/escaped strings and character syntax. JVM ownership is based on declared packages before directory layout; ambiguous package claims stay unattributed. Graph and import source errors remain fail-loud. The existing all-target lint and full-suite sandbox failures are not to be repaired by this refactor. No new dependency or runtime is needed.
