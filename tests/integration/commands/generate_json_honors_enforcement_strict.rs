
#[test]
fn generate_json_honors_enforcement_strict() {
    // An existing spec with a real validation error (a missing source file) that
    // `generate` cannot fix must fail `--enforcement strict` on the JSON path too.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/foo.rs"), "pub fn f() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/foo")).unwrap();
    fs::write(
        root.join("specs/foo/foo.spec.md"),
        "---\nmodule: foo\nversion: 1\nstatus: active\nfiles:\n  - src/foo.rs\n  - src/does_not_exist.rs\n---\n# foo\n## Purpose\np\n",
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["generate", "--enforcement", "strict"])
        .assert()
        .failure();
    let output = specsync()
        .current_dir(root)
        .args(["generate", "--enforcement", "strict", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .clone();
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("generate --format json stdout must be valid JSON even when the gate fails");
}

#[test]
fn generate_json_no_specs_emits_valid_json() {
    // Regression: the "No existing specs found…" diagnostic was printed to stdout even
    // under --format json, prepending non-JSON text and breaking any parser.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/foo.js"), "export function add() {}\n").unwrap();

    let output = specsync()
        .current_dir(root)
        .args(["generate", "--format", "json"])
        .assert()
        .get_output()
        .clone();
    serde_json::from_slice::<serde_json::Value>(&output.stdout).expect(
        "generate --format json stdout must be a clean JSON document with no specs present",
    );
}

// ─── compact: idempotent files and truthful text output ───────────────
// Requirement evidence: REQ-cli-007, REQ-archive-001,
// REQ-cmd-archive-tasks-001, REQ-cmd-compact-001, and REQ-compact-001.

fn setup_compact_project(tmp: &TempDir) -> std::path::PathBuf {
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/history")).unwrap();
    fs::write(
        root.join("src/history/mod.ts"),
        "export function record() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/history")).unwrap();

    let mut spec = valid_spec("history", &["src/history/mod.ts"]);
    for day in 1..=4 {
        spec.push_str(&format!(
            "| 2026-07-{day:02} | maintainer | Change {day} |\n"
        ));
    }
    fs::write(root.join("specs/history/history.spec.md"), spec).unwrap();
    root
}

#[test]
fn compact_dry_run_reports_counts_without_writing() {
    let tmp = TempDir::new().unwrap();
    let root = setup_compact_project(&tmp);
    let spec_path = root.join("specs/history/history.spec.md");
    let before = fs::read(&spec_path).unwrap();

    specsync()
        .args(["compact", "--keep", "2", "--dry-run", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Dry run — no files will be modified",
        ))
        .stdout(predicate::str::contains("would compact 2 entries (kept 2)"))
        .stdout(predicate::str::contains(
            "Would compact 2 entries across 1 spec",
        ));

    assert_eq!(fs::read(spec_path).unwrap(), before);
}

#[test]
fn compact_cli_is_idempotent_and_preserves_trailing_newline() {
    let tmp = TempDir::new().unwrap();
    let root = setup_compact_project(&tmp);
    let spec_path = root.join("specs/history/history.spec.md");

    specsync()
        .args(["compact", "--keep", "2", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("compacted 2 entries (kept 2)"))
        .stdout(predicate::str::contains(
            "Compacted 2 entries across 1 spec",
        ));

    let once = fs::read(&spec_path).unwrap();
    assert!(
        once.ends_with(b"\n"),
        "compact stripped the trailing newline"
    );

    specsync()
        .args(["compact", "--keep", "2", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No changelogs need compaction (all within limit).",
        ));

    assert_eq!(
        fs::read(spec_path).unwrap(),
        once,
        "the second compact run changed the file"
    );
}

#[test]
fn compact_json_formats_are_clean_truthful_and_equivalent() {
    let tmp = TempDir::new().unwrap();
    let root = setup_compact_project(&tmp);
    let spec_path = root.join("specs/history/history.spec.md");
    let before = fs::read(&spec_path).unwrap();
    let mut outputs = Vec::new();

    for format_args in [vec!["--format", "json"], vec!["--json"]] {
        let output = specsync()
            .args(["compact", "--keep", "2", "--dry-run"])
            .args(format_args)
            .arg("--root")
            .arg(&root)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(
            !output.stdout.contains(&0x1b),
            "JSON stdout contained an ANSI escape"
        );
        outputs.push(
            serde_json::from_slice::<serde_json::Value>(&output.stdout)
                .expect("compact JSON stdout must be one valid document"),
        );
    }

    assert_eq!(outputs[0], outputs[1], "--json must match --format json");
    let result = &outputs[0];
    assert_eq!(result["command"], "compact");
    assert_eq!(result["dry_run"], true);
    assert_eq!(result["would_change"], true);
    assert_eq!(result["applied"], false);
    assert_eq!(result["complete"], true);
    assert_eq!(result["partial"], false);
    assert_eq!(result["operations"]["planned"], 1);
    assert_eq!(result["operations"]["succeeded"], 0);
    assert_eq!(result["operations"]["failed"], 0);
    assert_eq!(result["entries_affected"], 2);
    assert_eq!(result["specs_affected"], 1);
    assert_eq!(
        result["results"][0]["spec_path"],
        "specs/history/history.spec.md"
    );
    assert_eq!(result["results"][0]["action"], "would_compact");
    assert_eq!(result["results"][0]["kept_entries"], 2);
    assert_eq!(fs::read(spec_path).unwrap(), before);
}

#[test]
fn compact_markdown_and_github_are_structured_and_preserve_dry_run() {
    let tmp = TempDir::new().unwrap();
    let root = setup_compact_project(&tmp);
    let spec_path = root.join("specs/history/history.spec.md");
    let before = fs::read(&spec_path).unwrap();

    for format in ["markdown", "github"] {
        specsync()
            .args(["compact", "--keep", "2", "--dry-run", "--format", format])
            .arg("--root")
            .arg(&root)
            .assert()
            .success()
            .stdout(predicate::str::contains("## SpecSync Compact Results"))
            .stdout(predicate::str::contains(
                "> Dry run — no files will be modified.",
            ))
            .stdout(predicate::str::contains(
                "| Spec | Action | Entries affected | Kept |",
            ))
            .stdout(predicate::str::contains(
                "| `specs/history/history.spec.md` | Would compact | 2 | 2 |",
            ))
            .stdout(predicate::str::contains(
                "**Summary:** Would compact 2 entries across 1 spec.",
            ));
    }

    assert_eq!(fs::read(spec_path).unwrap(), before);
}

#[test]
fn compact_parse_failure_exits_one_with_valid_json_and_zero_writes() {
    let tmp = TempDir::new().unwrap();
    let root = setup_compact_project(&tmp);
    let valid_path = root.join("specs/history/history.spec.md");
    let valid_before = fs::read(&valid_path).unwrap();

    fs::create_dir_all(root.join("src/broken")).unwrap();
    fs::write(
        root.join("src/broken/mod.ts"),
        "export function broken() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/broken")).unwrap();
    let broken_path = root.join("specs/broken/broken.spec.md");
    let broken = valid_spec("broken", &["src/broken/mod.ts"])
        .replace("|------|--------|--------|", "|------|--------|");
    fs::write(&broken_path, &broken).unwrap();

    let output = specsync()
        .args(["compact", "--keep", "2", "--format", "json", "--root"])
        .arg(&root)
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("failure stdout must remain valid JSON");

    assert_eq!(report["complete"], false);
    assert_eq!(report["partial"], false);
    assert_eq!(report["applied"], false);
    assert_eq!(report["operations"]["planned"], 1);
    assert_eq!(report["operations"]["succeeded"], 0);
    assert_eq!(report["operations"]["failed"], 1);
    assert_eq!(report["errors"][0]["operation"], "parse");
    assert_eq!(fs::read(valid_path).unwrap(), valid_before);
    assert_eq!(fs::read_to_string(broken_path).unwrap(), broken);
}

#[cfg(unix)]
#[test]
fn compact_json_preserves_an_actual_unix_backslash_filename() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/history")).unwrap();
    fs::write(
        root.join("src/history/mod.ts"),
        "export function record() {}\n",
    )
    .unwrap();
    let spec_dir = root.join(r"specs/history\literal");
    fs::create_dir_all(&spec_dir).unwrap();
    let mut spec = valid_spec("history", &["src/history/mod.ts"]);
    for day in 1..=3 {
        spec.push_str(&format!(
            "| 2026-07-{day:02} | maintainer | Change {day} |\n"
        ));
    }
    fs::write(spec_dir.join("history.spec.md"), spec).unwrap();

    let output = specsync()
        .args(["compact", "--keep", "1", "--dry-run", "--format", "json"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .get_output()
        .clone();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(
        report["results"][0]["spec_path"],
        r"specs/history\literal/history.spec.md"
    );
}

// ─── archive-tasks: text and structured output ───────────────────────

fn setup_archive_tasks_project(tmp: &TempDir) -> std::path::PathBuf {
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/work")).unwrap();
    fs::write(root.join("src/work/mod.ts"), "export function work() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/work")).unwrap();
    fs::write(
        root.join("specs/work/work.spec.md"),
        valid_spec("work", &["src/work/mod.ts"]),
    )
    .unwrap();
    fs::write(
        root.join("specs/work/tasks.md"),
        "---\nspec: work.spec.md\n---\n\n## Tasks\n\n- [x] Done one\n- [ ] Still open\n- [X] Done two\n",
    )
    .unwrap();
    root
}

#[test]
fn archive_tasks_text_dry_run_reports_without_writing() {
    let tmp = TempDir::new().unwrap();
    let root = setup_archive_tasks_project(&tmp);
    let tasks_path = root.join("specs/work/tasks.md");
    let before = fs::read(&tasks_path).unwrap();

    specsync()
        .args(["archive-tasks", "--dry-run", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Dry run — no files will be modified",
        ))
        .stdout(predicate::str::contains("would archive 2 tasks"))
        .stdout(predicate::str::contains(
            "Would archive 2 tasks across 1 file",
        ));

    assert_eq!(fs::read(tasks_path).unwrap(), before);
}

#[test]
fn archive_tasks_json_formats_are_clean_truthful_and_equivalent() {
    let tmp = TempDir::new().unwrap();
    let root = setup_archive_tasks_project(&tmp);
    let tasks_path = root.join("specs/work/tasks.md");
    let before = fs::read(&tasks_path).unwrap();
    let mut outputs = Vec::new();

    for format_args in [vec!["--format", "json"], vec!["--json"]] {
        let output = specsync()
            .args(["archive-tasks", "--dry-run"])
            .args(format_args)
            .arg("--root")
            .arg(&root)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(
            !output.stdout.contains(&0x1b),
            "JSON stdout contained an ANSI escape"
        );
        outputs.push(
            serde_json::from_slice::<serde_json::Value>(&output.stdout)
                .expect("archive-tasks JSON stdout must be one valid document"),
        );
    }

    assert_eq!(outputs[0], outputs[1], "--json must match --format json");
    let result = &outputs[0];
    assert_eq!(result["command"], "archive-tasks");
    assert_eq!(result["dry_run"], true);
    assert_eq!(result["would_change"], true);
    assert_eq!(result["applied"], false);
    assert_eq!(result["tasks_affected"], 2);
    assert_eq!(result["files_affected"], 1);
    assert_eq!(result["results"][0]["tasks_path"], "specs/work/tasks.md");
    assert_eq!(result["results"][0]["action"], "would_archive");
    assert_eq!(result["results"][0]["tasks_affected"], 2);
    assert_eq!(fs::read(tasks_path).unwrap(), before);
}

#[test]
fn archive_tasks_markdown_is_structured_and_preserves_dry_run() {
    let tmp = TempDir::new().unwrap();
    let root = setup_archive_tasks_project(&tmp);
    let tasks_path = root.join("specs/work/tasks.md");
    let before = fs::read(&tasks_path).unwrap();

    for format in ["markdown", "github"] {
        specsync()
            .args(["archive-tasks", "--dry-run", "--format", format, "--root"])
            .arg(&root)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "## SpecSync Archive Tasks Results",
            ))
            .stdout(predicate::str::contains(
                "> Dry run — no files will be modified.",
            ))
            .stdout(predicate::str::contains(
                "| Tasks file | Action | Tasks affected |",
            ))
            .stdout(predicate::str::contains(
                "| `specs/work/tasks.md` | Would archive | 2 |",
            ))
            .stdout(predicate::str::contains(
                "**Summary:** Would archive 2 tasks across 1 file.",
            ));
    }

    assert_eq!(fs::read(tasks_path).unwrap(), before);
}

#[test]
fn archive_tasks_apply_failure_exits_one_and_reports_zero_writes() {
    let tmp = TempDir::new().unwrap();
    let root = setup_archive_tasks_project(&tmp);
    let valid_tasks = root.join("specs/work/tasks.md");
    let valid_before = fs::read(&valid_tasks).unwrap();

    fs::create_dir_all(root.join("src/invalid")).unwrap();
    fs::write(
        root.join("src/invalid/mod.ts"),
        "export function invalid() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/invalid")).unwrap();
    fs::write(
        root.join("specs/invalid/invalid.spec.md"),
        valid_spec("invalid", &["src/invalid/mod.ts"]),
    )
    .unwrap();
    let invalid_tasks = root.join("specs/invalid/tasks.md");
    fs::write(&invalid_tasks, b"## Tasks\n\n- [x] invalid \xFF\n").unwrap();
    let invalid_before = fs::read(&invalid_tasks).unwrap();

    let output = specsync()
        .args(["archive-tasks", "--format", "json", "--root"])
        .arg(&root)
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("failure stdout must remain valid JSON");

    assert_eq!(report["complete"], false);
    assert_eq!(report["partial"], false);
    assert_eq!(report["applied"], false);
    assert_eq!(report["files_planned"], 1);
    assert_eq!(report["files_succeeded"], 0);
    assert_eq!(report["failed"][0]["operation"], "read");
    assert_eq!(fs::read(valid_tasks).unwrap(), valid_before);
    assert_eq!(fs::read(invalid_tasks).unwrap(), invalid_before);
}

// ─── hooks install: claude-code-hook must not clobber user settings ──────

#[test]
fn hooks_install_claude_code_hook_preserves_user_settings() {
    // Regression (H3): `hooks install --claude-code-hook` used to overwrite the
    // whole `hooks` object in .claude/settings.json, destroying the user's own
    // hooks. It must deep-merge instead.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".claude")).unwrap();
    fs::write(
        root.join(".claude/settings.json"),
        "{\n  \"permissions\": { \"allow\": [\"Bash(ls:*)\"] },\n  \"hooks\": {\n    \"PreToolUse\": [\n      { \"matcher\": \"Bash\", \"hooks\": [{ \"type\": \"command\", \"command\": \"audit.sh\" }] }\n    ]\n  }\n}\n",
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["hooks", "install", "--claude-code-hook"])
        .assert()
        .success();

    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert_eq!(
        parsed["permissions"]["allow"][0], "Bash(ls:*)",
        "unrelated settings must survive"
    );
    assert_eq!(
        parsed["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "audit.sh",
        "the user's own hooks must survive"
    );
    assert!(
        parsed["hooks"]["PostToolUse"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap()
            .contains("specsync"),
        "specsync's hook must be added"
    );
}
