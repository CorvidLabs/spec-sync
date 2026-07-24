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
    /// The `N/M exports documented` partial-coverage summary warning.
    ExportsDocumented,
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
            "exports-documented" => Some(Self::ExportsDocumented),
            _ => None,
        }
    }

    /// Classify a warning message into a category based on its text.
    pub fn classify(warning: &str) -> Option<Self> {
        if warning.contains("requirements") {
            return Some(Self::RequirementsCompanion);
        }
        if warning.starts_with("Section ##")
            && (warning.contains("stub") || warning.contains("unfinished draft"))
        {
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
        // The partial-coverage summary `N/M exports documented` — suppressible
        // so acknowledging undocumented exports can yield a clean --strict run.
        if warning.ends_with("exports documented")
            && warning
                .split_once('/')
                .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
        {
            return Some(Self::ExportsDocumented);
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
    /// Problems found while loading (invalid UTF-8 lines, unmatchable
    /// patterns). Surfaced by the caller so dead/typo'd rules are visible
    /// instead of silently doing nothing.
    pub warnings: Vec<String>,
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
    /// specs/api.spec.md:undocumented-export  # path:first order also accepted
    /// ```
    ///
    /// A leading UTF-8 BOM is tolerated. One invalid-UTF-8 line is skipped
    /// with a warning rather than poisoning the whole file. Lines that match
    /// no known category produce a warning — a suppression that can never
    /// fire must be visible.
    pub fn load(root: &Path) -> Self {
        let mut rules = Self::default();
        let ignore_path = root.join(".specsyncignore");
        let bytes = match fs::read(&ignore_path) {
            Ok(b) => b,
            Err(_) => return rules,
        };
        // A UTF-8 BOM is a non-semantic encoding marker; left in place it
        // silently disables the first pattern.
        let bytes: &[u8] = bytes
            .strip_prefix(b"\xef\xbb\xbf")
            .unwrap_or(bytes.as_slice());

        for (index, raw_line) in bytes.split(|b| *b == b'\n').enumerate() {
            let line_no = index + 1;
            let line = match std::str::from_utf8(raw_line) {
                Ok(l) => l.trim(),
                Err(_) => {
                    rules.warnings.push(format!(
                        ".specsyncignore line {line_no} is not valid UTF-8 — skipped"
                    ));
                    continue;
                }
            };
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Strip inline comments
            let line = line.split('#').next().unwrap_or(line).trim();
            if line.is_empty() {
                continue;
            }

            if let Some((left, right)) = line.split_once(':') {
                // Per-spec rule. Accept both `category:path` and the documented
                // `path:category` order — whichever side names a known
                // category is the category; the other side is the path.
                let (category, pattern) = match (
                    WarningCategory::from_str(left),
                    WarningCategory::from_str(right),
                ) {
                    (Some(category), _) => (category, right),
                    (None, Some(category)) => (category, left),
                    (None, None) => {
                        rules.warnings.push(format!(
                            ".specsyncignore line {line_no} has no known warning category: `{line}`"
                        ));
                        continue;
                    }
                };
                let pattern = pattern.trim().to_string();
                if pattern.is_empty() {
                    rules.warnings.push(format!(
                        ".specsyncignore line {line_no} has an empty spec path: `{line}`"
                    ));
                    continue;
                }
                rules.per_spec.entry(pattern).or_default().insert(category);
            } else if let Some(category) = WarningCategory::from_str(line) {
                // Global rule
                rules.global.insert(category);
            } else {
                rules.warnings.push(format!(
                    ".specsyncignore line {line_no} matches no warning category: `{line}`"
                ));
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
            if let Some(rest) = trimmed.strip_prefix("<!-- specsync-ignore:")
                && let Some(content) = rest.strip_suffix("-->")
            {
                for part in content.split(',') {
                    if let Some(cat) = WarningCategory::from_str(part.trim()) {
                        categories.insert(cat);
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
                "Missing companion requirements.md — run `specsync add-spec <name>` or `specsync generate` to scaffold one"
            ),
            Some(WarningCategory::RequirementsCompanion)
        );
        assert_eq!(
            WarningCategory::classify(
                "Inline requirements detected — specs are technical contracts; user stories and acceptance criteria belong in a companion requirements.md file"
            ),
            Some(WarningCategory::RequirementsCompanion)
        );
    }

    #[test]
    fn test_classify_stub_section() {
        assert_eq!(
            WarningCategory::classify("Section ## Purpose contains only unfinished draft text"),
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
            "Missing companion requirements.md — run `specsync add-spec <name>` or `specsync generate` to scaffold one",
            "specs/auth/auth.spec.md",
            &inline,
        ));
        assert!(!rules.is_suppressed(
            "Section ## Purpose contains only unfinished draft text",
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
            "Section ## Purpose contains only unfinished draft text",
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
    fn test_classify_exports_documented_summary() {
        assert_eq!(
            WarningCategory::classify("2/5 exports documented"),
            Some(WarningCategory::ExportsDocumented)
        );
        assert_eq!(
            WarningCategory::from_str("exports-documented"),
            Some(WarningCategory::ExportsDocumented)
        );
        // Not a summary — unrelated prose must not match.
        assert_ne!(
            WarningCategory::classify("Undocumented export 'foo' from src/a.ts"),
            Some(WarningCategory::ExportsDocumented)
        );
    }

    #[test]
    fn test_exports_documented_summary_suppressible() {
        let mut rules = IgnoreRules::default();
        rules.global.insert(WarningCategory::ExportsDocumented);
        let inline = HashSet::new();
        assert!(rules.is_suppressed(
            "2/5 exports documented",
            "specs/a/a.spec.md",
            &inline,
        ));
    }

    #[test]
    fn test_load_tolerates_utf8_bom() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".specsyncignore"),
            "\u{feff}undocumented-export\n",
        )
        .unwrap();
        let rules = IgnoreRules::load(tmp.path());
        assert!(
            rules.global.contains(&WarningCategory::UndocumentedExport),
            "a BOM must not disable the first pattern: {rules:?}"
        );
        assert!(rules.warnings.is_empty(), "{:?}", rules.warnings);
    }

    #[test]
    fn test_load_invalid_utf8_line_skipped_not_poisoning() {
        let tmp = tempfile::tempdir().unwrap();
        let mut bytes = b"undocumented-export\n".to_vec();
        bytes.extend_from_slice(b"bad-\xff\xfe-line\n");
        bytes.extend_from_slice(b"changelog\n");
        std::fs::write(tmp.path().join(".specsyncignore"), bytes).unwrap();

        let rules = IgnoreRules::load(tmp.path());
        // The valid lines still apply...
        assert!(rules.global.contains(&WarningCategory::UndocumentedExport));
        assert!(rules.global.contains(&WarningCategory::ChangelogEntries));
        // ...and the corrupt line is reported, not silent.
        assert_eq!(rules.warnings.len(), 1, "{:?}", rules.warnings);
        assert!(rules.warnings[0].contains("not valid UTF-8"));
        assert!(rules.warnings[0].contains("line 2"));
    }

    #[test]
    fn test_load_per_spec_path_first_order() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".specsyncignore"),
            "specs/legacy/:undocumented-export\n",
        )
        .unwrap();
        let rules = IgnoreRules::load(tmp.path());
        assert!(
            rules.per_spec
                .get("specs/legacy/")
                .is_some_and(|cats| cats.contains(&WarningCategory::UndocumentedExport)),
            "path:category order must work: {rules:?}"
        );
    }

    #[test]
    fn test_load_warns_on_unmatchable_pattern() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".specsyncignore"),
            "undocumanted-export\nmain:undocumented\n",
        )
        .unwrap();
        let rules = IgnoreRules::load(tmp.path());
        // Typo'd category warns with the offending line.
        assert!(
            rules
                .warnings
                .iter()
                .any(|w| w.contains("undocumanted-export") && w.contains("line 1")),
            "{:?}",
            rules.warnings
        );
        // `main:undocumented` — "undocumented" is a known category alias and
        // "main" is not, so this parses as per-spec pattern "main". Both sides
        // unresolvable would warn instead.
        assert!(
            rules.per_spec.contains_key("main")
                || rules.warnings.iter().any(|w| w.contains("line 2")),
            "{rules:?}"
        );
    }

    #[test]
    fn test_load_no_file() {
        let tmp = tempfile::tempdir().unwrap();
        let rules = IgnoreRules::load(tmp.path());
        assert!(rules.global.is_empty());
        assert!(rules.per_spec.is_empty());
    }
}
