## ADDED

### SPEC SECTION Internal Organization

Parser retains its public and crate-visible API paths through a small facade. Private owners separate compatibility frontmatter and the shared delimiter scan, strict checked issue-reference parsing, API symbol extraction and byte-preserving fence masking, and section analysis. Existing tests are grouped beside their owners. The split preserves strict-versus-compatibility boundaries, content-free checked errors, duplicate-key and positive-number rejection, CRLF normalization versus borrowed stripping, delimiter agreement, exact symbol spelling, and order-preserving deduplication without introducing new public behavior.
