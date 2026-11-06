// ─── REQ-change-144: clean delivery blobs are hashed one at a time (#976) ───
//
// Reopen of site CHG-0026 refused before it checked anything: its affected paths held 372 MiB of
// clean files, and the acceptance evidence loaded them all through one capped `git cat-file
// --batch` read. The seam `TEST_GIT_EVIDENCE_CAPS` shrinks that bound so a few small files exceed
// it, exactly as `public/nevermore` exceeds the production one.

const LARGE_SCOPE_FILES: usize = 8;
const LARGE_SCOPE_FILE_BYTES: usize = 512;
/// Smaller than the clean bytes of the scope, larger than any one of its files.
const SHRUNK_BATCH_CAP: usize = LARGE_SCOPE_FILE_BYTES * 2;
const BATCH_OVERFLOW: &str = "`git cat-file --batch` output exceeds deterministic bounds";

struct EvidenceCapsGuard;

impl Drop for EvidenceCapsGuard {
    fn drop(&mut self) {
        TEST_GIT_EVIDENCE_CAPS.with(|caps| caps.set((None, None)));
    }
}

fn shrink_evidence_caps(batch: Option<usize>, object: Option<usize>) -> EvidenceCapsGuard {
    TEST_GIT_EVIDENCE_CAPS.with(|caps| caps.set((batch, object)));
    EvidenceCapsGuard
}

fn large_scope_repository(root: &Path) {
    run_git(root, &["init", "-b", "main"]);
    run_git(root, &["config", "user.email", "test@example.com"]);
    run_git(root, &["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    fs::write(root.join("README.md"), "base\n").unwrap();
    fs::create_dir_all(root.join("assets")).unwrap();
    for index in 0..LARGE_SCOPE_FILES {
        fs::write(
            root.join(format!("assets/frame-{index}.bin")),
            vec![b'a' + index as u8; LARGE_SCOPE_FILE_BYTES],
        )
        .unwrap();
    }
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-m", "base"]);
}

/// A workflow-v1 change accepted over `src/` and `assets/`, committed, and then left stale by a
/// later committed edit of `src/lib.rs`: the site's CHG-0026 shape. With `manifest` false the
/// acceptance is rewritten to the manifest-less legacy shape, re-signed so it still
/// authenticates.
fn stale_large_scope_acceptance(root: &Path, manifest: bool) -> ChangeRecord {
    large_scope_repository(root);
    let mut record = completed_no_spec_record(root);
    record.affected_paths.push("assets/".into());
    record.affected_paths.sort();
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    if !manifest {
        let legacy_digest = acceptance_input_digest(root, &record, &[]).unwrap();
        let mut verification = load_verification(root, &record).unwrap();
        verification.acceptance_input_digest = Some(legacy_digest);
        verification.acceptance_manifest = None;
        verification.semantic_succession = None;
        write_json(
            &change_dir(root, &record.id).join("verification.json"),
            &verification,
        )
        .unwrap();
        let mut ledger = load_approvals(root, &record).unwrap();
        ledger
            .approvals
            .iter_mut()
            .rev()
            .find(|approval| approval.gate == "acceptance")
            .unwrap()
            .digest = closing_digest(&record, &verification);
        write_json(&change_dir(root, &record.id).join("approvals.json"), &ledger).unwrap();
    }
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-m", "accept"]);
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { false }\n").unwrap();
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-m", "change a delivery input after acceptance"]);
    record
}

fn reopen_stale_large_scope(root: &Path, record: &ChangeRecord) -> Result<ReopenResult, String> {
    reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "A delivery input changed after acceptance".into(),
    )
}

#[test]
fn reopen_reads_clean_delivery_files_past_one_batch_read_with_an_acceptance_manifest() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = stale_large_scope_acceptance(root, true);
    assert!(
        load_verification(root, &record)
            .unwrap()
            .acceptance_manifest
            .is_some()
    );
    let _caps = shrink_evidence_caps(Some(SHRUNK_BATCH_CAP), None);
    let reopened = reopen_stale_large_scope(root, &record)
        .unwrap_or_else(|error| panic!("reopen refused: {error}"));
    assert_eq!(reopened.change.state, ChangeState::Verifying);
}

#[test]
fn reopen_reads_clean_delivery_files_past_one_batch_read_without_an_acceptance_manifest() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = stale_large_scope_acceptance(root, false);
    assert!(
        load_verification(root, &record)
            .unwrap()
            .acceptance_manifest
            .is_none()
    );
    let _caps = shrink_evidence_caps(Some(SHRUNK_BATCH_CAP), None);
    let reopened = reopen_stale_large_scope(root, &record)
        .unwrap_or_else(|error| panic!("reopen refused: {error}"));
    assert_eq!(reopened.change.state, ChangeState::Verifying);
}

#[test]
fn a_clean_delivery_blob_over_the_per_object_bound_is_refused_by_name() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = stale_large_scope_acceptance(root, true);
    fs::write(
        root.join("assets/oversized.bin"),
        vec![b'z'; LARGE_SCOPE_FILE_BYTES * 3],
    )
    .unwrap();
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-m", "add an oversized asset"]);
    let _caps = shrink_evidence_caps(None, Some(LARGE_SCOPE_FILE_BYTES * 2));
    let error = match reopen_stale_large_scope(root, &record) {
        Ok(_) => panic!("reopen hashed a blob over the per-object bound"),
        Err(error) => error,
    };
    assert!(error.contains("assets/oversized.bin"), "{error}");
    assert!(!error.contains(BATCH_OVERFLOW), "{error}");
}

#[test]
fn accepted_input_validation_judges_evidence_past_one_batch_read() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let _record = stale_large_scope_acceptance(root, true);
    let _caps = shrink_evidence_caps(Some(SHRUNK_BATCH_CAP), None);
    let errors = check_project(root).errors;
    assert!(
        errors
            .iter()
            .all(|error| !error.contains(BATCH_OVERFLOW)),
        "validation refused on the read: {errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("accepted change verification is stale")),
        "validation did not judge the drifted input: {errors:?}"
    );
}

/// Every kind of delivery entry: clean, executable, modified in the working tree, deleted from
/// it, a symlink, an LFS pointer, and the sequence ledger.
fn mixed_delivery_scope(root: &Path) -> ChangeRecord {
    run_git(root, &["init", "-b", "main"]);
    run_git(root, &["config", "user.email", "test@example.com"]);
    run_git(root, &["config", "user.name", "Test"]);
    configure_official_lfs_driver(root);
    write_lifecycle_test_policy(root);
    fs::create_dir_all(root.join("assets")).unwrap();
    fs::write(root.join(".gitattributes"), "*.bin filter=lfs -text\n").unwrap();
    fs::write(root.join("assets/clean.txt"), "clean bytes\n").unwrap();
    fs::write(root.join("assets/run.txt"), "#!/bin/sh\necho run\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            root.join("assets/run.txt"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
    }
    fs::write(root.join("assets/edited.txt"), "committed\n").unwrap();
    fs::write(root.join("assets/gone.txt"), "deleted later\n").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink("clean.txt", root.join("assets/link.txt")).unwrap();
    fs::write(
        root.join("assets/pack.bin"),
        b"version https://git-lfs.github.com/spec/v1\noid sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\nsize 7\n",
    )
    .unwrap();
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-m", "mixed delivery scope"]);
    let mut record = completed_no_spec_record(root);
    record
        .affected_paths
        .extend(["assets/".to_string(), SEQUENCE_PATH.to_string()]);
    record.affected_paths.sort();
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    // Only a started change covers its affected paths.
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-m", "open the change"]);
    fs::write(root.join("assets/edited.txt"), "edited in the working tree\n").unwrap();
    fs::remove_file(root.join("assets/gone.txt")).unwrap();
    record
}

/// The manifest and digest are the same bytes whichever way the clean blobs are read. The
/// pinned values were computed on main f7180c34, before the streamed read existed, so a
/// recorded manifest or digest still matches after it (REQ-change-144).
#[test]
#[cfg(unix)]
fn streamed_acceptance_evidence_is_byte_identical_to_the_whole_scope_read() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = mixed_delivery_scope(root);
    let manifest = acceptance_manifest(root, &record, &[]).unwrap();

    // Each entry hashes exactly the bytes the whole-scope read returns for that path.
    let candidates: BTreeSet<String> = manifest
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect();
    let whole_scope = inspect_git_candidates(root, &candidates, false, false).unwrap();
    for entry in &manifest.entries {
        let captured = whole_scope.entries.get(&entry.path).unwrap();
        assert_eq!(
            entry.payload_digest,
            sha256_hex(&captured.payload),
            "{}",
            entry.path
        );
        assert_eq!(entry.mode, captured.mode, "{}", entry.path);
    }
    let paths: Vec<&str> = manifest
        .entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    for expected in [
        "assets/clean.txt",
        "assets/edited.txt",
        "assets/gone.txt",
        "assets/link.txt",
        "assets/pack.bin",
        "assets/run.txt",
    ] {
        assert!(paths.contains(&expected), "{expected} missing: {paths:?}");
    }

    assert_eq!(
        acceptance_manifest_digest(&manifest).unwrap(),
        PINNED_MIXED_SCOPE_MANIFEST_DIGEST
    );
    assert_eq!(
        acceptance_input_digest(root, &record, &[]).unwrap(),
        PINNED_MIXED_SCOPE_INPUT_DIGEST
    );
}

const PINNED_MIXED_SCOPE_MANIFEST_DIGEST: &str =
    "3ec852ec16f05d7531c2f69c583e3ade6f9ce48f96f9f72f3a335019d0fb2c65";
const PINNED_MIXED_SCOPE_INPUT_DIGEST: &str =
    "3cde45bbdf7a2cfa267ea76de4bfe82bf08953cc63a3ac6af5e7074ac06f5ffa";
