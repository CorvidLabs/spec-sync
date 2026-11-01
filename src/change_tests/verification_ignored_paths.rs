// Issue #825: an opt-in `.specsync/sdd.json` `verification_ignored_paths` list keeps another
// tool's record directory (Chough writes `.chough/`) out of the verification project-input digest
// and nowhere else.
//
// Labels: a DISCRIMINATOR fails on origin/main a80748ff and passes with the change; a CONTROL
// passes on both. Every test here except the ones marked NEW API writes the list as raw JSON and
// calls only functions that already existed on a80748ff, so the labels were confirmed by running
// this file against that code.

fn vip_git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Write `verification_ignored_paths` into the existing policy as raw JSON, so the fixture also
/// compiles and runs against a binary that does not know the key.
fn vip_set_list(root: &Path, entries: &[&str]) {
    let path = root.join(POLICY_PATH);
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["verification_ignored_paths"] = serde_json::json!(entries);
    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

fn vip_set_policy_value(root: &Path, key: &str, json: serde_json::Value) {
    let path = root.join(POLICY_PATH);
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value[key] = json;
    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

fn vip_review_next(id: &str) -> String {
    format!("run `specsync change review {id} --reviewer <human>` after the PR's scoped review passes")
}

fn vip_next_action(root: &Path, id: &str) -> String {
    summarize_change(root, &load_change(root, id).unwrap()).next_action
}

fn vip_verification_current(root: &Path, id: &str) -> bool {
    recorded_verification_is_current(root, &load_change(root, id).unwrap())
}

fn vip_review_currency(root: &Path, id: &str) -> Option<ScopedReviewCurrency> {
    recorded_scoped_review_currency(root, &load_change(root, id).unwrap())
}

/// A Git project in a subdirectory `prefix` of the repository (`""` for the repository root):
/// one workflow-v2 change for module `change` covering `src/`, approved, implemented and verified
/// with `entries` committed in the policy before the first check.
fn vip_fixture_at(prefix: &str, entries: &[&str]) -> (TempDir, PathBuf, String) {
    let temp = TempDir::new().unwrap();
    let repository = temp.path().to_path_buf();
    vip_git(&repository, &["init", "-b", "main"]);
    vip_git(&repository, &["config", "user.email", "test@example.com"]);
    vip_git(&repository, &["config", "user.name", "Test"]);
    fs::write(repository.join("README.md"), "base\n").unwrap();
    vip_git(&repository, &["add", "README.md"]);
    vip_git(&repository, &["commit", "-m", "base"]);
    let root = if prefix.is_empty() {
        repository.clone()
    } else {
        repository.join(prefix)
    };
    fs::create_dir_all(&root).unwrap();
    let record = current_workflow_record(&root, completed_no_spec_record(&root));
    if !entries.is_empty() {
        vip_set_list(&root, entries);
    }
    approve_definition(&root, &record.id, Some("Scope owner".into()), None).unwrap();
    vip_git(&root, &["add", "."]);
    vip_git(&root, &["commit", "-m", "Implement approved change"]);
    check_change(&root, Some(&record.id)).unwrap();
    vip_git(&root, &["add", "."]);
    vip_git(&root, &["commit", "-m", "Record verification"]);
    let verification = check_change(&root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed);
    assert_eq!(vip_next_action(&root, &record.id), vip_review_next(&record.id));
    (temp, root, record.id)
}

fn vip_fixture(entries: &[&str]) -> (TempDir, PathBuf, String) {
    vip_fixture_at("", entries)
}

fn vip_write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

// DISCRIMINATOR. Issue #825 step B with the list in place of the workarounds it tried.
// Verifies REQ-change-114.
#[test]
fn listed_prefix_untracked_file_keeps_verification_current() {
    let (_temp, root, id) = vip_fixture(&[".chough/"]);
    vip_write(&root, ".chough/evidence/obs.json", "{\"observed\":true}\n");
    assert!(
        vip_verification_current(&root, &id),
        "an untracked record under a listed prefix must not stale verification"
    );
    assert_eq!(vip_next_action(&root, &id), vip_review_next(&id));
}

// DISCRIMINATOR. Tracked edits, deletion, re-adding and committing before the review (step F).
// Verifies REQ-change-114.
#[test]
fn listed_prefix_tracked_edit_and_delete_keep_verification_current() {
    let (_temp, root, id) = vip_fixture(&[".chough/"]);
    vip_write(&root, ".chough/obs.json", "first\n");
    commit_paths(&root, &[".chough/obs.json"], "record observation");
    assert!(vip_verification_current(&root, &id), "committed record");
    vip_write(&root, ".chough/obs.json", "second\n");
    assert!(vip_verification_current(&root, &id), "edited tracked record");
    fs::remove_file(root.join(".chough/obs.json")).unwrap();
    assert!(vip_verification_current(&root, &id), "deleted tracked record");
    vip_write(&root, ".chough/obs.json", "third\n");
    commit_paths(&root, &[".chough/obs.json"], "re-record observation");
    assert!(vip_verification_current(&root, &id), "re-added and committed");
    assert_eq!(vip_next_action(&root, &id), vip_review_next(&id));
    record_scoped_review(&root, &id, "Independent reviewer".into()).unwrap();
    assert_eq!(
        vip_review_currency(&root, &id),
        Some(ScopedReviewCurrency::Current)
    );
}

// DISCRIMINATOR. The review binds the same filtered digest, so an uncommitted edit under the
// listed prefix after the review leaves it current and finalization accepts it.
// Verifies REQ-change-114.
#[test]
fn listed_prefix_uncommitted_edit_after_review_keeps_review_current() {
    let (_temp, root, id) = vip_fixture(&[".chough/"]);
    vip_write(&root, ".chough/obs.json", "first\n");
    commit_paths(&root, &[".chough/obs.json"], "record observation");
    record_scoped_review(&root, &id, "Independent reviewer".into()).unwrap();
    vip_write(&root, ".chough/obs.json", "edited after review\n");
    vip_write(&root, ".chough/new.json", "untracked after review\n");
    assert_eq!(
        vip_review_currency(&root, &id),
        Some(ScopedReviewCurrency::Current)
    );
    assert_eq!(
        vip_next_action(&root, &id),
        format!("run `specsync change finalize {id}`")
    );
    let archive = finalize_change(&root, &id).unwrap();
    assert!(archive.is_dir());
}

/// A non-Git project with a default lifecycle policy, a spec owning `src/lib.rs` and
/// `assets/schema.txt`, a principles file and a custom-artifact template.
fn vip_validation_fixture() -> TempDir {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    vip_set_policy_value(root, "principles_file", serde_json::json!("docs/principles.md"));
    vip_set_policy_value(
        root,
        "custom_artifacts",
        serde_json::json!({"runbook": "templates/runbook.md"}),
    );
    vip_write(root, "docs/principles.md", "Be careful.\n");
    vip_write(root, "templates/runbook.md", "# Runbook\n");
    vip_write(root, "src/lib.rs", "pub fn ready() -> bool { true }\n");
    vip_write(root, "assets/schema.txt", "schema\n");
    vip_write(
        root,
        "specs/auth/auth.spec.md",
        "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\n  - assets/schema.txt\n---\n\n# Auth\n",
    );
    temp
}

fn vip_assert_refused(root: &Path, entries: &[&str], named: &str, rule: &str) {
    vip_set_list(root, entries);
    let error = project_input_digest(root)
        .err()
        .unwrap_or_else(|| panic!("{entries:?} must be refused"));
    assert!(
        error.contains("invalid SDD policy: verification_ignored_paths"),
        "{entries:?}: {error}"
    );
    assert!(error.contains(named), "{entries:?} must name `{named}`: {error}");
    assert!(error.contains(rule), "{entries:?} must name rule `{rule}`: {error}");
}

// DISCRIMINATOR. Every shape and overlap rule refuses, naming the entry and the rule.
// Verifies REQ-change-115.
#[test]
fn invalid_verification_ignored_paths_fail_closed() {
    let temp = vip_validation_fixture();
    let root = temp.path();
    vip_set_list(root, &[".chough/"]);
    project_input_digest(root).expect("a valid entry is accepted");

    for pattern in ["*.json", ".chough/**", "[a]/", "{a,b}/", "!x/", "a?/"] {
        vip_assert_refused(root, &[pattern], pattern, "pattern character");
    }
    for absolute in ["/abs/", "C:/x/"] {
        vip_assert_refused(root, &[absolute], absolute, "is absolute");
    }
    vip_assert_refused(root, &["a\\b/"], "a\\b/", "backslash");
    for component in ["../x/", "./x/", "a//b/", "./", "a/../"] {
        vip_assert_refused(root, &[component], component, "component");
    }
    for slashless in [".chough", "."] {
        vip_assert_refused(root, &[slashless], slashless, "does not end in `/`");
    }
    vip_assert_refused(root, &[""], "``", "is empty");
    vip_assert_refused(root, &["a\u{1}b/"], "a\\u{1}b/", "control character");
    let long = format!("{}/", "a".repeat(128));
    vip_assert_refused(root, &[long.as_str()], &long, "128 bytes");
    let nine: Vec<String> = (0..9).map(|index| format!("tool{index}/")).collect();
    let nine: Vec<&str> = nine.iter().map(String::as_str).collect();
    vip_assert_refused(root, &nine, "9 entries", "at most 8");
    vip_assert_refused(root, &["a/", "a/"], "a/", "duplicates");
    vip_assert_refused(root, &["a/", "A/"], "A/", "duplicates");
    vip_assert_refused(root, &["a/", "a/b/"], "a/b/", "nested");
    for protected in [".specsync/", ".specsync/changes/", ".SpecSync/"] {
        vip_assert_refused(root, &[protected], protected, "`.specsync`");
    }
    for specs in ["specs/", "specs/auth/", "Specs/"] {
        vip_assert_refused(root, &[specs], specs, "the specs tree `specs`");
    }
    for source in ["src/", "src/gen/", "SRC/"] {
        vip_assert_refused(root, &[source], source, "source_dirs `src`");
    }
    vip_assert_refused(root, &["assets/"], "assets/", "files: entry `assets/schema.txt`");
    vip_assert_refused(root, &["docs/"], "docs/", "principles_file `docs/principles.md`");
    vip_assert_refused(
        root,
        &["templates/"],
        "templates/",
        "custom_artifacts template `templates/runbook.md`",
    );
    for meaningful in ["tests/", "tests/fixtures/", ".github/", ".github/workflows/", "site/"] {
        vip_assert_refused(root, &[meaningful], meaningful, "meaningful_paths");
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("src"), root.join(".chough")).unwrap();
        vip_assert_refused(root, &[".chough/"], ".chough/", "symlink at `.chough`");
        fs::remove_file(root.join(".chough")).unwrap();
        fs::create_dir_all(root.join("elsewhere/.chough")).unwrap();
        std::os::unix::fs::symlink(root.join("elsewhere"), root.join("link")).unwrap();
        vip_assert_refused(root, &["link/.chough/"], "link/.chough/", "symlink at `link`");
        fs::remove_file(root.join("link")).unwrap();
    }
    vip_set_list(root, &[".chough/", ".tool-cache/"]);
    project_input_digest(root).expect("valid entries are accepted again");
}

// DISCRIMINATOR. `change check` also reads, in-process, the configured `schema_dir` (to judge
// spec `db_tables`), every path-form `depends_on` entry of a spec, and every file a spec's
// `Consumed By` table names. An entry that equals, contains or lies inside any of them would let
// an edit there change the verdict without staling verification, so each is refused.
// Verifies REQ-change-115.
#[test]
fn entry_covering_an_in_process_check_input_is_refused() {
    let temp = vip_validation_fixture();
    let root = temp.path();
    vip_write(
        root,
        ".specsync/config.toml",
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\nschema_dir = \"db/schema\"\n",
    );
    vip_write(root, "db/schema/001_users.sql", "CREATE TABLE users (id INTEGER);\n");
    vip_write(
        root,
        "specs/billing/billing.spec.md",
        "---\nmodule: billing\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\ndb_tables:\n  - users\ndepends_on:\n  - vendor/shared/shared.spec.md\n  - auth\n  - CorvidLabs/other@ledger\n---\n\n# Billing\n\n## Public API\n\n### Consumed By\n\n| File | Usage |\n|------|-------|\n| `tools/consumer.rs` | Calls billing |\n",
    );
    vip_set_list(root, &[".chough/", "db2/"]);
    project_input_digest(root).expect("entries clear of every check input are accepted");
    for schema in ["db/", "db/schema/", "db/schema/migrations/", "DB/Schema/"] {
        vip_assert_refused(root, &[schema], schema, "schema_dir `db/schema`");
    }
    vip_assert_refused(
        root,
        &["vendor/"],
        "vendor/",
        "depends_on entry `vendor/shared/shared.spec.md` of spec `specs/billing/billing.spec.md`",
    );
    vip_assert_refused(
        root,
        &["tools/"],
        "tools/",
        "Consumed By reference `tools/consumer.rs` of spec `specs/billing/billing.spec.md`",
    );
    // A cross-project `depends_on` reference is resolved by `specsync resolve`, never read from
    // disk by `change check`, so it protects nothing.
    vip_set_list(root, &["corvidlabs/"]);
    project_input_digest(root).expect("a cross-project reference is not a local input");
}

// DISCRIMINATOR. `ignored_paths` keeps its one meaning and never unlocks an entry.
// Verifies REQ-change-115.
#[test]
fn ignored_paths_never_unlocks_an_entry() {
    let temp = vip_validation_fixture();
    let root = temp.path();
    vip_set_policy_value(
        root,
        "ignored_paths",
        serde_json::json!([".specsync/", "specs/", "tests/fixtures/", ".github/workflows/", ".chough/"]),
    );
    vip_assert_refused(root, &["tests/fixtures/"], "tests/fixtures/", "meaningful_paths `tests/`");
    vip_assert_refused(
        root,
        &[".github/workflows/"],
        ".github/workflows/",
        "meaningful_paths `.github/`",
    );
    vip_set_list(root, &[".chough/"]);
    project_input_digest(root).expect(".chough/ overlaps nothing under the default policy");
    vip_set_policy_value(root, "meaningful_paths", serde_json::json!(["."]));
    vip_assert_refused(root, &[".chough/"], ".chough/", "meaningful_paths `.`");
    vip_set_policy_value(root, "ignored_paths", serde_json::json!([]));
    vip_assert_refused(root, &[".chough/"], ".chough/", "meaningful_paths `.`");
}

// DISCRIMINATOR. With `["src/"]` every lifecycle verb that loads policy refuses before it records
// anything, status names the entry as its one next action, and audit reports it.
// Verifies REQ-change-115.
#[test]
fn invalid_list_is_refused_by_every_lifecycle_verb() {
    let (_temp, root, id) = vip_fixture(&[".chough/"]);
    let draft = create_change(
        &root,
        CreateChangeRequest {
            description: distinct_fixture_description(&root, "a draft beside the verified change"),
            kind: ChangeKind::BugFix,
            affected_specs: Vec::new(),
            affected_paths: vec!["src/lib.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("No public contract change".into()),
        },
    )
    .unwrap();
    vip_set_list(&root, &["src/"]);
    let named = |result: Result<String, String>, verb: &str| {
        let error = result.err().unwrap_or_else(|| panic!("{verb} must refuse"));
        assert!(
            error.contains("verification_ignored_paths") && error.contains("`src/`"),
            "{verb}: {error}"
        );
    };
    let workspace = root.join(CHANGES_PATH).join(&id);
    let read = |name: &str| fs::read(workspace.join(name)).ok();
    let approvals = read("approvals.json");
    let attempts = read("verification-attempts.json");
    let changes_before = fs::read_dir(root.join(CHANGES_PATH)).unwrap().count();

    named(
        create_change(
            &root,
            CreateChangeRequest {
                description: "a change created under an invalid list".into(),
                kind: ChangeKind::BugFix,
                affected_specs: Vec::new(),
                affected_paths: vec!["src/lib.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("No public contract change".into()),
            },
        )
        .map(|record| record.id),
        "change new",
    );
    assert_eq!(
        fs::read_dir(root.join(CHANGES_PATH)).unwrap().count(),
        changes_before,
        "no workspace is created"
    );
    named(
        answer_question(&root, &draft.id, "acceptance_criteria", "It works")
            .map(|record| record.id),
        "change answer acceptance_criteria",
    );
    named(
        answer_question(&root, &draft.id, "affected_paths", "src/lib.rs").map(|record| record.id),
        "change answer affected_paths",
    );
    named(
        approve_definition(&root, &id, Some("Scope owner".into()), None).map(|record| record.id),
        "change approve",
    );
    named(
        check_change(&root, Some(&id)).map(|_| String::new()),
        "change check",
    );
    named(
        record_scoped_review(&root, &id, "Independent reviewer".into()).map(|_| String::new()),
        "change review",
    );
    named(
        finalize_change(&root, &id).map(|path| path.display().to_string()),
        "change finalize",
    );
    assert_eq!(read("approvals.json"), approvals, "no approval appended");
    assert_eq!(
        read("verification-attempts.json"),
        attempts,
        "no verification attempt appended"
    );
    assert!(!workspace.join(SCOPED_REVIEW_FILE).exists(), "no review recorded");
    let next = vip_next_action(&root, &id);
    assert!(
        next.contains("verification_ignored_paths") && next.contains("`src/`"),
        "status names the entry: {next}"
    );
    let draft_next = vip_next_action(&root, &draft.id);
    assert!(
        draft_next.contains("verification_ignored_paths"),
        "every change names it: {draft_next}"
    );
    let report = audit_project(&root);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("verification_ignored_paths") && error.contains("`src/`")),
        "{:?}",
        report.errors
    );
}

// DISCRIMINATOR. A change that claims a listed path is refused at every verb that binds it, and an
// unrelated change is not.
// Verifies REQ-change-115.
#[test]
fn entry_overlapping_declared_affected_paths_is_refused_for_that_change() {
    let (_temp, root, id) = vip_fixture(&[]);
    let request = |description: &str, path: &str| CreateChangeRequest {
        description: distinct_fixture_description(&root, description),
        kind: ChangeKind::BugFix,
        affected_specs: Vec::new(),
        affected_paths: vec![path.into()],
        requested_artifacts: Vec::new(),
        no_spec_change: true,
        rationale: Some("No public contract change".into()),
    };
    // Created before the list exists, so the later verbs are the ones that must refuse.
    let claims = create_change(&root, request("claims the tool directory", ".chough/")).unwrap();
    vip_set_list(&root, &[".chough/"]);
    let overlap = |result: Result<String, String>, verb: &str, path: &str| {
        let error = result.err().unwrap_or_else(|| panic!("{verb} must refuse"));
        assert!(
            error.contains("verification_ignored_paths entry `.chough/`")
                && error.contains(&format!("affected path `{path}`")),
            "{verb}: {error}"
        );
    };
    for path in [".chough/", ".", ".chough/obs.json"] {
        overlap(
            create_change(&root, request("claims a listed path", path)).map(|record| record.id),
            "change new",
            path,
        );
    }
    let draft = create_change(&root, request("a draft that later claims it", "src/lib.rs")).unwrap();
    overlap(
        answer_question(&root, &draft.id, "affected_paths", ".chough/").map(|record| record.id),
        "change answer affected_paths",
        ".chough/",
    );
    let next = vip_next_action(&root, &claims.id);
    assert!(next.contains("affected path `.chough/`"), "{next}");
    // The verified change declares `src/`, so the same list leaves it alone: the `sdd.json` edit
    // stales its verification as any policy edit does, and it re-verifies and is reviewed.
    assert_eq!(vip_next_action(&root, &id), format!("run `specsync change check {id}`"));
    check_change(&root, Some(&id)).unwrap();
    assert_eq!(vip_next_action(&root, &id), vip_review_next(&id));
    record_scoped_review(&root, &id, "Independent reviewer".into()).unwrap();
}

// DISCRIMINATOR. Approval and check refuse a change whose declared path came before the list.
// Verifies REQ-change-115.
#[test]
fn entry_overlapping_declared_affected_paths_is_refused_at_approve_and_check() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    vip_git(root, &["init", "-b", "main"]);
    vip_git(root, &["config", "user.email", "test@example.com"]);
    vip_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    vip_git(root, &["add", "README.md"]);
    vip_git(root, &["commit", "-m", "base"]);
    let mut record = current_workflow_record(root, completed_no_spec_record(root));
    record.affected_paths = vec!["src/".into(), ".chough/".into()];
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    let approved = |root: &Path| approve_definition(root, &record.id, Some("Scope owner".into()), None);
    vip_set_list(root, &[".chough/"]);
    let error = approved(root).unwrap_err();
    assert!(
        error.contains("entry `.chough/`") && error.contains("affected path `.chough/`"),
        "{error}"
    );
    fs::remove_file(root.join(POLICY_PATH)).unwrap();
    write_lifecycle_test_policy(root);
    approved(root).unwrap();
    vip_git(root, &["add", "."]);
    vip_git(root, &["commit", "-m", "Implement approved change"]);
    vip_set_list(root, &[".chough/"]);
    let error = check_change(root, Some(&record.id)).unwrap_err();
    assert!(
        error.contains("entry `.chough/`") && error.contains("affected path `.chough/`"),
        "{error}"
    );
    assert!(
        !root
            .join(CHANGES_PATH)
            .join(&record.id)
            .join("verification-attempts.json")
            .exists(),
        "no verification attempt is recorded against the overlap"
    );
}

// DISCRIMINATOR. Entries are project-relative in a project nested inside its repository.
// Verifies REQ-change-114 and REQ-change-115.
#[test]
fn nested_project_root_excludes_only_its_own_prefix() {
    let (temp, root, id) = vip_fixture_at("app", &[".chough/"]);
    let repository = temp.path();
    vip_write(&root, ".chough/obs.json", "app record\n");
    assert!(vip_verification_current(&root, &id), "app/.chough/ is excluded");
    vip_write(&root, "notes/.chough/obs.json", "nested elsewhere\n");
    assert!(
        !vip_verification_current(&root, &id),
        "app/notes/.chough/ is not under the entry"
    );
    fs::remove_dir_all(root.join("notes")).unwrap();
    assert!(vip_verification_current(&root, &id));
    #[cfg(unix)]
    {
        // A repository-root `.chough` symlink is not this project's entry.
        std::os::unix::fs::symlink(repository.join("app/src"), repository.join(".chough"))
            .unwrap();
        assert!(project_input_digest(&root).is_ok());
        assert!(vip_verification_current(&root, &id));
        fs::remove_dir_all(root.join(".chough")).unwrap();
        std::os::unix::fs::symlink(root.join("src"), root.join(".chough")).unwrap();
        let error = project_input_digest(&root).unwrap_err();
        assert!(error.contains("symlink at `.chough`"), "{error}");
    }
}

// DISCRIMINATOR for the non-empty list, CONTROL for the absent one.
// Verifies REQ-cmd-change-024.
#[test]
fn summary_reports_the_active_list() {
    let (_temp, root, id) = vip_fixture(&[".chough/"]);
    let summary = serde_json::to_value(summarize_change(&root, &load_change(&root, &id).unwrap()))
        .unwrap();
    assert_eq!(
        summary["verification_ignored_paths"],
        serde_json::json!([".chough/"])
    );
    let (_temp, root, id) = vip_fixture(&[]);
    let summary = serde_json::to_value(summarize_change(&root, &load_change(&root, &id).unwrap()))
        .unwrap();
    assert!(summary.get("verification_ignored_paths").is_none());
}

// CONTROL (an `sdd.json` edit already stales verification on a80748ff). Adding, editing or
// removing an entry stales verification and review.
// Verifies REQ-change-114.
#[test]
fn changing_the_list_after_verification_stales_it() {
    let (_temp, root, id) = vip_fixture(&[".chough/"]);
    let original = fs::read(root.join(POLICY_PATH)).unwrap();
    vip_set_list(&root, &[".chough/", ".tool/"]);
    assert!(!vip_verification_current(&root, &id), "adding an entry");
    vip_set_list(&root, &[".tool/"]);
    assert!(!vip_verification_current(&root, &id), "editing an entry");
    vip_set_list(&root, &[]);
    assert!(!vip_verification_current(&root, &id), "removing every entry");
    fs::write(root.join(POLICY_PATH), &original).unwrap();
    // A rewritten tracked file is named by its bytes until Git refreshes its stat data.
    vip_git(&root, &["update-index", "-q", "--refresh"]);
    assert!(vip_verification_current(&root, &id), "restoring the bytes");
    record_scoped_review(&root, &id, "Independent reviewer".into()).unwrap();
    vip_set_list(&root, &[]);
    assert!(matches!(
        vip_review_currency(&root, &id),
        Some(ScopedReviewCurrency::Stale(_))
    ));
}

// CONTROL. Outside every listed prefix, and for a path equal to an entry without its slash,
// verification stales exactly as before.
// Verifies REQ-change-114.
#[test]
fn unlisted_paths_still_stale_verification() {
    let (_temp, root, id) = vip_fixture(&[".chough/"]);
    vip_write(&root, "notes/obs.json", "unlisted\n");
    assert!(!vip_verification_current(&root, &id), "notes/obs.json");
    fs::remove_dir_all(root.join("notes")).unwrap();
    assert!(vip_verification_current(&root, &id));
    let source = fs::read(root.join("src/lib.rs")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { false }\n").unwrap();
    assert!(!vip_verification_current(&root, &id), "src/lib.rs");
    fs::write(root.join("src/lib.rs"), source).unwrap();
    vip_git(&root, &["update-index", "-q", "--refresh"]);
    assert!(vip_verification_current(&root, &id));
    vip_write(&root, ".chough", "a file named like the entry\n");
    assert!(!vip_verification_current(&root, &id), "a file named .chough");
    assert_eq!(vip_next_action(&root, &id), format!("run `specsync change check {id}`"));
}

// DISCRIMINATOR. The descendant walk is unchanged: a commit after the review that touches only
// the listed prefix still stales the review THROUGH THE WALK, and finalization refuses. On
// a80748ff the same commit stales the review through the project-input digest instead, so the
// walk's message is absent there.
// Verifies REQ-change-114.
#[test]
fn listed_prefix_commit_after_review_still_stales_review() {
    let (_temp, root, id) = vip_fixture(&[".chough/"]);
    record_scoped_review(&root, &id, "Independent reviewer".into()).unwrap();
    vip_write(&root, ".chough/obs.json", "committed after review\n");
    commit_paths(&root, &[".chough/obs.json"], "record observation after review");
    // The digest no longer moves for `.chough/`, so staleness must come from the unchanged
    // descendant walk, whose message names the path the post-review commit touched.
    match vip_review_currency(&root, &id) {
        Some(ScopedReviewCurrency::Stale(message)) => assert!(
            message.contains("changed `.chough/obs.json`, which is outside this change's own lifecycle records"),
            "{message}"
        ),
        other => panic!("the review must be stale through the descendant walk: {other:?}"),
    }
    assert!(finalize_change(&root, &id).is_err());
}

// CONTROL. An unlisted commit after the review stales it as before.
// Verifies REQ-change-114.
#[test]
fn unlisted_commit_after_review_still_stales_review() {
    let (_temp, root, id) = vip_fixture(&[".chough/"]);
    record_scoped_review(&root, &id, "Independent reviewer".into()).unwrap();
    vip_write(&root, "notes/x", "unlisted\n");
    commit_paths(&root, &["notes/x"], "unlisted commit after review");
    assert!(matches!(
        vip_review_currency(&root, &id),
        Some(ScopedReviewCurrency::Stale(_))
    ));
}

// CONTROL. Coverage (`path_is_meaningful`) answers identically with and without the list.
// Verifies REQ-change-114.
#[test]
fn ignored_paths_and_coverage_are_unchanged_by_the_list() {
    let without = SddPolicy::default();
    let mut value = serde_json::to_value(&without).unwrap();
    value["verification_ignored_paths"] = serde_json::json!([".chough/"]);
    let with: SddPolicy = serde_json::from_value(value).unwrap();
    for path in [
        ".chough/obs.json",
        ".chough",
        "src/lib.rs",
        "tests/fixtures/a",
        ".specsync/sdd.json",
        "specs/change/change.spec.md",
        "notes/x",
        "README.md",
    ] {
        assert_eq!(
            path_is_meaningful(path, &with),
            path_is_meaningful(path, &without),
            "{path}"
        );
    }
    assert_eq!(with.ignored_paths, without.ignored_paths);
    assert_eq!(with.meaningful_paths, without.meaningful_paths);
}

// CONTROL. The default policy and adoption never write the key.
// Verifies REQ-change-114.
#[test]
fn write_default_policy_bytes_are_unchanged() {
    let temp = TempDir::new().unwrap();
    write_default_policy(temp.path(), Vec::new()).unwrap();
    let written = fs::read_to_string(temp.path().join(POLICY_PATH)).unwrap();
    assert!(!written.contains("verification_ignored_paths"), "{written}");
    let serialized = serde_json::to_string(&SddPolicy::default()).unwrap();
    assert!(!serialized.contains("verification_ignored_paths"));
}

/// Files of the byte-identity fixture, written with fixed modes so the digest is reproducible.
fn vip_pinned_tree(root: &Path, policy: &str) {
    for (relative, content) in [
        ("README.md", "pinned\n"),
        ("src/lib.rs", "pub fn ready() -> bool { true }\n"),
        (".specsync/sdd.json", policy),
    ] {
        vip_write(root, relative, content);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(root.join(relative), fs::Permissions::from_mode(0o644)).unwrap();
        }
    }
}

// CONTROL. With the key absent or empty the digest is byte-identical to a80748ff. Both hex values
// were computed by this test on unchanged a80748ff.
// Verifies REQ-change-114.
#[test]
fn empty_or_absent_list_digest_is_byte_identical() {
    let absent = TempDir::new().unwrap();
    vip_pinned_tree(absent.path(), "{\n  \"version\": 2,\n  \"enabled\": true\n}\n");
    let empty = TempDir::new().unwrap();
    vip_pinned_tree(
        empty.path(),
        "{\n  \"version\": 2,\n  \"enabled\": true,\n  \"verification_ignored_paths\": []\n}\n",
    );
    assert_eq!(
        project_input_digest(absent.path()).unwrap(),
        "5362d2e2fe30d1afc52ab44df300e00531414113387ace53b6f9fe58de8e0d79"
    );
    assert_eq!(
        project_input_digest(empty.path()).unwrap(),
        "dae74f5c5ce75935a4ed42c037c98ffe774d6979448d108acec0215d1befa316"
    );
}

// CONTROL. The same byte-identity pin through Git discovery, the path a Git project takes: the
// tree is committed with fixed content and modes, so `git_project_paths` (and, with the change,
// its excluding variant and the cache key it extended) supplies the paths. Both hex values were
// computed by this test on unchanged a80748ff.
// Verifies REQ-change-114.
#[test]
fn empty_or_absent_list_git_digest_is_byte_identical() {
    let digest_for = |policy: &str| {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        vip_git(root, &["init", "-b", "main"]);
        vip_git(root, &["config", "user.email", "test@example.com"]);
        vip_git(root, &["config", "user.name", "Test"]);
        vip_git(root, &["config", "core.autocrlf", "false"]);
        vip_git(root, &["config", "core.filemode", "true"]);
        vip_pinned_tree(root, policy);
        vip_git(root, &["add", "--all"]);
        vip_git(root, &["commit", "-m", "pinned"]);
        vip_write(root, "notes/untracked.txt", "untracked\n");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                root.join("notes/untracked.txt"),
                fs::Permissions::from_mode(0o644),
            )
            .unwrap();
        }
        project_input_digest(root).unwrap()
    };
    assert_eq!(
        digest_for("{\n  \"version\": 2,\n  \"enabled\": true\n}\n"),
        "1300fd3d22328f24c98ef2fa3147baae56addae669a280ee39acd8fc203a5da8"
    );
    assert_eq!(
        digest_for(
            "{\n  \"version\": 2,\n  \"enabled\": true,\n  \"verification_ignored_paths\": []\n}\n"
        ),
        "7750e475791e9070f809a2cae4a4532e4d930facec0fb9ec8765a3a9449dbec6"
    );
}

// CONTROL. With `[]` written explicitly in a Git project, neither verification.json nor the status
// JSON gains the key.
// Verifies REQ-change-114.
#[test]
fn empty_list_adds_no_key_to_verification_or_status_json() {
    let (_temp, root, id) = vip_fixture(&[]);
    vip_set_list(&root, &[]);
    commit_paths(&root, &[POLICY_PATH], "write an empty list");
    check_change(&root, Some(&id)).unwrap();
    let verification =
        fs::read_to_string(root.join(CHANGES_PATH).join(&id).join("verification.json")).unwrap();
    assert!(!verification.contains("verification_ignored"), "{verification}");
    let status =
        serde_json::to_string(&summarize_change(&root, &load_change(&root, &id).unwrap())).unwrap();
    assert!(!status.contains("verification_ignored"), "{status}");
}

// CONTROL. An accepted change whose acceptance was squash-merged is authenticated through
// `consider_accepted_evidence_anchor` exactly as it is without the list: the list adds nothing to
// verification.json, approvals.json or state.json, so the same in-history anchor matches.
// Verifies REQ-change-114.
#[test]
fn accepted_evidence_anchor_is_found_identically_with_the_list() {
    let mut accepted_states = Vec::new();
    for entries in [&[][..], &[".chough/"][..]] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        vip_git(root, &["init", "-b", "main"]);
        vip_git(root, &["config", "user.email", "test@example.com"]);
        vip_git(root, &["config", "user.name", "Test"]);
        vip_git(root, &["config", "core.autocrlf", "false"]);
        vip_write(root, "src/lib.rs", "pub fn ready() -> bool { false }\n");
        write_lifecycle_test_policy(root);
        if !entries.is_empty() {
            vip_set_list(root, entries);
        }
        vip_git(root, &["add", "."]);
        vip_git(root, &["commit", "-m", "base"]);
        vip_git(root, &["switch", "-c", "feature"]);
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        vip_git(root, &["add", "."]);
        vip_git(root, &["commit", "-m", "implement"]);
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        vip_git(root, &["add", "."]);
        vip_git(root, &["commit", "-m", "accept"]);
        vip_git(root, &["switch", "main"]);
        vip_git(root, &["merge", "--squash", "feature"]);
        vip_git(root, &["commit", "-m", "squash feature"]);
        vip_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
        if !entries.is_empty() {
            // A record the other tool writes after the squash changes nothing either.
            vip_write(root, ".chough/obs.json", "after the squash\n");
        }
        let (_anchor, state_bytes, accepted) = authenticated_accepted_transition(root, &record)
            .unwrap_or_else(|error| panic!("{entries:?}: {error}"));
        assert!(ensure_closing_approval_valid(root, &record).is_ok(), "{entries:?}");
        assert!(accepted_change_is_recorded_in_current_history(root, &record));
        let text = String::from_utf8(state_bytes).unwrap();
        assert!(!text.contains("verification_ignored"), "{text}");
        let state: serde_json::Value = serde_json::from_str(&text).unwrap();
        accepted_states.push((
            state.as_object().unwrap().keys().cloned().collect::<Vec<_>>(),
            accepted.state,
        ));
    }
    assert_eq!(accepted_states[0], accepted_states[1]);
}

// CONTROL. Archived evidence is not re-verified against the working-tree digest: adding the list
// to the working tree after archiving, then writing under it and editing it, leaves the archived
// change's terminal evidence (the full archive-integrity walk, `check_project`) exactly as it was.
// The only new finding either walk may report is the existing policy-change guard, which names any
// `sdd.json` edit no change covers (the docs say editing the list needs a covering change); a
// binary that ignores the key does not see an edit at all.
// Verifies REQ-change-114.
#[test]
fn archived_change_is_unaffected_by_a_later_list() {
    let (_temp, root, id) = vip_fixture(&[]);
    record_scoped_review(&root, &id, "Independent reviewer".into()).unwrap();
    finalize_change(&root, &id).unwrap();
    vip_git(&root, &["add", "--all"]);
    vip_git(&root, &["commit", "-m", "finalize"]);
    let before = (audit_project(&root), check_project(&root));
    assert!(
        before.1.terminal_evidence.iter().any(|result| result.id == id),
        "the archive-integrity walk must examine the archived change: {:?}",
        before.1
    );
    vip_set_list(&root, &[".chough/"]);
    vip_write(&root, ".chough/obs.json", "written after the archive\n");
    let listed = (audit_project(&root), check_project(&root));
    vip_set_list(&root, &[".chough/", ".tool-cache/"]);
    vip_write(&root, ".chough/obs.json", "edited after the archive\n");
    let edited = (audit_project(&root), check_project(&root));
    let policy_guard = "committed SDD policy changed without";
    for (audit, full) in [&listed, &edited] {
        for (after, baseline) in [(audit, &before.0), (full, &before.1)] {
            for error in &after.errors {
                assert!(
                    baseline.errors.contains(error) || error.contains(policy_guard),
                    "{error}"
                );
            }
            assert_eq!(after.warnings, baseline.warnings);
        }
        assert_eq!(full.terminal_evidence, before.1.terminal_evidence);
    }
}

// CONTROL. No persisted record gains a field: the top-level keys of verification.json,
// review.json and finalization.json are the same with and without the list.
// Verifies REQ-change-114.
#[test]
fn no_persisted_record_gains_a_field() {
    let keys = |path: &Path| -> Vec<String> {
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(path).unwrap_or_else(|error| {
                panic!("{}: {error}", path.display())
            }))
            .unwrap();
        value.as_object().unwrap().keys().cloned().collect()
    };
    let mut observed = Vec::new();
    for entries in [&[][..], &[".chough/"][..]] {
        let (_temp, root, id) = vip_fixture(entries);
        let workspace = root.join(CHANGES_PATH).join(&id);
        let verification = keys(&workspace.join("verification.json"));
        record_scoped_review(&root, &id, "Independent reviewer".into()).unwrap();
        let review = keys(&workspace.join(SCOPED_REVIEW_FILE));
        let archive = finalize_change(&root, &id).unwrap();
        let finalization = keys(&archive.join("finalization.json"));
        let archived_verification = keys(&archive.join("verification.json"));
        for text in [
            fs::read_to_string(archive.join("verification.json")).unwrap(),
            fs::read_to_string(archive.join(SCOPED_REVIEW_FILE)).unwrap(),
            fs::read_to_string(archive.join("finalization.json")).unwrap(),
        ] {
            assert!(!text.contains("verification_ignored"), "{text}");
        }
        observed.push((verification, review, finalization, archived_verification));
    }
    assert_eq!(observed[0], observed[1]);
}

// CONTROL. A change's acceptance manifest does not depend on the list (the policy file is not one
// of this change's covered inputs).
// Verifies REQ-change-114.
#[test]
fn acceptance_input_digest_is_unchanged_by_the_list() {
    let (_temp, root, id) = vip_fixture(&[]);
    let record = load_change(&root, &id).unwrap();
    let without = acceptance_manifest(&root, &record, &[]).unwrap();
    vip_set_list(&root, &[".chough/"]);
    vip_write(&root, ".chough/obs.json", "record\n");
    let with = acceptance_manifest(&root, &record, &[]).unwrap();
    assert_eq!(
        acceptance_manifest_digest(&with).unwrap(),
        acceptance_manifest_digest(&without).unwrap()
    );
}


// DISCRIMINATOR (NEW API: `frame_verification_ignored_paths` does not exist on a80748ff, so this
// test cannot compile there). The list frame keeps a digest computed with a list distinct from one
// without it and from any tree whose paths or bytes spell the tags; an empty list adds nothing.
// Verifies REQ-change-114.
#[test]
fn verification_ignored_frame_cannot_collide_with_path_frames() {
    let kind = acceptance_kind_bytes(&AcceptanceInputKind::File);
    let fixed = |digest: &mut FramedDigest| digest.entry("src/lib.rs", kind, 0o100644, b"code\n");
    let with_list = |entries: &[&str]| {
        let entries: Vec<String> = entries.iter().map(|entry| entry.to_string()).collect();
        let mut digest = FramedDigest::new(PROJECT_DIGEST_DOMAIN);
        frame_verification_ignored_paths(&mut digest, &entries);
        fixed(&mut digest);
        digest.finish()
    };
    let without = {
        let mut digest = FramedDigest::new(PROJECT_DIGEST_DOMAIN);
        fixed(&mut digest);
        digest.finish()
    };
    let spoof_content = {
        let mut digest = FramedDigest::new(PROJECT_DIGEST_DOMAIN);
        digest.entry("verification-ignored", kind, 0o100644, &1_u64.to_be_bytes());
        digest.entry("verification-ignored-prefix", kind, 0o100644, b".chough/");
        fixed(&mut digest);
        digest.finish()
    };
    let spoof_object = {
        let mut digest = FramedDigest::new(PROJECT_DIGEST_DOMAIN);
        digest.object_entry("verification-ignored", kind, 0o100644, &1_u64.to_be_bytes());
        digest.object_entry("verification-ignored-prefix", kind, 0o100644, b".chough/");
        fixed(&mut digest);
        digest.finish()
    };
    let listed = with_list(&[".chough/"]);
    assert_eq!(with_list(&[]), without, "an empty list adds no frame");
    for other in [
        &without,
        &with_list(&[".chough/", "x/"]),
        &with_list(&[".tool/"]),
        &spoof_content,
        &spoof_object,
    ] {
        assert_ne!(&listed, other);
    }
    assert!(path_is_verification_ignored(".chough/a", &[".chough/".to_string()]));
    assert!(!path_is_verification_ignored(".chough", &[".chough/".to_string()]));
    assert!(!path_is_verification_ignored("notes/.chough/a", &[".chough/".to_string()]));
}
