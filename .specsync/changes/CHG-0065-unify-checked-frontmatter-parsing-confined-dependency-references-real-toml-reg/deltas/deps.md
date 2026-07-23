## ADDED

### REQUIREMENT REQ-deps-002

Dependency graph construction SHALL consume checked specs and typed references without silently
dropping declarations or inflating graph results.

Acceptance Criteria

- A spec read or checked-frontmatter failure is retained as a hard graph error rather than omitted.
- Flow and block `depends_on` lists produce the same graph edges.
- Repeated equivalent local declarations produce one edge, one edge count, and one Mermaid or DOT
  relationship.
- Bare modules, canonical spec paths, and registry-mapped custom spec paths resolve to one module
  identity; remote references are syntax-checked but excluded from the local graph.
- Valid `implements` and `tracks` issue-number lists remain typed metadata and create no dependency
  node, edge, cycle, count, Mermaid relationship, or DOT relationship.
- Malformed, unsafe, missing, or identity-conflicting dependencies produce the same failures in
  validation, JSON, Mermaid, and DOT modes.
- Graph node and edge ordering remains deterministic.

## MODIFIED

### SPEC SECTION Invariants

1. Graph construction consumes checked frontmatter and shared typed dependency references; a read,
   parse, identity, confinement, registry, or missing-target failure is retained in the report.
2. Block and flow dependency sequences produce identical results, and normalized duplicate
   declarations produce one ordered edge and one rendered relationship.
3. Bare modules resolve through registry mappings before conventional
   `specs/<module>/<module>.spec.md` locations; canonical paths and registry-mapped custom paths
   converge on the same module identity.
4. Remote references are syntax-checked but excluded from the local graph.
5. `implements` and `tracks` remain typed issue metadata and never contribute dependency nodes,
   edges, cycle detection, counts, Mermaid, or DOT output.
6. Every renderer consumes the same checked graph report and surfaces the same hard graph errors;
   diagram modes cannot rebuild a permissive graph.
7. Module and edge ordering is deterministic across validation, JSON, Mermaid, DOT, and
   topological sorting.
8. Circular dependency detection traverses the complete normalized graph, and
   `topological_sort` returns `None` rather than a partial order when a cycle exists.
9. Undeclared imports remain advisory warnings and source-import extraction retains its documented
   language-specific behavior.
10. Existing but unreadable specs or declared source files are hard errors rather than silent
    omissions.
