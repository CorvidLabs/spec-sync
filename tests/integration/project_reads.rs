//! SpecSync never reads a project file through a link that leaves the project, never waits on a
//! file that is not a regular file, and never reads past a size bound (REQ-util-003). Git stores
//! symbolic links, so a repository can make any project path a link to `/dev/zero`, a FIFO or a
//! file of whoever clones it.
//!
//! Each fixture is one temporary directory holding `project/` and `outside/`, so a link that
//! leaves the project still reaches nothing but the fixture. A FIFO stands in for a device: a
//! command that opens it to read blocks, and the timeout turns that into a failure instead of a
//! hang. Over-bound files are sparse. Nothing here reads `/dev/zero`.

use crate::helpers::*;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Long enough for any command here on a loaded machine, short enough to fail fast on a hang.
const TIMEOUT: Duration = Duration::from_secs(20);

struct Fixture {
    _tmp: TempDir,
    root: PathBuf,
    outside: PathBuf,
}

impl Fixture {
    /// Run `specsync <args> --root <project>` under the timeout. Panics if the command had to be
    /// killed: that is a read that waited on a file instead of refusing it.
    fn run(&self, args: &[&str]) -> Output {
        let started = Instant::now();
        let output = specsync()
            .args(args)
            .arg("--root")
            .arg(&self.root)
            .timeout(TIMEOUT)
            .output()
            .unwrap();
        assert!(
            output.status.code().is_some(),
            "`specsync {}` was still running after {:?}: a read waited on a file instead of refusing it",
            args.join(" "),
            started.elapsed()
        );
        output
    }

    /// Replace `relative` with a FIFO that nothing ever writes to.
    fn fifo(&self, relative: &str) {
        let path = self.root.join(relative);
        let _ = fs::remove_file(&path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let status = std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap();
        assert!(status.success(), "mkfifo {}", path.display());
    }

    /// Replace `relative` with a relative link to `outside/<name>`, as a clone carries it.
    fn link_out(&self, relative: &str, name: &str, content: &str) {
        fs::write(self.outside.join(name), content).unwrap();
        let path = self.root.join(relative);
        let _ = fs::remove_file(&path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let depth = Path::new(relative).components().count();
        symlink(format!("{}outside/{name}", "../".repeat(depth)), &path).unwrap();
    }

    /// Replace `relative` with a sparse file of `len` bytes.
    fn sparse(&self, relative: &str, len: u64) {
        let path = self.root.join(relative);
        let _ = fs::remove_file(&path);
        fs::File::create(&path).unwrap().set_len(len).unwrap();
    }
}

fn said(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// A project with one spec, set up by `init`, with the change workflow switched on.
fn project() -> Fixture {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
    let fixture = Fixture {
        _tmp: tmp,
        root,
        outside,
    };
    let init = fixture.run(&["init"]);
    assert!(init.status.success(), "{}", said(&init));
    let adopt = fixture.run(&["change", "adopt"]);
    assert!(adopt.status.success(), "{}", said(&adopt));
    fixture
}

// ─── The policy and the transaction journal (the tester's finding) ──────

/// Verifies REQ-change-161. A FIFO at `.specsync/sdd.json` made `change status --json`, which
/// Chough's `next` and `status` run, wait forever.
#[test]
fn change_status_does_not_wait_on_a_policy_that_is_not_a_regular_file() {
    let fx = project();
    fx.fifo(".specsync/sdd.json");
    let output = fx.run(&["change", "status", "--json"]);
    assert!(
        said(&output).contains(".specsync/sdd.json is not a regular file"),
        "{}",
        said(&output)
    );
}

/// Verifies REQ-change-161. A policy linked to a file outside the project is not read.
#[test]
fn change_status_does_not_read_a_policy_linked_outside_the_project() {
    let fx = project();
    let policy = fs::read_to_string(fx.root.join(".specsync/sdd.json")).unwrap();
    fx.link_out(".specsync/sdd.json", "policy.json", &policy);
    let output = fx.run(&["change", "status", "--json"]);
    assert!(
        said(&output).contains(".specsync/sdd.json is a symbolic link that leaves the project"),
        "the policy was read through a link to a file outside the project: {}",
        said(&output)
    );
}

/// Verifies REQ-change-161 and REQ-util-003. The policy is read to at most its bound.
#[test]
fn change_status_refuses_a_policy_over_its_bound() {
    let fx = project();
    fx.sparse(".specsync/sdd.json", 4 * 1024 * 1024 + 1);
    let output = fx.run(&["change", "status", "--json"]);
    assert!(
        said(&output).contains(".specsync/sdd.json is larger than 4194304 bytes"),
        "{}",
        said(&output)
    );
}

/// Verifies REQ-change-161. The transaction journal is read before every lifecycle write.
#[test]
fn a_lifecycle_write_does_not_wait_on_or_follow_the_transaction_journal() {
    for planted in ["fifo", "outside"] {
        let fx = project();
        if planted == "fifo" {
            fx.fifo(".specsync/change-transaction.json");
        } else {
            fx.link_out(".specsync/change-transaction.json", "journal.json", "[]");
        }
        let output = fx.run(&[
            "change",
            "new",
            "Probe the journal",
            "--spec",
            "auth",
            "--path",
            "src/auth/service.ts",
        ]);
        let reason = if planted == "fifo" {
            "is not a regular file"
        } else {
            "is a symbolic link that leaves the project"
        };
        assert!(
            said(&output).contains(&format!(".specsync/change-transaction.json {reason}")),
            "{planted}: {}",
            said(&output)
        );
        assert!(!output.status.success(), "{planted}");
    }
}

/// Verifies REQ-cmd-change-038. `change audit` reads the policy too.
#[test]
fn change_audit_does_not_wait_on_the_policy() {
    let fx = project();
    fx.fifo(".specsync/sdd.json");
    let output = fx.run(&["change", "audit"]);
    assert!(
        said(&output).contains(".specsync/sdd.json is not a regular file"),
        "{}",
        said(&output)
    );
}

// ─── Config, ignore rules, the hash cache ────────────────────────────────

/// Verifies REQ-config-016. Config loading refuses the FIFO without waiting; the confined
/// discovery that follows then names it and stops the run.
#[test]
fn check_does_not_wait_on_a_config_file_that_is_not_a_regular_file() {
    let fx = project();
    fx.fifo(".specsync/config.toml");
    let output = fx.run(&["check"]);
    assert!(!output.status.success());
    assert!(
        said(&output).contains(".specsync/config.toml"),
        "{}",
        said(&output)
    );
}

/// Verifies REQ-ignore-002. An ignore file that cannot be read is no ignore file, as before;
/// what changes is that a FIFO no longer holds the run.
#[test]
fn check_does_not_wait_on_an_ignore_file_that_is_not_a_regular_file() {
    let fx = project();
    fx.fifo(".specsyncignore");
    let output = fx.run(&["check"]);
    assert!(output.status.success(), "{}", said(&output));
}

/// Verifies REQ-hash-cache-008. A FIFO companion is hashed to classify the spec.
#[test]
fn check_does_not_wait_on_a_companion_that_is_not_a_regular_file() {
    let fx = project();
    fx.fifo("specs/auth/requirements.md");
    fx.run(&["check"]);
}

/// Verifies REQ-hash-cache-008. The hash cache is refused over its bound, and so not saved over.
#[test]
fn check_refuses_a_hash_cache_over_its_bound() {
    let fx = project();
    fx.sparse(".specsync/hashes.json", 64 * 1024 * 1024 + 1);
    let output = fx.run(&["check"]);
    assert!(
        said(&output).contains(".specsync/hashes.json is larger than 67108864 bytes"),
        "{}",
        said(&output)
    );
    assert_eq!(
        fs::metadata(fx.root.join(".specsync/hashes.json"))
            .unwrap()
            .len(),
        64 * 1024 * 1024 + 1,
        "a refused cache is not saved over"
    );
}

/// Verifies REQ-validator-049. A requirements file linked outside the project is not read, so
/// the duplicate ID in the outside file is never reported.
#[test]
fn check_does_not_read_requirements_linked_outside_the_project() {
    let fx = project();
    fx.link_out(
        "specs/auth/requirements.md",
        "requirements.md",
        "# Requirements\n\n### REQ-auth-001\n\nOne.\n\n### REQ-auth-001\n\nOne again.\n",
    );
    let output = fx.run(&["check", "--force"]);
    assert!(
        !said(&output).contains("appears 2 times"),
        "requirement IDs were read through a link to a file outside the project: {}",
        said(&output)
    );
}

/// Verifies REQ-exports-011. A source file a spec lists, linked outside the project, is not read
/// for exports.
#[test]
fn check_does_not_read_exports_from_a_source_linked_outside_the_project() {
    let fx = project();
    fx.link_out(
        "src/auth/service.ts",
        "service.ts",
        "export function login() {}\nexport function outsideOnlySymbol() {}\n",
    );
    let output = fx.run(&["check", "--force"]);
    assert!(
        !said(&output).contains("outsideOnlySymbol"),
        "exports were read through a link to a file outside the project: {}",
        said(&output)
    );
}

// ─── State and hook files ────────────────────────────────────────────────

/// Verifies REQ-mcp-012.
#[test]
fn mcp_diff_does_not_wait_on_a_lock_that_is_not_a_regular_file() {
    let fx = project();
    fx.fifo(".specsync/mcp-tools.lock.json");
    let output = fx.run(&["mcp", "diff"]);
    assert!(!output.status.success());
    assert!(
        said(&output).contains(".specsync/mcp-tools.lock.json is not a regular file"),
        "{}",
        said(&output)
    );
}

/// Verifies REQ-agents-012.
#[test]
fn agents_install_does_not_wait_on_a_manifest_that_is_not_a_regular_file() {
    let fx = project();
    fx.fifo(".specsync/agent-artifacts.json");
    let output = fx.run(&["agents", "install", "--claude"]);
    assert!(!output.status.success());
    assert!(
        said(&output).contains(".specsync/agent-artifacts.json is not a regular file"),
        "{}",
        said(&output)
    );
}

/// Verifies REQ-archive-002.
#[test]
fn archive_tasks_does_not_wait_on_a_tasks_file_that_is_not_a_regular_file() {
    let fx = project();
    fx.fifo("specs/auth/tasks.md");
    fx.run(&["archive-tasks"]);
}

/// Verifies REQ-hooks-007.
#[test]
fn hooks_status_does_not_wait_on_a_hook_file_that_is_not_a_regular_file() {
    let fx = project();
    fx.fifo("CLAUDE.md");
    fx.run(&["hooks", "status"]);
}

/// Verifies REQ-util-003. A link that stays inside the project is still read.
#[test]
fn a_policy_linked_inside_the_project_is_still_read() {
    let fx = project();
    fs::rename(
        fx.root.join(".specsync/sdd.json"),
        fx.root.join(".specsync/policy.json"),
    )
    .unwrap();
    symlink("policy.json", fx.root.join(".specsync/sdd.json")).unwrap();
    let output = fx.run(&["change", "status", "--json"]);
    assert!(output.status.success(), "{}", said(&output));
    assert!(!said(&output).contains("sdd.json is"), "{}", said(&output));
}

// ─── A committed transaction journal (F1) ────────────────────────────────

/// The journal exactly as SpecSync writes one, with its integrity envelope, so the test does not
/// rely on the bare-array form a fix might refuse for that reason alone.
fn durable_journal(entries: &[(&str, Option<&str>)]) -> String {
    use sha2::{Digest, Sha256};
    let rendered: Vec<String> = entries
        .iter()
        .map(|(path, original)| {
            format!(
                r#"{{"path":{},"original":{}}}"#,
                serde_json::to_string(path).unwrap(),
                serde_json::to_string(original).unwrap()
            )
        })
        .collect();
    let array = format!("[{}]", rendered.join(","));
    let digest = format!("{:x}", Sha256::digest(array.as_bytes()));
    format!(
        r#"{{"schema_version":1,"entry_count":{},"entries_digest":"{digest}","entries":{array}}}"#,
        entries.len()
    )
}

/// Verifies REQ-change-162 through the CLI. `change new`, the first step of the change
/// workflow in a fresh clone, takes the lock and so replays the journal: a committed one must
/// never rewrite `.git/config`.
#[test]
fn change_new_never_replays_a_committed_journal_into_git() {
    let fx = project();
    let git = |args: &[&str]| {
        assert!(
            std::process::Command::new("git")
                .args(args)
                .current_dir(&fx.root)
                .status()
                .unwrap()
                .success(),
            "git {args:?}"
        );
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "journal@example.invalid"]);
    git(&["config", "user.name", "journal test"]);
    git(&["config", "commit.gpgsign", "false"]);
    let config = fs::read(fx.root.join(".git/config")).unwrap();
    fs::write(
        fx.root.join(".specsync/change-transaction.json"),
        durable_journal(&[(".git/config", Some("[core]\n\tfsmonitor = touch planted\n"))]),
    )
    .unwrap();
    // `init`'s `.specsync/.gitignore` ignores the journal; a repository that wants one tracked
    // adds it with `-f`.
    git(&["add", "-A"]);
    git(&["add", "-f", ".specsync/change-transaction.json"]);
    git(&["commit", "-q", "-m", "a repository that carries a journal"]);

    let output = fx.run(&[
        "change",
        "new",
        "Probe the journal",
        "--spec",
        "auth",
        "--path",
        "src/auth/service.ts",
    ]);

    assert_eq!(
        fs::read(fx.root.join(".git/config")).unwrap(),
        config,
        "a committed journal rewrote .git/config: {}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
    assert!(
        said(&output).contains("tracked by Git"),
        "{}",
        said(&output)
    );
}

// ─── The guard ───────────────────────────────────────────────────────────

/// Files whose every read is outside this rule: `util` holds the bounded read itself; the others
/// already read through a confined handle with their own bounds, read Git's output, or read only a
/// path the person names on the command line.
const ALLOWED_FILES: &[&str] = &[
    "src/util.rs",
    "src/watch.rs",
    "src/git_utils.rs",
    "src/commands/issues.rs",
    "src/commands/import.rs",
    "src/mcp/confinement.rs",
    "src/mcp/snapshot.rs",
    "src/manifest/max_gradle_manifest_bytes.rs",
    "src/validator/consumed_by_re.rs",
];

/// A direct read elsewhere carries this marker, on its line or the line above, with the reason
/// the file it reads is not a project file or is read through a bounded handle already.
const ALLOWED_MARKER: &str = "not a project file read:";

const READ_CALLS: &[&str] = &[
    "fs::read(",
    "fs::read_to_string(",
    "File::open(",
    ".read_to_end(",
    ".read_to_string(",
    "from_reader(",
];

/// The lines of `text` outside `#[cfg(test)]` items, numbered from 1.
fn non_test_lines(text: &str) -> Vec<(usize, &str)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut kept = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if lines[index].trim_start().starts_with("#[cfg(test)]") {
            let mut next = index + 1;
            while next < lines.len() {
                let line = lines[next].trim_start();
                if line.starts_with("#[") || line.starts_with("///") || line.is_empty() {
                    next += 1;
                } else {
                    break;
                }
            }
            if next < lines.len() && lines[next].trim_end().ends_with(';') {
                index = next + 1;
                continue;
            }
            let mut depth: i64 = 0;
            let mut opened = false;
            while next < lines.len() {
                depth += lines[next].matches('{').count() as i64;
                depth -= lines[next].matches('}').count() as i64;
                opened |= lines[next].contains('{');
                next += 1;
                if opened && depth <= 0 {
                    break;
                }
            }
            index = next;
            continue;
        }
        kept.push((index + 1, lines[index]));
        index += 1;
    }
    kept
}

fn rust_sources(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_sources(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// Verifies REQ-util-003 and the module requirements that rely on it. Every read of a project
/// file goes through the bounded read; a new direct read fails here until it does, or until it
/// says why the file it reads is not a project file.
#[test]
fn no_project_file_is_read_outside_the_bounded_read() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = Vec::new();
    rust_sources(&crate_root.join("src"), &mut sources);
    sources.sort();
    let mut direct = Vec::new();
    for path in sources {
        let relative = path
            .strip_prefix(crate_root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let name = path.file_name().unwrap().to_string_lossy();
        if ALLOWED_FILES.contains(&relative.as_str())
            || name == "tests.rs"
            || relative.contains("_tests")
            || relative.contains("test_support")
        {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let lines = non_test_lines(&text);
        for (position, (number, line)) in lines.iter().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") || !READ_CALLS.iter().any(|call| line.contains(call)) {
                continue;
            }
            let previous = position
                .checked_sub(1)
                .map(|before| lines[before].1)
                .unwrap_or("");
            if line.contains(ALLOWED_MARKER) || previous.contains(ALLOWED_MARKER) {
                continue;
            }
            direct.push(format!("{relative}:{number}: {}", code.trim()));
        }
    }
    assert!(
        direct.is_empty(),
        "{} direct file read(s) bypass the bounded project read (REQ-util-003):\n{}",
        direct.len(),
        direct.join("\n")
    );
}
