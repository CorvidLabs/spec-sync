use super::archive_guidance::{
    ArchiveCandidate, archive_sentence_without_command, pull_request_archive_sentence,
    ship_next_action,
};
use super::render::lesson_bundle_line;
use super::shipping::pull_request_ready_next;

// Honest label: DISCRIMINATOR. Lessons are a convention, not a merge gate.
// On the unfixed binary this string started with "write lessons into…".
#[test]
fn ship_does_not_gate_merge_on_writing_lessons() {
    let next = ship_next_action(
        true,
        true,
        &[],
        &["specs/change/context.md".to_string()],
        "/archive/lesson-bundle.md",
    );

    assert_eq!(next, "Merge the pull request when the checks are green.");
    assert!(!next.contains("write lessons"), "got {next:?}");
}

// Honest label: this is the CONTROL. A change owning no specs has nothing to fold, and its
// guidance must be BYTE-IDENTICAL to the guidance before this fix — so the expected strings
// are spelled out rather than probed for the absence of "write lessons", which would still
// pass if the tail were reworded. All four push/wait combinations, including `(false, true)`:
// `--wait` is independent of `--push` in the CLI, so that arm is reachable.
#[test]
fn ship_guidance_is_unchanged_when_there_is_nothing_to_fold() {
    let cases = [
        (
            true,
            true,
            "Merge the pull request when the checks are green.",
        ),
        (
            true,
            false,
            "Wait until the checks are green, then merge the pull request.",
        ),
        (
            false,
            true,
            "Push this archive, wait until the checks are green, then merge the pull request.",
        ),
        (
            false,
            false,
            "Push this archive, wait until the checks are green, then merge the pull request.",
        ),
    ];

    for (push, wait, expected) in cases {
        assert_eq!(
            ship_next_action(push, wait, &[], &[], "/archive/lesson-bundle.md"),
            expected
        );
    }
}

// Sibling changes still block the merge, and that warning must survive the prefix rather
// than being displaced by it.
#[test]
fn ship_keeps_the_sibling_blocker_alongside_the_fold_back() {
    let next = ship_next_action(
        false,
        false,
        &["other-change".to_string()],
        &["specs/cmd_change/context.md".to_string()],
        "/archive/lesson-bundle.md",
    );

    assert!(
        !next.contains("write lessons"),
        "lessons must not displace the sibling blocker: {next:?}"
    );
    assert_eq!(
        next,
        "Archive `other-change` next. This pull request is not finished."
    );
    assert!(!next.to_ascii_lowercase().contains("merge"), "{next}");
}

#[test]
fn finalize_text_names_the_lesson_bundle_beside_the_merge_instruction() {
    let bundle = lesson_bundle_line();
    let next = ship_next_action(
        true,
        true,
        &[],
        &["specs/change/context.md".to_string()],
        "/archive/lesson-bundle.md",
    );
    assert!(bundle.starts_with("Lesson bundle: "), "{bundle}");
    assert!(bundle.contains("lesson-bundle.md"), "{bundle}");
    assert!(!bundle.contains("Next:"), "{bundle}");
    assert_eq!(next, "Merge the pull request when the checks are green.");
    assert!(!next.contains("write lessons"), "{next}");
    assert!(!next.contains("Lesson bundle"), "{next}");
}

#[test]
fn ship_commits_the_archive_and_names_merge_as_what_is_left() {
    let next = ship_next_action(false, false, &[], &[], "lesson-bundle.md");
    assert_eq!(
        next,
        "Push this archive, wait until the checks are green, then merge the pull request."
    );
    assert!(!pull_request_archive_sentence(&[]).contains("not finished"));
    assert_eq!(
        pull_request_archive_sentence(&[]),
        "All changes are archived. Ready to merge when the other checks are green."
    );
}

#[test]
fn change_pr_next_step_waits_for_green_checks_and_does_not_merge() {
    let next = pull_request_ready_next();
    assert_eq!(
        next,
        "Wait until the checks are green, then merge the pull request."
    );
    assert!(!next.contains("gh pr merge"), "{next}");
}

#[test]
fn status_says_which_open_change_to_archive_first() {
    let open = vec![
        ArchiveCandidate {
            id: "draft-notes".into(),
            state: "draft".into(),
            ship: false,
        },
        ArchiveCandidate {
            id: "zebra-feature".into(),
            state: "verifying".into(),
            ship: true,
        },
    ];
    let sentence = pull_request_archive_sentence(&open);
    assert!(
        sentence.starts_with("This pull request is not finished. Archive `zebra-feature` first."),
        "{sentence}"
    );
    assert!(sentence.contains("Run `specsync change ship zebra-feature`."));
    assert!(!sentence.contains("Ready to merge"), "{sentence}");
}

#[test]
fn accepted_is_named_before_a_verifying_change() {
    let open = vec![
        ArchiveCandidate {
            id: "later-verify".into(),
            state: "verifying".into(),
            ship: true,
        },
        ArchiveCandidate {
            id: "sooner-accept".into(),
            state: "accepted".into(),
            ship: false,
        },
    ];
    let sentence = pull_request_archive_sentence(&open);
    assert!(
        sentence.contains("Archive `sooner-accept` first"),
        "{sentence}"
    );
    assert!(
        sentence.contains("Run `specsync change status sooner-accept`."),
        "{sentence}"
    );
    assert_eq!(
        archive_sentence_without_command(&sentence),
        "This pull request is not finished. Archive `sooner-accept` first. Still open: `sooner-accept`, `later-verify`."
    );
}
