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

/// Files whose every read is exempt, each with the reason. Everything else marks each direct read
/// on its own line (REQ-util-003).
const ALLOWED_FILES: &[(&str, &str)] = &[
    ("src/util.rs", "the bounded project read itself lives here"),
    (
        "src/util/local_state.rs",
        "SpecSync's own state folder outside the work tree, read through no-follow, bounded handles (REQ-util-006)",
    ),
    (
        "src/git_utils.rs",
        "reads the pipes of the Git processes it starts, never a project file",
    ),
    (
        "src/mcp/confinement.rs",
        "the MCP server's own confined, bounded reads beneath its retained root",
    ),
    (
        "src/mcp/snapshot.rs",
        "copies the MCP snapshot through no-follow, bounded handles beneath its retained root",
    ),
    (
        "src/mcp/publication.rs",
        "reads back only what it staged in its own confined publication directory",
    ),
    (
        "src/watch.rs",
        "reads through its retained, no-follow, bounded watch handles, and a child's own output",
    ),
    (
        "src/manifest/max_gradle_manifest_bytes.rs",
        "reads manifests through the coverage traversal's retained, no-follow, bounded handles",
    ),
    (
        "src/commands/issues.rs",
        "reads project config only through no-follow, identity-checked, bounded handles",
    ),
];

/// A direct read elsewhere carries this marker, on its line or the line above, with the reason
/// the file it reads is not a project file or is read through a bounded handle already.
const ALLOWED_MARKER: &str = "not a project file read:";

/// Calls that read a file, or open one or a directory handle to read through (REQ-util-008). A
/// pattern that starts with a name matches only where no longer name ends in it.
const READ_CALLS: &[&str] = &[
    "fs::read(",
    "read_to_string(",
    "File::open(",
    ".read_to_end(",
    "from_reader(",
    "fs::copy(",
    ".read(true)",
    "BufReader::new(",
    "io::copy(",
    ".open_with(",
    ".open_dir(",
    "open_dir_nofollow(",
];

/// Free functions of `std::fs` that read, which an import would let a call name bare.
const READ_FUNCTIONS: &[&str] = &["read", "read_to_string", "copy"];

/// One line of a source file outside `#[cfg(test)]` items: its number from 1, the line as
/// written (where the markers are), and the same line with comments and literals blanked.
pub(crate) struct SourceLine<'a> {
    pub number: usize,
    pub raw: &'a str,
    pub code: &'a str,
}

/// `text` with every comment, string, raw string, byte string and character literal blanked to
/// spaces, byte for byte, newlines kept: what is left is code, at the same offsets and lines
/// (REQ-util-008). A lifetime or a label stays code.
pub(crate) fn code_only(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let blank = |out: &mut String, c: char| {
        if c == '\n' {
            out.push('\n');
        } else {
            out.extend(std::iter::repeat_n(' ', c.len_utf8()));
        }
    };
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                blank(&mut out, chars[i]);
                i += 1;
            }
            continue;
        }
        if c == '/' && next == Some('*') {
            let mut depth = 0usize;
            while i < chars.len() {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    blank(&mut out, chars[i]);
                    i += 1;
                }
            }
            continue;
        }
        let after_name = i > 0 && ident(chars[i - 1]);
        if !after_name && (c == 'r' || (c == 'b' && next == Some('r'))) {
            let start = if c == 'b' { i + 2 } else { i + 1 };
            let mut quote = start;
            while chars.get(quote) == Some(&'#') {
                quote += 1;
            }
            if chars.get(quote) == Some(&'"') {
                let hashes = quote - start;
                for &prefix in &chars[i..=quote] {
                    blank(&mut out, prefix);
                }
                i = quote + 1;
                while i < chars.len() {
                    if chars[i] == '"' && (0..hashes).all(|k| chars.get(i + 1 + k) == Some(&'#')) {
                        out.extend(std::iter::repeat_n(' ', 1 + hashes));
                        i += 1 + hashes;
                        break;
                    }
                    blank(&mut out, chars[i]);
                    i += 1;
                }
                continue;
            }
        }
        if c == '"' {
            out.push(' ');
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    blank(&mut out, chars[i]);
                    i += 1;
                    if i < chars.len() {
                        blank(&mut out, chars[i]);
                        i += 1;
                    }
                    continue;
                }
                let closing = chars[i] == '"';
                blank(&mut out, chars[i]);
                i += 1;
                if closing {
                    break;
                }
            }
            continue;
        }
        if c == '\'' {
            let end = if next == Some('\\') {
                let mut j = i + 3;
                while j < chars.len() && chars[j] != '\'' && chars[j] != '\n' {
                    j += 1;
                }
                (chars.get(j) == Some(&'\'')).then_some(j)
            } else if next.is_some_and(|n| n != '\n') && chars.get(i + 2) == Some(&'\'') {
                Some(i + 2)
            } else {
                None
            };
            if let Some(end) = end {
                for &literal in &chars[i..=end] {
                    blank(&mut out, literal);
                }
                i = end + 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

/// The byte offset just past the `#[cfg(test)]` item whose attribute ends at `from` in `code`:
/// past further attributes, then to the `;` that ends the item or the brace that closes it.
fn end_of_item(code: &[u8], mut from: usize) -> usize {
    loop {
        while from < code.len() && code[from].is_ascii_whitespace() {
            from += 1;
        }
        if code[from..].starts_with(b"#[") {
            let mut depth = 0i64;
            while from < code.len() {
                match code[from] {
                    b'[' => depth += 1,
                    b']' => {
                        depth -= 1;
                        if depth == 0 {
                            from += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                from += 1;
            }
            continue;
        }
        break;
    }
    // An item (`fn`, `mod`, `impl`, `use` ...) ends at its `;` or its closing brace. Anything
    // else under the attribute, a field, a match arm or a statement, also ends at a `,` outside
    // brackets, so it never takes the rest of the enclosing block with it.
    let rest = std::str::from_utf8(&code[from..]).unwrap_or_default();
    let first_word: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    let is_item = [
        "fn",
        "mod",
        "impl",
        "struct",
        "enum",
        "trait",
        "pub",
        "const",
        "static",
        "use",
        "type",
        "unsafe",
        "async",
        "extern",
        "macro_rules",
    ]
    .contains(&first_word.as_str());
    let mut braces = 0i64;
    let mut brackets = 0i64;
    while from < code.len() {
        match code[from] {
            b'(' | b'[' => brackets += 1,
            b')' | b']' => brackets -= 1,
            b';' if braces == 0 && brackets == 0 => return from + 1,
            b',' if !is_item && braces == 0 && brackets <= 0 => return from + 1,
            b'{' => braces += 1,
            b'}' => {
                braces -= 1;
                if braces <= 0 {
                    return from + 1;
                }
            }
            _ => {}
        }
        from += 1;
    }
    code.len()
}

/// The lines of `text` outside `#[cfg(test)]` items, found in `code` (the same text through
/// [`code_only`]), so a brace inside a literal or a comment never moves an item's end.
pub(crate) fn non_test_lines<'a>(text: &'a str, code: &'a str) -> Vec<SourceLine<'a>> {
    assert_eq!(text.len(), code.len(), "code_only must keep every offset");
    let bytes = code.as_bytes();
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(
            bytes
                .iter()
                .enumerate()
                .filter(|(_, b)| **b == b'\n')
                .map(|(at, _)| at + 1),
        )
        .collect();
    let line_of = |offset: usize| line_starts.partition_point(|start| *start <= offset) - 1;
    let mut test_line = vec![false; line_starts.len()];
    for attribute in ["#[cfg(test)]", "#[cfg(all(test"] {
        for (at, _) in code.match_indices(attribute) {
            let attribute_end = code[at..].find(']').map_or(code.len(), |end| at + end + 1);
            let end = end_of_item(bytes, attribute_end);
            for line in line_of(at)..=line_of(end.saturating_sub(1).max(at)) {
                test_line[line] = true;
            }
        }
    }
    text.split('\n')
        .zip(code.split('\n'))
        .enumerate()
        .filter(|(index, _)| !test_line[*index])
        .map(|(index, (raw, code))| SourceLine {
            number: index + 1,
            raw,
            code,
        })
        .collect()
}

/// Whether `code` holds one of `calls`. A call that starts with a name counts only where no
/// longer name ends in it, so `symlink(` is not found in `is_symlink(`.
pub(crate) fn has_call(code: &str, calls: &[&str]) -> bool {
    let name = |c: char| c.is_alphanumeric() || c == '_';
    calls.iter().any(|call| {
        let starts_with_name = call.chars().next().is_some_and(name);
        code.match_indices(call)
            .any(|(at, _)| !starts_with_name || !code[..at].chars().next_back().is_some_and(name))
    })
}

/// Whether `code` calls a whole-file method of a cap-std directory handle: `Dir::read(path)`,
/// which `Read::read(&mut buffer)` is told apart from by its argument, or `Dir::write(path,
/// contents)`, which `Write::write(buffer)` is told apart from by its second argument.
pub(crate) fn has_dir_file_call(code: &str, method: &str) -> bool {
    let call = format!(".{method}(");
    code.match_indices(&call).any(|(at, _)| {
        let rest = &code[at + call.len()..];
        let mut depth = 0i64;
        let mut arguments = rest.len();
        let mut commas = 0;
        for (offset, c) in rest.char_indices() {
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' if depth == 0 => {
                    arguments = offset;
                    break;
                }
                ')' | ']' | '}' => depth -= 1,
                ',' if depth == 0 => commas += 1,
                _ => {}
            }
        }
        let arguments = rest[..arguments].trim();
        match method {
            "read" => {
                !arguments.is_empty()
                    && !arguments.starts_with("&mut")
                    && arguments != "true"
                    && arguments != "false"
            }
            _ => commas > 0,
        }
    })
}

/// Whether the line is a `use std::fs::` import that names one of `functions`, so a bare call
/// would hide the read or write from the call list.
pub(crate) fn imports_fs_function(code: &str, functions: &[&str]) -> bool {
    let Some(at) = code.find("std::fs::") else {
        return false;
    };
    if !code.trim_start().starts_with("use ") && !code.trim_start().starts_with("pub use ") {
        return false;
    }
    code[at + "std::fs::".len()..]
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|name| functions.contains(&name))
}

/// Whether the call at `position` carries one of `markers`: on its own line, or on a line above
/// it in the same statement, which a builder chain spreads over several lines. The look back
/// stops at the end of the statement before (`;`, `{` or `}`), judged on code, not comments.
pub(crate) fn marked_in_statement(lines: &[SourceLine], position: usize, markers: &[&str]) -> bool {
    let has_marker = |text: &str| markers.iter().any(|marker| text.contains(marker));
    if has_marker(lines[position].raw) {
        return true;
    }
    for line in lines[..position].iter().rev() {
        if has_marker(line.raw) {
            return true;
        }
        let code = line.code.trim_end();
        if code.ends_with(';') || code.ends_with('{') || code.ends_with('}') {
            return false;
        }
    }
    false
}

/// The source files Cargo compiled into the `specsync` binary, from the dep-info it writes beside
/// it: an `include!`d file is there whatever it is named, and a `#[cfg(test)]` module file is
/// not (REQ-util-008). Fails when the list cannot be read, so a scan never passes over nothing.
pub(crate) fn compiled_sources() -> Vec<(String, PathBuf)> {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dep_info = Path::new(env!("CARGO_BIN_EXE_specsync")).with_extension("d");
    let text = fs::read_to_string(&dep_info).unwrap_or_else(|error| {
        panic!(
            "cannot read {}: {error}; the scans take their files from Cargo's dep-info",
            dep_info.display()
        )
    });
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&' ') {
            token.push(' ');
            chars.next();
        } else if c.is_whitespace() {
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
        } else {
            token.push(c);
        }
    }
    tokens.push(token);
    let source = crate_root.join("src");
    let mut found: Vec<(String, PathBuf)> = tokens
        .iter()
        .map(|token| PathBuf::from(token.trim_end_matches(':')))
        .filter(|path| {
            path.extension().is_some_and(|extension| extension == "rs")
                && path.starts_with(&source)
                && path.is_file()
        })
        .map(|path| {
            let relative = path
                .strip_prefix(crate_root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            (relative, path)
        })
        .collect();
    found.sort();
    found.dedup();
    assert!(
        found.iter().any(|(relative, _)| relative == "src/main.rs"),
        "{} does not list src/main.rs, so it is not the binary's dep-info",
        dep_info.display()
    );
    found
}

/// Verifies REQ-util-008. The lexer the scans use blanks comments and literals and nothing else,
/// so a brace in a literal never ends a test module early or late.
#[test]
fn the_scan_lexer_blanks_only_comments_and_literals() {
    let text = "fn a() { let s = \"{ x\"; let c = '{'; let e = '\\''; let r = r#\"}\"#; }\n\
                fn b<'a>(x: &'a str) -> &'a str { /* { /* } */ } */ x } // }\n\
                let bytes = b\"{\"; let byte = b'}'; let raw = br\"{\";\n";
    let code = code_only(text);
    assert_eq!(code.len(), text.len());
    assert_eq!(
        code.matches('{').count(),
        code.matches('}').count(),
        "{code}"
    );
    assert!(code.contains("fn b<'a>(x: &'a str) -> &'a str {"), "{code}");
    let module = "#[cfg(test)]\nmod tests {\n    const S: &str = \"{ unbalanced\";\n}\nfn after() { fs::read(p); }\n";
    let code = code_only(module);
    let kept: Vec<usize> = non_test_lines(module, &code)
        .iter()
        .map(|line| line.number)
        .collect();
    assert_eq!(
        kept,
        vec![5, 6],
        "the line after the test module is production code"
    );
}

/// Verifies REQ-util-003, REQ-util-008 and the module requirements that rely on them. Every
/// read of a project file goes through the bounded read; a new direct read fails here until it
/// does, or until it says why the file it reads is not a project file.
#[test]
fn no_project_file_is_read_outside_the_bounded_read() {
    let mut direct = Vec::new();
    for (relative, path) in compiled_sources() {
        if ALLOWED_FILES.iter().any(|(file, _)| *file == relative) {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let code = code_only(&text);
        let lines = non_test_lines(&text, &code);
        for (position, line) in lines.iter().enumerate() {
            if !has_call(line.code, READ_CALLS)
                && !has_dir_file_call(line.code, "read")
                && !imports_fs_function(line.code, READ_FUNCTIONS)
            {
                continue;
            }
            if marked_in_statement(&lines, position, &[ALLOWED_MARKER]) {
                continue;
            }
            direct.push(format!("{relative}:{}: {}", line.number, line.raw.trim()));
        }
    }
    assert!(
        direct.is_empty(),
        "{} direct file read(s) bypass the bounded project read (REQ-util-003):\n{}",
        direct.len(),
        direct.join("\n")
    );
}
