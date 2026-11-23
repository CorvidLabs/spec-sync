#[test]
fn partial_clone_recovery_accepts_only_complete_missing_object_ids() {
    let sha1 = "1234567890abcdef1234567890abcdef12345678";
    let sha256 = "89abcdef0123456789abcdef0123456789abcdef0123456789abcdef01234567";
    let listing = [
        format!("{sha1} specs/change/requirements.md"),
        "?0123456789abcdef0123456789abcdef0123456".to_string(),
        format!("?{sha1} extra"),
        "?0123456789abcdef0123456789abcdef0123456z".to_string(),
        format!("?{sha1}"),
        format!("?{sha256}"),
    ]
    .join("\n");

    assert_eq!(
        missing_git_objects(&listing).collect::<Vec<_>>(),
        [sha1, sha256]
    );
}

#[test]
fn partial_clone_checks_the_subcommand_after_git_config_overrides() {
    let mut args = vec!["-c", "core.quotePath=false", "-c", "color.ui=never"];
    let options = args.len();
    args.extend(["cat-file", "-e", "HEAD:specs/change/requirements.md"]);
    assert!(reads_history_objects(&args));

    args.truncate(options);
    args.extend(["rev-parse", "--is-shallow-repository"]);
    assert!(!reads_history_objects(&args));
}
