## ADDED

### REQUIREMENT REQ-rehash-001

The rehash command SHALL rebuild the local hash cache from current canonical inputs and fail clearly when persistence fails.

Acceptance Criteria
- `cmd_rehash` loads canonical configuration and discovers non-template specs directly through validator APIs,
  without depending on the parent command registry, then rebuilds a fresh `HashCache` from scratch
- The rebuilt cache is written to `.specsync/hashes.json`
- On success, prints a confirmation including the number of specs hashed
- On `cache.save` failure, prints an `error:` message to stderr and exits with code 1

## MODIFIED

### SPEC SECTION Invariants

1. Loads canonical configuration and discovers spec files through `config::load_config` and
   `validator::find_spec_files`
2. Filters underscore-prefixed template specs and exits successfully with generation guidance when no specs exist
3. Builds a fresh `HashCache` from scratch (not incremental)
4. Saves cache to `.specsync/hashes.json`
5. Exits with code 1 if cache save fails

### SPEC SECTION Dependencies

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| validator | `find_spec_files` |
| hash_cache | `HashCache::default`, `update_cache`, `save` |
