// A repository can commit `.specsync/change-transaction.json`, and every writing change command
// replays the journal when it takes the lock. Replay is for the person's own interrupted write:
// a journal Git tracks is never replayed, an entry may only name a file the change workflow
// writes, a `.git` component is refused in any letter case, and the 5.x bare-array form is no
// longer replayed (SETUP-8, REQ-change-162).

/// A project with a spec, its requirements and a Git repository, so `.git/` is real.
fn journal_project() -> TempDir {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(root.join("specs/auth/auth.spec.md"), "# Auth\n").unwrap();
    fs::write(root.join("specs/auth/requirements.md"), "# Requirements\n").unwrap();
    for args in [
        &["init", "-q"][..],
        &["config", "user.email", "journal@example.invalid"][..],
        &["config", "user.name", "journal test"][..],
        &["config", "commit.gpgsign", "false"][..],
    ] {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    }
    temp
}

fn journal_git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap()
            .success(),
        "git {args:?}"
    );
}

fn entry(path: &str, original: Option<&str>) -> TransactionEntry {
    TransactionEntry {
        path: path.to_string(),
        original: original.map(str::to_string),
    }
}

/// The journal exactly as this build writes one, integrity envelope and all.
fn durable_journal(entries: Vec<TransactionEntry>) -> String {
    serde_json::to_string(&TransactionJournal {
        schema_version: 1,
        entry_count: entries.len(),
        entries_digest: transaction_entries_digest(&entries).unwrap(),
        entries,
    })
    .unwrap()
}

fn plant_journal(root: &Path, journal: &str) {
    fs::write(root.join(TRANSACTION_PATH), journal).unwrap();
}

/// Verifies REQ-change-162. No journal entry may rewrite or delete a file under `.git`, in any
/// letter case or with a trailing dot, so a repository cannot set `core.fsmonitor` or replace a
/// hook through a journal it commits.
#[test]
fn a_journal_never_rewrites_or_deletes_git_files() {
    for (target, original) in [
        (".git/config", Some("[core]\n\tfsmonitor = touch planted\n")),
        (".GIT/config", Some("[core]\n\tfsmonitor = touch planted\n")),
        (".git./config", Some("[core]\n\tfsmonitor = touch planted\n")),
        (".git/HEAD", None),
        (".git/hooks/pre-commit", Some("#!/bin/sh\ntouch planted\n")),
    ] {
        let temp = journal_project();
        let root = temp.path();
        fs::create_dir_all(root.join(".git/hooks")).unwrap();
        fs::write(root.join(".git/hooks/pre-commit"), "#!/bin/sh\nexit 0\n").unwrap();
        let config = fs::read(root.join(".git/config")).unwrap();
        let head = fs::read(root.join(".git/HEAD")).unwrap();
        let hook = fs::read(root.join(".git/hooks/pre-commit")).unwrap();
        plant_journal(root, &durable_journal(vec![entry(target, original)]));

        let error = recover_pending_transaction(root).unwrap_err();

        assert_eq!(
            fs::read(root.join(".git/config")).unwrap(),
            config,
            "{target}: the journal rewrote .git/config"
        );
        assert_eq!(
            fs::read(root.join(".git/HEAD")).unwrap(),
            head,
            "{target}: the journal touched .git/HEAD"
        );
        assert_eq!(
            fs::read(root.join(".git/hooks/pre-commit")).unwrap(),
            hook,
            "{target}: the journal rewrote a hook"
        );
        assert!(
            !root.join(".GIT").exists() || root.join(".git").exists(),
            "{target}"
        );
        assert!(
            !root.join(".git.").exists(),
            "{target}: the journal created a .git. directory"
        );
        assert!(
            error.contains(&format!("entry `{target}`")) && error.contains("Git's own files"),
            "{target}: {error}"
        );
        assert!(
            root.join(TRANSACTION_PATH).exists(),
            "{target}: a refused journal is left for the person to inspect"
        );
    }
}

/// Verifies REQ-change-162. A journal Git tracks came with the repository, so it is never
/// replayed, even when every entry names an allowed file: in the index, or only in `HEAD`.
#[test]
fn a_journal_git_tracks_is_never_replayed() {
    for staged_out in [false, true] {
        let temp = journal_project();
        let root = temp.path();
        plant_journal(
            root,
            &durable_journal(vec![entry("specs/auth/auth.spec.md", Some("TAMPERED\n"))]),
        );
        journal_git(root, &["add", "-A"]);
        journal_git(root, &["commit", "-q", "-m", "fixture"]);
        if staged_out {
            journal_git(root, &["rm", "-q", "--cached", TRANSACTION_PATH]);
        }

        let error = recover_pending_transaction(root).unwrap_err();

        assert_eq!(
            fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap(),
            "# Auth\n",
            "a tracked journal was replayed (staged out: {staged_out})"
        );
        assert!(
            error.contains("tracked by Git") && error.contains("git rm"),
            "{error}"
        );
    }
}

/// Verifies REQ-change-162. One entry outside what the change workflow writes refuses the
/// whole journal: nothing is replayed, not even the allowed entries before it.
#[test]
fn one_entry_outside_the_change_workflow_refuses_the_whole_journal() {
    let temp = journal_project();
    let root = temp.path();
    plant_journal(
        root,
        &durable_journal(vec![
            entry("specs/auth/auth.spec.md", Some("RESTORED\n")),
            entry("Makefile", Some("all:\n\ttouch planted\n")),
        ]),
    );

    let error = recover_pending_transaction(root).unwrap_err();

    assert!(!root.join("Makefile").exists(), "an entry outside was written");
    assert_eq!(
        fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap(),
        "# Auth\n",
        "part of a refused journal was replayed"
    );
    assert!(
        error.contains("entry `Makefile`") && error.contains(".broken"),
        "{error}"
    );
}

/// Verifies REQ-change-162. The bare-array form only 5.x wrote has no integrity envelope, so it
/// is no longer replayed.
#[test]
fn a_bare_array_journal_is_no_longer_replayed() {
    let temp = journal_project();
    let root = temp.path();
    plant_journal(
        root,
        r#"[{"path":"specs/auth/auth.spec.md","original":"RESTORED\n"}]"#,
    );

    let error = recover_pending_transaction(root).unwrap_err();

    assert_eq!(
        fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap(),
        "# Auth\n"
    );
    assert!(
        error.contains("5.x") && error.contains(".broken"),
        "{error}"
    );
}

/// Verifies REQ-change-162. CONTROL: the person's own interrupted write is still put back, in
/// the spec tree, in a change record, and as a deletion of a file the write created.
#[test]
fn a_real_interrupted_write_is_still_recovered() {
    let temp = journal_project();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/auth/history")).unwrap();
    fs::write(root.join("specs/auth/history/new-change.md"), "half\n").unwrap();
    fs::create_dir_all(root.join(".specsync/changes/new-change")).unwrap();
    fs::write(root.join(".specsync/changes/new-change/state.json"), "half").unwrap();
    plant_journal(
        root,
        &durable_journal(vec![
            entry("specs/auth/auth.spec.md", Some("# Auth before\n")),
            entry("specs/auth/requirements.md", Some("# Requirements before\n")),
            entry("specs/auth/history/new-change.md", None),
            entry(".specsync/changes/new-change/state.json", Some("{}")),
        ]),
    );

    recover_pending_transaction(root).unwrap();

    assert_eq!(
        fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap(),
        "# Auth before\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("specs/auth/requirements.md")).unwrap(),
        "# Requirements before\n"
    );
    assert!(!root.join("specs/auth/history/new-change.md").exists());
    assert_eq!(
        fs::read_to_string(root.join(".specsync/changes/new-change/state.json")).unwrap(),
        "{}"
    );
    assert!(!root.join(TRANSACTION_PATH).exists());
}

/// Verifies REQ-change-162. A spec the local registry maps outside `specs_dir` is written by the
/// change workflow too, so its spec, `requirements.md` and history fragments are recovered; a
/// neighbouring file it does not write is not.
#[test]
fn a_registry_mapped_spec_is_recovered_and_its_neighbours_are_not() {
    let temp = journal_project();
    let root = temp.path();
    fs::create_dir_all(root.join("docs/api/history")).unwrap();
    fs::write(root.join("docs/api/api.spec.md"), "# Api\n").unwrap();
    fs::write(
        root.join(".specsync/registry.toml"),
        "[registry]\nname = \"journal\"\n\n[specs]\napi = \"docs/api/api.spec.md\"\n",
    )
    .unwrap();
    plant_journal(
        root,
        &durable_journal(vec![
            entry("docs/api/api.spec.md", Some("# Api before\n")),
            entry("docs/api/requirements.md", Some("# Api requirements\n")),
            entry("docs/api/history/new-change.md", None),
        ]),
    );
    recover_pending_transaction(root).unwrap();
    assert_eq!(
        fs::read_to_string(root.join("docs/api/api.spec.md")).unwrap(),
        "# Api before\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("docs/api/requirements.md")).unwrap(),
        "# Api requirements\n"
    );

    plant_journal(
        root,
        &durable_journal(vec![entry("docs/api/notes.md", Some("planted\n"))]),
    );
    let error = recover_pending_transaction(root).unwrap_err();
    assert!(!root.join("docs/api/notes.md").exists());
    assert!(error.contains("entry `docs/api/notes.md`"), "{error}");
}

/// Verifies REQ-change-162. The write turns a backslash into a separator, so the path rule must
/// too: `.git\config` is `.git/config`, and `specs/.git\config` reaches a `.git` component under
/// the spec tree. A repository config of `specs_dir = "."` must not let either through.
#[test]
fn a_backslash_entry_cannot_reach_git_files() {
    for (specs_dir, target) in [(".", ".git\\config"), ("specs", "specs/.git\\config")] {
        let temp = journal_project();
        let root = temp.path();
        fs::write(
            root.join(".specsync/config.toml"),
            format!("specs_dir = \"{specs_dir}\"\nsource_dirs = [\"src\"]\n"),
        )
        .unwrap();
        let config = fs::read(root.join(".git/config")).unwrap();
        plant_journal(
            root,
            &durable_journal(vec![entry(target, Some("[core]\n\tfsmonitor = touch planted\n"))]),
        );

        let error = recover_pending_transaction(root).unwrap_err();

        assert_eq!(
            fs::read(root.join(".git/config")).unwrap(),
            config,
            "{target} rewrote .git/config"
        );
        assert!(
            !root.join("specs/.git").exists(),
            "{target} created a .git directory in the spec tree"
        );
        assert!(error.contains("Git's own files"), "{target}: {error}");
    }
}

/// Verifies REQ-change-162. Under `specs_dir` the change workflow writes only spec files,
/// `requirements.md` and history fragments, so a `specs_dir` of `.` does not open the whole tree
/// to a replay.
#[test]
fn specs_dir_dot_does_not_widen_replay() {
    let temp = journal_project();
    let root = temp.path();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \".\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    plant_journal(
        root,
        &durable_journal(vec![entry("Makefile", Some("all:\n\ttouch planted\n"))]),
    );
    let error = recover_pending_transaction(root).unwrap_err();
    assert!(!root.join("Makefile").exists(), "specs_dir . admitted Makefile");
    assert!(error.contains("entry `Makefile`"), "{error}");

    fs::create_dir_all(root.join("auth/history")).unwrap();
    plant_journal(
        root,
        &durable_journal(vec![
            entry("auth/auth.spec.md", Some("# Auth\n")),
            entry("auth/requirements.md", Some("# Requirements\n")),
            entry("auth/history/a-change.md", Some("---\n")),
        ]),
    );
    recover_pending_transaction(root).unwrap();
    assert_eq!(
        fs::read_to_string(root.join("auth/auth.spec.md")).unwrap(),
        "# Auth\n"
    );
}

