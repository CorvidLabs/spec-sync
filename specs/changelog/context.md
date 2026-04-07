# changelog — Context

The changelog module was added in v3.3.0 (#141) to provide automated release notes by diffing spec state between git refs. It relies on git plumbing commands (`ls-tree`, `show`) to read spec files at arbitrary commits without touching the working tree.
