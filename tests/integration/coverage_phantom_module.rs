//! #529 — a module name nothing owns is not a module without a spec.
//!
//! Coverage answered "does this module have a spec?" by looking for a spec
//! DIRECTORY of the same name. A repository that maps its files by language
//! (`strutil_py`, `strutil_js`, …) never creates `specs/strutil/`, so the
//! missing NAME was read as a missing SPEC and the report named an uncovered
//! module `strutil/` beside `File coverage: 5/5 (100%)` — the campaign's defect
//! class with the sign flipped: a value invented where there was no input.
//!
//! Both derivations are asserted here. The reported issue names the flat-file
//! stem; the subdirectory derivation is a second implementation of the same
//! claim and fails identically. Every case is paired with a control that must
//! still report the module, so suppressing the feature outright cannot pass.

use crate::helpers::*;
use std::fs;
use tempfile::TempDir;

fn run(root: &std::path::Path, args: &[&str]) -> (String, i32) {
    let output = specsync()
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        output.status.code().unwrap_or(-1),
    )
}

fn modules(json: &str) -> Vec<String> {
    let value: serde_json::Value = serde_json::from_str(json).expect("coverage json");
    value["modules"]
        .as_array()
        .expect("modules array")
        .iter()
        .map(|module| module["name"].as_str().unwrap_or_default().to_string())
        .collect()
}

fn uncovered_files(json: &str) -> Vec<String> {
    let value: serde_json::Value = serde_json::from_str(json).expect("coverage json");
    value["uncovered_files"]
        .as_array()
        .expect("uncovered_files array")
        .iter()
        .map(|file| file["file"].as_str().unwrap_or_default().to_string())
        .collect()
}

/// `src/strutil.{py,mjs,rb}` mapped by three language-specific specs.
fn language_specific_flat_project(root: &std::path::Path, map_javascript: bool) {
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/strutil.py"), "def upper(s):\n    return s\n").unwrap();
    fs::write(
        root.join("src/strutil.mjs"),
        "export function upper(s) {\n  return s;\n}\n",
    )
    .unwrap();
    fs::write(root.join("src/strutil.rb"), "def upper(s)\n  s\nend\n").unwrap();
    for (module, file) in [
        ("strutil_py", "src/strutil.py"),
        ("strutil_rb", "src/strutil.rb"),
    ] {
        fs::create_dir_all(root.join("specs").join(module)).unwrap();
        fs::write(
            root.join("specs")
                .join(module)
                .join(format!("{module}.spec.md")),
            valid_spec(module, &[file]),
        )
        .unwrap();
    }
    if map_javascript {
        fs::create_dir_all(root.join("specs/strutil_js")).unwrap();
        fs::write(
            root.join("specs/strutil_js/strutil_js.spec.md"),
            valid_spec("strutil_js", &["src/strutil.mjs"]),
        )
        .unwrap();
    }
}

/// `src/textkit/case.{py,mjs}` mapped by two language-specific specs.
fn language_specific_directory_project(root: &std::path::Path, map_javascript: bool) {
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/textkit")).unwrap();
    fs::write(
        root.join("src/textkit/case.py"),
        "def upper(s):\n    return s\n",
    )
    .unwrap();
    fs::write(
        root.join("src/textkit/case.mjs"),
        "export function upper(s) {\n  return s;\n}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/textkit_py")).unwrap();
    fs::write(
        root.join("specs/textkit_py/textkit_py.spec.md"),
        valid_spec("textkit_py", &["src/textkit/case.py"]),
    )
    .unwrap();
    if map_javascript {
        fs::create_dir_all(root.join("specs/textkit_js")).unwrap();
        fs::write(
            root.join("specs/textkit_js/textkit_js.spec.md"),
            valid_spec("textkit_js", &["src/textkit/case.mjs"]),
        )
        .unwrap();
    }
}

#[test]
fn a_fully_mapped_stem_is_not_reported_as_a_module_without_a_spec() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    language_specific_flat_project(root, true);

    let (json, rc) = run(root, &["coverage", "--format", "json"]);
    assert_eq!(rc, 0, "{json}");
    assert!(
        uncovered_files(&json).is_empty(),
        "fixture must be fully mapped, got {:?}",
        uncovered_files(&json)
    );
    assert!(
        !modules(&json).contains(&"strutil".to_string()),
        "coverage invented module `strutil` over files that are all mapped: {:?}",
        modules(&json)
    );

    let (text, rc) = run(root, &["coverage"]);
    assert_eq!(rc, 0, "{text}");
    assert!(
        !text.contains("strutil/"),
        "text report still names the phantom parent module:\n{text}"
    );
    assert!(
        text.contains("All source modules have spec directories"),
        "text report should be internally consistent with 3/3 files covered:\n{text}"
    );
}

#[test]
fn an_unmapped_sibling_keeps_the_stem_reported() {
    // VACUITY CONTROL. `src/strutil.mjs` is mapped by nothing, so `strutil` is
    // a real gap. A fix that simply stopped emitting parent modules would pass
    // the test above and fail this one.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    language_specific_flat_project(root, false);

    let (json, rc) = run(root, &["coverage", "--format", "json"]);
    assert_eq!(rc, 0, "{json}");
    assert_eq!(uncovered_files(&json), ["src/strutil.mjs"]);
    assert!(
        modules(&json).contains(&"strutil".to_string()),
        "a stem with an unmapped file must stay reported: {:?}",
        modules(&json)
    );

    let (text, rc) = run(root, &["coverage"]);
    assert_eq!(rc, 0, "{text}");
    assert!(
        text.contains("strutil/"),
        "text report must still name the real gap:\n{text}"
    );
}

#[test]
fn a_fully_mapped_directory_is_not_reported_as_a_module_without_a_spec() {
    // The sibling derivation, one directory up from the reported site.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    language_specific_directory_project(root, true);

    let (json, rc) = run(root, &["coverage", "--format", "json"]);
    assert_eq!(rc, 0, "{json}");
    assert!(uncovered_files(&json).is_empty(), "{json}");
    assert!(
        !modules(&json).contains(&"textkit".to_string()),
        "coverage invented module `textkit` over files that are all mapped: {:?}",
        modules(&json)
    );
}

#[test]
fn an_unmapped_file_keeps_the_directory_reported() {
    // VACUITY CONTROL for the directory derivation.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    language_specific_directory_project(root, false);

    let (json, rc) = run(root, &["coverage", "--format", "json"]);
    assert_eq!(rc, 0, "{json}");
    assert_eq!(uncovered_files(&json), ["src/textkit/case.mjs"]);
    assert!(
        modules(&json).contains(&"textkit".to_string()),
        "a directory with an unmapped file must stay reported: {:?}",
        modules(&json)
    );
}

#[test]
fn a_directory_holding_nothing_measurable_stays_reported() {
    // VACUITY CONTROL for the defect class itself: owning no discovered source
    // file is the absence of INPUT, not evidence of coverage.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    language_specific_flat_project(root, true);
    fs::create_dir_all(root.join("src/assets")).unwrap();
    fs::write(root.join("src/assets/logo.svg"), "<svg/>\n").unwrap();

    let (json, rc) = run(root, &["coverage", "--format", "json"]);
    assert_eq!(rc, 0, "{json}");
    assert!(
        modules(&json).contains(&"assets".to_string()),
        "a directory with nothing measured must stay reported: {:?}",
        modules(&json)
    );
}
