## ADDED

### REQUIREMENT REQ-github-027

Continuous integration SHALL validate the syntax of every maintained Mermaid diagram and reject lifecycle diagrams whose states or send-back transitions disagree with the implementation (DOCS-5, DOCS-5.a).

Acceptance Criteria
- The existing required site test lane parses every Mermaid fence in the maintained README, CONTRIBUTING, docs tree, site Markdown/MDX/Astro content and checked-in skill files with a pinned Mermaid parser, reporting the file and line on failure. A missing parser is a failure, not a skipped pass.
- The HLD change-state diagram contains exactly the ChangeState variants and a send-back-to-Draft edge from every SEND_BACK_STATES entry; source extraction failure or an unrecognized source shape fails explicitly.
- Mutation controls prove malformed Mermaid, a missing lifecycle state, an extra state and a missing send-back edge are rejected. Parsing alone is not claimed to prove behavior.
- CI runs the checks for root/reference documentation changes and site changes through its existing selected site/full lanes, without weakening any gate, bound or timeout and without network link probes.
