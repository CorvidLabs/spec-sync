// 6.4.1 (S5): the transaction journal is kept where cleanup tools and macOS do not purge it.

/// S5. The transaction journal lives in the user's state folder, not in the cache folder macOS and
/// cleanup tools may empty, so an interrupted multi-file write can still be rolled back.
#[test]
fn the_journal_is_not_kept_in_the_cache_folder() {
    let temp = journal_project();
    let state = transaction_state(temp.path(), true).unwrap().unwrap();
    let journal = state.file_path(TRANSACTION_FILE);
    let cache = std::env::temp_dir().join("specsync-unit-test-cache-home");
    assert!(
        !journal.starts_with(&cache),
        "the journal is in the cache folder: {}",
        journal.display()
    );
}

/// C1 (#1135, 6.4.1). The change read cache is read through SpecSync's own state handles: a FIFO
/// at its path is refused, never waited on.
#[test]
fn the_change_read_cache_is_never_waited_on() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    let path = persistent_read_cache_path(root).expect("the read cache is enabled");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let _ = fs::remove_file(&path);
    assert!(Command::new("mkfifo").arg(&path).status().unwrap().success());

    let (sender, receiver) = std::sync::mpsc::channel();
    let (reader_root, id) = (root.to_path_buf(), record.id.clone());
    std::thread::spawn(move || {
        let _scope = begin_change_read_scope(&reader_root);
        let _ = sender.send(archived_status_read(&reader_root, &id));
    });
    let finished = receiver.recv_timeout(std::time::Duration::from_secs(20));
    if finished.is_err() {
        // Let a reader still waiting on the FIFO go, so the test process can end.
        let _ = fs::OpenOptions::new().write(true).open(&path);
    }
    let _ = fs::remove_file(&path);
    assert!(finished.is_ok(), "reading the change read cache waited on a FIFO");
}
