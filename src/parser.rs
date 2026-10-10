//! Spec metadata, issue-reference validation, API symbols, and section analysis.

mod api_symbols;
mod checked_issues;
mod frontmatter;
mod sections;

use crate::types::Frontmatter;

pub(crate) use api_symbols::{blank_fenced_code, get_duplicate_spec_symbols};
pub use api_symbols::{get_all_api_table_symbols, get_spec_symbols, is_export_header};
pub use checked_issues::parse_checked_issue_references;
pub use frontmatter::{parse_frontmatter, strip_frontmatter};
pub(crate) use sections::is_boilerplate_line;
pub use sections::{
    body_has_section, find_section_offset, find_stub_sections, get_missing_sections,
    get_near_miss_sections, section_has_content,
};

/// Parsed spec file: frontmatter + markdown body.
pub struct ParsedSpec {
    pub frontmatter: Frontmatter,
    pub body: String,
    /// Hard frontmatter problems (duplicate keys, wrong shapes, unclosed
    /// brackets/quotes). The offending text is included in each message.
    pub errors: Vec<String>,
    /// Soft frontmatter problems (non-numeric versions, ignored garbage lines).
    pub warnings: Vec<String>,
}
