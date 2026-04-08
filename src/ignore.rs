use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Warning categories that can be suppressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarningCategory {
    RequirementsCompanion,
    StubSection,
    UndocumentedExport,
    Deprecated,
    UnknownStatus,
    UnknownAgentPolicy,
    SchemaColumn,
    SchemaTypeMismatch,
    ConsumedBy,
    ChangelogEntries,
    SpecSize,
    MinInvariants,
    RequireDependsOn,
}

impl WarningCategory {
    /// Parse a category name from a string (case-insensitive, supports kebab-case).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "requirements-companion" | "requirements" => Some(Self::RequirementsCompanion),
            "stub-section" | "stub" => Some(Self::StubSection),
            "undocumented-export" | "undocumented" => Some(Self::UndocumentedExport),
            "deprecated" => Some(Self::Deprecated),
            "unknown-status" => Some(Self::UnknownStatus),
            "unknown-agent-policy" => Some(Self::UnknownAgentPolicy),
            "schema-column" => Some(Self::SchemaColumn),
            "schema-type-mismatch" | "schema-mismatch" => Some(Self::SchemaTypeMismatch),
            "consumed-by" => Some(Self::ConsumedBy),
            "changelog-entries" | "changelog" => Some(Self::ChangelogEntries),
            "spec-size" => Some(Self::SpecSize),
            "min-invariants" | "invariants" => Some(Self::MinInvariants),
            "require-depends-on" | "depends-on" => Some(Self::RequireDependsOn),
            _ => None,
        }
    }

    /// Classify a warning message into a category based on its text.
    pub fn classify(warning: &str) -> Option<Self> {
        if warning.contains("requirements") {
            return Some(Self::RequirementsCompanion);
        }
        if warning.contains("stub") && warning.starts_with("Section ##") {
            return Some(Self::StubSection);
        }
        if warning.starts_with("Undocumented export '") || warning.starts_with("Export '") {
            return Some(Self::UndocumentedExport);
        }
        if warning.contains("deprecated") {
            return Some(Self::Deprecated);
        }
        if warning.starts_with("Unknown status") {
            return Some(Self::UnknownStatus);
        }
        if warning.starts_with("Unknown agent_policy") {
            return Some(Self::UnknownAgentPolicy);
        }
        if warning.starts_with("Schema column") && warning.contains("type mismatch") {
            return Some(Self::SchemaTypeMismatch);
        }
        if warning.starts_with("Schema column") {
            return Some(Self::SchemaColumn);
        }
        if warning.starts_with("Consumed By") {
            return Some(Self::ConsumedBy);
        }
        if warning.contains("Change Log has") && warning.contains("entries") {
            return Some(Self::ChangelogEntries);
        }
        if warning.contains("KB") && warning.contains("exceeds limit") {
            return Some(Self::SpecSize);
        }
        if warning.contains("invariant(s) found") {
            return Some(Self::MinInvariants);
        }
        if warning.contains("rule: require_depends_on") {
            return Some(Self::RequireDependsOn);
        }
        None
    }
}

/// Rules for suppressing warnings, loaded from `.specsyncignore` and inline comments.
#[derive(Debug, Default)]
pub struct IgnoreRules {
    /// Categories suppressed globally (all specs).
    pub global: HashSet<WarningCategory>,
    /// Categories suppressed for specific spec paths.
    pub per_spec: std::collections::HashMap<String, HashSet<WarningCategory>>,
}

impl IgnoreRules {
    /// Load ignore rules from `.specsyncignore` file in the project root.
    ///
    /// Format (one rule per line):
    /// ```text
    /// # Comment
    /// requirements-companion           # suppress globally
    /// stub-section:specs/auth/         # suppress for specs under this path
    /// undocumented-export:specs/api.spec.md  # suppress for specific spec
    /// ```
    pub fn load(root: &Path) -> Self {
        let mut rules = Self::default();
        let ignore_path = root.join(".specsyncignore");
        let content = match fs::read_to_string(&ignore_path) {
            Ok(c) => c,
            Err(_) => return rules,
        };

        for line in content.lines() {
            let line = line.trim();
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Strip inline comments
            let line = line.split('#').next().unwrap_or(line).trim();

            if let Some((category_str, spec_pattern)) = line.split_once(':') {
                // Per-spec rule: category:path
                if let Some(category) = WarningCategory::from_str(category_str) {
                    let pattern = spec_pattern.trim().to_string();
                    rules.per_spec.entry(pattern).or_default().insert(category);
                }
            } else if let Some(category) = WarningCategory::from_str(line) {
                // Global rule
                rules.global.insert(category);
            }
        }

        rules
    }

    /// Parse inline ignore directives from a spec file body.
    ///
    /// Format: `<!-- specsync-ignore: category1, category2 -->`
    pub fn parse_inline(body: &str) -> HashSet<WarningCategory> {
        let mut categories = HashSet::new();
        for line in body.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("<!-- specsync-ignore:") {
                if let Some(content) = rest.strip_suffix("-->") {
                    for part in content.split(',') {
                        if let Some(cat) = WarningCategory::from_str(part.trim()) {
                            categories.insert(cat);
                        }
                    }
                }
            }
        }
        categories
    }

    /// Check if a warning should be suppressed for a given spec path.
    pub fn is_suppressed(
        &self,
        warning: &str,
        spec_rel_path: &str,
        inline_ignores: &HashSet<WarningCategory>,
    ) -> bool {
        let category = match WarningCategory::classify(warning) {
            Some(c) => c,
            None => return false,
        };

        // Check global suppression
        if self.global.contains(&category) {
            return true;
        }

        // Check inline suppression
        if inline_ignores.contains(&category) {
            return true;
        }

        // Check per-spec suppression
        for (pattern, categories) in &self.per_spec {
            if categories.contains(&category) && spec_rel_path.starts_with(pattern.as_str()) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_requirements_companion() {
        assert_eq!(
            WarningCategory::classify(
                "Missing companion requirements.md — run `specsync generate` or create one manually"
            ),
            Some(WarningCategory::RequirementsCompanion)
        );
        assert_eq!(
            WarningCategory::classify(
                "Requirements appear inline in the spec body — move to a companion requirements.md file"
            ),
            Some(WarningCategory::RequirementsCompanion)
        );
    }

    #[test]
    fn test_classify_stub_section() {
        assert_eq!(
            WarningCategory::classify(
                "Section ## Purpose contains only stub/placeholder text (TBD, N/A, TODO, etc.)"
            ),
            Some(WarningCategory::StubSection)
        );
    }

    #[test]
    fn test_classify_undocumented_export() {
        assert_eq!(
            WarningCategory::classify("Undocumented export 'foo' from src/bar.ts"),
            Some(WarningCategory::UndocumentedExport)
        );
        assert_eq!(
            WarningCategory::classify("Export 'baz' not in spec (undocumented)"),
            Some(WarningCategory::UndocumentedExport)
        );
    }

    #[test]
    fn test_classify_schema_type_before_column() {
        // Type mismatch should match before generic schema-column
        assert_eq!(
            WarningCategory::classify(
                "Schema column `users.name` type mismatch: spec says TEXT but migrations say VARCHAR"
            ),
            Some(WarningCategory::SchemaTypeMismatch)
        );
        assert_eq!(
            WarningCategory::classify(
                "Schema column `users.age` exists in migrations but not documented in spec"
            ),
            Some(WarningCategory::SchemaColumn)
        );
    }

    #[test]
    fn test_from_str_aliases() {
        assert_eq!(
            WarningCategory::from_str("requirements"),
            Some(WarningCategory::RequirementsCompanion)
        );
        assert_eq!(
            WarningCategory::from_str("requirements-companion"),
            Some(WarningCategory::RequirementsCompanion)
        );
        assert_eq!(
            WarningCategory::from_str("stub"),
            Some(WarningCategory::StubSection)
        );
        assert_eq!(
            WarningCategory::from_str("REQUIREMENTS_COMPANION"),
            Some(WarningCategory::RequirementsCompanion)
        );
    }

    #[test]
    fn test_parse_inline() {
        let body = "## Purpose\nSomething\n<!-- specsync-ignore: requirements-companion, stub-section -->\n## API\n";
        let cats = IgnoreRules::parse_inline(body);
        assert!(cats.contains(&WarningCategory::RequirementsCompanion));
        assert!(cats.contains(&WarningCategory::StubSection));
        assert!(!cats.contains(&WarningCategory::UndocumentedExport));
    }

    #[test]
    fn test_is_suppressed_global() {
        let mut rules = IgnoreRules::default();
        rules.global.insert(WarningCategory::RequirementsCompanion);

        let inline = HashSet::new();
        assert!(rules.is_suppressed(
            "Missing companion requirements.md — run `specsync generate`",
            "specs/auth/auth.spec.md",
            &inline,
        ));
        assert!(!rules.is_suppressed(
            "Section ## Purpose contains only stub/placeholder text",
            "specs/auth/auth.spec.md",
            &inline,
        ));
    }

    #[test]
    fn test_is_suppressed_inline() {
        let rules = IgnoreRules::default();
        let mut inline = HashSet::new();
        inline.insert(WarningCategory::StubSection);

        assert!(rules.is_suppressed(
            "Section ## Purpose contains only stub/placeholder text (TBD, N/A, TODO, etc.)",
            "specs/auth/auth.spec.md",
            &inline,
        ));
    }

    #[test]
    fn test_is_suppressed_per_spec() {
        let mut rules = IgnoreRules::default();
        let mut cats = HashSet::new();
        cats.insert(WarningCategory::UndocumentedExport);
        rules.per_spec.insert("specs/legacy/".to_string(), cats);

        let inline = HashSet::new();
        assert!(rules.is_suppressed(
            "Undocumented export 'oldFunc' from src/legacy.ts",
            "specs/legacy/api.spec.md",
            &inline,
        ));
        assert!(!rules.is_suppressed(
            "Undocumented export 'newFunc' from src/core.ts",
            "specs/core/core.spec.md",
            &inline,
        ));
    }

    #[test]
    fn test_load_specsyncignore() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".specsyncignore"),
            "# Global suppressions\nrequirements-companion\n\n# Per-spec\nstub-section:specs/legacy/\n",
        )
        .unwrap();

        let rules = IgnoreRules::load(tmp.path());
        assert!(
            rules
                .global
                .contains(&WarningCategory::RequirementsCompanion)
        );
        assert!(!rules.global.contains(&WarningCategory::StubSection));
        assert!(rules.per_spec.contains_key("specs/legacy/"));
        assert!(rules.per_spec["specs/legacy/"].contains(&WarningCategory::StubSection));
    }

    #[test]
    fn test_load_no_file() {
        let tmp = tempfile::tempdir().unwrap();
        let rules = IgnoreRules::load(tmp.path());
        assert!(rules.global.is_empty());
        assert!(rules.per_spec.is_empty());
    }
}
