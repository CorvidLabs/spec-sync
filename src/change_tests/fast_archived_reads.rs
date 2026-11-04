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

/// The timed benchmark against a Raven-sized project: 124 archived changes and 120 specs.
/// Run on demand (`fledge run bench-archived-reads`); `SPECSYNC_BENCH_ARCHIVES` and
/// `SPECSYNC_BENCH_SPECS` change the fixture size.
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
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
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
        "fixture: {archives} archived changes, {specs} specs, built in {:.1}s",
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
    time("cold read scope");
    let warm = time("warm read scope");
    let started = Instant::now();
    {
        let _scope = begin_change_read_scope(root);
        list_changes(root).unwrap();
    }
    eprintln!("change list: {:.3}s", started.elapsed().as_secs_f64());
    assert!(
        warm < Duration::from_secs(2),
        "a warm archived status read took {:.3}s against the 2 s budget",
        warm.as_secs_f64()
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
