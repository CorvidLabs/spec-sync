use super::super::parse_gradle_settings;
use crate::manifest::GradleSettingsModule;

#[test]
fn gradle_settings_support_groovy_kotlin_multiline_and_project_dir_overrides() {
    let modules = parse_gradle_settings(
        r#"
include ':groovy:member',
        ':second'
include(
    ":kotlin:member",
    ':vendor:member'
);
project(':vendor:member').projectDir = file('vendor/custom-member');
project(":second").projectDir = new File(rootDir, "modules/second")
"#,
    )
    .unwrap();

    assert_eq!(
        modules,
        vec![
            GradleSettingsModule {
                name: "groovy/member".to_string(),
                path: "groovy/member".to_string(),
            },
            GradleSettingsModule {
                name: "kotlin/member".to_string(),
                path: "kotlin/member".to_string(),
            },
            GradleSettingsModule {
                name: "second".to_string(),
                path: "modules/second".to_string(),
            },
            GradleSettingsModule {
                name: "vendor/member".to_string(),
                path: "vendor/custom-member".to_string(),
            },
        ]
    );
}

#[test]
fn gradle_settings_reject_unsupported_project_dir_bases_and_suffixes() {
    let outside_base = parse_gradle_settings(
        r#"
include(":outside")
project(":outside").projectDir = new File(rootDir.parentFile, "outside")
"#,
    )
    .unwrap_err();
    assert!(outside_base.contains("base must be exactly rootDir"));

    let project_root =
        parse_gradle_settings("include(\":root\")\nproject(\":root\").projectDir = file(\".\")\n")
            .unwrap();
    assert_eq!(project_root[0].path, ".");

    for assignment in [
        r#"file("modules/safe").parentFile"#,
        r#"new File(rootDir, "modules/safe") + "/outside""#,
    ] {
        let settings =
            format!("include(\":safe\")\nproject(\":safe\").projectDir = {assignment}\n");
        let error = parse_gradle_settings(&settings).unwrap_err();
        assert!(error.contains("Unsupported trailing Gradle projectDir assignment expression"));
    }
}

#[test]
fn gradle_settings_reject_project_root_escapes() {
    for settings in [
        r#"
include(":outside")
project(":outside").projectDir = file("../outside")
"#,
        r#"
include(":outside")
project(":outside").setProjectDir(file("../outside"))
"#,
        r#"include(":..:outside")"#,
        r#"include("C:/outside")"#,
        r#"include("C:outside")"#,
        r#"include(":C:\\outside")"#,
        r#"
include(":outside")
project(":outside").projectDir = file("safe/../../outside")
"#,
        r#"
include(":outside")
project(":outside").projectDir = file("C:\\outside")
"#,
        r#"
include(":outside")
project(":outside").projectDir = file("\\\\server\\share")
"#,
        r#"
include(":safe")
project("C:/outside").projectDir = file("modules/safe")
"#,
        r#"
include(":safe")
project("C:outside").projectDir = file("modules/safe")
"#,
        r#"
include(":outside")
project(":outside").setProjectDir(file("\u002e\u002e/outside"))
"#,
        r#"
include(":outside")
project(":outside").setProjectDir(file("\056\056/outside"))
"#,
    ] {
        let error = parse_gradle_settings(settings).unwrap_err();
        assert!(
            error.contains("must remain beneath the project root"),
            "unexpected Gradle confinement error: {error}"
        );
    }
}

#[test]
fn gradle_settings_preserve_rooted_nested_names_that_resemble_drive_relative_paths() {
    let modules = parse_gradle_settings(
        r#"
include(":C:member")
project(":C:member").projectDir = file("modules/member")
"#,
    )
    .unwrap();
    assert_eq!(
        modules,
        vec![GradleSettingsModule {
            name: "C/member".to_string(),
            path: "modules/member".to_string(),
        }]
    );
}

#[test]
fn gradle_settings_support_literal_set_project_dir_forms() {
    let modules = parse_gradle_settings(
        r#"
include(":first", ":second")
project(":first").setProjectDir(file("modules/first"))
project(":second").setProjectDir(new File(rootDir, "modules/second"))
"#,
    )
    .unwrap();

    assert_eq!(
        modules,
        vec![
            GradleSettingsModule {
                name: "first".to_string(),
                path: "modules/first".to_string(),
            },
            GradleSettingsModule {
                name: "second".to_string(),
                path: "modules/second".to_string(),
            },
        ]
    );
}

#[test]
fn gradle_settings_reject_dynamic_or_ambiguous_set_project_dir_forms() {
    for setter in [
        r#"project(":member").setProjectDir(projectDirProvider.get())"#,
        r#"project(":member").setProjectDir(file("modules/member"), file("other"))"#,
        r#"project(":member").setProjectDir(file("modules/member")).parentFile"#,
        r#"project(":member").setProjectDir(file("$outside"))"#,
        r#"project(":member").setProjectDir(file("${outside}"))"#,
        r#"project(":member").projectDir = file("$outside")"#,
        r#"project(":member").projectDir = file("\u0024outside")"#,
        r#"project(":member").projectDir = file("\044outside")"#,
        r#"project(":member").projectDir = newFile(rootDir, "../outside")"#,
        r#"project(":member").setProjectDir(newFile(rootDir, "../outside"))"#,
    ] {
        let settings = format!("include(\":member\")\n{setter}\n");
        let error = parse_gradle_settings(&settings).unwrap_err();
        assert!(
            error.contains("Unsupported Gradle projectDir assignment")
                || error.contains("Unsupported or dynamic Gradle expression")
                || error.contains("must contain one path")
                || error.contains("Unsupported trailing Gradle projectDir assignment"),
            "unexpected setProjectDir parser error: {error}"
        );
    }
}

#[test]
fn gradle_settings_reject_dynamic_and_unrecognized_project_mutations() {
    for mutation in [
        r#"project(":member").setProperty("projectDir", file("../outside"))"#,
        r#"val changed = project(":member").setProperty("projectDir", file("../outside"))"#,
        r#"project(":member")["projectDir"] = file("../outside")"#,
        r#"project(":member").properties["projectDir"] = file("../outside")"#,
        "project(\":member\")[\n    \"projectDir\"\n]\n    = file(\"../outside\")",
        r#"project(":member").configure { projectDir = file("../outside") }"#,
        r#"project(":member").customDirectory = file("../outside")"#,
        r#"if (enabled) project(":member").customDirectory = file("../outside")"#,
    ] {
        let settings = format!("include(\":member\")\n{mutation}\n");
        let error = parse_gradle_settings(&settings).unwrap_err();
        assert!(
            error.contains("Unsupported")
                && (error.contains("project mutation") || error.contains("projectDir mutation")),
            "unexpected executable project mutation error: {error}"
        );
    }

    let modules = parse_gradle_settings(
        r#"
include(":member")
project(":member").projectDir
project(":member")["projectDir"] == file("modules/member")
project(":member").properties["projectDir"]
val unchanged = project(":member").projectDir == file("modules/member")
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

#[test]
fn gradle_directive_control_flow_validation_is_local() {
    let modules = parse_gradle_settings(
        r#"
if (enabled) println("unrelated")
for (entry in entries) {
    println(entry)
}
include(":member")
project(":member").projectDir = file("modules/member")
"#,
    )
    .unwrap();
    assert_eq!(
        modules,
        vec![GradleSettingsModule {
            name: "member".to_string(),
            path: "modules/member".to_string(),
        }]
    );

    for settings in [
        r#"if (enabled) include(":member")"#,
        "if (\n    enabled\n)\ninclude(\":member\")",
        "if (enabled) {\ninclude(\":member\")\n}",
        "if (\n    enabled\n)\nproject(\":member\").projectDir = file(\"modules/member\")",
        "if (enabled) {\nproject(\":member\").setProjectDir(file(\"modules/member\"))\n}",
    ] {
        let error = parse_gradle_settings(settings).unwrap_err();
        assert!(
            error.contains("Unsupported")
                && (error.contains("conditional") || error.contains("block-scoped")),
            "unexpected directive-local control-flow error: {error}"
        );
    }
}

#[test]
fn gradle_settings_reject_indirect_or_conditional_mutations_but_allow_reads() {
    for settings in [
        r#"
include(":member")
val member = project(":member")
member.setProjectDir(file("../outside"))
"#,
        r#"
include(":member")
val member = project(":member")
member.projectDir = file("../outside")
"#,
        r#"if (enabled) include(":member")"#,
        "if (enabled)\ninclude(\":member\")",
        "if (enabled) {\ninclude(\":member\")\n}",
        r#"settings.include(":member")"#,
        r#"
include(":member")
if (enabled) project(":member").projectDir = file("modules/member")
"#,
        r#"
include(":member")
project(":member").projectDir += file("modules/member")
"#,
        r#"
include(":member")
project(":member") {
    projectDir = file("../outside")
}
"#,
        r#"
include(":member")
project(":member") {
    setProjectDir(file("../outside"))
}
"#,
        r#"
include(":member")
project(":member") . setProjectDir(file("../outside"))
"#,
        r#"
include(":member")
if (enabled) {
    project(":member").projectDir = file("modules/member")
}
"#,
        r#"include(""":member""")"#,
    ] {
        let error = parse_gradle_settings(settings).unwrap_err();
        assert!(
            error.contains("Unsupported") || error.contains("must contain a literal module"),
            "unexpected indirect Gradle mutation error: {error}"
        );
    }

    let modules = parse_gradle_settings(
        r#"
include(":member")
println(project(":member").projectDir)
val unchanged = project(":member").projectDir == file("modules/member")
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

#[test]
fn gradle_settings_ignore_multiline_literals_and_nested_comments() {
    let modules = parse_gradle_settings(
        r#"
val kotlinDocumentation = """
include(":kotlin-phantom")
project(":real").projectDir = file("../outside")
"""
def groovyDocumentation = '''
include(":groovy-phantom")
project(":real").setProjectDir(file("../outside"))
'''
/*
  include(":outer-comment-phantom")
  /* include(":nested-comment-phantom") */
  project(":real").projectDir = file("../outside")
*/
val regularDocumentation = ".projectDir include(\":quoted-phantom\")"
include(":real")
"#,
    )
    .unwrap();

    assert_eq!(
        modules,
        vec![GradleSettingsModule {
            name: "real".to_string(),
            path: "real".to_string(),
        }]
    );
}

#[test]
fn gradle_settings_reject_interpolated_includes_without_partial_modules() {
    for include in [r#"include(":$module")"#, r#"include(":${module}")"#] {
        let error = parse_gradle_settings(include).unwrap_err();
        assert!(
            error.contains("Unsupported or dynamic Gradle expression"),
            "unexpected interpolated include error: {error}"
        );
    }
}

#[test]
fn gradle_settings_preserve_literal_dollars() {
    let modules = parse_gradle_settings(
        r#"
include(':literal$module', ":escaped\$module")
project(':literal$module').setProjectDir(file('modules/$literal'))
project(":escaped\$module").setProjectDir(file("modules/escaped\$literal"))
"#,
    )
    .unwrap();

    assert_eq!(
        modules,
        vec![
            GradleSettingsModule {
                name: "escaped$module".to_string(),
                path: "modules/escaped$literal".to_string(),
            },
            GradleSettingsModule {
                name: "literal$module".to_string(),
                path: "modules/$literal".to_string(),
            },
        ]
    );
}

#[test]
fn gradle_settings_ignore_comments_and_decode_escaped_values() {
    let commented = parse_gradle_settings("include ':member' // \"comment\n").unwrap();
    assert_eq!(commented[0].name, "member");

    let modules = parse_gradle_settings(
        r#"
include(
    ":member", // ignored quote: "
    ':quoted\'member',
    ":slash//member" /* ignored quote: ' */
)
project(":member").projectDir = file("modules\\member")
"#,
    )
    .unwrap();

    assert_eq!(modules[0].name, "member");
    assert_eq!(modules[0].path, "modules/member");
    assert_eq!(modules[1].name, "quoted'member");
    assert_eq!(modules[2].name, "slash//member");
}
