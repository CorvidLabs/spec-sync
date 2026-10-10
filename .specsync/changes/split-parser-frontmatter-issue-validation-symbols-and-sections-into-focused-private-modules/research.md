---
change: split-parser-frontmatter-issue-validation-symbols-and-sections-into-focused-private-modules
artifact: research
---

# Research

The pristine main unit log contains 69 parser tests, all passing; the original 2409-line parser has been retained for exact body, type, constant, attribute and literal comparisons. Existing source boundaries separate compatibility frontmatter, checked issue validation, API symbols/fence masking and section analysis. Strict issue parsing deserializes complete YAML with duplicate rejection and sanitizes errors, independently of compatibility parsing. The delimiter agreement regression intentionally compares all three readers. parse_frontmatter returns normalized text; strip_frontmatter preserves borrowed input bytes. API masking preserves original byte offsets. Parent confirmed current open PRs do not touch these parser files; the semantically adjacent lifecycle consumer is out of scope.
