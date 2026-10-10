//! JVM lexical extraction and package attribution regressions.

use super::*;
use crate::deps::test_support::{create_source, create_spec};
use crate::deps::{unresolved_imports_note, valid_declarations_line, validate_deps};
use tempfile::TempDir;

#[test]
fn test_extract_kotlin_imports_packages() {
    let imports = extract_kotlin_imports(
        "package com.example.feature\n\n\
             import com.example.core.Core\n\
             import com.example.util.*\n\
             import com.example.nested.Outer.Inner\n\
             import com.example.top.helper\n\
             import com.example.alias.Thing as Renamed\n\
             import Bare\n",
    );

    assert_eq!(
        imports,
        HashSet::from([
            "com.example.core".to_string(),
            "com.example.util".to_string(),
            "com.example.nested".to_string(),
            "com.example.top".to_string(),
            "com.example.alias".to_string(),
        ]),
        "a Kotlin import names a declaration, so the package is everything before it"
    );
}

#[test]
fn test_extract_kotlin_imports_ignores_comments_and_strings() {
    let imports = extract_kotlin_imports(
        "package com.example.feature\n\
             // import com.example.linecomment.Thing\n\
             /* import com.example.block.Thing\n\
                /* import com.example.nestedblock.Thing */\n\
             */\n\
             val doc = \"\"\"\nimport com.example.raw.Thing\n\"\"\"\n\
             val line = \"import com.example.quoted.Thing\"\n\
             import com.example.real.Thing\n",
    );

    assert_eq!(imports, HashSet::from(["com.example.real".to_string()]));
}

#[test]
fn test_undeclared_kotlin_import() {
    // #477: Kotlin imports were never collected, so `deps` called an
    // unexamined graph valid.
    let tmp = TempDir::new().unwrap();
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/core/Core.kt",
        "package com.example.core\n\nclass Core\n",
    );
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/feature/Feature.kt",
        "package com.example.feature\n\nimport com.example.core.Core\n\nclass Feature(val core: Core)\n",
    );
    create_spec(
        tmp.path(),
        "core",
        &[],
        &["src/main/kotlin/com/example/core/Core.kt"],
    );
    create_spec(
        tmp.path(),
        "feature",
        &[],
        &["src/main/kotlin/com/example/feature/Feature.kt"],
    );

    let report = validate_deps(tmp.path(), "specs");
    assert!(
        report
            .undeclared_imports
            .contains(&("feature".to_string(), "core".to_string())),
        "expected com.example.core to resolve to the spec owning it: {:?}",
        report.undeclared_imports
    );
    assert!(
        report.unanalyzed_languages.is_empty(),
        "Kotlin is analysable now: {:?}",
        report.unanalyzed_languages
    );
}

#[test]
fn test_declared_kotlin_import_not_flagged() {
    let tmp = TempDir::new().unwrap();
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/core/Core.kt",
        "package com.example.core\n\nclass Core\n",
    );
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/feature/Feature.kt",
        "package com.example.feature\n\nimport com.example.core.Core\n",
    );
    create_spec(
        tmp.path(),
        "core",
        &[],
        &["src/main/kotlin/com/example/core/Core.kt"],
    );
    create_spec(
        tmp.path(),
        "feature",
        &["specs/core/core.spec.md"],
        &["src/main/kotlin/com/example/feature/Feature.kt"],
    );

    let report = validate_deps(tmp.path(), "specs");
    assert!(
        report.undeclared_imports.is_empty(),
        "a declared dependency must not be flagged: {:?}",
        report.undeclared_imports
    );
}

#[test]
fn test_kotlin_import_of_subpackage_maps_to_owning_spec() {
    // `com.example.core.internal` has no spec of its own; the longest
    // package prefix that does own files wins.
    let tmp = TempDir::new().unwrap();
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/core/Core.kt",
        "package com.example.core\n\nclass Core\n",
    );
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/feature/Feature.kt",
        "package com.example.feature\n\nimport com.example.core.internal.Detail\n",
    );
    create_spec(
        tmp.path(),
        "core",
        &[],
        &["src/main/kotlin/com/example/core/Core.kt"],
    );
    create_spec(
        tmp.path(),
        "feature",
        &[],
        &["src/main/kotlin/com/example/feature/Feature.kt"],
    );

    let report = validate_deps(tmp.path(), "specs");
    assert!(
        report
            .undeclared_imports
            .contains(&("feature".to_string(), "core".to_string())),
        "expected the subpackage import to resolve to core: {:?}",
        report.undeclared_imports
    );
}

#[test]
fn test_kotlin_third_party_package_is_not_mistaken_for_a_spec_module() {
    // `java.util` must not resolve to the spec module named `util` just
    // because they share a last segment: the project's package topology is
    // known and `java.util` is not part of it.
    let tmp = TempDir::new().unwrap();
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/util/Util.kt",
        "package com.example.util\n\nclass Util\n",
    );
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/feature/Feature.kt",
        "package com.example.feature\n\nimport java.util.UUID\n",
    );
    create_spec(
        tmp.path(),
        "util",
        &[],
        &["src/main/kotlin/com/example/util/Util.kt"],
    );
    create_spec(
        tmp.path(),
        "feature",
        &[],
        &["src/main/kotlin/com/example/feature/Feature.kt"],
    );

    let report = validate_deps(tmp.path(), "specs");
    assert!(
        report.undeclared_imports.is_empty(),
        "java.util is not the spec module `util`: {:?}",
        report.undeclared_imports
    );
    // And it is NOT an attribution failure either: a third-party package
    // correctly maps to no spec module, so disclosing it would be noise.
    assert!(
        report.unresolved_imports.is_empty(),
        "java.util is foreign, not unattributed: {:?}",
        report.unresolved_imports
    );
}

#[test]
fn test_kotlin_flat_layout_resolves_by_declared_package() {
    // No file sits in a package directory, so the directory tells us nothing
    // about package ownership. The `package` statement each file declares
    // does, and that is what resolution uses.
    let tmp = TempDir::new().unwrap();
    create_source(
        tmp.path(),
        "Core.kt",
        "package com.example.core\n\nclass Core\n",
    );
    create_source(
        tmp.path(),
        "Feature.kt",
        "package com.example.feature\n\nimport com.example.core.Core\n",
    );
    create_spec(tmp.path(), "core", &[], &["Core.kt"]);
    create_spec(tmp.path(), "feature", &[], &["Feature.kt"]);

    let report = validate_deps(tmp.path(), "specs");
    assert!(
        report
            .undeclared_imports
            .contains(&("feature".to_string(), "core".to_string())),
        "expected the flat-layout import to still be attributed: {:?}",
        report.undeclared_imports
    );
    assert!(
        report.unresolved_imports.is_empty(),
        "nothing was left unattributed: {:?}",
        report.unresolved_imports
    );
}

#[test]
fn test_kotlin_import_resolves_when_directory_does_not_match_package() {
    // #477, one level down: the source root is `src/kt`, which does not end
    // with either file's package path. Matching a package against directory
    // suffixes finds no owner here — and a resolution that finds no owner
    // used to be dropped, leaving an empty report and exit 0.
    let tmp = TempDir::new().unwrap();
    create_source(
        tmp.path(),
        "src/kt/Core.kt",
        "package com.example.core\n\nclass Core\n",
    );
    create_source(
        tmp.path(),
        "src/kt/Feature.kt",
        "package com.example.feature\n\nimport com.example.core.Core\n",
    );
    create_spec(tmp.path(), "core", &[], &["src/kt/Core.kt"]);
    create_spec(tmp.path(), "feature", &[], &["src/kt/Feature.kt"]);

    let report = validate_deps(tmp.path(), "specs");
    assert!(
        report
            .undeclared_imports
            .contains(&("feature".to_string(), "core".to_string())),
        "the declared package must resolve regardless of directory: {:?}",
        report.undeclared_imports
    );
    assert!(
        report.unresolved_imports.is_empty(),
        "nothing was left unattributed: {:?}",
        report.unresolved_imports
    );
}

#[test]
fn test_kotlin_import_of_project_package_with_no_owner_is_disclosed() {
    // The import was collected and is plainly this project's own namespace,
    // but no spec owns `com.example.internal`. Silently dropping it is the
    // #477 defect: no edge, no finding, exit 0. It must be disclosed.
    let tmp = TempDir::new().unwrap();
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/feature/Feature.kt",
        "package com.example.feature\n\nimport com.example.internal.Detail\n",
    );
    create_spec(
        tmp.path(),
        "feature",
        &[],
        &["src/main/kotlin/com/example/feature/Feature.kt"],
    );

    let report = validate_deps(tmp.path(), "specs");
    assert_eq!(
        report.unresolved_imports,
        vec![("feature".to_string(), "com.example.internal".to_string())],
        "an import the tool could not map must be reported, not filtered away"
    );
    assert_eq!(
        valid_declarations_line(&report),
        "All dependency declarations are valid for the imports that resolved.",
        "the success sentence must not claim more than was checked"
    );
    let note = unresolved_imports_note(&report).expect("the gap must be disclosed");
    assert!(
        note.contains("feature imports com.example.internal"),
        "got: {note}"
    );
    // Disclosure is advisory: it is not an error and does not gate.
    assert!(report.errors.is_empty() && report.warnings.is_empty());
}

#[test]
fn test_ambiguous_kotlin_package_is_disclosed_not_guessed() {
    // Two specs own files in `com.example.core`, so no single module owns
    // the package. Guessing one would be wrong; dropping the import in
    // silence is the bug. Say it could not be mapped.
    let tmp = TempDir::new().unwrap();
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/core/One.kt",
        "package com.example.core\n\nclass One\n",
    );
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/core/Two.kt",
        "package com.example.core\n\nclass Two\n",
    );
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/feature/Feature.kt",
        "package com.example.feature\n\nimport com.example.core.One\n",
    );
    create_spec(
        tmp.path(),
        "core_one",
        &[],
        &["src/main/kotlin/com/example/core/One.kt"],
    );
    create_spec(
        tmp.path(),
        "core_two",
        &[],
        &["src/main/kotlin/com/example/core/Two.kt"],
    );
    create_spec(
        tmp.path(),
        "feature",
        &[],
        &["src/main/kotlin/com/example/feature/Feature.kt"],
    );

    let report = validate_deps(tmp.path(), "specs");
    assert_eq!(
        report.unresolved_imports,
        vec![("feature".to_string(), "com.example.core".to_string())],
        "an ambiguous package is unattributed, and unattributed is disclosed"
    );
}

#[test]
fn test_kotlin_import_resolves_to_spec_owning_java_sources() {
    // A Kotlin file importing a package whose spec owns `.java` sources still
    // produces the edge — JVM siblings share one package namespace.
    let tmp = TempDir::new().unwrap();
    create_source(
        tmp.path(),
        "src/main/java/com/example/core/Core.java",
        "package com.example.core;\n\npublic class Core {}\n",
    );
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/feature/Feature.kt",
        "package com.example.feature\n\nimport com.example.core.Core\n",
    );
    create_spec(
        tmp.path(),
        "core",
        &[],
        &["src/main/java/com/example/core/Core.java"],
    );
    create_spec(
        tmp.path(),
        "feature",
        &[],
        &["src/main/kotlin/com/example/feature/Feature.kt"],
    );

    let report = validate_deps(tmp.path(), "specs");
    assert!(
        report
            .undeclared_imports
            .contains(&("feature".to_string(), "core".to_string())),
        "expected the Java-owned package to be attributed: {:?}",
        report.undeclared_imports
    );
    // Java itself still has no import extractor, and that is disclosed.
    assert_eq!(report.unanalyzed_languages, vec![("Java".to_string(), 1)]);
}

#[test]
fn test_kotlin_self_package_import_not_flagged() {
    let tmp = TempDir::new().unwrap();
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/core/Core.kt",
        "package com.example.core\n\nclass Core\n",
    );
    create_source(
        tmp.path(),
        "src/main/kotlin/com/example/core/Helper.kt",
        "package com.example.core\n\nimport com.example.core.Core\n",
    );
    create_spec(
        tmp.path(),
        "core",
        &[],
        &[
            "src/main/kotlin/com/example/core/Core.kt",
            "src/main/kotlin/com/example/core/Helper.kt",
        ],
    );

    let report = validate_deps(tmp.path(), "specs");
    assert!(
        report.undeclared_imports.is_empty(),
        "a module must not depend on itself: {:?}",
        report.undeclared_imports
    );
}

#[test]
fn test_jvm_declared_package_reads_the_header() {
    assert_eq!(
        jvm_declared_package("package com.example.core\n\nclass Core\n"),
        Some("com.example.core".to_string())
    );
    assert_eq!(
        jvm_declared_package("package com.example.core;\n\npublic class Core {}\n"),
        Some("com.example.core".to_string()),
        "Java terminates the clause with a semicolon"
    );
    assert_eq!(
        jvm_declared_package("@file:JvmName(\"CoreKt\")\n\npackage com.example.core\n"),
        Some("com.example.core".to_string()),
        "Kotlin file annotations precede the package clause"
    );
    assert_eq!(
        jvm_declared_package("// package com.example.wrong\npackage com.example.core\n"),
        Some("com.example.core".to_string()),
        "a commented-out clause is not a declaration"
    );
    assert_eq!(
        jvm_declared_package("package com.example\npackage core\n\nclass Core\n"),
        Some("com.example.core".to_string()),
        "Scala chains clauses, and they nest"
    );
    assert_eq!(
        jvm_declared_package("class Core\n\npackage com.example.core\n"),
        None,
        "only the header declares the package"
    );
    assert_eq!(jvm_declared_package("class Core\n"), None);
}
