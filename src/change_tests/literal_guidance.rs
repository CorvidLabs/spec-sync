// #929, #930, #931 (CHANGE-7.a.2, CHANGE-7.b, REQ-change-093). A recovery step is
// meant to be pasted into a shell, so these tests run what it prints: an archive path is one
// literal path to Git, a change ID is one positional to SpecSync, and a directory name that holds
// control bytes is printed as text that still names that directory.

/// Every backtick span in `message`, in order.
fn literal_guidance_spans(message: &str) -> Vec<String> {
    message
        .split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// Runs `command` through `bash -c` in `root` and returns its standard output.
fn literal_guidance_bash(root: &Path, command: &str) -> String {
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "`{command}` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// The argument vector bash builds from a printed `specsync ...` command.
fn literal_guidance_argv(command: &str) -> Vec<String> {
    let rest = command
        .strip_prefix("specsync ")
        .unwrap_or_else(|| panic!("not a specsync command: {command}"));
    let temp = TempDir::new().unwrap();
    let words = literal_guidance_bash(temp.path(), &format!("printf '%s\\0' {rest}"));
    std::iter::once("specsync".to_string())
        .chain(words.split('\0').filter(|word| !word.is_empty()).map(str::to_string))
        .collect()
}

/// A repository with `packages` committed under `pkg/`, each holding `f` and `state.json`, one
/// commit per package in the order given.
fn literal_guidance_repository(packages: &[&str]) -> TempDir {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    for package in packages {
        let directory = root.join("pkg").join(package);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("f"), "committed\n").unwrap();
        fs::write(directory.join("state.json"), "{}\n").unwrap();
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-q", "-m", &format!("add {package}")]);
    }
    temp
}

// #929. `*` and `?` are legal in a change ID, so a package path can hold them. The printed restore
// must put back only the package it names and leave a sibling it would glob-match alone.
#[test]
fn the_printed_restore_puts_back_only_the_named_package_even_with_glob_characters() {
    let temp = literal_guidance_repository(&["2026-11-06-fix*", "2026-11-06-fixA"]);
    let root = temp.path();
    for package in ["2026-11-06-fix*", "2026-11-06-fixA"] {
        fs::write(root.join("pkg").join(package).join("f"), "edited\n").unwrap();
    }
    let step = corrupt_archive_recovery(
        "fix*",
        &CorruptArchive::DiffersFromHead {
            package: "pkg/2026-11-06-fix*".into(),
        },
    );
    let restore = literal_guidance_spans(&step)
        .into_iter()
        .find(|span| span.starts_with("git restore"))
        .unwrap_or_else(|| panic!("no restore in {step}"));
    literal_guidance_bash(root, &restore);
    assert_eq!(
        fs::read_to_string(root.join("pkg/2026-11-06-fix*/f")).unwrap(),
        "committed\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("pkg/2026-11-06-fixA/f")).unwrap(),
        "edited\n",
        "the restore for `fix*` discarded an edit to `fixA`"
    );
}

// #929. The printed lookup of the archiving commit names the commit that added this package's
// record, not a later one that added a sibling the glob also matches.
#[test]
fn the_printed_archiving_commit_lookup_matches_only_the_named_package() {
    let temp = literal_guidance_repository(&["2026-11-06-fix?", "2026-11-06-fixA"]);
    let root = temp.path();
    let first = quiet_git(root, &["rev-parse", "HEAD~1"]).trim().to_string();
    let step = corrupt_archive_recovery(
        "fix?",
        &CorruptArchive::DamageCommitted {
            package: "pkg/2026-11-06-fix?".into(),
        },
    );
    let lookup = literal_guidance_spans(&step)
        .into_iter()
        .find(|span| span.starts_with("git log"))
        .unwrap_or_else(|| panic!("no archiving-commit lookup in {step}"));
    assert_eq!(literal_guidance_bash(root, &lookup).trim(), first);
}

// #929. The probes behind that choice read the package as one literal path too: an edit to a
// sibling the glob matches is not this package differing from `HEAD`.
#[test]
fn the_package_probe_reads_the_package_as_one_literal_path() {
    let temp = literal_guidance_repository(&["2026-11-06-fix*", "2026-11-06-fixA"]);
    let root = temp.path();
    fs::write(root.join("pkg/2026-11-06-fixA/f"), "edited\n").unwrap();
    assert!(!archive_package_differs_from(root, "HEAD", "pkg/2026-11-06-fix*"));
    assert!(archive_package_differs_from(root, "HEAD", "pkg/2026-11-06-fixA"));
}

// #930. A change ID may begin with `-`. Every printed status command must hand it to SpecSync as
// the change ID, never as a flag.
#[test]
fn the_printed_status_command_reads_a_leading_hyphen_id_as_the_change_id() {
    for corrupt in [
        CorruptArchive::DiffersFromHead {
            package: "pkg/2026-11-06---json".into(),
        },
        CorruptArchive::DamageCommitted {
            package: "pkg/2026-11-06---json".into(),
        },
        CorruptArchive::OutsidePackage,
    ] {
        let step = corrupt_archive_recovery("--json", &corrupt);
        let commands: Vec<String> = literal_guidance_spans(&step)
            .into_iter()
            .filter(|span| span.starts_with("specsync "))
            .collect();
        assert!(!commands.is_empty(), "{step}");
        for command in commands {
            let argv = literal_guidance_argv(&command);
            let cli = <crate::cli::Cli as clap::Parser>::try_parse_from(&argv)
                .unwrap_or_else(|error| panic!("`{command}` does not parse: {error}"));
            match cli.command {
                Some(crate::cli::Command::Change {
                    action: crate::cli::ChangeAction::Status { id, .. },
                }) => assert_eq!(id.as_deref(), Some("--json"), "`{command}`"),
                _ => panic!("`{command}` did not parse as `change status`"),
            }
        }
    }
}

// #931, CHANGE-7.b. An archive directory name is not validated, so it can hold a newline or an
// escape byte. The printed step carries no control byte, and the restore it prints still names
// that exact directory.
#[test]
fn a_control_byte_in_an_archive_directory_name_is_printed_as_text_that_still_restores_it() {
    let package = "2026-11-06-a\nb\u{1b}[31mc";
    let temp = literal_guidance_repository(&[package]);
    let root = temp.path();
    fs::write(root.join("pkg").join(package).join("f"), "edited\n").unwrap();
    let step = corrupt_archive_recovery(
        "a",
        &CorruptArchive::DiffersFromHead {
            package: format!("pkg/{package}"),
        },
    );
    assert!(
        !step.chars().any(char::is_control),
        "control bytes reach the printed step: {step:?}"
    );
    let restore = literal_guidance_spans(&step)
        .into_iter()
        .find(|span| span.starts_with("git restore"))
        .unwrap_or_else(|| panic!("no restore in {step:?}"));
    literal_guidance_bash(root, &restore);
    assert_eq!(
        fs::read_to_string(root.join("pkg").join(package).join("f")).unwrap(),
        "committed\n"
    );
}
