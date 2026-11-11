// Issue #433: a corrupt transaction journal bricked every write with a raw serde error that
// named neither the file nor the recovery, and the stale-evidence refusals printed a bare
// `specsync change reopen <id>` whose required `--actor`/`--reason` flags the pasted command
// then demanded.

/// The corrupt-journal error names the journal path and the set-aside recovery.
#[test]
fn a_corrupt_transaction_journal_names_the_file_and_the_recovery() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(TRANSACTION_PATH), "NOT JSON{{{").unwrap();
    let error = recover_pending_transaction(root).unwrap_err();
    assert!(
        error.contains(".specsync/change-transaction.json"),
        "the error names the file: {error}"
    );
    assert!(
        error.contains(".broken"),
        "the error names the set-aside recovery: {error}"
    );
    assert!(
        error.contains("expected value"),
        "the underlying parse error is preserved: {error}"
    );
}

/// A journal with a damaged integrity envelope is reported the same way.
#[test]
fn a_journal_with_a_damaged_integrity_envelope_names_the_file_and_the_recovery() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(TRANSACTION_PATH),
        r#"{"schema_version":1,"entry_count":2,"entries_digest":"0000","entries":[]}"#,
    )
    .unwrap();
    let error = recover_pending_transaction(root).unwrap_err();
    assert!(
        error.contains(".specsync/change-transaction.json"),
        "the error names the file: {error}"
    );
    assert!(
        error.contains(".broken"),
        "the error names the set-aside recovery: {error}"
    );
}

/// A journal that cannot be READ (non-UTF-8 bytes) gets the same named path and recovery as
/// one that cannot be parsed (t384's round-2 finding on #1041).
#[test]
fn a_non_utf8_journal_names_the_file_and_the_recovery() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(TRANSACTION_PATH), [0xff, 0xfe, 0x00, 0x7b]).unwrap();
    let error = recover_pending_transaction(root).unwrap_err();
    assert!(
        error.contains(".specsync/change-transaction.json"),
        "the error names the file: {error}"
    );
    assert!(error.contains(".broken"), "the error names the recovery: {error}");
    assert!(
        error.contains("stream did not contain valid UTF-8"),
        "the underlying read error is preserved: {error}"
    );
}

/// A directory at the journal path is an unreadable journal, reported the same way.
#[test]
fn a_directory_at_the_journal_path_names_the_file_and_the_recovery() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(TRANSACTION_PATH)).unwrap();
    let error = recover_pending_transaction(root).unwrap_err();
    assert!(
        error.contains(".specsync/change-transaction.json"),
        "the error names the file: {error}"
    );
    assert!(error.contains(".broken"), "the error names the recovery: {error}");
}

/// The printed recovery is quoted: in a project whose path contains a space it runs verbatim
/// through the shell (t384's round-2 finding on #1041).
#[cfg(unix)]
#[test]
fn the_printed_recovery_runs_verbatim_in_a_spaced_root() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("with space");
    fs::create_dir_all(root.join(".specsync")).unwrap();
    let journal = root.join(TRANSACTION_PATH);
    fs::write(&journal, "NOT JSON{{{").unwrap();
    let error = recover_pending_transaction(&root).unwrap_err();
    // Extract the printed mv command and run it through a real shell.
    let start = error.find("`mv ").map(|i| i + 1).expect("a quoted mv is printed");
    let end = error[start..].find('`').map(|i| start + i).unwrap();
    let printed = &error[start..end];
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(printed)
        .status()
        .unwrap();
    assert!(status.success(), "the printed recovery runs as printed: {printed}");
    assert!(!journal.exists(), "the journal was set aside");
    assert!(
        root.join(".specsync/change-transaction.json.broken").exists(),
        "the set-aside copy exists"
    );
}

/// A legacy entry pointing outside the project is refused with the journal named.
#[test]
fn a_journal_entry_escaping_the_root_names_the_journal() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(TRANSACTION_PATH),
        r#"[{"path":"../outside.txt","original":null}]"#,
    )
    .unwrap();
    let error = recover_pending_transaction(root).unwrap_err();
    assert!(
        error.contains(".specsync/change-transaction.json"),
        "the refusal names the journal holding the entry: {error}"
    );
    assert!(error.contains("../outside.txt"), "and the entry: {error}");
}
