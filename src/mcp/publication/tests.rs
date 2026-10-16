use super::super::confinement::file_identity;
use super::super::test_support::{
    assert_no_mcp_transaction_debris, open_test_directory, quarantine_contains, setup_project,
};
use super::*;
use std::fs;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PostLinkDestinationFailure {
    Open,
    IdentityRead,
    IdentityMismatch,
}

#[test]
fn failed_generated_batch_rolls_back_files_and_retains_ambiguous_empty_parent() {
    let tmp = setup_project();
    let directory = open_test_directory(tmp.path());
    fs::create_dir_all(tmp.path().join("specs/blocked")).unwrap();
    fs::write(
        tmp.path().join("specs/blocked/blocked.spec.md"),
        "existing\n",
    )
    .unwrap();

    let error = write_generated_specs_transactionally(
        &directory,
        vec![
            (
                PathBuf::from("specs/created/created.spec.md"),
                "created\n".to_string(),
            ),
            (
                PathBuf::from("specs/blocked/blocked.spec.md"),
                "replacement\n".to_string(),
            ),
        ],
    )
    .unwrap_err();

    assert!(
        error.contains("Cannot atomically publish confined MCP generated spec"),
        "unexpected publication error: {error}"
    );
    assert!(!tmp.path().join("specs/created/created.spec.md").exists());
    assert!(tmp.path().join("specs/created").is_dir());
    assert!(
        fs::read_dir(tmp.path().join("specs/created"))
            .unwrap()
            .next()
            .is_none()
    );
    assert_eq!(
        fs::read_to_string(tmp.path().join("specs/blocked/blocked.spec.md")).unwrap(),
        "existing\n"
    );
    assert_no_mcp_transaction_debris(tmp.path());
}

#[cfg(unix)]
#[test]
fn generation_rejects_a_replaced_public_parent_before_publication() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join("specs")).unwrap();
    let directory = open_test_directory(tmp.path());
    let mut staged = stage_new_confined(
        &directory,
        Path::new("specs/target/generated.spec.md"),
        b"generated\n",
        "generated spec",
    )
    .unwrap();

    fs::rename(
        tmp.path().join("specs/target"),
        tmp.path().join("specs/moved-target"),
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("specs/target")).unwrap();
    fs::write(
        tmp.path().join("specs/target/generated.spec.md"),
        "replacement\n",
    )
    .unwrap();

    let error = publish_staged_file(&mut staged, "generated spec").unwrap_err();
    assert!(
        error.contains("public parent specs/target changed after staging"),
        "unexpected publication error: {error}"
    );
    let rollback = rollback_staged_batch(&[staged], error.clone());

    assert_eq!(rollback, error);
    assert_eq!(
        fs::read_to_string(tmp.path().join("specs/target/generated.spec.md")).unwrap(),
        "replacement\n"
    );
    assert!(
        !tmp.path()
            .join("specs/moved-target/generated.spec.md")
            .exists()
    );
    assert!(
        fs::read_dir(tmp.path().join("specs/moved-target"))
            .unwrap()
            .next()
            .is_none()
    );
}

#[cfg(unix)]
#[test]
fn generation_cleans_staged_quarantine_after_post_link_parent_replacement() {
    let tmp = setup_project();
    let directory = open_test_directory(tmp.path());
    let mut staged = stage_new_confined(
        &directory,
        Path::new("specs/target/generated.spec.md"),
        b"generated\n",
        "generated spec",
    )
    .unwrap();

    let error = publish_staged_file_with_hooks(
        &mut staged,
        "generated spec",
        |_, _| {},
        |_, _| {
            fs::rename(
                tmp.path().join("specs/target"),
                tmp.path().join("specs/moved-target"),
            )
            .unwrap();
            fs::create_dir_all(tmp.path().join("specs/target")).unwrap();
            fs::write(
                tmp.path().join("specs/target/generated.spec.md"),
                "replacement\n",
            )
            .unwrap();
        },
    )
    .unwrap_err();
    assert!(
        error.contains("public parent specs/target changed after staging"),
        "unexpected publication error: {error}"
    );

    let rollback = rollback_staged_batch(&[staged], error.clone());

    assert_eq!(rollback, error);
    assert_eq!(
        fs::read_to_string(tmp.path().join("specs/target/generated.spec.md")).unwrap(),
        "replacement\n"
    );
    assert!(
        fs::read_dir(tmp.path().join("specs/moved-target"))
            .unwrap()
            .next()
            .is_none()
    );
    assert_no_mcp_transaction_debris(tmp.path());
}

#[cfg(unix)]
#[test]
fn generation_appends_post_link_quarantine_cleanup_failure_and_preserves_replacement() {
    let tmp = setup_project();
    let directory = open_test_directory(tmp.path());
    let mut staged = stage_new_confined(
        &directory,
        Path::new("specs/target/generated.spec.md"),
        b"generated\n",
        "generated spec",
    )
    .unwrap();

    let error = publish_staged_file_with_hooks(
        &mut staged,
        "generated spec",
        |_, _| {},
        |quarantine, entry| {
            fs::rename(
                tmp.path().join("specs/target"),
                tmp.path().join("specs/moved-target"),
            )
            .unwrap();
            fs::create_dir_all(tmp.path().join("specs/target")).unwrap();
            fs::write(
                tmp.path().join("specs/target/generated.spec.md"),
                "public replacement\n",
            )
            .unwrap();
            quarantine.remove_file(entry).unwrap();
            quarantine
                .write(entry, b"quarantine replacement\n")
                .unwrap();
        },
    )
    .unwrap_err();
    assert!(
        error.contains("public parent specs/target changed after staging"),
        "public-parent failure was lost: {error}"
    );
    assert!(
        error.contains("Refusing to delete a replacement in the private quarantine"),
        "quarantine cleanup failure was not appended: {error}"
    );
    assert!(quarantine_contains(
        &staged.parent,
        b"quarantine replacement\n"
    ));

    let rollback = rollback_staged_batch(&[staged], error.clone());

    assert_eq!(rollback, error);
    assert_eq!(
        fs::read_to_string(tmp.path().join("specs/target/generated.spec.md")).unwrap(),
        "public replacement\n"
    );
    assert!(
        !tmp.path()
            .join("specs/moved-target/generated.spec.md")
            .exists()
    );
    let moved_parent = open_test_directory(&tmp.path().join("specs/moved-target"));
    assert!(quarantine_contains(
        &moved_parent,
        b"quarantine replacement\n"
    ));
}

#[test]
fn generation_cleans_exact_quarantine_for_every_post_link_destination_failure() {
    for failure in [
        PostLinkDestinationFailure::Open,
        PostLinkDestinationFailure::IdentityRead,
        PostLinkDestinationFailure::IdentityMismatch,
    ] {
        let tmp = setup_project();
        let destination = tmp.path().join("specs/target/generated.spec.md");
        let directory = open_test_directory(tmp.path());
        let mut staged = stage_new_confined(
            &directory,
            Path::new("specs/target/generated.spec.md"),
            b"generated\n",
            "generated spec",
        )
        .unwrap();

        let error = publish_staged_file_with_verification_hooks(
            &mut staged,
            "generated spec",
            |_, _| {},
            |_, _| match failure {
                PostLinkDestinationFailure::Open => {
                    fs::remove_file(&destination).unwrap();
                }
                PostLinkDestinationFailure::IdentityMismatch => {
                    fs::remove_file(&destination).unwrap();
                    fs::write(&destination, b"public replacement\n").unwrap();
                }
                PostLinkDestinationFailure::IdentityRead => {}
            },
            |file| {
                if failure == PostLinkDestinationFailure::IdentityRead {
                    Err(io::Error::other(
                        "injected destination identity-read failure",
                    ))
                } else {
                    file_identity(file)
                }
            },
        )
        .unwrap_err();

        let expected = match failure {
            PostLinkDestinationFailure::Open => "Cannot verify published MCP",
            PostLinkDestinationFailure::IdentityRead => "Cannot identify published MCP",
            PostLinkDestinationFailure::IdentityMismatch => {
                "was replaced before identity verification"
            }
        };
        assert!(error.contains(expected), "{failure:?}: {error}");
        assert_no_mcp_transaction_debris(tmp.path());

        let rollback = rollback_staged_batch(&[staged], error);
        if failure == PostLinkDestinationFailure::IdentityMismatch {
            assert!(
                rollback.contains("Refusing to remove replacement file"),
                "{failure:?}: {rollback}"
            );
            assert_eq!(
                fs::read(&destination).unwrap(),
                b"public replacement\n",
                "{failure:?}: public replacement was removed"
            );
        } else {
            assert!(
                !destination.exists(),
                "{failure:?}: exact published identity survived rollback"
            );
        }
    }
}

#[test]
fn generation_appends_cleanup_failure_for_every_post_link_destination_failure() {
    for failure in [
        PostLinkDestinationFailure::Open,
        PostLinkDestinationFailure::IdentityRead,
        PostLinkDestinationFailure::IdentityMismatch,
    ] {
        let tmp = setup_project();
        let destination = tmp.path().join("specs/target/generated.spec.md");
        let directory = open_test_directory(tmp.path());
        let mut staged = stage_new_confined(
            &directory,
            Path::new("specs/target/generated.spec.md"),
            b"generated\n",
            "generated spec",
        )
        .unwrap();

        let error = publish_staged_file_with_verification_hooks(
            &mut staged,
            "generated spec",
            |_, _| {},
            |quarantine, entry| {
                match failure {
                    PostLinkDestinationFailure::Open => {
                        fs::remove_file(&destination).unwrap();
                    }
                    PostLinkDestinationFailure::IdentityMismatch => {
                        fs::remove_file(&destination).unwrap();
                        fs::write(&destination, b"public replacement\n").unwrap();
                    }
                    PostLinkDestinationFailure::IdentityRead => {}
                }
                quarantine.remove_file(entry).unwrap();
                quarantine
                    .write(entry, b"quarantine replacement\n")
                    .unwrap();
            },
            |file| {
                if failure == PostLinkDestinationFailure::IdentityRead {
                    Err(io::Error::other(
                        "injected destination identity-read failure",
                    ))
                } else {
                    file_identity(file)
                }
            },
        )
        .unwrap_err();

        let expected = match failure {
            PostLinkDestinationFailure::Open => "Cannot verify published MCP",
            PostLinkDestinationFailure::IdentityRead => "Cannot identify published MCP",
            PostLinkDestinationFailure::IdentityMismatch => {
                "was replaced before identity verification"
            }
        };
        assert!(error.contains(expected), "{failure:?}: {error}");
        assert!(
            error.contains("Refusing to delete a replacement in the private quarantine"),
            "{failure:?}: {error}"
        );
        assert!(
            quarantine_contains(&staged.parent, b"quarantine replacement\n"),
            "{failure:?}: quarantine replacement was removed"
        );

        let rollback = rollback_staged_batch(&[staged], error);
        if failure == PostLinkDestinationFailure::IdentityMismatch {
            assert!(
                rollback.contains("Refusing to remove replacement file"),
                "{failure:?}: {rollback}"
            );
            assert_eq!(
                fs::read(&destination).unwrap(),
                b"public replacement\n",
                "{failure:?}: public replacement was removed"
            );
        } else {
            assert!(
                !destination.exists(),
                "{failure:?}: exact published identity survived rollback"
            );
        }
        let parent = open_test_directory(&tmp.path().join("specs/target"));
        assert!(
            quarantine_contains(&parent, b"quarantine replacement\n"),
            "{failure:?}: cleanup failure lost the quarantine replacement"
        );
    }
}

#[test]
fn staging_detects_a_same_entry_replacement_after_identity_capture() {
    let tmp = setup_project();
    let directory = open_test_directory(tmp.path());
    let mut staged = stage_new_confined_with_hook(
        &directory,
        Path::new("specs/replaced/replaced.spec.md"),
        b"generated\n",
        "generated spec",
        |parent, name| {
            parent.remove_file(name).unwrap();
            parent.write(name, b"replacement\n").unwrap();
        },
    )
    .unwrap();

    let error = publish_staged_file(&mut staged, "generated spec").unwrap_err();

    assert!(error.contains("Refusing to publish replacement staged MCP"));
    assert_eq!(
        staged.parent.read(&staged.temporary_name).unwrap(),
        b"replacement\n"
    );
    assert!(!tmp.path().join("specs/replaced/replaced.spec.md").exists());
}

#[test]
fn publication_quarantines_a_replacement_swapped_after_verification() {
    let tmp = setup_project();
    let directory = open_test_directory(tmp.path());
    let mut staged = stage_new_confined(
        &directory,
        Path::new("specs/replaced/replaced.spec.md"),
        b"generated\n",
        "generated spec",
    )
    .unwrap();

    let error = publish_staged_file_with_hook(&mut staged, "generated spec", |parent, name| {
        parent.remove_file(name).unwrap();
        parent.write(name, b"replacement\n").unwrap();
    })
    .unwrap_err();

    assert!(error.contains("quarantined replacement was preserved"));
    assert!(!tmp.path().join("specs/replaced/replaced.spec.md").exists());
    assert!(quarantine_contains(&staged.parent, b"replacement\n"));
}
