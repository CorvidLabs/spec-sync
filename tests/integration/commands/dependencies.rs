//! Dependency discovery, strict validation, and unreadable-input failures.

use crate::helpers::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ─── specsync deps: --strict gates on undeclared-import warnings ─────────

#[test]
fn deps_strict_gates_on_undeclared_imports() {
    // Regression (H6): `deps --strict` was a silent no-op — undeclared imports
    // were reported as warnings but never failed the exit code.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_undeclared_import_project(root);

    // Default deps is advisory (reports the warning, exits 0).
    specsync().current_dir(root).arg("deps").assert().success();
    // --strict fails on the undeclared import; non-JSON formats get the human
    // "treated as errors" note on stderr.
    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("treated as errors"));
    // JSON output gates AND stays fully machine-readable: stdout is parseable
    // JSON carrying the warning, and the human strict note is suppressed
    // entirely — not even on stderr — so a JSON consumer sees only structured
    // data plus the exit code (no ANSI, nothing to parse around).
    let output = specsync()
        .current_dir(root)
        .args(["deps", "--strict", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("deps --strict json stdout must be valid JSON");
    assert!(
        !parsed["undeclared_imports"].as_array().unwrap().is_empty(),
        "expected the undeclared import to be reported in JSON"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("treated as errors") && !stderr.contains("--strict mode"),
        "JSON mode must not emit the human strict note, even on stderr; got: {stderr:?}"
    );
}

#[test]
fn deps_strict_passes_when_dependency_is_declared() {
    // No false failure: once `api` declares `depends_on: [db]`, --strict is clean.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_undeclared_import_project(root);
    fs::write(
        root.join("specs/api/api.spec.md"),
        "---\nmodule: api\nversion: 1\nstatus: active\nfiles:\n  - src/api/api.ts\ndepends_on:\n  - db\n---\n# api\n## Purpose\np\n",
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .success();
}

#[test]
fn deps_strict_gates_on_undeclared_kotlin_import() {
    // Regression (#477): `deps` had no Kotlin import extractor at all, so a
    // Kotlin tree produced zero edges and `--strict` called the unexamined
    // graph valid — the same shape that already failed in Python.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/main/kotlin/com/example/core")).unwrap();
    fs::create_dir_all(root.join("src/main/kotlin/com/example/feature")).unwrap();
    fs::write(
        root.join("src/main/kotlin/com/example/core/Core.kt"),
        "package com.example.core\n\nclass Core\n",
    )
    .unwrap();
    fs::write(
        root.join("src/main/kotlin/com/example/feature/Feature.kt"),
        "package com.example.feature\n\nimport com.example.core.Core\n\nclass Feature(val core: Core)\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/core")).unwrap();
    fs::create_dir_all(root.join("specs/feature")).unwrap();
    fs::write(
        root.join("specs/core/core.spec.md"),
        valid_spec("core", &["src/main/kotlin/com/example/core/Core.kt"]),
    )
    .unwrap();
    fs::write(
        root.join("specs/feature/feature.spec.md"),
        valid_spec(
            "feature",
            &["src/main/kotlin/com/example/feature/Feature.kt"],
        ),
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "source imports 'core' but it is not in depends_on",
        ));

    // Declaring the dependency clears it — the analysis honours depends_on.
    fs::write(
        root.join("specs/feature/feature.spec.md"),
        valid_spec(
            "feature",
            &["src/main/kotlin/com/example/feature/Feature.kt"],
        )
        .replace("depends_on: []", "depends_on:\n  - core"),
    )
    .unwrap();
    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .success();
}

#[test]
fn deps_discloses_languages_it_cannot_analyse() {
    // A language with no import extractor contributes no edges. `deps` must say
    // so rather than let the silence read as "no undeclared imports" (#477).
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/core.go"), "package core\n").unwrap();
    fs::create_dir_all(root.join("specs/core")).unwrap();
    fs::write(
        root.join("specs/core/core.spec.md"),
        valid_spec("core", &["src/core.go"]),
    )
    .unwrap();

    // Disclosed, but advisory: an unanalysable language is not a failure.
    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Import analysis is not implemented for Go (1 file(s))").and(
                predicate::str::contains(
                    "All dependency declarations are valid for the languages analysed.",
                ),
            ),
        );

    let output = specsync()
        .current_dir(root)
        .args(["deps", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .clone();
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("deps json stdout must be valid JSON");
    assert_eq!(
        parsed["unanalyzed_languages"],
        serde_json::json!([{"language": "Go", "files": 1}]),
        "JSON consumers must be able to tell 'not analysed' from 'clean'"
    );
}

#[test]
fn deps_does_not_disclose_languages_that_have_no_imports() {
    // The disclosure must name languages whose imports went unread — not every
    // file whose extension happens to map to a Language. A YAML file has no
    // imports to miss and a shell script names a path, so a project like this
    // one is fully analysed and must say so without hedging (#477).
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/core.rs"), "pub fn run() {}\n").unwrap();
    fs::write(root.join("src/ci.yml"), "jobs: {}\n").unwrap();
    fs::write(root.join("src/tool.sh"), "#!/usr/bin/env bash\ntrue\n").unwrap();
    fs::create_dir_all(root.join("specs/core")).unwrap();
    fs::write(
        root.join("specs/core/core.spec.md"),
        valid_spec("core", &["src/core.rs", "src/ci.yml", "src/tool.sh"]),
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("All dependency declarations are valid.")
                .and(predicate::str::contains("Import analysis is not implemented").not()),
        );
}

#[test]
fn deps_reports_kotlin_imports_it_could_not_resolve() {
    // The fix for #477 collected Kotlin imports and then dropped every one it
    // could not map to a module, reporting the remainder as a clean graph. An
    // import the tool could not resolve must be distinguishable in the output
    // from an import that resolves to nothing.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/kt")).unwrap();
    fs::write(
        root.join("src/kt/Feature.kt"),
        "package com.example.feature\n\nimport com.example.internal.Detail\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/feature")).unwrap();
    fs::write(
        root.join("specs/feature/feature.spec.md"),
        valid_spec("feature", &["src/kt/Feature.kt"]),
    )
    .unwrap();

    // Advisory, like the unanalysed-language note: disclosed, not gated.
    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "1 import(s) could not be mapped to a spec module, so they were not checked \
                 against depends_on: feature imports com.example.internal",
            )
            .and(predicate::str::contains(
                "All dependency declarations are valid for the imports that resolved.",
            )),
        );

    let output = specsync()
        .current_dir(root)
        .args(["deps", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .clone();
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("deps json stdout must be valid JSON");
    assert_eq!(
        parsed["unresolved_imports"],
        serde_json::json!([{"module": "feature", "import": "com.example.internal"}]),
        "JSON consumers must be able to tell 'could not resolve' from 'nothing to resolve'"
    );
    assert_eq!(
        parsed["undeclared_imports"],
        serde_json::json!([]),
        "an unresolved import is not evidence of an undeclared dependency"
    );
}

#[test]
fn deps_fails_loud_on_unreadable_source_file() {
    // Regression: a declared source file that can't be read as UTF-8 silently
    // contributed no imports, so `deps` could pass while hiding undeclared imports.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    let mut bad = b"export function apiFn() {}\n".to_vec();
    bad.push(0xFF);
    fs::write(root.join("src/a.ts"), bad).unwrap();
    fs::create_dir_all(root.join("specs/m")).unwrap();
    fs::write(
        root.join("specs/m/m.spec.md"),
        valid_spec("m", &["src/a.ts"]),
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .arg("deps")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "could not be read as UTF-8 for dependency analysis",
        ));
}

#[test]
fn deps_fails_loud_on_unreadable_spec_file() {
    // Regression: a spec file that can't be read as UTF-8 was silently dropped from
    // the dependency graph, defeating cycle / missing-dep detection for that module.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "pub fn f() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/m")).unwrap();
    let mut bad = b"---\nmodule: m\nfiles:\n  - src/a.rs\n---\n# m\n".to_vec();
    bad.push(0xFF);
    fs::write(root.join("specs/m/m.spec.md"), bad).unwrap();

    specsync()
        .current_dir(root)
        .arg("deps")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "spec file could not be read as UTF-8",
        ));
}

#[test]
fn deps_strict_mermaid_still_gates() {
    // Regression: `deps --mermaid`/`--dot` early-returned before the strict gate, so
    // `deps --strict --mermaid` silently exited 0 on the same undeclared import that
    // `deps --strict` fails on. The diagram must still print, but the gate must apply.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_undeclared_import_project(root);

    // Diagram is emitted on stdout AND the strict gate fails.
    let output = specsync()
        .current_dir(root)
        .args(["deps", "--strict", "--mermaid"])
        .assert()
        .failure()
        .get_output()
        .clone();
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("graph LR"),
        "the mermaid diagram must still be printed to stdout"
    );
    // Without --strict, a render is advisory (exit 0).
    specsync()
        .current_dir(root)
        .args(["deps", "--mermaid"])
        .assert()
        .success();
}
