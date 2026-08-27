## MODIFIED

### REQUIREMENT REQ-change-083

A minted change slug SHALL be a legal directory component on every platform a SpecSync repository may be checked out on, Windows included, whether or not SpecSync publishes a binary for that platform, and SHALL remain readable when the description is too long to keep.

Acceptance Criteria
- The platforms this rule covers are the platforms a repository may be checked out on, not the platforms SpecSync publishes binaries for. Narrowing the published set does not narrow this rule, because the directory a slug becomes is created in someone else's clone.
- The length limit bounds the bytes of the name that reaches the filesystem rather than the characters of the description it came from, and is sized so the deepest path a change produces stays within the shortest maximum path length of any host platform, which is Windows `MAX_PATH` at 260.
- A name that must be shortened is cut at a word boundary rather than mid-word whenever a boundary is near enough for the result to stay legible, because the description is stored in full elsewhere and the directory name exists to be read.
- A description that would reduce to a reserved directory name does not become one, including the name substituted when a description reduces to nothing.
- A description that needs none of this produces exactly the name it produced before.

### REQUIREMENT REQ-change-084

A change identity SHALL be accepted or refused on the properties that make a string a safe path component, and SHALL NOT be required to begin with any particular prefix.

Acceptance Criteria
- An identity carrying no ordinal is accepted, because a prefix is text any caller can type and is therefore evidence neither that an identity is well-formed nor that SpecSync minted it.
- An identity is refused when it is empty, is not a single path component, contains a path separator or a control character, exceeds the longest name a path component may hold, or is a name a host platform reserves, Windows device names included.
- Every identity shape SpecSync has previously minted remains acceptable, so relaxing what is required does not orphan history.
