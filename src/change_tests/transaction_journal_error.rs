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
