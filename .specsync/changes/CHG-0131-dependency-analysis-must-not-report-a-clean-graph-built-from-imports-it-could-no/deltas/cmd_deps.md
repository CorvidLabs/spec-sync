## ADDED

### REQUIREMENT REQ-cmd-deps-003

Every output format SHALL disclose what the analysis could not read and could not attribute.

Acceptance Criteria
- Imports read but not mappable to a spec module are named in text, JSON, markdown, and on stderr in diagram mode, from one shared formatting site.
- A language with an import construct but no extractor is disclosed; a language with no import concept at all is not.
- The success sentence is qualified when either disclosure is non-empty.
- Neither disclosure is an error or a warning, and neither changes the exit code.
