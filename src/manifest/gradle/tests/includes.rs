use super::super::parse_gradle_settings;
use crate::manifest::{
    GradleSettingsModule, discover_from_manifests, discover_from_manifests_checked,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn gradle_manifest_discovery_rejects_dynamic_include_without_partial_modules() {
    let tmp = tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
    fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        tmp.path().join("settings.gradle.kts"),
        r#"include(":safe", dynamicModule)"#,
    )
    .unwrap();

    let error = discover_from_manifests_checked(tmp.path()).unwrap_err();
    assert!(error.contains("Unsupported or dynamic Gradle expression"));
    let compatibility_result = discover_from_manifests(tmp.path());
    assert!(compatibility_result.modules.is_empty());
    assert!(compatibility_result.source_dirs.is_empty());
}

#[test]
fn gradle_settings_reject_unsupported_include_prefixed_workspace_mutators() {
    // `includeBuild` is no longer in this list: it is judged by its
    // argument, and its own tests below cover both verdicts. `includeFlat`
    // resolves against the PARENT of the root, so its argument is outside
    // the project by construction, and `includeWorkspace` is not a form
    // this parser models — neither becomes supportable by reading its path.
    for settings in [
        r#"includeFlat("../outside")"#,
        r#"includeWorkspace("../outside")"#,
        r#"settings.includeFlat "../outside""#,
    ] {
        let error = parse_gradle_settings(settings).unwrap_err();
        assert!(
            error.contains("Unsupported Gradle workspace mutator"),
            "unexpected include-prefixed mutator error: {error}"
        );
    }

    let modules = parse_gradle_settings(
        r#"
val includeFlat = "documentation"
println(includeFlat)
include(":member")
"#,
    )
    .unwrap();
    assert_eq!(
        modules,
        vec![GradleSettingsModule {
            name: "member".to_string(),
            path: "member".to_string(),
        }]
    );
}

/// THE REGRESSION (#723). An in-repo composite build is ordinary Gradle,
/// and it was refused on the token prefix alone — the argument was never
/// read. Every fixture the old guard was written against used
/// `"../outside"`, so no test could fail for this reason: the guard was
/// built for escapes and caught every composite build, and the two cases
/// were indistinguishable. This is the case that had no coverage at all.
///
/// The included build is accepted and then ignored: it names a separate
/// build, so the root build's `include(...)` list is what modules still
/// come from, and `:app` must survive beside it untouched.
#[test]
fn gradle_settings_accept_an_in_repo_include_build() {
    for settings in [
        "includeBuild(\"vendor/podo-shared\")\ninclude(\":app\")\n",
        "includeBuild 'vendor/podo-shared'\ninclude(':app')\n",
        "settings.includeBuild(\"vendor/podo-shared\")\ninclude(\":app\")\n",
        "includeBuild(\"./vendor/../vendor/podo-shared\")\ninclude(\":app\")\n",
        "includeBuild(\n    \"vendor/podo-shared\"\n)\ninclude(\":app\")\n",
        // Conditional forms, on one line and across three. They must agree:
        // a composite build contributes no module whether or not its branch
        // runs, so its position cannot change what is discovered. These two
        // disagreed at first — only the one-line form put the closing brace
        // on the declaration's own line.
        "if (loadShared) { includeBuild(\"vendor/podo-shared\") }\ninclude(\":app\")\n",
        "if (loadShared) {\n    includeBuild(\"vendor/podo-shared\")\n}\ninclude(\":app\")\n",
        "if (loadShared) { includeBuild 'vendor/podo-shared' }\ninclude(':app')\n",
    ] {
        let modules = parse_gradle_settings(settings)
            .unwrap_or_else(|error| panic!("in-repo includeBuild refused: {error}"));
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "app".to_string(),
                path: "app".to_string(),
            }],
            "unexpected modules for {settings:?}"
        );
    }
}

/// THE REGRESSION (#725). `includeBuild(path) { dependencySubstitution { … } }`
/// is the NORMAL way to write a composite build — it is the whole reason to
/// include one — and #723 left it refused while accepting the bare
/// `includeBuild(path)` minority form. The parser was therefore accepting
/// the uncommon spelling and refusing the common one.
///
/// The block carries substitution rules, not project declarations, so it
/// contributes no module and no source directory. Once the path argument has
/// been read and confined, a balanced block can be skipped whole, and `:app`
/// must still survive beside it untouched — the same claim the bare form
/// makes.
///
/// The two brace cases are the reason the skip needs a real scan rather than
/// a search for `}`: a brace inside a string literal must not move the
/// depth, and a brace inside a comment must already be gone.
#[test]
fn gradle_settings_accept_an_include_build_with_a_configuration_block() {
    for settings in [
        // The reported form, verbatim (#725).
        "includeBuild(\"vendor/podo-shared\") {\n    dependencySubstitution {\n        substitute(module(\"com.example:podo\")).using(project(\":\"))\n    }\n}\n\ninclude(\":app\")\n",
        // The same declaration on one line.
        "includeBuild(\"vendor/podo-shared\") { dependencySubstitution { substitute(module(\"com.example:podo\")).using(project(\":\")) } }\ninclude(\":app\")\n",
        "includeBuild('vendor/podo-shared') { dependencySubstitution { } }\ninclude(':app')\n",
        "settings.includeBuild(\"vendor/podo-shared\") { dependencySubstitution {} }\ninclude(\":app\")\n",
        "includeBuild(\"vendor/podo-shared\") { dependencySubstitution {} };\ninclude(\":app\")\n",
        "includeBuild(\"./vendor/../vendor/podo-shared\") { dependencySubstitution {} }\ninclude(\":app\")\n",
        // A brace inside a STRING is not a brace. Counted naively this block
        // never closes and the file is refused.
        "includeBuild(\"vendor/podo-shared\") {\n    dependencySubstitution {\n        substitute(module(\"com.example:podo{\")).using(project(\":\"))\n    }\n}\ninclude(\":app\")\n",
        // A brace inside a COMMENT is not a brace either. Counted naively
        // this block closes early and the rest of the comment is trailing
        // junk.
        "includeBuild(\"vendor/podo-shared\") {\n    // the closing brace } belongs to prose\n    dependencySubstitution {}\n}\ninclude(\":app\")\n",
        "includeBuild(\"vendor/podo-shared\") {\n    /* } */ dependencySubstitution {}\n}\ninclude(\":app\")\n",
        // Position is not judged for the block form either.
        "if (loadShared) { includeBuild(\"vendor/podo-shared\") { dependencySubstitution {} } }\ninclude(\":app\")\n",
        "if (loadShared) {\n    includeBuild(\"vendor/podo-shared\") {\n        dependencySubstitution {}\n    }\n}\ninclude(\":app\")\n",
    ] {
        let modules = parse_gradle_settings(settings).unwrap_or_else(|error| {
            panic!(
                "includeBuild with a configuration block refused: {error}; settings={settings:?}"
            )
        });
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "app".to_string(),
                path: "app".to_string(),
            }],
            "unexpected modules for {settings:?}"
        );
    }
}

/// Skipping the block must not hide what is written INSIDE it. The block is
/// stepped over for the purpose of finding where the declaration ends; the
/// text stays in the file, and every other guard still walks it.
#[test]
fn gradle_settings_still_judge_directives_inside_an_include_build_block() {
    for (settings, expected) in [
        (
            "includeBuild(\"vendor/shared\") {\n    include(\":smuggled\")\n}\n",
            "Unsupported block-scoped Gradle include",
        ),
        (
            "includeBuild(\"vendor/shared\") {\n    includeFlat(\"../outside\")\n}\n",
            "Unsupported Gradle workspace mutator",
        ),
        (
            "include(\":member\")\nincludeBuild(\"vendor/shared\") {\n    project(\":member\").projectDir = file(\"../outside\")\n}\n",
            "Unsupported block-scoped Gradle projectDir mutation",
        ),
        (
            "include(\":member\")\nincludeBuild(\"vendor/shared\") { project(\":member\").setProperty(\"projectDir\", outside) }\n",
            "Unsupported dynamic Gradle project mutation",
        ),
    ] {
        let error = match parse_gradle_settings(settings) {
            Ok(modules) => panic!("{settings} was accepted as {modules:?}"),
            Err(error) => error,
        };
        assert!(
            error.contains(expected),
            "unexpected error for a directive inside an includeBuild block: {error}; settings={settings:?}"
        );
    }
}

/// Honest label: the refusals asserted first are the CONTROL, and they
/// pass on the unfixed parser too — that is the point. Reading the argument
/// must not turn `includeBuild` into an accept-anything token, so what
/// breaks this loop is a relaxation that stopped confining the path or
/// stopped requiring one complete literal: a `../` escape leaves the
/// repository, and an interpolated, dynamic, multi-argument, or
/// trailing-expression form names something this parser cannot resolve from
/// the settings text at all.
///
/// The diagnostics asserted second are what is genuinely new. The old
/// refusal said `Unsupported Gradle workspace mutator includeBuild` for
/// every one of these — it was reached without ever reading the argument,
/// so it could not say which argument was wrong, and it said exactly the
/// same thing about the valid in-repo build above. Naming the path is how a
/// reader can tell an escape from a form the parser does not model.
///
/// The block-carrying fixtures added for #725 sit under the same honest
/// label. Every one of them is refused on the unfixed parser too — but only
/// because the block itself was refused, which said nothing about the
/// argument and would have said the same thing about a perfectly safe path.
/// Now the block is skipped, so the refusal has to come from the argument or
/// not at all: these assert that skipping it did not make the block a way to
/// carry an escape, an interpolation, an extra argument, or an unknowable
/// extent past the checks in front of it.
#[test]
fn gradle_settings_still_refuse_an_escaping_or_dynamic_include_build() {
    for (settings, expected) in [
        (
            r#"includeBuild("../outside")"#,
            "must remain beneath the project root",
        ),
        (
            r#"includeBuild("/etc/outside")"#,
            "must remain beneath the project root",
        ),
        (
            r#"includeBuild("vendor/../../outside")"#,
            "must remain beneath the project root",
        ),
        (
            r#"settings.includeBuild "../outside""#,
            "must remain beneath the project root",
        ),
        (
            r#"includeBuild("${vendorRoot}/shared")"#,
            "Unsupported or dynamic Gradle expression",
        ),
        (
            r#"includeBuild(vendorRoot)"#,
            "Unsupported or dynamic Gradle expression",
        ),
        (
            r#"includeBuild("vendor/a", "vendor/b")"#,
            "must name one literal path",
        ),
        (
            r#"includeBuild("vendor/shared") = shared"#,
            "Unsupported trailing Gradle includeBuild declaration expression",
        ),
        // The block is skipped, never the ARGUMENT check in front of it
        // (#725). Each of these is the accepted form of the test above with
        // a configuration block appended, and each must reach the same
        // verdict it reaches without one.
        (
            "includeBuild(\"../outside\") { dependencySubstitution {} }",
            "must remain beneath the project root",
        ),
        (
            "includeBuild(\"../outside\") {\n    dependencySubstitution {\n        substitute(module(\"com.example:podo\")).using(project(\":\"))\n    }\n}\n",
            "must remain beneath the project root",
        ),
        (
            r#"includeBuild("${vendorRoot}/shared") { dependencySubstitution {} }"#,
            "Unsupported or dynamic Gradle expression",
        ),
        (
            r#"includeBuild("vendor/a", "vendor/b") { dependencySubstitution {} }"#,
            "must name one literal path",
        ),
        // An unbalanced block cannot be skipped, because its extent is
        // exactly what is unknown.
        (
            "includeBuild(\"vendor/shared\") {\n    dependencySubstitution {\n",
            "configuration block has unbalanced braces",
        ),
        // A brace inside a string literal does not close the block. Without
        // a quote-aware scan the `"}"` below would balance it and this would
        // be accepted with the real closing brace left over.
        (
            r#"includeBuild("vendor/shared") { dependencySubstitution { substitute("}") } "#,
            "configuration block has unbalanced braces",
        ),
        (r#"includeBuild(".")"#, "must identify a project path"),
        // Position is not judged, but the ARGUMENT still is — wherever the
        // declaration sits.
        (
            r#"if (loadShared) { includeBuild("../outside") }"#,
            "must remain beneath the project root",
        ),
    ] {
        // The control: refused at all. True before this change and after.
        let error = match parse_gradle_settings(settings) {
            Ok(modules) => panic!("{settings} was accepted as {modules:?}"),
            Err(error) => error,
        };
        // What is new: the refusal names the argument, not the token.
        assert!(
            error.contains(expected),
            "unexpected includeBuild error for {settings}: {error}"
        );
    }
}
