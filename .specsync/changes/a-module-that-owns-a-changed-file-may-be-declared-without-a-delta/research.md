---
change: a-module-that-owns-a-changed-file-may-be-declared-without-a-delta
artifact: research
---

# Research

Ten places read a delta for every affected module (collect_requirement_ids.rs and send_back.rs);
the other loops over `affected_specs` read spec files, verification routing or lessons, which a
code-only module keeps. `read_approved_delta_binding` lists as changed every approved module
whose current body is not intact, so dropping a module from the per-module read cannot hide a
deleted approved delta.
