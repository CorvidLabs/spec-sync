## MODIFIED

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| types | `Language` enum for extension-to-language mapping |

**Consumed By**

| Module | What is used |
|--------|-------------|
| validator | `get_exported_symbols`, `has_extension`, `is_test_file` |
| scoring | `get_exported_symbols` |
| generator | `has_extension`, `is_test_file` |
| config | `has_extension` |

**Frontmatter Synchronization**

Implementation SHALL add `specs/util/util.spec.md` to `depends_on`. The apparent `config` import was test-fixture
text and is intentionally excluded by code-only Rust dependency extraction.
