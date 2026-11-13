// Archived lifecycle reads stay cheap however many archives a project holds (REQ-change-119).
//
// The guards here are hardware independent: they count the Git processes one read spawns rather
// than timing it. `raven_sized_archived_status_benchmark` is the timed benchmark and is run on
// demand with `fledge run bench-archived-reads`.

fn fast_read_repository(root: &Path) {
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    quiet_git(root, &["add", "README.md"]);
    quiet_git(root, &["commit", "-m", "base"]);
}

/// Drives one no-spec change through approve, check, review and finalize, then commits the
/// archive the way a merged pull request carries it.
fn finalized_archived_change(root: &Path) -> ChangeRecord {
    let record = current_workflow_record(root, completed_no_spec_record(root));
    // `create_change` re-adopts workflow v2 at the current HEAD; a project keeps the baseline it
    // committed when it first adopted, so the fixture does too.
    if quiet_git(root, &["ls-files", WORKFLOW_V2_BASELINE_PATH]).is_empty() {
        quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
        quiet_git(root, &["commit", "-m", "adopt workflow v2"]);
    } else {
        quiet_git(root, &["checkout", "--", WORKFLOW_V2_BASELINE_PATH]);
    }
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "implement approved change"]);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    finalize_change(root, &record.id).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "archive change"]);
    load_change(root, &record.id).unwrap()
}

/// One `change status <id> --json` read, as the command layer performs it.
fn archived_status_read(root: &Path, id: &str) -> Result<String, String> {
    let record = load_change(root, id)?;
    let summary = summarize_change_with_strict(root, &record, false);
    serde_json::to_string(&summary).map_err(|error| error.to_string())
}

fn git_processes_for_one_scoped_read(root: &Path, id: &str) -> (Result<String, String>, usize) {
    let _scope = begin_change_read_scope(root);
    reset_test_git_process_count();
    let result = archived_status_read(root, id);
    (result, test_git_process_count())
}

#[test]
fn archived_status_git_processes_do_not_grow_with_other_archives() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let target = finalized_archived_change(root);
    finalized_archived_change(root);
    git_processes_for_one_scoped_read(root, &target.id).0.unwrap();
    let (first, with_two) = git_processes_for_one_scoped_read(root, &target.id);
    first.unwrap();

    for _ in 0..3 {
        finalized_archived_change(root);
    }
    git_processes_for_one_scoped_read(root, &target.id).0.unwrap();
    let (second, with_five) = git_processes_for_one_scoped_read(root, &target.id);
    second.unwrap();

    assert!(
        with_five < with_two + 3,
        "three more archived changes added {} Git processes to one archived status read \
         ({with_two} with two archives, {with_five} with five)",
        with_five.saturating_sub(with_two)
    );
}

/// A repository whose first `legacy` changes were made under workflow v1 and committed as the
/// trusted pre-v2 cutoff, then adopted workflow v2 and finalized one change. Returns that
/// archived change. Most real archives look like this: their archived workflow-v1 changes must
/// each be shown to have existed at the cutoff.
fn repository_with_legacy_changes(root: &Path, legacy: usize) -> ChangeRecord {
    legacy_repository(root, legacy);
    finalized_archived_change(root)
}

/// A repository with `legacy` archived workflow-v1 changes committed as the trusted pre-v2
/// cutoff, then adopted to workflow v2.
fn legacy_repository(root: &Path, legacy: usize) {
    legacy_repository_with(root, legacy, |_| {});
}

/// `legacy_repository`, running `before_cutoff` after the legacy archives and before the trusted
/// cutoff commit.
fn legacy_repository_with(root: &Path, legacy: usize, before_cutoff: impl FnOnce(&Path)) {
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.version = 1;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    for _ in 0..legacy {
        let record = completed_no_spec_record(root);
        assert_eq!(record.workflow_version, 1);
        // Carried through the workflow-v1 lifecycle to its archive, as most real legacy
        // changes were before their project adopted workflow v2.
        approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
        start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        accept_change(root, &record.id, Some("Acceptor".into()), None).unwrap();
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "accept legacy change"]);
        quiet_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
        archive_change(root, &record.id).unwrap();
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "archive legacy change"]);
    }
    before_cutoff(root);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "--allow-empty", "-m", "trusted workflow-v1 cutoff"]);
    quiet_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    adopt(root, false, None).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "adopt workflow v2"]);
}

/// Git processes one warm `change status` of an archived change spawns in a repository holding
/// `legacy` workflow-v1 changes.
fn warm_archived_status_processes_with_legacy_changes(legacy: usize) -> usize {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let target = repository_with_legacy_changes(root, legacy);
    git_processes_for_one_scoped_read(root, &target.id).0.unwrap();
    let (result, processes) = git_processes_for_one_scoped_read(root, &target.id);
    result.unwrap();
    processes
}

// S9 (SPEED-1.a): an archived change's status must not walk history once per workflow-v1 change
// in the project. On this repository 73 such walks are about 49 s of a warm read.
#[test]
fn warm_archived_status_git_processes_do_not_grow_with_legacy_changes() {
    let with_two = warm_archived_status_processes_with_legacy_changes(2);
    let with_eight = warm_archived_status_processes_with_legacy_changes(8);
    eprintln!(
        "warm archived status Git processes: {with_two} with two workflow-v1 changes, \
         {with_eight} with eight"
    );
    assert!(
        with_eight < with_two + 3,
        "six more workflow-v1 changes added {} Git processes to one warm archived status read \
         ({with_two} with two, {with_eight} with eight)",
        with_eight.saturating_sub(with_two)
    );
}

// REQ-change-136: the cutoff-tree answer equals the history-walk answer it replaces.
#[test]
fn cutoff_tree_verdict_equals_the_history_walk_verdict() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    repository_with_legacy_changes(root, 2);
    let cutoff = read_workflow_v2_baseline(root)
        .unwrap()
        .unwrap()
        .0
        .cutoff_commit
        .unwrap();
    let records = list_all_changes_checked(root).unwrap();
    let legacy: Vec<&ChangeRecord> = records
        .values()
        .filter(|record| record.workflow_version == 1)
        .collect();
    assert_eq!(legacy.len(), 2);
    for record in &legacy {
        assert!(workflow_v1_record_exists_at_cutoff(root, record, &cutoff).unwrap());
        assert!(workflow_v1_record_present_in_cutoff_tree(root, record, &cutoff).unwrap());
    }

    // A workflow-v1 record that first appears after the cutoff.
    let mut late = (*legacy[0]).clone();
    late.id = "late-legacy-change".into();
    let late_dir = root.join(CHANGES_PATH).join(late.id.as_str());
    fs::create_dir_all(&late_dir).unwrap();
    write_json(&late_dir.join("state.json"), &late).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "workflow-v1 record after the cutoff"]);
    let walked = workflow_v1_record_exists_at_cutoff(root, &late, &cutoff).unwrap();
    assert!(!walked, "a record added after the cutoff is absent at it");
    assert_eq!(
        workflow_v1_record_present_in_cutoff_tree(root, &late, &cutoff).unwrap(),
        walked
    );

    // A presence is remembered in the persistent read cache under the immutable cutoff; the
    // absence above is a refusal and is never stored.
    {
        let _scope = begin_change_read_scope(root);
        assert!(workflow_v1_record_present_in_cutoff_tree(root, legacy[0], &cutoff).unwrap());
        assert!(!workflow_v1_record_present_in_cutoff_tree(root, &late, &cutoff).unwrap());
    }
    let cache: serde_json::Value = serde_json::from_slice(
        &fs::read(persistent_read_cache_path(root).expect("the read cache is enabled")).unwrap(),
    )
    .unwrap();
    let remembered = cache["workflow_v1_at_cutoff"].as_array().unwrap();
    assert!(remembered.contains(&serde_json::json!(format!("{cutoff}\n{}", legacy[0].id))));
    assert!(!remembered.contains(&serde_json::json!(format!("{cutoff}\n{}", late.id))));
}

// #939 (SPEED-1.a.3, REQ-change-145): when the cutoff's archive tree cannot be read, as in an
// offline `--filter=tree:0` clone that never fetched it, the cutoff-tree check must say it could
// not read it rather than answer that the workflow-v1 change was not there.
#[test]
fn an_unreadable_cutoff_archive_tree_is_an_error_not_an_absent_change() {
    let temp = TempDir::new().unwrap();
    let origin = temp.path().join("origin");
    fs::create_dir_all(&origin).unwrap();
    // A workflow-v2 change archived after adoption changes the archive tree, so the tree at the
    // cutoff is an older object that a `tree:0` clone does not hold.
    repository_with_legacy_changes(&origin, 1);
    let cutoff = read_workflow_v2_baseline(&origin)
        .unwrap()
        .unwrap()
        .0
        .cutoff_commit
        .unwrap();
    let legacy: ChangeRecord = list_all_changes_checked(&origin)
        .unwrap()
        .into_values()
        .find(|record| record.workflow_version == 1)
        .unwrap();
    assert!(workflow_v1_record_present_in_cutoff_tree(&origin, &legacy, &cutoff).unwrap());

    quiet_git(&origin, &["config", "uploadpack.allowFilter", "true"]);
    let url = format!("file://{}", origin.display());
    quiet_git(temp.path(), &["clone", "-q", "--filter=tree:0", url.as_str(), "clone"]);
    let clone = temp.path().join("clone");
    quiet_git(&clone, &["remote", "set-url", "origin", "file:///nonexistent-promisor"]);

    let answer = workflow_v1_record_present_in_cutoff_tree(&clone, &legacy, &cutoff);
    assert!(
        answer
            .as_ref()
            .is_err_and(|error| error.contains("could not be read")),
        "an unreadable cutoff archive tree answered {answer:?}"
    );
}

// #940 (SPEED-1.a.2, REQ-change-146): `find_change_dir` identifies an archive by the id its
// state declares, not by its directory name. A workflow-v1 archive moved before the cutoff to a
// directory that does not end in `-<id>` was accepted by the history walk, so the cutoff-tree
// check must accept it too.
#[test]
fn cutoff_tree_finds_a_workflow_v1_archive_renamed_before_the_cutoff() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    legacy_repository_with(root, 1, |root| {
        let archive = fs::read_dir(root.join(ARCHIVE_PATH))
            .unwrap()
            .flatten()
            .next()
            .unwrap()
            .path();
        fs::rename(&archive, archive.with_file_name("renamed-archive")).unwrap();
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "move the legacy archive"]);
    });
    let cutoff = read_workflow_v2_baseline(root)
        .unwrap()
        .unwrap()
        .0
        .cutoff_commit
        .unwrap();
    let renamed = root.join(ARCHIVE_PATH).join("renamed-archive");
    let record: ChangeRecord =
        serde_json::from_slice(&fs::read(renamed.join("state.json")).unwrap()).unwrap();
    assert_eq!(record.workflow_version, 1);
    assert_eq!(find_change_dir(root, &record.id).unwrap(), renamed);
    assert!(
        workflow_v1_record_exists_at_cutoff(root, &record, &cutoff).unwrap(),
        "the history walk accepts the moved archive"
    );
    assert!(
        workflow_v1_record_present_in_cutoff_tree(root, &record, &cutoff).unwrap(),
        "the cutoff-tree check must accept what the history walk accepts"
    );
    load_change(root, &record.id).expect("the moved workflow-v1 archive loads");
}

/// Git processes one archived status read spawns with no persistent read cache.
fn cold_archived_status_processes(root: &Path, id: &str) -> usize {
    if let Some(path) = persistent_read_cache_path(root) {
        let _ = fs::remove_file(path);
    }
    let (result, processes) = git_processes_for_one_scoped_read(root, id);
    result.unwrap();
    processes
}

/// Git processes building the archive-introduction index spawns with no persistent read cache.
fn cold_archive_introduction_index_processes(root: &Path) -> usize {
    if let Some(path) = persistent_read_cache_path(root) {
        let _ = fs::remove_file(path);
    }
    let _scope = begin_change_read_scope(root);
    reset_test_git_process_count();
    archive_introduction_index(root).unwrap();
    test_git_process_count()
}

// S9 (REQ-change-135): the archive-introduction index is built from one walk, so a cold build
// must not cost Git processes per archive-introducing commit.
#[test]
fn cold_archive_introduction_index_does_not_grow_with_archive_introductions() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    finalized_archived_change(root);
    finalized_archived_change(root);
    let with_two = cold_archive_introduction_index_processes(root);
    for _ in 0..4 {
        finalized_archived_change(root);
    }
    let with_six = cold_archive_introduction_index_processes(root);
    eprintln!("cold archive-introduction index Git processes: {with_two} with two archives, {with_six} with six");
    assert!(
        with_six < with_two + 3,
        "four more archive-introducing commits added {} Git processes to a cold index build \
         ({with_two} with two archives, {with_six} with six)",
        with_six.saturating_sub(with_two)
    );
}

/// Git processes four more archives may add to one cold archived status read: each archive's own
/// workflow-version history is still checked once, by a prefetch worker that starts from the
/// command's repository facts (REQ-git-utils-011). Measured 135; before workers were seeded, 250.
const COLD_ARCHIVED_STATUS_GIT_PROCESSES_FOR_FOUR_ARCHIVES: usize = 150;

// S9 (REQ-git-utils-011): prefetch workers start from what the command already knows, so a cold
// archived status does not re-ask repository facts once per archive.
#[test]
fn cold_archived_status_prefetch_workers_reuse_repository_facts() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let target = finalized_archived_change(root);
    finalized_archived_change(root);
    // Workers run only when the lifecycle-state history index cannot answer (REQ-change-141), so
    // this guard turns the index off to keep measuring them.
    let with_two =
        without_lifecycle_state_history(|| cold_archived_status_processes(root, &target.id));
    for _ in 0..4 {
        finalized_archived_change(root);
    }
    let with_six =
        without_lifecycle_state_history(|| cold_archived_status_processes(root, &target.id));
    eprintln!("cold archived status Git processes: {with_two} with two archives, {with_six} with six");
    assert!(
        with_six <= with_two + COLD_ARCHIVED_STATUS_GIT_PROCESSES_FOR_FOUR_ARCHIVES,
        "four more archives added {} Git processes to one cold archived status read \
         ({with_two} with two archives, {with_six} with six)",
        with_six.saturating_sub(with_two)
    );
}

// #915 (SPEED-1.a.1, REQ-change-140 and 141): a cold archived status must not cost Git processes
// per archive. Before the lifecycle-state history index, each archive's own workflow-version
// history check cost about 34 (99 with two archives, 234 with six).
#[test]
fn cold_archived_status_git_processes_do_not_grow_with_archives() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let target = finalized_archived_change(root);
    finalized_archived_change(root);
    let with_two = cold_archived_status_processes(root, &target.id);
    for _ in 0..4 {
        finalized_archived_change(root);
    }
    let with_six = cold_archived_status_processes(root, &target.id);
    eprintln!("cold archived status Git processes: {with_two} with two archives, {with_six} with six");
    assert!(
        with_six < with_two + 3,
        "four more archives added {} Git processes to one cold archived status read \
         ({with_two} with two archives, {with_six} with six)",
        with_six.saturating_sub(with_two)
    );
}

// REQ-change-135: one `diff-tree --stdin` lists exactly what the per-commit reads list,
// including a merge commit that brings an archive in.
#[test]
fn one_walk_archive_introductions_equal_per_commit_introductions() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let first = finalized_archived_change(root);
    finalized_archived_change(root);
    // A side branch brings in one more archived package, and a merge commit adds it to main.
    let archive_root = git_repo_relative_path(root, ARCHIVE_PATH).unwrap();
    let source = find_change_dir(root, &first.id).unwrap();
    quiet_git(root, &["switch", "-q", "-c", "side"]);
    let copy = root
        .join(&archive_root)
        .join(format!("2026-01-02-{}", first.id));
    fs::create_dir_all(&copy).unwrap();
    for entry in fs::read_dir(&source).unwrap().flatten() {
        if entry.path().is_file() {
            fs::copy(entry.path(), copy.join(entry.file_name())).unwrap();
        }
    }
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-q", "-m", "side archive"]);
    quiet_git(root, &["switch", "-q", "main"]);
    fs::write(root.join("README.md"), "main moves on\n").unwrap();
    quiet_git(root, &["commit", "-q", "-am", "main moves on"]);
    quiet_git(root, &["merge", "-q", "--no-ff", "-m", "merge side archive", "side"]);
    let pathspec = format!(":(top,literal){archive_root}");
    let prefix = format!("{archive_root}/");
    let history = quiet_git(
        root,
        &[
            "log",
            "--format=%H",
            "--diff-filter=A",
            "--no-renames",
            "HEAD",
            "--",
            pathspec.as_str(),
        ],
    );
    let commits: Vec<&str> = history.lines().filter(|line| !line.is_empty()).collect();
    assert!(commits.len() >= 3, "{commits:?}");
    let listings = archive_introduction_listings(root, &commits, &pathspec)
        .expect("one walk lists every commit");
    for commit in &commits {
        let per_commit =
            archive_introductions_at_commit_uncached(root, commit, &pathspec, &prefix, None)
                .unwrap();
        let listed = listings.get(*commit).map(Vec::as_slice).unwrap_or(&[]);
        let one_walk =
            archive_introductions_at_commit_uncached(root, commit, &pathspec, &prefix, Some(listed))
                .unwrap();
        assert_eq!(one_walk, per_commit, "{commit}");
    }
    // Commit selection is unchanged: `log --diff-filter=A` without `-m` names the side branch's
    // own commit, not the merge, as the one that introduced the copied package.
    let side_state = format!("{archive_root}/2026-01-02-{}/state.json", first.id);
    let side = quiet_git(root, &["rev-parse", "side"]);
    assert!(
        listings
            .get(side.trim())
            .is_some_and(|paths| paths.contains(&side_state)),
        "the side commit lists {side_state}"
    );
}

#[test]
fn cached_archived_reads_answer_exactly_like_live_reads() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let first = finalized_archived_change(root);
    let second = finalized_archived_change(root);
    for record in [&first, &second] {
        let live = archived_status_read(root, &record.id).unwrap();
        let cold = git_processes_for_one_scoped_read(root, &record.id).0.unwrap();
        let warm = git_processes_for_one_scoped_read(root, &record.id).0.unwrap();
        assert_eq!(cold, live);
        assert_eq!(warm, live);
    }
    assert!(
        persistent_read_cache_path(root).is_some_and(|path| path.is_file()),
        "a scoped read should leave its verdicts under the Git directory"
    );
}

#[test]
fn cached_archived_history_still_refuses_a_later_hidden_downgrade() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    git_processes_for_one_scoped_read(root, &record.id).0.unwrap();

    let state_path = find_change_dir(root, &record.id).unwrap().join("state.json");
    let original = fs::read(&state_path).unwrap();
    let mut downgraded: serde_json::Value = serde_json::from_slice(&original).unwrap();
    let object = downgraded.as_object_mut().unwrap();
    object.remove("workflow_version");
    object.remove("workflow_origin_version");
    fs::write(
        &state_path,
        format!("{}\n", serde_json::to_string_pretty(&downgraded).unwrap()),
    )
    .unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "attempt workflow downgrade"]);
    fs::write(&state_path, &original).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "hide workflow downgrade"]);

    let live = archived_status_read(root, &record.id).unwrap_err();
    assert!(live.contains("workflow-version history changed immutable identity"));
    let scoped = git_processes_for_one_scoped_read(root, &record.id).0.unwrap_err();
    assert_eq!(scoped, live);
}

/// Rewrites the archived `state.json` without its workflow-version fields, commits that, then
/// commits the original bytes back, so the tip looks untouched but history holds a downgrade.
fn commit_hidden_workflow_downgrade(root: &Path, state_path: &Path) {
    let original = fs::read(state_path).unwrap();
    let mut downgraded: serde_json::Value = serde_json::from_slice(&original).unwrap();
    let object = downgraded.as_object_mut().unwrap();
    object.remove("workflow_version");
    object.remove("workflow_origin_version");
    fs::write(
        state_path,
        format!("{}\n", serde_json::to_string_pretty(&downgraded).unwrap()),
    )
    .unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "attempt workflow downgrade"]);
    fs::write(state_path, &original).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "hide workflow downgrade"]);
}

#[test]
fn cached_archived_history_refuses_a_hidden_downgrade_in_a_renamed_archive() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);

    // Move the archive to a directory whose name no longer ends in `-<id>`; `find_change_dir`
    // still finds it by the id its state declares.
    let original_dir = find_change_dir(root, &record.id).unwrap();
    let renamed_dir = original_dir.with_file_name("renamed-archive");
    fs::rename(&original_dir, &renamed_dir).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "rename archive directory"]);
    assert_eq!(find_change_dir(root, &record.id).unwrap(), renamed_dir);

    // A warm read caches a passing verdict for the renamed path.
    git_processes_for_one_scoped_read(root, &record.id).0.unwrap();
    git_processes_for_one_scoped_read(root, &record.id).0.unwrap();

    commit_hidden_workflow_downgrade(root, &renamed_dir.join("state.json"));

    let live = archived_status_read(root, &record.id).unwrap_err();
    assert!(live.contains("workflow-version history changed immutable identity"));
    let scoped = git_processes_for_one_scoped_read(root, &record.id).0.unwrap_err();
    assert_eq!(scoped, live);
}

#[test]
fn cached_archived_reads_recheck_an_edited_archive_and_a_moved_head() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    let warm = git_processes_for_one_scoped_read(root, &record.id).0.unwrap();

    // An uncommitted edit to the archived state changes its digest, so nothing cached applies.
    let state_path = find_change_dir(root, &record.id).unwrap().join("state.json");
    let original = fs::read(&state_path).unwrap();
    let mut edited: ChangeRecord = serde_json::from_slice(&original).unwrap();
    edited.workflow_origin_version = None;
    write_json(&state_path, &edited).unwrap();
    let live = archived_status_read(root, &record.id);
    assert_eq!(git_processes_for_one_scoped_read(root, &record.id).0, live);
    fs::write(&state_path, &original).unwrap();

    // A HEAD the cached verdict was not computed on an ancestor of recomputes it.
    quiet_git(root, &["checkout", "--quiet", "--orphan", "unrelated"]);
    quiet_git(root, &["commit", "--quiet", "-m", "unrelated history"]);
    let live = archived_status_read(root, &record.id);
    assert_eq!(git_processes_for_one_scoped_read(root, &record.id).0, live);
    quiet_git(root, &["checkout", "--quiet", "main"]);
    assert_eq!(
        git_processes_for_one_scoped_read(root, &record.id).0.unwrap(),
        warm
    );
}

#[test]
fn path_is_state_candidate_matches_only_one_change_s_state_paths() {
    let changes = ".specsync/changes";
    let archive = ".specsync/archive/changes";
    let id = "fix-it";
    assert!(path_is_state_candidate(".specsync/changes/fix-it/state.json", changes, archive, id));
    assert!(path_is_state_candidate(
        ".specsync/archive/changes/2026-01-01-fix-it/state.json",
        changes,
        archive,
        id
    ));
    assert!(!path_is_state_candidate(
        ".specsync/archive/changes/2026-01-01-fix-it/approvals.json",
        changes,
        archive,
        id
    ));
    assert!(!path_is_state_candidate(
        ".specsync/archive/changes/nested/2026-01-01-fix-it/state.json",
        changes,
        archive,
        id
    ));
    assert!(!path_is_state_candidate(
        ".specsync/archive/changes/2026-01-01-fix-it-too/state.json",
        changes,
        archive,
        id
    ));
}

/// The timed benchmark against a Raven-sized project: 124 archived changes, 120 specs and 24
/// archived workflow-v1 changes from before the project adopted workflow v2. Run on demand
/// (`fledge run bench-archived-reads`); `SPECSYNC_BENCH_ARCHIVES`, `SPECSYNC_BENCH_SPECS`,
/// `SPECSYNC_BENCH_LEGACY` and `SPECSYNC_BENCH_ACTIVE` change the fixture size.
#[test]
#[ignore = "timed benchmark; run with `fledge run bench-archived-reads`"]
fn raven_sized_archived_status_benchmark() {
    let size = |name: &str, default: usize| {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    };
    let archives = size("SPECSYNC_BENCH_ARCHIVES", 124);
    let specs = size("SPECSYNC_BENCH_SPECS", 120);
    let legacy = size("SPECSYNC_BENCH_LEGACY", 24);
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    if legacy == 0 {
        fast_read_repository(root);
    } else {
        legacy_repository(root, legacy);
    }
    for index in 0..specs {
        let module = format!("bench-{index}");
        fs::create_dir_all(root.join("specs").join(&module)).unwrap();
        fs::create_dir_all(root.join("bench")).unwrap();
        fs::write(
            root.join(format!("bench/{module}.rs")),
            format!("pub fn bench_{index}() {{}}\n"),
        )
        .unwrap();
        fs::write(
            root.join("specs").join(&module).join(format!("{module}.spec.md")),
            format!(
                "---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - bench/{module}.rs\n---\n\n# Bench {index}\n\n## Purpose\n\nBenchmark fixture.\n"
            ),
        )
        .unwrap();
    }
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "benchmark specs"]);
    let built = Instant::now();
    let mut ids = Vec::new();
    for _ in 0..archives {
        ids.push(finalized_archived_change(root).id);
    }
    eprintln!(
        "fixture: {archives} archived changes, {legacy} workflow-v1 archives, {specs} specs, built in {:.1}s",
        built.elapsed().as_secs_f64()
    );
    let target = ids.last().unwrap().clone();
    let time = |label: &str| {
        let started = Instant::now();
        let (result, processes) = git_processes_for_one_scoped_read(root, &target);
        result.unwrap();
        let elapsed = started.elapsed();
        eprintln!(
            "{label}: {:.3}s, {processes} Git processes",
            elapsed.as_secs_f64()
        );
        elapsed
    };
    let live_started = Instant::now();
    archived_status_read(root, &target).unwrap();
    eprintln!("unscoped: {:.3}s", live_started.elapsed().as_secs_f64());
    // Cold means no persistent cache at all, not merely a fresh process.
    if let Some(path) = persistent_read_cache_path(root) {
        let _ = fs::remove_file(path);
    }
    time("cold read scope");
    let warm = time("warm read scope");

    // `change list` reads every archive and the open changes beside them.
    let active = size("SPECSYNC_BENCH_ACTIVE", 3);
    for _ in 0..active {
        current_workflow_record(root, completed_no_spec_record(root));
    }
    let list = |label: &str| {
        let started = Instant::now();
        let _scope = begin_change_read_scope(root);
        reset_test_git_process_count();
        list_changes(root).unwrap();
        let elapsed = started.elapsed();
        eprintln!(
            "{label} ({active} active): {:.3}s, {} Git processes",
            elapsed.as_secs_f64(),
            test_git_process_count()
        );
        elapsed
    };
    list("change list, first");
    let warm_list = list("change list, warm");
    assert!(
        warm < Duration::from_secs(2),
        "a warm archived status read took {:.3}s against the 2 s budget",
        warm.as_secs_f64()
    );
    assert!(
        warm_list < Duration::from_secs(2),
        "a warm change list took {:.3}s against the 2 s budget",
        warm_list.as_secs_f64()
    );
}


/// A blob-less partial clone of a repository whose archive state changed after it was added,
/// with its promisor remote gone, so the blob the archive was introduced with cannot be read.
/// Returns the clone root and the introducing commit.
fn offline_partial_clone_missing_an_archive_blob(temp: &TempDir) -> (PathBuf, String, String) {
    let origin = temp.path().join("origin");
    fs::create_dir_all(&origin).unwrap();
    fast_read_repository(&origin);
    let state = format!("{ARCHIVE_PATH}/2026-01-01-offline/state.json");
    fs::create_dir_all(origin.join(&state).parent().unwrap()).unwrap();
    fs::write(origin.join(&state), "{\"introduced\":true}\n").unwrap();
    quiet_git(&origin, &["add", "--all"]);
    quiet_git(&origin, &["commit", "-m", "introduce archive"]);
    let introduction = quiet_git(&origin, &["rev-parse", "HEAD"]);
    fs::write(origin.join(&state), "{\"introduced\":false}\n").unwrap();
    quiet_git(&origin, &["commit", "-am", "edit archive"]);
    quiet_git(&origin, &["config", "uploadpack.allowFilter", "true"]);

    let clone = temp.path().join("clone");
    let url = format!("file://{}", origin.display());
    quiet_git(
        temp.path(),
        &["clone", "-q", "--filter=blob:none", url.as_str(), "clone"],
    );
    quiet_git(&clone, &["remote", "set-url", "origin", "file:///nonexistent-promisor"]);
    (clone, introduction, state)
}

#[test]
fn an_unreadable_introduced_archive_blob_is_never_cached_as_absent() {
    let temp = TempDir::new().unwrap();
    let (clone, introduction, _) = offline_partial_clone_missing_an_archive_blob(&temp);
    let archive_root = ARCHIVE_PATH.to_string();
    let (introduced, conclusive) = archive_introductions_at_commit_uncached(
        &clone,
        &introduction,
        &format!(":(top,literal){archive_root}"),
        &format!("{archive_root}/"),
        None,
    )
    .unwrap();
    assert!(introduced.is_empty());
    assert!(
        !conclusive,
        "diff-tree listed the archived state but its blob could not be read; that must not be \
         stored as 'introduced nothing'"
    );
}

#[test]
fn an_unreadable_blob_at_a_commit_is_an_error_not_an_absent_file() {
    let temp = TempDir::new().unwrap();
    let (clone, introduction, state) = offline_partial_clone_missing_an_archive_blob(&temp);
    let _scope = begin_change_read_scope(&clone);
    for _ in 0..2 {
        let read = scoped_review_file_at_commit(&clone, &introduction, &state);
        assert!(
            read.is_err(),
            "a path the commit's tree lists but whose blob is unavailable read as {read:?}"
        );
    }
    // A path the commit really does not have is still plainly absent.
    assert_eq!(
        scoped_review_file_at_commit(&clone, &introduction, "no-such-file").unwrap(),
        None
    );
}

#[test]
fn a_pass_is_never_stamped_with_a_head_that_moved_during_the_read() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    let state_path = find_change_dir(root, &record.id).unwrap().join("state.json");
    {
        let _scope = begin_change_read_scope(root);
        // The read starts: nothing cached yet, so the verdict would be computed live at this HEAD.
        assert!(!cached_workflow_history_is_current(root, &record));
        // HEAD moves mid-read to a commit whose history hides a downgrade.
        commit_hidden_workflow_downgrade(root, &state_path);
        // The pass computed for the earlier HEAD must not be recorded against the new one.
        remember_workflow_history_pass(root, &record);
    }
    let live = archived_status_read(root, &record.id).unwrap_err();
    assert!(live.contains("workflow-version history changed immutable identity"));
    let scoped = git_processes_for_one_scoped_read(root, &record.id).0.unwrap_err();
    assert_eq!(scoped, live);
}

fn cached_history_commit(root: &Path, id: &str) -> Option<String> {
    let bytes = fs::read(persistent_read_cache_path(root)?).ok()?;
    let cache: PersistentReadCache = serde_json::from_slice(&bytes).ok()?;
    cache.workflow_history.get(id).map(|entry| entry.commit.clone())
}

#[test]
fn a_cache_hit_moves_its_entry_forward_to_the_current_head() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    git_processes_for_one_scoped_read(root, &record.id).0.unwrap();
    let stamped = cached_history_commit(root, &record.id).unwrap();
    for index in 0..3 {
        fs::write(root.join("README.md"), format!("unrelated {index}\n")).unwrap();
        quiet_git(root, &["commit", "-am", "unrelated work"]);
    }
    git_processes_for_one_scoped_read(root, &record.id).0.unwrap();
    let head = quiet_git(root, &["rev-parse", "HEAD"]);
    assert_ne!(stamped, head);
    assert_eq!(
        cached_history_commit(root, &record.id).as_deref(),
        Some(head.as_str()),
        "a proven-current hit should restamp its entry so later reads walk only newer commits"
    );
}

#[test]
fn a_cache_flush_never_shares_its_temporary_file_with_another_writer() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("specsync").join("read-cache-v1.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    // Another thread of this process is mid-write: the bare per-process name is taken.
    fs::create_dir_all(path.with_extension(format!("tmp-{}", std::process::id()))).unwrap();
    let frame = PersistentReadCacheFrame {
        path: Some(path.clone()),
        loaded: Some(PersistentReadCache {
            schema: READ_CACHE_SCHEMA,
            ..PersistentReadCache::default()
        }),
        dirty: true,
        ..PersistentReadCacheFrame::default()
    };
    flush_persistent_read_cache(&frame);
    let written: PersistentReadCache =
        serde_json::from_slice(&fs::read(&path).expect("the cache was not written")).unwrap();
    assert_eq!(written.schema, READ_CACHE_SCHEMA);
    let leftovers: Vec<_> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name())
        .filter(|name| name.to_string_lossy().contains(".tmp-") && !name.to_string_lossy().ends_with(&std::process::id().to_string()))
        .collect();
    assert!(leftovers.is_empty(), "temporary files left behind: {leftovers:?}");
}

#[test]
fn git_processes_spawned_by_prefetch_threads_are_counted() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let records: Vec<ChangeRecord> = (0..4).map(|_| finalized_archived_change(root)).collect();
    let archives = || fs::read_dir(root.join(ARCHIVE_PATH)).unwrap().collect::<Vec<_>>();
    // Workers run only when the lifecycle-state history index cannot answer (REQ-change-141), so
    // both halves turn it off to compare the per-archive work with the workers' count of it.
    without_lifecycle_state_history(|| {
    let serial = {
        let _scope = begin_change_read_scope(root);
        reset_test_git_process_count();
        for record in &records {
            validate_workflow_version_history_uncached(root, record).unwrap();
        }
        test_git_process_count()
    };
    // Finalize stores passing verdicts in the persistent cache (REQ-change-121), so the setup
    // above leaves every archive cached; start cold so the prefetch does the per-archive work.
    if let Some(path) = persistent_read_cache_path(root) {
        let _ = fs::remove_file(path);
    }
    let prefetched = {
        let _scope = begin_change_read_scope(root);
        reset_test_git_process_count();
        prefetch_archived_workflow_histories(root, &archives());
        test_git_process_count()
    };
    // The prefetch does the same per-archive history work on worker threads, so the guard that
    // budgets Git processes must see at least as many as the serial walk.
    assert!(
        prefetched >= serial,
        "prefetch reported {prefetched} Git processes for work that spawns {serial} serially"
    );
    });
}

#[test]
fn the_persistent_cache_is_off_while_replace_refs_rewrite_history() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    assert!(persistent_read_cache_path(root).is_some());
    fs::write(root.join("README.md"), "replacement\n").unwrap();
    quiet_git(root, &["commit", "-am", "replacement"]);
    let replacement = quiet_git(root, &["rev-parse", "HEAD"]);
    quiet_git(root, &["replace", "HEAD~1", replacement.as_str()]);
    assert_eq!(persistent_read_cache_path(root), None);
}

#[test]
fn the_persistent_cache_is_off_while_grafts_rewrite_history() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    assert!(persistent_read_cache_path(root).is_some());
    let head = quiet_git(root, &["rev-parse", "HEAD"]);
    fs::create_dir_all(root.join(".git/info")).unwrap();
    fs::write(root.join(".git/info/grafts"), format!("{head}\n")).unwrap();
    assert_eq!(persistent_read_cache_path(root), None);
}

#[test]
fn the_persistent_cache_is_off_in_a_partial_clone() {
    let temp = TempDir::new().unwrap();
    let (clone, _, _) = offline_partial_clone_missing_an_archive_blob(&temp);
    assert_eq!(persistent_read_cache_path(&clone), None);
}

#[test]
fn a_second_worktree_reads_warm_from_the_shared_cache() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("main");
    fs::create_dir_all(&root).unwrap();
    fast_read_repository(&root);
    let record = finalized_archived_change(&root);
    git_processes_for_one_scoped_read(&root, &record.id).0.unwrap();

    let second = temp.path().join("second");
    quiet_git(
        &root,
        &["worktree", "add", "-q", "--detach", second.to_str().unwrap(), "HEAD"],
    );
    assert_eq!(
        persistent_read_cache_path(&second),
        persistent_read_cache_path(&root),
        "worktrees of one repository share one history, so they should share one cache"
    );
    assert!(cached_history_commit(&second, &record.id).is_some());
    let live = archived_status_read(&second, &record.id).unwrap();
    assert_eq!(git_processes_for_one_scoped_read(&second, &record.id).0.unwrap(), live);
}

#[test]
fn a_pass_is_stored_only_for_the_state_it_was_computed_for() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    let state_path = find_change_dir(root, &record.id).unwrap().join("state.json");
    {
        let _scope = begin_change_read_scope(root);
        assert!(!cached_workflow_history_is_current(root, &record));
        // While the verdict for `record` is being computed, the file on disk changes to a state
        // that loads cleanly but whose history does not hold: a different workflow origin.
        let mut downgraded: serde_json::Value =
            serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();
        let object = downgraded.as_object_mut().unwrap();
        let origin = object["workflow_origin_version"].as_u64().unwrap();
        object.insert(
            "workflow_origin_version".into(),
            serde_json::json!(if origin == 1 { 2 } else { 1 }),
        );
        fs::write(
            &state_path,
            format!("{}\n", serde_json::to_string_pretty(&downgraded).unwrap()),
        )
        .unwrap();
        remember_workflow_history_pass(root, &record);
    }
    // The pass was computed for `record`, not for the bytes now on disk, so nothing may be stored
    // under those bytes' digest. (Archive integrity checks also refuse this edit at the command
    // layer today; the cache must not depend on that.)
    assert_eq!(cached_history_commit(root, &record.id), None);
    let live = archived_status_read(root, &record.id).unwrap_err();
    let scoped = git_processes_for_one_scoped_read(root, &record.id).0.unwrap_err();
    assert_eq!(scoped, live);
}

/// A Git repository whose SpecSync project is `app/`, so `git rev-parse --show-prefix` is `app/`
/// when history queries run with the project root as their working directory.
fn subdirectory_project() -> (TempDir, PathBuf) {
    subdirectory_project_named("app")
}

/// The same repository shape with the project in a directory called `name`.
fn subdirectory_project_named(name: &str) -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let repository = temp.path();
    quiet_git(repository, &["init", "-b", "main"]);
    quiet_git(repository, &["config", "user.email", "test@example.com"]);
    quiet_git(repository, &["config", "user.name", "Test"]);
    fs::write(repository.join("README.md"), "repository root\n").unwrap();
    quiet_git(repository, &["add", "README.md"]);
    quiet_git(repository, &["commit", "-m", "repository root"]);
    let root = repository.join(name);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(&root, &["add", "seed.txt"]);
    quiet_git(&root, &["commit", "-m", "seed"]);
    assert_eq!(git_repo_prefix(&root).unwrap(), format!("{name}/"));
    (temp, root)
}

/// Archives one workflow-v2 change by moving its package, the same shape the cross-date history
/// test uses, inside whatever project `root` is.
fn archived_workflow_record(root: &Path) -> ChangeRecord {
    let mut record = current_workflow_record(root, completed_no_spec_record(root));
    record.state = ChangeState::Accepted;
    save_change(root, &record).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "record accepted workflow identity"]);
    let archive = root.join(ARCHIVE_PATH).join(format!("2026-07-14-{}", record.id));
    fs::create_dir_all(archive.parent().unwrap()).unwrap();
    fs::rename(change_dir(root, &record.id), &archive).unwrap();
    record.state = ChangeState::Archived;
    write_json(&archive.join("state.json"), &record).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "archive on first date"]);
    record
}

// Issue #851. A project in a repository subdirectory double-applies `git_repo_prefix` when a
// history pathspec lacks `:(top)`, so a hidden workflow-version downgrade is invisible to the
// live walk and to the read cache's invalidation walk. Both must still refuse it.
#[test]
fn subdirectory_project_refuses_a_hidden_workflow_downgrade_live_and_cached() {
    let (_temp, root) = subdirectory_project();
    let record = archived_workflow_record(&root);
    git_processes_for_one_scoped_read(&root, &record.id)
        .0
        .expect("clean archived history in a subdirectory project is readable");

    let state_path = find_change_dir(&root, &record.id).unwrap().join("state.json");
    commit_hidden_workflow_downgrade(&root, &state_path);

    let live = archived_status_read(&root, &record.id).unwrap_err();
    assert!(
        live.contains("workflow-version history changed immutable identity"),
        "{live}"
    );
    let scoped = git_processes_for_one_scoped_read(&root, &record.id).0.unwrap_err();
    assert_eq!(scoped, live);
}

// Issue #851. The dated-archive glob is what admits an earlier package directory. A downgrade
// committed only on that earlier path, then hidden by a reopen and a later rearchive, is still
// part of the change's workflow-version history when the project is not the repository root.
#[test]
fn subdirectory_project_follows_an_earlier_archive_path_through_a_hidden_downgrade() {
    let (_temp, root) = subdirectory_project();
    follows_an_earlier_archive_path_through_a_hidden_downgrade(&root);
}

// A project directory whose name holds glob metacharacters. Interpolated unescaped into a
// `:(top,glob)` pathspec, `app[1]` is a character class that matches `app1`, never `app[1]`, so
// the dated-archive discovery finds nothing and the hidden downgrade is invisible again.
#[test]
fn glob_metacharacter_project_directory_follows_an_earlier_archive_path() {
    let (_temp, root) = subdirectory_project_named("app[1]");
    follows_an_earlier_archive_path_through_a_hidden_downgrade(&root);
}

fn follows_an_earlier_archive_path_through_a_hidden_downgrade(root: &Path) {
    let root = root.to_path_buf();
    let mut record = archived_workflow_record(&root);
    let first_archive = find_change_dir(&root, &record.id).unwrap();
    commit_hidden_workflow_downgrade(&root, &first_archive.join("state.json"));

    fs::rename(&first_archive, change_dir(&root, &record.id)).unwrap();
    record.state = ChangeState::Verifying;
    save_change(&root, &record).unwrap();
    quiet_git(&root, &["add", "--all"]);
    quiet_git(&root, &["commit", "-m", "reopen archived record"]);

    let second_archive = root.join(ARCHIVE_PATH).join(format!("2026-07-15-{}", record.id));
    fs::rename(change_dir(&root, &record.id), &second_archive).unwrap();
    record.state = ChangeState::Archived;
    write_json(&second_archive.join("state.json"), &record).unwrap();
    quiet_git(&root, &["add", "--all"]);
    quiet_git(&root, &["commit", "-m", "rearchive on second date"]);

    let state_paths = workflow_version_state_paths(&root, &record).unwrap();
    assert!(
        state_paths.iter().any(|path| path.contains("2026-07-14-")),
        "earlier archive path missing from {state_paths:?}"
    );
    assert!(
        state_paths.iter().any(|path| path.contains("2026-07-15-")),
        "current archive path missing from {state_paths:?}"
    );
    let error = load_change(&root, &record.id).unwrap_err();
    assert!(
        error.contains("workflow-version history changed immutable identity"),
        "{error}"
    );
}

/// The read-cache identity every build wrote before #878: the package version and the bundled
/// limits, and nothing that names the code. Every build of one version shared it, and the cache
/// lives in the Git common directory, so sibling worktrees on different code shared it too.
/// Frozen on purpose, because it stands for caches already on disk; do not update it.
fn read_cache_identity_before_878() -> String {
    format!(
        "{}+{}",
        env!("CARGO_PKG_VERSION"),
        sha256_hex(include_str!("../lifecycle-validation-limits.json").as_bytes())
    )
}

/// Verifies REQ-change-134 (#878). DISCRIMINATOR: the persistent read cache holds a passing
/// workflow-history verdict stamped by a build from before #878 at this same version. This build
/// must not reuse it: it recomputes the verdict and rewrites the cache under its own identity.
#[test]
fn a_read_cache_written_by_a_build_before_878_is_not_reused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    git_processes_for_one_scoped_read(root, &record.id).0.unwrap();
    let path = persistent_read_cache_path(root).expect("the fixture repository caches");
    let mut cache: PersistentReadCache =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert!(
        cache.workflow_history.contains_key(&record.id),
        "the first read cached the archived change's passing verdict"
    );
    cache.specsync_version = read_cache_identity_before_878();
    fs::write(&path, serde_json::to_vec(&cache).unwrap()).unwrap();

    git_processes_for_one_scoped_read(root, &record.id).0.unwrap();
    let rewritten: PersistentReadCache =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_ne!(
        rewritten.specsync_version,
        read_cache_identity_before_878(),
        "a read cache stamped by a build from before #878 must not be reused"
    );
    assert_eq!(rewritten.specsync_version, read_cache_build_identity());
    assert!(rewritten.workflow_history.contains_key(&record.id));
}

/// Every change record the project holds, active and archived, read from its own `state.json`.
fn every_change_record(root: &Path) -> Vec<ChangeRecord> {
    let mut records = Vec::new();
    for directory in [root.join(CHANGES_PATH), root.join(ARCHIVE_PATH)] {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
        entries.sort();
        for entry in entries {
            if let Ok(bytes) = fs::read(entry.join("state.json"))
                && let Ok(record) = serde_json::from_slice::<ChangeRecord>(&bytes)
            {
                records.push(record);
            }
        }
    }
    records
}

/// What the workflow-version history check concludes about `record` inside one read scope:
/// its state paths, the commits touching them oldest first with their parents, and its verdict.
type WorkflowHistoryAnswer = (
    Result<Vec<String>, String>,
    Option<Vec<(String, Vec<String>)>>,
    Result<(), String>,
);

fn workflow_history_answer(root: &Path, record: &ChangeRecord, indexed: bool) -> WorkflowHistoryAnswer {
    let answer = || {
        let _scope = begin_change_read_scope(root);
        assert_eq!(lifecycle_state_history(root).is_some(), indexed);
        let state_paths = workflow_version_state_paths(root, record);
        let history = state_paths.as_ref().ok().map(|paths| {
            let commits: Vec<String> = match lifecycle_state_history(root) {
                Some(history) => history
                    .state_path_history(paths, lifecycle_validation_limits().scoped_review_max_descendants)
                    .unwrap()
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                None => live_workflow_version_history(root, paths)
                    .unwrap()
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(str::to_string)
                    .collect(),
            };
            commits
                .into_iter()
                .map(|commit| {
                    let parents = match lifecycle_state_history(root) {
                        Some(history) => history.parents(&commit).unwrap().to_vec(),
                        None => quiet_git(root, &["rev-list", "--parents", "-n", "1", &commit])
                            .split_whitespace()
                            .skip(1)
                            .map(str::to_string)
                            .collect(),
                    };
                    (commit, parents)
                })
                .collect()
        });
        let verdict = validate_workflow_version_history_uncached(root, record);
        (state_paths, history, verdict)
    };
    if indexed {
        answer()
    } else {
        without_lifecycle_state_history(answer)
    }
}

/// REQ-change-140: for every change, the one-walk index gives exactly the per-change answers.
/// Returns how many changes were compared.
fn assert_lifecycle_index_equals_per_change_walks(root: &Path) -> usize {
    let records = every_change_record(root);
    for record in &records {
        let live = workflow_history_answer(root, record, false);
        let indexed = workflow_history_answer(root, record, true);
        assert_eq!(indexed, live, "change {} ({:?})", record.id, record.state);
    }
    records.len()
}

/// Copy every file of `source` into `target`.
fn copy_package(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap().flatten() {
        if entry.path().is_file() {
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
}

// REQ-change-140: merges from either side, a package archived again on a side branch, a hidden
// downgrade, a non-canonical archive directory and an active change all get the same answers
// and the same first refusal from the index as from per-change walks.
#[test]
fn lifecycle_index_answers_equal_per_change_walks_across_merges_and_refusals() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let first = finalized_archived_change(root);
    let second = finalized_archived_change(root);
    let third = finalized_archived_change(root);
    assert_eq!(assert_lifecycle_index_equals_per_change_walks(root), 3);
    let archive_root = root.join(ARCHIVE_PATH);

    // A side branch archives `first` again under another date and a merge brings it in.
    quiet_git(root, &["switch", "-q", "-c", "side"]);
    copy_package(
        &find_change_dir(root, &first.id).unwrap(),
        &archive_root.join(format!("2026-01-02-{}", first.id)),
    );
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-q", "-m", "side archive"]);
    quiet_git(root, &["switch", "-q", "main"]);
    fs::write(root.join("README.md"), "main moves on\n").unwrap();
    quiet_git(root, &["commit", "-q", "-am", "main moves on"]);
    quiet_git(root, &["merge", "-q", "--no-ff", "-m", "merge side archive", "side"]);
    let first_answer = workflow_history_answer(root, &first, true);
    assert!(first_answer.0.is_err() || first_answer.2.is_err(), "{first_answer:?}");
    assert_lifecycle_index_equals_per_change_walks(root);
    // The copy stays in history only, so later steps can load the project.
    fs::remove_dir_all(archive_root.join(format!("2026-01-02-{}", first.id))).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-q", "-m", "remove the side copy"]);
    assert_lifecycle_index_equals_per_change_walks(root);

    // A second-parent-only edit: a hidden downgrade of `second` made on a branch and merged.
    quiet_git(root, &["switch", "-q", "-c", "downgrade"]);
    commit_hidden_workflow_downgrade(root, &find_change_dir(root, &second.id).unwrap().join("state.json"));
    quiet_git(root, &["switch", "-q", "main"]);
    fs::write(root.join("README.md"), "main moves again\n").unwrap();
    quiet_git(root, &["commit", "-q", "-am", "main moves again"]);
    quiet_git(root, &["merge", "-q", "--no-ff", "-m", "merge downgrade", "downgrade"]);
    let second_answer = workflow_history_answer(root, &second, true);
    assert!(
        second_answer.2.as_ref().is_err_and(|error| error.contains("changed immutable identity")),
        "{:?}",
        second_answer.2
    );
    assert_lifecycle_index_equals_per_change_walks(root);

    // A directory that matches `*-<id>` but is not a dated archive, left only in history.
    let stray = archive_root.join(format!("not-a-date-{}", third.id));
    copy_package(&find_change_dir(root, &third.id).unwrap(), &stray);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-q", "-m", "non-canonical archive copy"]);
    fs::remove_dir_all(&stray).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-q", "-m", "remove the copy"]);
    let third_answer = workflow_history_answer(root, &third, true);
    assert!(
        third_answer.0.as_ref().is_err_and(|error| error.contains("non-canonical archive path")),
        "{:?}",
        third_answer.0
    );
    assert_lifecycle_index_equals_per_change_walks(root);

    // An active change alongside the archives.
    let active = current_workflow_record(root, completed_no_spec_record(root));
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-q", "-m", "active change"]);
    assert!(every_change_record(root).iter().any(|record| record.id == active.id));
    assert_lifecycle_index_equals_per_change_walks(root);
}

// REQ-change-140: a root commit that already holds lifecycle state counts against the empty tree.
#[test]
fn lifecycle_index_answers_equal_per_change_walks_from_a_root_commit() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    finalized_archived_change(root);
    finalized_archived_change(root);
    quiet_git(root, &["checkout", "-q", "--orphan", "flat"]);
    quiet_git(root, &["commit", "-q", "-m", "everything in one root commit"]);
    assert_eq!(assert_lifecycle_index_equals_per_change_walks(root), 2);
}

// REQ-change-140 and #851: a project in a repository subdirectory whose name holds glob
// characters, following an earlier archive path through a hidden downgrade.
#[test]
fn lifecycle_index_answers_equal_per_change_walks_in_a_glob_named_subdirectory() {
    let (_temp, root) = subdirectory_project_named("app[1]");
    let mut record = archived_workflow_record(&root);
    let first_archive = find_change_dir(&root, &record.id).unwrap();
    assert_lifecycle_index_equals_per_change_walks(&root);
    commit_hidden_workflow_downgrade(&root, &first_archive.join("state.json"));
    fs::rename(&first_archive, change_dir(&root, &record.id)).unwrap();
    record.state = ChangeState::Verifying;
    save_change(&root, &record).unwrap();
    quiet_git(&root, &["add", "--all"]);
    quiet_git(&root, &["commit", "-m", "reopen archived record"]);
    let second_archive = root.join(ARCHIVE_PATH).join(format!("2026-07-15-{}", record.id));
    fs::rename(change_dir(&root, &record.id), &second_archive).unwrap();
    record.state = ChangeState::Archived;
    write_json(&second_archive.join("state.json"), &record).unwrap();
    quiet_git(&root, &["add", "--all"]);
    quiet_git(&root, &["commit", "-m", "rearchive on second date"]);
    let answer = workflow_history_answer(&root, &record, true);
    assert!(
        answer.2.as_ref().is_err_and(|error| error.contains("changed immutable identity")),
        "{:?}",
        answer.2
    );
    assert_eq!(assert_lifecycle_index_equals_per_change_walks(&root), 1);
}

// REQ-change-140: the per-change bounds refuse at exactly the count Git's `--max-count` walks
// would, for both the archived-path discovery and the state-path history.
#[test]
fn lifecycle_index_applies_the_per_change_bounds_like_the_walks() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    let _scope = begin_change_read_scope(root);
    let history = lifecycle_state_history(root).unwrap();
    let state_paths = workflow_version_state_paths(root, &record).unwrap();
    let pathspecs: Vec<String> = state_paths
        .iter()
        .map(|path| format!(":(top,literal){path}"))
        .collect();
    let touching = |max: usize| {
        let max_count = format!("--max-count={max}");
        let mut arguments = vec!["rev-list", "--full-history", max_count.as_str(), "HEAD", "--"];
        arguments.extend(pathspecs.iter().map(String::as_str));
        quiet_git(root, &arguments).lines().filter(|line| !line.is_empty()).count()
    };
    let total = touching(100_000);
    assert!(total >= 2, "the fixture needs a history of at least two commits");
    for bound in [total - 1, total, total + 1] {
        let walk_refuses = touching(bound + 1) > bound;
        assert_eq!(history.state_path_history(&state_paths, bound).is_none(), walk_refuses, "bound {bound}");
    }
    let archived = history
        .archived_state_paths(&record.id, 1000)
        .unwrap();
    let archive_pathspec = format!(
        ":(top,glob){}/*-{}/state.json",
        escape_glob_pathspec(&git_repo_relative_path(root, ARCHIVE_PATH).unwrap()),
        escape_glob_pathspec(&record.id)
    );
    let archived_commits = quiet_git(root, &["rev-list", "--full-history", "HEAD", "--", archive_pathspec.as_str()])
        .lines()
        .filter(|line| !line.is_empty())
        .count();
    assert!(!archived.is_empty() && archived_commits >= 1);
    for bound in [archived_commits.saturating_sub(1).max(1), archived_commits, archived_commits + 1] {
        assert_eq!(
            history.archived_state_paths(&record.id, bound).is_err(),
            archived_commits > bound,
            "bound {bound}"
        );
    }
}

// REQ-change-141: in an offline partial clone, a state blob the history lists but Git cannot
// read still refuses the archive through the batch reader, exactly as without the index.
#[test]
fn an_unreadable_workflow_history_blob_refuses_with_and_without_the_index() {
    let temp = TempDir::new().unwrap();
    let origin = temp.path().join("origin");
    fs::create_dir_all(&origin).unwrap();
    fast_read_repository(&origin);
    let record = finalized_archived_change(&origin);
    quiet_git(&origin, &["config", "uploadpack.allowFilter", "true"]);
    let url = format!("file://{}", origin.display());
    quiet_git(temp.path(), &["clone", "-q", "--filter=blob:none", url.as_str(), "clone"]);
    let clone = temp.path().join("clone");
    quiet_git(&clone, &["remote", "set-url", "origin", "file:///nonexistent-promisor"]);
    let record = load_change(&origin, &record.id).unwrap();
    let indexed = workflow_history_answer(&clone, &record, true).2;
    let live = workflow_history_answer(&clone, &record, false).2;
    assert!(
        indexed.as_ref().is_err_and(|error| error.contains("could not be read")),
        "{indexed:?}"
    );
    assert_eq!(indexed, live);
}

// REQ-change-140: parsing folds `-m`'s repeated merge blocks, and refuses output that is not in
// the walk's shape, so the caller falls back to per-change walks instead of judging from it.
#[test]
fn lifecycle_history_parse_folds_merge_blocks_and_refuses_other_shapes() {
    let a = "a".repeat(40);
    let b = "b".repeat(40);
    let c = "c".repeat(40);
    let text = format!(
        "\x01{a} {b} {c}\0\nx/state.json\0\x01{a} {b} {c}\0\ny/state.json\0x/state.json\0\x01{b}\0\ny/state.json\0"
    );
    let commits = parse_lifecycle_state_history(&text).unwrap();
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].parents, vec![b.clone(), c.clone()]);
    assert_eq!(commits[0].listed, vec!["x/state.json".to_string(), "y/state.json".to_string()]);
    assert!(commits[1].parents.is_empty());
    assert_eq!(parse_lifecycle_state_history("").unwrap(), Vec::new());
    assert!(parse_lifecycle_state_history("garbage").is_none());
    assert!(parse_lifecycle_state_history("\x01not-an-id\0").is_none());
    let split = format!("\x01{a} {b}\0\nx\0\x01{b}\0\x01{a} {b}\0\ny\0");
    assert!(parse_lifecycle_state_history(&split).is_none(), "a commit listed twice apart");
}

/// Each archive's captured subtree, keyed by archive and then by path, as `(mode, bytes)`.
type ArchiveSubtreeSnapshots = BTreeMap<String, BTreeMap<String, (u32, Vec<u8>)>>;

/// A repository whose `count` legacy archives were each introduced by their own commit before
/// the cutoff, with the baseline that enumerates them, its ledger digest, and their current
/// subtrees captured the way `load_legacy_archive_baseline_context` captures them.
fn legacy_archives_introduced_before_cutoff(
    root: &Path,
    count: usize,
) -> (LegacyArchiveBaselineV1, String, ArchiveSubtreeSnapshots) {
    fast_read_repository(root);
    let mut entries = Vec::new();
    for index in 0..count {
        let id = format!("CHG-{index:04}-legacy-archive");
        let archive_path = format!("{ARCHIVE_PATH}/2026-07-11-{id}");
        fs::create_dir_all(root.join(&archive_path)).unwrap();
        fs::write(
            root.join(&archive_path).join("state.json"),
            format!("{{\"id\":\"{id}\"}}\n"),
        )
        .unwrap();
        fs::write(root.join(&archive_path).join("change.md"), format!("# {id}\n")).unwrap();
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", &format!("archive {id}")]);
        let introduction_commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();
        entries.push((id, archive_path, introduction_commit));
    }
    let cutoff_commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let scopes = entries.iter().map(|(_, path, _)| path.clone()).collect();
    let (project_paths, evidence) =
        stable_discovered_evidence(root, Some(&scopes), &BTreeSet::new(), false, false).unwrap();
    let mut snapshots = BTreeMap::new();
    let mut baseline_entries = Vec::new();
    for (id, archive_path, introduction_commit) in entries {
        let snapshot =
            archive_snapshot_from_evidence(&project_paths, &evidence, &archive_path).unwrap();
        baseline_entries.push(LegacyArchiveBaselineEntryV1 {
            id: id.clone(),
            archive_path,
            introduction_commit,
            subtree_digest: legacy_archive_subtree_digest(&snapshot).unwrap(),
        });
        snapshots.insert(id, snapshot);
    }
    let baseline = LegacyArchiveBaselineV1 {
        schema_version: 1,
        domain: "specsync.legacy-archive-baseline.v1".into(),
        authority_change_id: "CHG-9999-legacy-archive-baseline".into(),
        cutoff_commit,
        entries: baseline_entries,
    };
    let (baseline, digest) =
        validate_legacy_archive_baseline_bytes(json_content(&baseline).unwrap().as_bytes())
            .unwrap();
    (baseline, digest, snapshots)
}

/// The Git processes one read scope spends proving the legacy baseline's introductions.
fn legacy_introduction_processes(
    root: &Path,
    baseline: &LegacyArchiveBaselineV1,
    digest: &str,
    snapshots: &BTreeMap<String, BTreeMap<String, (u32, Vec<u8>)>>,
) -> (Result<(), String>, usize) {
    let _scope = begin_change_read_scope(root);
    reset_test_git_process_count();
    let result = validate_legacy_archive_introductions_cached(root, digest, baseline, snapshots);
    (result, test_git_process_count())
}

/// #901 (Codex P1): an archived read of a legacy archive proves the shared baseline's
/// introductions, which walks every introduction commit and two trees per archive. That proof is
/// a pure function of the baseline bytes, the immutable cutoff and introduction commits, and the
/// archives' current subtrees, so once it has passed for exactly those inputs a later read does
/// not walk it again, and its Git processes stay flat as legacy archives are added.
#[test]
fn warm_legacy_baseline_introductions_do_not_grow_with_legacy_archives() {
    let warm = |count: usize| {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let (baseline, digest, snapshots) = legacy_archives_introduced_before_cutoff(root, count);
        let (cold, cold_processes) =
            legacy_introduction_processes(root, &baseline, &digest, &snapshots);
        cold.unwrap();
        let (warm, warm_processes) =
            legacy_introduction_processes(root, &baseline, &digest, &snapshots);
        warm.unwrap();
        (cold_processes, warm_processes)
    };
    let (two_cold, two_warm) = warm(2);
    let (eight_cold, eight_warm) = warm(8);
    assert!(
        eight_cold > two_cold,
        "the cold proof walks every introduction: {two_cold} vs {eight_cold}"
    );
    assert_eq!(
        two_warm, eight_warm,
        "a warm read re-walked the legacy introductions: {two_warm} Git processes with 2 legacy archives, {eight_warm} with 8"
    );
}

/// The cached pass is keyed by everything the proof reads, and a refusal is never cached: a
/// changed archive subtree or a changed baseline is refused exactly as the live proof refuses it,
/// on every read.
#[test]
fn a_cached_legacy_introduction_pass_never_hides_a_changed_archive_or_baseline() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (baseline, digest, snapshots) = legacy_archives_introduced_before_cutoff(root, 3);
    legacy_introduction_processes(root, &baseline, &digest, &snapshots).0.unwrap();
    let (warm, warm_processes) = legacy_introduction_processes(root, &baseline, &digest, &snapshots);
    warm.unwrap();

    // A tampered archive in the working tree.
    let mut tampered = snapshots.clone();
    let first = baseline.entries[0].id.clone();
    tampered
        .get_mut(&first)
        .unwrap()
        .insert("change.md".into(), (0o100644, b"# tampered\n".to_vec()));
    let live = validate_legacy_archive_introductions(root, &baseline, &tampered).unwrap_err();
    for _ in 0..2 {
        let (refused, processes) =
            legacy_introduction_processes(root, &baseline, &digest, &tampered);
        assert_eq!(refused.unwrap_err(), live);
        assert!(processes > warm_processes, "a refusal is re-derived live every time");
    }

    // A baseline that binds a different introduction commit.
    let mut rebound = baseline.clone();
    rebound.entries[0].introduction_commit = baseline.entries[1].introduction_commit.clone();
    let (rebound, rebound_digest) =
        validate_legacy_archive_baseline_bytes(json_content(&rebound).unwrap().as_bytes())
            .unwrap();
    assert_ne!(rebound_digest, digest);
    let live = validate_legacy_archive_introductions(root, &rebound, &snapshots).unwrap_err();
    let (refused, _) = legacy_introduction_processes(root, &rebound, &rebound_digest, &snapshots);
    assert_eq!(refused.unwrap_err(), live);

    // The untouched inputs still pass from the cache.
    let (again, again_processes) =
        legacy_introduction_processes(root, &baseline, &digest, &snapshots);
    again.unwrap();
    assert_eq!(again_processes, warm_processes);
}

// REQ-change-140: the index is only reused while history under one `HEAD` cannot change. A
// replace ref (or a graft, or deepening a shallow clone) changes what Git walks without moving
// `HEAD`, so a later read scope must answer as a fresh per-change walk does.
#[test]
fn lifecycle_index_is_not_reused_once_a_replace_ref_rewrites_history() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    // The first read scope builds the index for this HEAD.
    assert_eq!(
        workflow_history_answer(root, &record, true),
        workflow_history_answer(root, &record, false)
    );
    // Replace the archiving commit with one that changes nothing, under the same HEAD.
    let archived = quiet_git(root, &["rev-parse", "HEAD~1"]);
    let parent = quiet_git(root, &["rev-parse", "HEAD~2"]);
    let parent_tree = quiet_git(root, &["rev-parse", "HEAD~2^{tree}"]);
    let replacement = quiet_git(
        root,
        &["commit-tree", parent_tree.trim(), "-p", parent.trim(), "-m", "replacement"],
    );
    quiet_git(root, &["replace", archived.trim(), replacement.trim()]);
    let live = without_lifecycle_state_history(|| {
        let _scope = begin_change_read_scope(root);
        (
            workflow_version_state_paths(root, &record),
            validate_workflow_version_history_uncached(root, &record),
        )
    });
    let indexed = {
        let _scope = begin_change_read_scope(root);
        (
            workflow_version_state_paths(root, &record),
            validate_workflow_version_history_uncached(root, &record),
        )
    };
    assert_eq!(indexed, live, "a read scope after the replace ref must answer like a fresh walk");
    let _scope = begin_change_read_scope(root);
    assert!(
        lifecycle_state_history(root).is_none(),
        "the index is off while replace refs rewrite history"
    );
}

// REQ-change-140: candidate discovery must not depend on `log.showRoot`. Here an earlier archive
// path appears only in the root commit and in a merge that moved it, so a `git log` that hides
// the root commit's names finds nothing, while the index (which passes `--root`) finds it.
#[test]
fn lifecycle_index_and_fallback_agree_when_log_show_root_is_off() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    let earlier = find_change_dir(root, &record.id).unwrap();
    quiet_git(root, &["checkout", "-q", "--orphan", "flat"]);
    quiet_git(root, &["commit", "-q", "-m", "everything in one root commit"]);
    quiet_git(root, &["switch", "-q", "-c", "side"]);
    fs::write(root.join("side.txt"), "side\n").unwrap();
    quiet_git(root, &["add", "side.txt"]);
    quiet_git(root, &["commit", "-q", "-m", "side work"]);
    quiet_git(root, &["switch", "-q", "flat"]);
    fs::write(root.join("README.md"), "flat moves on\n").unwrap();
    quiet_git(root, &["commit", "-q", "-am", "flat moves on"]);
    // The merge itself moves the package to a later date; no ordinary commit names either path.
    quiet_git(root, &["merge", "-q", "--no-ff", "--no-commit", "side"]);
    let later = earlier.with_file_name(format!("2026-01-03-{}", record.id));
    fs::rename(&earlier, &later).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-q", "-m", "merge side and move the archive"]);
    quiet_git(root, &["config", "log.showRoot", "false"]);
    assert_eq!(find_change_dir(root, &record.id).unwrap(), later);
    let live = workflow_history_answer(root, &record, false);
    let indexed = workflow_history_answer(root, &record, true);
    assert_eq!(indexed, live);
    let earlier_path = git_repo_relative_path(
        root,
        &portable_project_path(root, &earlier.join("state.json")),
    )
    .unwrap();
    assert!(
        live.0.as_ref().unwrap().contains(&earlier_path),
        "the root commit's archive path is a candidate whatever log.showRoot says: {:?}",
        live.0
    );
}
/// Git processes one warm `change status` of an archived change spawns in a repository that also
/// tracks `extra` ordinary files.
fn warm_archived_status_processes_with_tracked_files(extra: usize) -> usize {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let target = finalized_archived_change(root);
    if extra > 0 {
        let dir = root.join("src-extra");
        fs::create_dir_all(&dir).unwrap();
        for index in 0..extra {
            fs::write(dir.join(format!("file-{index:05}.txt")), format!("{index}\n")).unwrap();
        }
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-q", "-m", "more tracked files"]);
    }
    git_processes_for_one_scoped_read(root, &target.id).0.unwrap();
    let (result, processes) = git_processes_for_one_scoped_read(root, &target.id);
    result.unwrap();
    processes
}

// Warm path (SPEED-1.a): a warm status of an archived change must not digest the whole working
// tree. Today `summarize_change_with_effective` computes `scoped_review_current` for every state,
// which takes the project-input digest twice over every tracked file (on Raven, 130 of 183 Git
// processes and about 60% of the time), although an archived change's review is history.
#[test]
fn warm_archived_status_git_processes_do_not_grow_with_tracked_files() {
    let few = warm_archived_status_processes_with_tracked_files(0);
    let many = warm_archived_status_processes_with_tracked_files(3000);
    eprintln!("warm archived status Git processes: {few} with no extra files, {many} with 3000");
    assert!(
        many < few + 3,
        "3000 more tracked files added {} Git processes to one warm archived status read \
         ({few} without them, {many} with them)",
        many.saturating_sub(few)
    );
}

/// What the full scoped-review currency check answers for `record` today: its recorded review
/// compared with the current project-input digest.
fn full_scoped_review_currency(root: &Path, record: &ChangeRecord) -> bool {
    load_scoped_review(root, record).is_ok_and(|review| scoped_review_is_current(root, record, &review))
}

fn summary_scoped_review_current(root: &Path, id: &str) -> bool {
    let record = load_change(root, id).unwrap();
    summarize_change_with_strict(root, &record, false).scoped_review_current
}

// REQ-change-147: an archived change's `scoped_review_current` is the value the full check
// gives, without taking the digest: finalize moves the package, so the recorded review never
// matches the archived tree again. Pinned right after finalize (uncommitted and committed), after
// later commits, after an unrelated working-tree edit, and for an archive moved to another name.
#[test]
fn an_archived_change_reports_the_scoped_review_currency_the_full_check_gives() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);

    // Just finalized, archive not yet committed.
    let record = current_workflow_record(root, completed_no_spec_record(root));
    if quiet_git(root, &["ls-files", WORKFLOW_V2_BASELINE_PATH]).is_empty() {
        quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
        quiet_git(root, &["commit", "-m", "adopt workflow v2"]);
    }
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "implement approved change"]);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    finalize_change(root, &record.id).unwrap();
    let mut cases = Vec::new();
    let archived = load_change(root, &record.id).unwrap();
    assert_eq!(archived.state, ChangeState::Archived);
    cases.push(("finalized, uncommitted", full_scoped_review_currency(root, &archived), summary_scoped_review_current(root, &record.id)));

    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "archive change"]);
    cases.push(("archive committed", full_scoped_review_currency(root, &archived), summary_scoped_review_current(root, &record.id)));

    finalized_archived_change(root);
    cases.push(("after a later change", full_scoped_review_currency(root, &archived), summary_scoped_review_current(root, &record.id)));

    fs::write(root.join("README.md"), "edited in the working tree\n").unwrap();
    cases.push(("unrelated working-tree edit", full_scoped_review_currency(root, &archived), summary_scoped_review_current(root, &record.id)));
    quiet_git(root, &["checkout", "--", "README.md"]);

    let dir = find_change_dir(root, &record.id).unwrap();
    fs::rename(&dir, dir.with_file_name("renamed-archive")).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "rename archive directory"]);
    let moved = load_change(root, &record.id).unwrap();
    cases.push(("archive moved", full_scoped_review_currency(root, &moved), summary_scoped_review_current(root, &record.id)));

    for (case, full, summary) in &cases {
        eprintln!("{case}: full check {full}, summary {summary}");
    }
    for (case, full, summary) in &cases {
        assert_eq!(summary, full, "{case}: the summary must report what the full check gives");
    }
}

// REQ-change-147: a just-finalized package that Git ignores (`.git/info/exclude` lists the
// archive) is still uncommitted, so the summary must still report what the full check gives.
#[test]
fn an_ignored_just_finalized_archive_reports_the_full_review_currency() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    if quiet_git(root, &["ls-files", WORKFLOW_V2_BASELINE_PATH]).is_empty() {
        quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
        quiet_git(root, &["commit", "-m", "adopt workflow v2"]);
    }
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "implement approved change"]);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    let exclude = root.join(".git/info/exclude");
    let mut patterns = fs::read_to_string(&exclude).unwrap_or_default();
    patterns.push_str(&format!("/{ARCHIVE_PATH}/\n"));
    fs::write(&exclude, patterns).unwrap();
    finalize_change(root, &record.id).unwrap();
    let archived = load_change(root, &record.id).unwrap();
    assert_eq!(archived.state, ChangeState::Archived);
    let full = full_scoped_review_currency(root, &archived);
    assert_eq!(
        summary_scoped_review_current(root, &record.id),
        full,
        "an ignored, uncommitted archive must still be judged by the full check"
    );
}

// G-deadline (#954, REQ-change-149): the archive-introduction index checks its deadline before
// its bulk `diff-tree --stdin`, so a deadline that has passed stops it even when there is no
// archive-introducing commit to loop over. (On a worker-style deadline the check returns an
// error instead of exiting, which lets the test observe it.)
#[test]
fn an_expired_deadline_stops_the_archive_index_before_it_lists_anything() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let _scope = begin_change_read_scope(root);
    let _deadline = begin_worker_read_deadline(Some(ReadDeadline {
        until: Instant::now(),
        limit_secs: 0,
    }));
    let result = archive_introduction_index(root);
    assert!(
        result
            .as_ref()
            .is_err_and(|error| error.contains("archive-introduction index")),
        "an expired deadline let the archive-introduction index answer {:?}",
        result.map(|index| index.len())
    );
}

fn expired_worker_deadline() -> ReadDeadlineGuard {
    begin_worker_read_deadline(Some(ReadDeadline {
        until: Instant::now(),
        limit_secs: 0,
    }))
}

// REQ-change-149 (#1005 review): active-change loading checks its deadline after its last
// workspace too, so a deadline that passed during (or before) the final entry still stops it.
// With no workspace to loop over, only that final check can see it.
#[test]
fn active_change_loading_checks_its_deadline_after_the_last_workspace() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    fs::create_dir_all(root.join(CHANGES_PATH)).unwrap();
    let _deadline = expired_worker_deadline();
    let result = list_changes_uncached(root);
    assert!(
        result
            .as_ref()
            .is_err_and(|error| error.contains("active change loading")),
        "an expired deadline let active-change loading finish: {:?}",
        result.map(|roster| roster.records.len())
    );
}

// REQ-change-149 (#1005 review): the workflow-version history walk checks its deadline after
// its last commit too. A change whose state paths no commit has touched yet has an empty walk.
#[test]
fn the_history_walk_checks_its_deadline_after_the_last_commit() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    let _deadline = expired_worker_deadline();
    let result = validate_workflow_version_history_uncached(root, &record);
    assert!(
        result
            .as_ref()
            .is_err_and(|error| error.contains("workflow-version history")),
        "an expired deadline let the history walk finish: {result:?}"
    );
}

// REQ-change-149 (#1005 review): a limit too large for the clock means no deadline even inside
// an outer one; the guard restores the outer deadline when it drops.
#[test]
fn an_overflowing_limit_inside_an_outer_deadline_means_no_deadline() {
    let _outer = expired_worker_deadline();
    assert!(read_deadline_passed());
    {
        let _inner = begin_read_deadline(u64::MAX, "change status", "specsync change status", true);
        assert!(
            !read_deadline_passed(),
            "an overflowing limit kept the outer, expired deadline in force"
        );
    }
    assert!(read_deadline_passed(), "the outer deadline comes back when the inner guard drops");
}

// #850: the persistent read cache stays bounded. An archive-introduction entry records whether the
// package had an `approvals.json`, not its bytes; the cache keeps at most 4096 introduction
// entries; and a cache file over 8 MiB is not loaded, so the next write starts it fresh.

/// Each introducing commit with the approvals ledger it committed, per change id.
type IntroducedLedgers = BTreeMap<String, Vec<(String, Option<Vec<u8>>)>>;

/// The approvals ledger each introduction in `index` committed, per change id.
fn introduced_ledgers(_root: &Path, index: &ArchiveIntroductionIndex) -> IntroducedLedgers {
    index
        .iter()
        .map(|(id, introductions)| {
            let ledgers = introductions
                .iter()
                .map(|introduction| (introduction.commit.clone(), introduction.approvals.clone()))
                .collect();
            (id.clone(), ledgers)
        })
        .collect()
}

/// The archive-introduction index built afresh: a commit that touches no archive moves `HEAD`, so
/// the in-memory index misses while every archive introduction stays the same. With `scoped`, the
/// build reads and fills the persistent read cache.
fn rebuilt_archive_introduction_index(root: &Path, scoped: bool) -> Arc<ArchiveIntroductionIndex> {
    let marker = root.join("rebuild-marker.txt");
    let count = fs::read_to_string(&marker).map_or(0, |text| text.len());
    fs::write(&marker, "x".repeat(count + 1)).unwrap();
    quiet_git(root, &["add", "rebuild-marker.txt"]);
    quiet_git(root, &["commit", "-m", "move HEAD"]);
    let _scope = scoped.then(|| begin_change_read_scope(root));
    archive_introduction_index(root).unwrap()
}

#[test]
fn the_read_cache_holds_no_approvals_bytes_and_reads_them_back_unchanged() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    finalized_archived_change(root);
    finalized_archived_change(root);
    let live = introduced_ledgers(root, &rebuilt_archive_introduction_index(root, false));
    assert_eq!(live.len(), 2, "{live:?}");
    assert!(
        live.values().flatten().all(|(_, ledger)| ledger.is_some()),
        "every archived package committed an approvals ledger: {live:?}"
    );
    if let Some(path) = persistent_read_cache_path(root) {
        let _ = fs::remove_file(path);
    }
    let cold = introduced_ledgers(root, &rebuilt_archive_introduction_index(root, true));
    assert_eq!(cold, live);
    let cache = fs::read_to_string(persistent_read_cache_path(root).unwrap()).unwrap();
    assert!(
        cache.contains("archive_introductions") && !cache.contains("approved_scope"),
        "the read cache stores approvals.json bytes inline:\n{cache}"
    );
    let warm = introduced_ledgers(root, &rebuilt_archive_introduction_index(root, true));
    assert_eq!(warm, live, "a cached introduction must read back the same ledger");
}

#[test]
fn the_read_cache_keeps_at_most_4096_archive_introductions() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("read-cache.json");
    let mut cache = PersistentReadCache {
        schema: READ_CACHE_SCHEMA,
        specsync_version: read_cache_build_identity(),
        ..PersistentReadCache::default()
    };
    for commit in 0..4097 {
        cache.archive_introductions.insert(
            format!("{commit:040x}\n:(top,literal).specsync/archive/changes"),
            Vec::new(),
        );
    }
    let frame = PersistentReadCacheFrame {
        path: Some(path.clone()),
        loaded: Some(cache),
        dirty: true,
        ..PersistentReadCacheFrame::default()
    };
    flush_persistent_read_cache(&frame);
    let written: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let kept = written["archive_introductions"].as_object().unwrap();
    assert!(kept.len() <= 4096, "the read cache kept {} archive introductions", kept.len());
}

#[test]
fn a_read_cache_file_over_8_mib_starts_fresh() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let path = persistent_read_cache_path(root).expect("the read cache is enabled");
    let oversized = PersistentReadCache {
        schema: READ_CACHE_SCHEMA,
        specsync_version: read_cache_build_identity(),
        repo_prefix: git_repo_prefix(root).unwrap_or_default(),
        legacy_introductions: (0..90_000).map(|key| format!("{key:0>100}")).collect(),
        ..PersistentReadCache::default()
    };
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, serde_json::to_vec(&oversized).unwrap()).unwrap();
    assert!(fs::metadata(&path).unwrap().len() > 8 * 1024 * 1024);
    let _scope = begin_change_read_scope(root);
    let loaded = with_persistent_read_cache(root, |_, frame| {
        frame
            .loaded
            .as_ref()
            .map(|cache| cache.legacy_introductions.len())
    })
    .flatten();
    assert_eq!(loaded, Some(0), "a cache file over 8 MiB was loaded");
}
