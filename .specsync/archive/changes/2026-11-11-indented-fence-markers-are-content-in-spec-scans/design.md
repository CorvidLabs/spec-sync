---
change: indented-fence-markers-are-content-in-spec-scans
artifact: design
---

# Design

Two scanners share one rule: a line indented four or more spaces (or a tab) is
content under CommonMark and never updates fence state. `export_table`
(src/commands/check.rs) checks indentation before the fence-marker match;
`parser::blank_fenced_code` (src/parser.rs) computes the marker only for
unindented lines, so an indented line inside an open fence is still blanked as
fence content. No new types, no format changes; the regression test drives
`check --fix` end to end.
