---
change: generate-and-scaffold-never-claim-files-another-spec-owns
artifact: requirements
---

# Requirements


- `specsync scaffold foo`/`new foo`/`generate` never write a spec whose `files:` list contains a path another spec already lists.
- A candidate set that is entirely owned by existing specs yields no generated spec.
- Explicit `config.modules` mappings are honored as authored (the config is the owner), but generated specs still never double-map a file.
