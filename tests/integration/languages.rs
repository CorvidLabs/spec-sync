use crate::helpers::*;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ─── 8. Multi-language ──────────────────────────────────────────────────

#[test]
fn multi_lang_typescript() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/ts-mod")).unwrap();
    fs::write(
        root.join("src/ts-mod/index.ts"),
        "export function greet(name: string): string { return `Hi ${name}`; }\nexport type Greeting = string;\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/ts-mod")).unwrap();
    let spec = valid_spec("ts-mod", &["src/ts-mod/index.ts"]);
    fs::write(root.join("specs/ts-mod/ts-mod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("specs checked"));
}

#[test]
fn multi_lang_rust() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/rs-mod")).unwrap();
    fs::write(
        root.join("src/rs-mod/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\npub struct Config { pub name: String }\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/rs-mod")).unwrap();
    let spec = valid_spec("rs-mod", &["src/rs-mod/lib.rs"]);
    fs::write(root.join("specs/rs-mod/rs-mod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success();
}

#[test]
fn multi_lang_go() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/gomod")).unwrap();
    fs::write(
        root.join("src/gomod/handler.go"),
        "package gomod\n\nfunc HandleRequest() error { return nil }\n\ntype Request struct {\n\tBody string\n}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/gomod")).unwrap();
    let spec = valid_spec("gomod", &["src/gomod/handler.go"]);
    fs::write(root.join("specs/gomod/gomod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success();
}

#[test]
fn multi_lang_python() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/pymod")).unwrap();
    fs::write(
        root.join("src/pymod/core.py"),
        "def process_data(data):\n    return data\n\nclass DataProcessor:\n    pass\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/pymod")).unwrap();
    let spec = valid_spec("pymod", &["src/pymod/core.py"]);
    fs::write(root.join("specs/pymod/pymod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success();
}

#[test]
fn multi_lang_php() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/phpmod")).unwrap();
    fs::write(
        root.join("src/phpmod/Service.php"),
        r#"<?php

namespace App\Auth;

class AuthService {
    public const DEFAULT_TTL = 3600;

    public function validate(string $token): bool {
        return true;
    }

    private function internalCheck(): void {}
}

interface Authenticator {
    public function authenticate(): bool;
}

function standalone_helper(): void {}
"#,
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/phpmod")).unwrap();
    let spec = valid_spec("phpmod", &["src/phpmod/Service.php"]);
    fs::write(root.join("specs/phpmod/phpmod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("specs checked"));
}

#[test]
fn multi_lang_ruby() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/rbmod")).unwrap();
    fs::write(
        root.join("src/rbmod/service.rb"),
        r#"
module Authentication
  class AuthService
    DEFAULT_TTL = 3600

    attr_reader :token

    def validate(token)
      true
    end

    def self.create(config)
      new
    end

    private

    def internal_check
      false
    end
  end
end

def standalone_helper
  true
end
"#,
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/rbmod")).unwrap();
    let spec = valid_spec("rbmod", &["src/rbmod/service.rb"]);
    fs::write(root.join("specs/rbmod/rbmod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("specs checked"));
}

// ─── Batch Operations ────────────────────────────────────────────────────

#[test]
fn score_all_format_table_outputs_headers() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .args(["score", "--all", "--format", "table", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Spec"))
        .stdout(predicate::str::contains("Score"))
        .stdout(predicate::str::contains("Grade"));
}

#[test]
fn score_all_format_csv_outputs_header_row() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .args(["score", "--all", "--format", "csv", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "spec,score,grade,frontmatter,sections,api,depth,freshness",
        ));
}

#[test]
fn score_all_format_csv_includes_summary_row() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .args(["score", "--all", "--format", "csv", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("SUMMARY,"));
}

#[test]
fn score_format_table_without_all_flag_still_works() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // --all is optional; table output should work without it
    specsync()
        .args(["score", "--format", "table", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Grade"));
}

#[test]
fn generate_uncovered_flag_accepted() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // --uncovered is a no-op when all modules are specced
    specsync()
        .args(["generate", "--uncovered", "--root"])
        .arg(&root)
        .assert()
        .success();
}

#[test]
fn generate_batch_empty_list_skips_gracefully() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Request a module that doesn't exist in coverage
    specsync()
        .args(["generate", "--batch", "nonexistent-module", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("not found in coverage report")
                .or(predicate::str::contains("Nothing to generate")),
        );
}

#[test]
fn import_without_args_or_flags_shows_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);

    specsync()
        .args(["import", "--root"])
        .arg(&root)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("SOURCE is required").or(predicate::str::contains("required")),
        );
}

#[test]
fn import_from_dir_imports_markdown_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();

    // Write a simple markdown doc to import
    fs::write(
        root.join("docs/my-feature.md"),
        "# My Feature\n\nThis is a great feature.\n\n- [ ] Do something\n- [ ] Do another thing\n",
    )
    .unwrap();

    specsync()
        .args(["import", "--from-dir", "docs", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Batch Import"))
        .stdout(predicate::str::contains("1 imported").or(predicate::str::contains("imported")));
}

#[test]
fn import_from_dir_skips_existing_specs() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("docs")).unwrap();

    // Pre-create the spec
    let spec_dir = root.join("specs/my-feature");
    fs::create_dir_all(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("my-feature.spec.md"),
        valid_spec("my-feature", &[]),
    )
    .unwrap();

    // Write the same-named doc
    fs::write(
        root.join("docs/my-feature.md"),
        "# My Feature\n\nAlready exists.\n",
    )
    .unwrap();

    specsync()
        .args(["import", "--from-dir", "docs", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("skipped"));
}

#[test]
fn import_from_dir_nonexistent_directory_errors() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs")).unwrap();

    specsync()
        .args(["import", "--from-dir", "nonexistent-dir", "--root"])
        .arg(&root)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Directory not found")
                .or(predicate::str::contains("not found")),
        );
}

