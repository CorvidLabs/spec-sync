//! Shared filesystem fixtures for dependency-analysis regressions.

use std::fs;
use std::path::Path;

/// Helper: create a spec file in the temp dir.
pub(super) fn create_spec(tmp: &Path, module: &str, depends_on: &[&str], files: &[&str]) {
    let spec_dir = tmp.join("specs").join(module);
    fs::create_dir_all(&spec_dir).unwrap();

    let deps_yaml = if depends_on.is_empty() {
        "depends_on: []".to_string()
    } else {
        let items: String = depends_on.iter().map(|d| format!("  - {d}\n")).collect();
        format!("depends_on:\n{items}")
    };

    let files_yaml = if files.is_empty() {
        "files: []".to_string()
    } else {
        let items: String = files.iter().map(|f| format!("  - {f}\n")).collect();
        format!("files:\n{items}")
    };

    let content = format!(
        "---\nmodule: {module}\nversion: 1\nstatus: active\n{files_yaml}\ndb_tables: []\n{deps_yaml}\n---\n\n# {module}\n\n## Purpose\nTest\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n"
    );

    fs::write(spec_dir.join(format!("{module}.spec.md")), content).unwrap();
}

pub(super) fn create_source(tmp: &Path, path: &str, content: &str) {
    let full = tmp.join(path);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full, content).unwrap();
}
