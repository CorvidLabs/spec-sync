
fn semantic_acceptance_item_exists_for_module(
    root: &Path,
    record: &ChangeRecord,
    module: &str,
) -> Result<bool, String> {
    let content =
        read_bounded_change_text(&delta_path_checked(root, record, module)?, "semantic delta")?;
    Ok(parse_delta(&content)?.iter().any(|item| {
        matches!(
            item.target,
            DeltaTarget::Requirement | DeltaTarget::SpecSection
        ) && item.operation != DeltaOperation::Removed
            && !item.content.trim().is_empty()
    }))
}

/// The acceptance-entry digest `predecessor_id` signs for `path` in the checkout at `tree`.
///
/// `tree` is a detached worktree of one commit for `acceptance_entry_digest_at_commit`, and the
/// project root itself for the one successor whose acceptance is not in history yet (see
/// `semantic_tuple_transition_is_valid`).
fn acceptance_entry_digest_in_tree(
    tree: &Path,
    predecessor_id: &str,
    path: &str,
) -> Result<String, String> {
    let mut predecessor = load_change(tree, predecessor_id)?;
    predecessor.state = ChangeState::Accepted;
    predecessor.affected_paths = vec![path.to_string()];
    let manifest = acceptance_manifest(tree, &predecessor, &[])?;
    manifest
        .entries
        .iter()
        .find(|entry| entry.path == path)
        .map(|entry| entry.entry_digest.clone())
        .ok_or_else(|| {
            format!("succession base tree predecessor `{predecessor_id}` has no entry for `{path}`")
        })
}

fn acceptance_entry_digest_at_commit(
    root: &Path,
    commit: &str,
    predecessor_id: &str,
    path: &str,
) -> Result<String, String> {
    let temporary = create_effective_contract_workspace()?;
    let tree = temporary.join("tree");
    let added = Command::new("git")
        .args([
            "worktree",
            "add",
            "--detach",
            tree.to_string_lossy().as_ref(),
            commit,
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to inspect succession base tree: {error}"))?;
    if !added.status.success() {
        let _ = fs::remove_dir_all(&temporary);
        return Err(format!(
            "failed to inspect succession base tree: {}",
            String::from_utf8_lossy(&added.stderr).trim()
        ));
    }
    let result = acceptance_entry_digest_in_tree(&tree, predecessor_id, path);
    let removed = Command::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            tree.to_string_lossy().as_ref(),
        ])
        .current_dir(root)
        .output();
    let _ = fs::remove_dir_all(&temporary);
    if !removed.is_ok_and(|output| output.status.success()) {
        return Err("failed to remove succession base workspace".into());
    }
    result
}

/// `pending` is the archive preflight's closing token, forwarded to the successor's transition
/// lookup so a package being re-finalized after a reopen (#540) can present its closing evidence.
/// See `PendingArchiveClose`.
fn semantic_tuple_transition_is_valid(
    root: &Path,
    successor: &ChangeRecord,
    tuple: &SemanticSuccessionTupleV1,
    pending: Option<&PendingArchiveClose<'_>>,
) -> Result<bool, String> {
    let Some(base) = successor.base_commit.as_deref() else {
        return Ok(false);
    };
    let base_old =
        acceptance_entry_digest_at_commit(root, base, &tuple.predecessor_id, &tuple.path)?;
    if base_old != tuple.predecessor_entry_digest {
        return Ok(false);
    }
    let (anchor, _, _) = match authenticated_accepted_transition_for(root, successor, pending) {
        Ok(transition) => transition,
        Err(_) => return Ok(false),
    };
    // History holds no acceptance transition for the package `finalize` is closing right now,
    // nor for an archive whose commit has not been made yet: the transition is the working-tree
    // closing evidence, and its label is not a commit. Handing the label to git read as "the
    // tuple does not hold", which is how the only successor able to cover a legacy change was
    // dropped during its own finalize. That evidence was admitted for the tree it was signed
    // against -- the working tree, whose base ancestry `accept_change_with_gate` checked against
    // HEAD -- so that is where the successor entry is read from.
    let closing = anchor == WORKING_TREE_CLOSING_EVIDENCE_ANCHOR;
    let ancestor = Command::new("git")
        .args([
            "merge-base",
            "--is-ancestor",
            base,
            if closing { "HEAD" } else { anchor.as_str() },
        ])
        .current_dir(root)
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to validate accepted transition ancestry: {error}"))?;
    if !ancestor.success() {
        return Ok(false);
    }
    let successor_digest = if closing {
        acceptance_entry_digest_in_tree(root, &successor.id, &tuple.path)
    } else {
        acceptance_entry_digest_at_commit(root, &anchor, &successor.id, &tuple.path)
    };
    Ok(successor_digest.is_ok_and(|digest| digest == tuple.successor_entry_digest))
}

fn acceptance_input_owners(
    root: &Path,
    record: &ChangeRecord,
    relative: &str,
    overrides: &[(PathBuf, String)],
    evidence: &GitEvidence,
    unowned_source: UnownedProductionSource,
) -> Result<Vec<String>, String> {
    let config = crate::config::load_config(root);
    // Configured ownership is decided before the reserved exact classes. A test, fixture, or
    // delivery path that a declared module owns through `[modules."<name>"] owns` is that
    // module's input, not `@exact:test` or `@exact:delivery` -- which is what lets a later
    // change supersede it instead of reopening the change that signed it. A spec's `files:`
    // list does not do this: a mapped test stays exact-only, as it always has.
    let mut owners: Vec<String> = record
        .affected_specs
        .iter()
        .filter(|module| configured_module_owns_path(&config, module, relative))
        .cloned()
        .collect();
    if owners.is_empty() {
        if path_is_governed_test_or_fixture(relative) {
            return Ok(vec![EXACT_TEST_OWNER.to_string()]);
        }
        if path_is_recognized_delivery_metadata(relative) {
            return Ok(vec![EXACT_DELIVERY_OWNER.to_string()]);
        }
    }
    let override_content: BTreeMap<String, &str> = overrides
        .iter()
        .map(|(path, content)| Ok((strict_portable_project_path(root, path)?, content.as_str())))
        .collect::<Result<_, String>>()?;
    for module in &record.affected_specs {
        let (spec_path, requirements_path) =
            canonical_module_paths(root, &config.specs_dir, module)?;
        let spec_relative = strict_portable_project_path(root, &spec_path)?;
        let parent = spec_path
            .parent()
            .ok_or_else(|| format!("canonical spec path has no parent: {}", spec_path.display()))?;
        let canonical_owned = relative == spec_relative
            || relative == strict_portable_project_path(root, &requirements_path)?
            || canonical_companion_names(module).iter().any(|name| {
                strict_portable_project_path(root, &parent.join(name))
                    .is_ok_and(|path| path == relative)
            });
        let source_owned =
            if canonical_owned {
                false
            } else {
                let content = if let Some(content) = override_content.get(&spec_relative) {
                    Some((*content).to_string())
                } else {
                    evidence
                        .entry(&spec_relative)
                        .ok()
                        .and_then(|entry| String::from_utf8(entry.payload.clone()).ok())
                };
                content
                    .as_deref()
                    .and_then(crate::parser::parse_frontmatter)
                    .is_some_and(|parsed| {
                        parsed.frontmatter.files.iter().any(|path| {
                            normalize_project_path(path).is_ok_and(|path| path == relative)
                        })
                    })
            };
        if canonical_owned || source_owned {
            owners.push(module.clone());
        }
    }
    owners.extend(
        record
            .acceptance_owner_corrections
            .iter()
            .filter(|correction| correction.path == relative)
            .map(|correction| correction.module.clone()),
    );
    if owners.is_empty() {
        if path_is_production_source(root, relative) {
            if unowned_source == UnownedProductionSource::AssignExactDelivery {
                // Legacy reconstruction only: adoption-era records predate canonical
                // ownership, so unowned production source routes to the exact delivery
                // owner instead of aborting the immutable archived ledger.
                owners.push(EXACT_DELIVERY_OWNER.to_string());
            } else {
                return Err(format!(
                    "acceptance input `{relative}` is production source without deterministic canonical ownership"
                ));
            }
        } else {
            owners.push(EXACT_DELIVERY_OWNER.to_string());
        }
    }
    owners.sort();
    owners.dedup();
    Ok(owners)
}

fn path_is_recognized_delivery_metadata(path: &str) -> bool {
    if path.starts_with(".github/") || path.starts_with(".specsync/") || path.starts_with("docs/") {
        return true;
    }
    is_protected_sdd_path(path)
        || (!path.contains('/')
            && matches!(
                path,
                "fledge.toml"
                    | ".trust.toml"
                    | "README.md"
                    | "LICENSE"
                    | "AGENTS.md"
                    | "Cargo.toml"
                    | "Cargo.lock"
                    | "package.json"
                    | "package-lock.json"
                    | "bun.lock"
                    | "bun.lockb"
                    | "pnpm-lock.yaml"
                    | "yarn.lock"
                    | "pyproject.toml"
                    | "uv.lock"
                    | "requirements.txt"
                    | "go.mod"
                    | "go.sum"
                    | "Package.swift"
                    | "Package.resolved"
                    | "action.yml"
                    | "action.yaml"
            ))
}

fn path_is_governed_test_or_fixture(path: &str) -> bool {
    path.starts_with("tests/")
        || path.starts_with("test/")
        || path.starts_with("Tests/")
        || path.contains("/tests/")
        || path.contains("/test/")
        || path.contains("/fixtures/")
        || path.contains("/__fixtures__/")
}

/// Whether `[modules."<module>"] owns` in the project configuration gives `module` the path
/// `relative`: an exact project-relative path, or a directory that owns everything beneath it,
/// matched the way `affected_paths` scopes are. Ownership declared here reaches acceptance
/// manifests and semantic succession only -- it is not a source mapping, so `specsync check`
/// demands no spec coverage for an owned path. The lifecycle's own ledger under `.specsync/`
/// is never configurable: its reserved exact owners are what the sequence-ledger succession
/// rule reads.
fn configured_module_owns_path(
    config: &crate::types::SpecSyncConfig,
    module: &str,
    relative: &str,
) -> bool {
    if !ownership_is_configurable(relative) {
        return false;
    }
    config.modules.get(module).is_some_and(|definition| {
        definition.owns.iter().any(|scope| {
            normalize_project_path(scope).is_ok_and(|scope| path_matches_scope(relative, &scope))
        })
    })
}

/// Whether the project configuration may grant a module ownership of `relative`.
fn ownership_is_configurable(relative: &str) -> bool {
    !relative.starts_with(".specsync/") && !is_protected_sdd_path(relative)
}

fn path_is_production_source(root: &Path, path: &str) -> bool {
    crate::exports::is_source_file(Path::new(path))
        && crate::config::load_config(root)
            .source_dirs
            .iter()
            .any(|source| {
                let normalized = source.trim_matches('/');
                normalized == "."
                    || path == normalized
                    || path.starts_with(&format!("{normalized}/"))
            })
        && !path_is_governed_test_or_fixture(path)
}

fn rooted_git_command(root: &Path) -> Command {
    let mut command = Command::new("git");
    command.current_dir(root);
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_PREFIX",
        "GIT_CEILING_DIRECTORIES",
        "GIT_DISCOVERY_ACROSS_FILESYSTEM",
        "GIT_EXEC_PATH",
        "GIT_NAMESPACE",
        "GIT_SHALLOW_FILE",
        "GIT_GRAFT_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        command.env_remove(key);
    }
    command
}

fn configured_git_command(root: &Path) -> Command {
    let mut command = rooted_git_command(root);
    command.env_remove("GIT_CONFIG");
    command.env_remove("GIT_CONFIG_PARAMETERS");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_CONFIG_KEY_")
            || key.to_string_lossy().starts_with("GIT_CONFIG_VALUE_")
        {
            command.env_remove(key);
        }
    }
    command.env_remove("GIT_CONFIG_COUNT");
    command.env("GIT_CONFIG_NOSYSTEM", "1");
    command.env(
        "GIT_CONFIG_GLOBAL",
        if cfg!(windows) { "NUL" } else { "/dev/null" },
    );
    if let Some(index) = std::env::var_os("GIT_INDEX_FILE") {
        let index = PathBuf::from(index);
        let effective = if index.is_absolute() {
            index
        } else {
            root.join(index)
        };
        command.env("GIT_INDEX_FILE", effective);
    } else {
        command.env_remove("GIT_INDEX_FILE");
    }
    command
}

fn read_bounded_pipe<Reader: Read>(
    mut reader: Reader,
    cap: usize,
    overflow: Arc<AtomicBool>,
) -> std::io::Result<Vec<u8>> {
    let mut retained = Vec::with_capacity(cap.min(64 * 1024));
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = cap.saturating_sub(retained.len());
        let keep = remaining.min(read);
        retained.extend_from_slice(&buffer[..keep]);
        if keep != read {
            overflow.store(true, Ordering::Release);
        }
    }
    Ok(retained)
}

fn run_git_bounded(
    root: &Path,
    args: &[&str],
    input: Option<Vec<u8>>,
    stdout_cap: usize,
) -> Result<BoundedCommandOutput, String> {
    let mut command = configured_git_command(root);
    run_git_command_bounded(&mut command, args, input, stdout_cap)
}

fn run_git_command_bounded(
    command: &mut Command,
    args: &[&str],
    input: Option<Vec<u8>>,
    stdout_cap: usize,
) -> Result<BoundedCommandOutput, String> {
    run_git_command_bounded_with_deadline(command, args, input, stdout_cap, GIT_COMMAND_DEADLINE)
}

fn run_git_command_bounded_with_deadline(
    command: &mut Command,
    args: &[&str],
    input: Option<Vec<u8>>,
    stdout_cap: usize,
    deadline: Duration,
) -> Result<BoundedCommandOutput, String> {
    command
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    record_test_git_process();
    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to run `git {}`: {error}", args.join(" ")))?;
    #[cfg(test)]
    TEST_GIT_CHILD_SPAWNED.with(|hook| {
        if let Some(hook) = hook.borrow_mut().take() {
            hook(child.id());
        }
    });
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err("failed to capture Git stdout".to_string());
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            drop(stdout);
            let _ = child.kill();
            let _ = child.wait();
            return Err("failed to capture Git stderr".to_string());
        }
    };
    let overflow = Arc::new(AtomicBool::new(false));
    let stdout_overflow = Arc::clone(&overflow);
    let stderr_overflow = Arc::clone(&overflow);
    let stdout_reader =
        std::thread::spawn(move || read_bounded_pipe(stdout, stdout_cap, stdout_overflow));
    let stderr_reader = std::thread::spawn(move || {
        read_bounded_pipe(stderr, MAX_GIT_DIAGNOSTIC_BYTES, stderr_overflow)
    });
    let writer = input.map(|input| {
        let mut stdin = child.stdin.take();
        std::thread::spawn(move || -> std::io::Result<()> {
            if let Some(stdin) = stdin.as_mut() {
                stdin.write_all(&input)?;
            }
            Ok(())
        })
    });
    let started = Instant::now();
    let mut command_error = None;
    let status = loop {
        if overflow.load(Ordering::Acquire) {
            let _ = child.kill();
            break child
                .wait()
                .map_err(|error| format!("failed to reap Git subprocess: {error}"));
        }
        if started.elapsed() >= deadline {
            command_error = Some(format!(
                "`git {}` exceeded its {:?} wall-clock deadline",
                args.join(" "),
                deadline
            ));
            let _ = child.kill();
            break child
                .wait()
                .map_err(|error| format!("failed to reap timed-out Git subprocess: {error}"));
        }
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => {}
            Err(error) => {
                command_error = Some(format!("failed to inspect Git subprocess: {error}"));
                let _ = child.kill();
                break child
                    .wait()
                    .map_err(|wait_error| format!("failed to reap Git subprocess: {wait_error}"));
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| "Git stdout reader panicked".to_string())
        .and_then(|result| result.map_err(|error| format!("failed to read Git stdout: {error}")));
    let stderr = stderr_reader
        .join()
        .map_err(|_| "Git stderr reader panicked".to_string())
        .and_then(|result| result.map_err(|error| format!("failed to read Git stderr: {error}")));
    let write_result = writer.map(|writer| {
        let joined = writer.join();
        #[cfg(test)]
        TEST_GIT_STDIN_WRITER_JOINED.with(|observed| observed.set(true));
        joined
            .map_err(|_| "Git stdin writer panicked".to_string())
            .and_then(|result| {
                result.map_err(|error| format!("failed to write Git stdin: {error}"))
            })
    });
    let status = status?;
    let stdout = stdout?;
    let stderr = stderr?;
    if !overflow.load(Ordering::Acquire)
        && command_error.is_none()
        && let Some(write_result) = write_result
    {
        write_result?;
    }
    if overflow.load(Ordering::Acquire) {
        return Err(format!(
            "`git {}` output exceeds deterministic bounds",
            args.join(" ")
        ));
    }
    if let Some(error) = command_error {
        return Err(error);
    }
    Ok(BoundedCommandOutput {
        status,
        stdout,
        stderr,
    })
}

fn run_git_required(
    root: &Path,
    args: &[&str],
    input: Option<Vec<u8>>,
    stdout_cap: usize,
) -> Result<Vec<u8>, String> {
    let output = run_git_bounded(root, args, input, stdout_cap)?;
    if !output.status.success() {
        return Err(format!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

/// The four `core.*` keys the checkout overrides are derived from, read in one query.
///
/// These were four separate `git config --get` spawns. On a lifecycle-heavy run that is the
/// single largest source of subprocess cost: one suite run issued 15,359 `git config` spawns,
/// which this takes to 3,842. At ~15 ms per spawn that is the dominant term in every command
/// that inspects the checkout, not only in tests.
///
/// Deliberately NOT cached. Every call still spawns, so a configuration change between calls is
/// still observed — the saving comes from asking once for four answers, never from remembering
/// an answer.
///
/// `core.fsmonitor` is deliberately excluded. It is read through `configured_git_command`, which
/// scrubs system, global and injected configuration; folding it into this query — which is built
/// on `rooted_git_command`, and must be, to preserve the injected/global precedence the callers
/// rely on — would silently change how fsmonitor resolves.
const CHECKOUT_CORE_KEYS: [&str; 4] = [
    "core.autocrlf",
    "core.eol",
    "core.symlinks",
    "core.filemode",
];

/// Reads the four keys in a single `--get-regexp`, keeping the LAST value per key.
///
/// Equivalence to four `--get` calls was verified against git 2.50.1 rather than assumed:
/// a multi-valued key lists in order and `--get` returns the last, so last-wins matches;
/// a valueless key yields a record with no `\n`, i.e. the empty value `--get` reports at rc=0;
/// a mixed-case section (`[CORE] FileMode`) is emitted lowercased, as the callers already
/// expect; no matching key gives rc=1 with empty stdout and stderr, exactly as `--get` does for
/// an unset key; and a malformed config gives rc=128 with stderr on both, so the fail-loudly
/// rule is preserved rather than collapsed into "unset".
fn core_config_snapshot_from_command(
    command: &mut Command,
) -> Result<BTreeMap<String, String>, String> {
    command.env("GIT_PAGER", "cat");
    let output = run_git_command_bounded(
        command,
        &[
            "config",
            "-z",
            "--get-regexp",
            "^core[.](autocrlf|eol|symlinks|filemode)$",
        ],
        None,
        // Four `--get` calls each returned about six bytes, so the 128-byte bound they shared
        // was never near the limit. `--get-regexp` returns every occurrence of all four keys
        // across every scope, so the ordinary global-plus-local layout is already 144 bytes and
        // tripped the deterministic-bounds guard — turning a routine read into a hard error and
        // breaking every git-evidence capture on that machine. Bounded like the sibling
        // `core.fsmonitor` read, which faces the same "one query, unknown number of records"
        // shape.
        16 * 1024,
    )?;
    let mut snapshot = BTreeMap::new();
    if !output.status.success() {
        // No matching key is rc=1 with nothing on either stream — the same shape `--get`
        // reports for an unset key. Anything else (a malformed config is rc=128 with stderr)
        // must still fail loudly; reading it as "unset" would turn a broken repository into a
        // silently default one.
        if output.stdout.is_empty() && output.stderr.is_empty() {
            return Ok(snapshot);
        }
        return Err(format!(
            "failed to inspect effective Git core configuration: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = std::str::from_utf8(&output.stdout)
        .map_err(|_| "effective Git core configuration is not UTF-8".to_string())?;
    for record in text.split('\0').filter(|record| !record.is_empty()) {
        // `key\nvalue`, or a bare `key` when the key is present with no value.
        let (key, value) = match record.split_once('\n') {
            Some((key, value)) => (key, value),
            None => (record, ""),
        };
        // Later occurrences win, matching what `--get` returns for a multi-valued key.
        snapshot.insert(key.trim().to_string(), value.to_string());
    }
    Ok(snapshot)
}

fn normalize_checkout_core_value(key: &str, raw: &str) -> Result<String, String> {
    let value = raw.trim().to_ascii_lowercase();
    let normalized = match key {
        "core.autocrlf" => match value.as_str() {
            "" | "true" | "yes" | "on" | "1" => "true",
            "false" | "no" | "off" | "0" => "false",
            "input" => "input",
            _ => {
                return Err(format!(
                    "effective Git core.autocrlf has unsupported value `{value}`"
                ));
            }
        },
        "core.eol" => match value.as_str() {
            "lf" | "crlf" | "native" => return Ok(value),
            _ => {
                return Err(format!(
                    "effective Git {key} has unsupported value `{value}`"
                ));
            }
        },
        _ => match value.as_str() {
            "" | "true" | "yes" | "on" | "1" => "true",
            "false" | "no" | "off" | "0" => "false",
            _ => {
                return Err(format!(
                    "effective Git {key} has unsupported value `{value}`"
                ));
            }
        },
    };
    Ok(normalized.to_string())
}

/// Derives just `core.autocrlf` from the shared snapshot.
///
/// Production reads all four keys together through `effective_checkout_overrides_uncached`, so
/// this now exists for the tests that assert local/global/injected precedence on a `Command`
/// the caller builds. Keeping it means those tests exercise the same snapshot path production
/// uses, rather than a parallel one that could drift from it.
#[cfg(test)]
fn checkout_autocrlf_from_command(command: &mut Command) -> Result<Option<String>, String> {
    let snapshot = core_config_snapshot_from_command(command)?;
    let Some(raw) = snapshot.get("core.autocrlf") else {
        return Ok(None);
    };
    return normalize_checkout_core_value("core.autocrlf", raw).map(Some);
}

fn effective_checkout_overrides(root: &Path) -> Result<Vec<String>, String> {
    if let Some(overrides) = read_scope_value(root, |scope| scope.checkout_overrides.clone()) {
        return overrides;
    }
    let result = effective_checkout_overrides_uncached(root);
    update_read_scope(root, |scope| {
        scope.checkout_overrides = Some(result.clone());
    });
    result
}

fn effective_checkout_overrides_uncached(root: &Path) -> Result<Vec<String>, String> {
    // One spawn for four keys. This used to be four `git config --get` invocations, which
    // dominated the subprocess cost of every checkout inspection.
    let mut command = rooted_git_command(root);
    let snapshot = core_config_snapshot_from_command(&mut command)?;
    let mut overrides = Vec::new();
    for key in CHECKOUT_CORE_KEYS {
        // A key absent from the snapshot is unset, which is what rc=1 meant per-key before.
        let Some(raw) = snapshot.get(key) else {
            continue;
        };
        overrides.push(format!(
            "{key}={}",
            normalize_checkout_core_value(key, raw)?
        ));
    }
    Ok(overrides)
}

fn git_repository_present(root: &Path) -> Result<bool, String> {
    if let Some(present) = read_scope_value(root, |scope| scope.repository_present.clone()) {
        return present;
    }
    let result = git_repository_present_uncached(root);
    update_read_scope(root, |scope| {
        scope.repository_present = Some(result.clone());
    });
    result
}

fn git_repository_present_uncached(root: &Path) -> Result<bool, String> {
    let output = run_git_bounded(root, &["rev-parse", "--is-inside-work-tree"], None, 32)?;
    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|_| "Git repository detection output is not UTF-8".to_string())?
        .trim();
    if output.status.success() {
        return match stdout {
            "true" => Ok(true),
            "false" => Err(
                "Git metadata was detected, but the governed root is not inside a work tree".into(),
            ),
            _ => Err(format!(
                "Git repository detection returned an invalid response `{stdout}`"
            )),
        };
    }
    let markers = git_metadata_markers_present(root)?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let ordinary_non_repository = stderr.to_ascii_lowercase().contains("not a git repository");
    if !markers && stdout.is_empty() && ordinary_non_repository {
        return Ok(false);
    }
    let diagnostic = stderr.trim();
    Err(if diagnostic.is_empty() {
        "Git repository detection failed without a diagnostic".into()
    } else {
        format!("Git repository detection failed: {diagnostic}")
    })
}

fn git_metadata_markers_present(root: &Path) -> Result<bool, String> {
    for ancestor in root.ancestors() {
        match fs::symlink_metadata(ancestor.join(".git")) {
            Ok(_) => return Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "failed to inspect Git metadata marker at {}: {error}",
                    ancestor.display()
                ));
            }
        }
    }
    let head = root.join("HEAD");
    let objects = root.join("objects");
    Ok(fs::symlink_metadata(head).is_ok() && fs::symlink_metadata(objects).is_ok())
}

fn repository_context(root: &Path) -> Result<RepositoryContext, String> {
    if let Some(context) = read_scope_value(root, |scope| scope.repository_context.clone()) {
        return context;
    }
    let result = repository_context_uncached(root);
    update_read_scope(root, |scope| {
        scope.repository_context = Some(result.clone());
    });
    result
}

fn repository_context_uncached(root: &Path) -> Result<RepositoryContext, String> {
    if !git_repository_present(root)? {
        let canonical = root
            .canonicalize()
            .map_err(|error| format!("failed to resolve governed root identity: {error}"))?;
        return Ok(RepositoryContext {
            git: false,
            identity: canonical.to_string_lossy().into_owned(),
        });
    }
    let mut digest = FramedDigest::new(b"specsync.repository-context.v1");
    for (tag, args) in [
        (b"git-dir".as_slice(), ["rev-parse", "--absolute-git-dir"]),
        (b"common-dir".as_slice(), ["rev-parse", "--git-common-dir"]),
        (b"worktree".as_slice(), ["rev-parse", "--show-toplevel"]),
    ] {
        let output = run_git_required(root, &args, None, 16 * 1024)?;
        digest.frame(tag, &output);
    }
    Ok(RepositoryContext {
        git: true,
        identity: digest.finish(),
    })
}

fn git_evidence(root: &Path, candidates: &BTreeSet<String>) -> Result<GitEvidence, String> {
    cached_git_evidence(root, candidates, false)
}

fn git_regular_file_evidence(
    root: &Path,
    candidates: &BTreeSet<String>,
) -> Result<GitEvidence, String> {
    cached_git_evidence(root, candidates, true)
}

fn cached_git_evidence(
    root: &Path,
    candidates: &BTreeSet<String>,
    regular_files_only: bool,
) -> Result<GitEvidence, String> {
    let key = GitEvidenceCacheKey {
        regular_files_only,
        candidates: candidates.iter().cloned().collect(),
    };
    if let Some(evidence) = read_scope_value(root, |scope| scope.git_evidence.get(&key).cloned()) {
        return evidence;
    }
    let result = git_evidence_with_policy(root, candidates, regular_files_only, |_, _| {});
    update_read_scope(root, |scope| {
        if scope.git_evidence.len() < MAX_CHANGE_READ_CACHE_ENTRIES {
            scope.git_evidence.insert(key, result.clone());
        }
    });
    result
}

#[cfg(test)]
fn git_evidence_with_hook<Hook>(
    root: &Path,
    candidates: &BTreeSet<String>,
    after_inspection: Hook,
) -> Result<GitEvidence, String>
where
    Hook: FnMut(usize, &Path),
{
    git_evidence_with_policy(root, candidates, false, after_inspection)
}

fn git_evidence_with_policy<Hook>(
    root: &Path,
    candidates: &BTreeSet<String>,
    regular_files_only: bool,
    mut after_inspection: Hook,
) -> Result<GitEvidence, String>
where
    Hook: FnMut(usize, &Path),
{
    if candidates.len() > MAX_GIT_EVIDENCE_PATHS
        || candidates.iter().map(String::len).sum::<usize>() > MAX_GIT_EVIDENCE_PATH_BYTES
        || candidates.iter().any(|path| path.len() > 4096)
    {
        return Err("Git evidence candidate inventory exceeds deterministic bounds".into());
    }
    if !git_repository_present(root)? {
        let entries = capture_non_git_candidates(root, candidates, regular_files_only)?;
        return Ok(GitEvidence {
            modes: BTreeMap::new(),
            entries,
        });
    }
    for attempt in 0..2 {
        let before_index = git_index_fingerprint(root)?;
        let before = inspect_git_candidates(root, candidates, regular_files_only, false)?;
        after_inspection(attempt, root);
        let after = inspect_git_candidates(root, candidates, regular_files_only, false)?;
        let after_index = git_index_fingerprint(root)?;
        if before_index == after_index && before == after {
            return Ok(GitEvidence {
                modes: before.modes,
                entries: before.entries,
            });
        }
        if attempt == 1 {
            return Err(
                "Git index changed or candidate state changed during evidence inspection".into(),
            );
        }
    }
    unreachable!()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InspectedGitCandidates {
    modes: BTreeMap<String, u32>,
    objects: BTreeMap<String, String>,
    worktree: GitWorktreeState,
    entries: BTreeMap<String, GitCapturedEntry>,
}

fn git_index_fingerprint(root: &Path) -> Result<String, String> {
    let index = run_git_required(root, &["rev-parse", "--git-path", "index"], None, 16 * 1024)?;
    let index = parse_git_path(root, &index, "effective Git index")?;
    let shared = run_git_required(root, &["rev-parse", "--shared-index-path"], None, 16 * 1024)?;
    let mut paths = vec![index];
    if !shared.iter().all(u8::is_ascii_whitespace) {
        paths.push(parse_git_path(root, &shared, "split Git index")?);
    }
    paths.sort();
    paths.dedup();
    fingerprint_git_index_paths(paths)
}

fn fingerprint_git_index_paths(paths: Vec<PathBuf>) -> Result<String, String> {
    let mut digest = FramedDigest::new(b"specsync.git-index-generation.v1");
    let mut total = 0_usize;
    for path in paths {
        match fs::symlink_metadata(&path) {
            Ok(before) => {
                if !before.is_file() || before.file_type().is_symlink() {
                    return Err(format!(
                        "Git index dependency is not a regular file: {}",
                        path.display()
                    ));
                }
                let length = usize::try_from(before.len()).map_err(|_| {
                    "Git index dependencies exceed deterministic bounds".to_string()
                })?;
                total = total.checked_add(length).ok_or_else(|| {
                    "Git index dependencies exceed deterministic bounds".to_string()
                })?;
                if total > MAX_GIT_INDEX_BYTES {
                    return Err("Git index dependencies exceed deterministic bounds".into());
                }
                let mut file = OpenOptions::new()
                    .read(true)
                    .open(&path)
                    .map_err(|error| format!("failed to open {}: {error}", path.display()))?;
                let opened = file
                    .metadata()
                    .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
                if !same_file_metadata(&before, &opened) {
                    return Err(format!(
                        "Git index dependency changed before hashing: {}",
                        path.display()
                    ));
                }
                digest.frame(b"path", path.to_string_lossy().as_bytes());
                digest.frame_reader(b"bytes", before.len(), &mut file)?;
                let after = fs::symlink_metadata(&path)
                    .map_err(|error| format!("failed to re-inspect {}: {error}", path.display()))?;
                if !same_file_metadata(&before, &after) {
                    return Err(format!(
                        "Git index dependency changed during hashing: {}",
                        path.display()
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && total == 0 => {
                digest.frame(b"missing-index", path.to_string_lossy().as_bytes());
            }
            Err(error) => return Err(format!("failed to fingerprint {}: {error}", path.display())),
        }
    }
    Ok(digest.finish())
}

fn same_file_metadata(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    if left.len() != right.len()
        || left.file_type() != right.file_type()
        || left.modified().ok() != right.modified().ok()
    {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        left.dev() == right.dev() && left.ino() == right.ino()
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn parse_git_path(root: &Path, output: &[u8], description: &str) -> Result<PathBuf, String> {
    let value = std::str::from_utf8(output)
        .map_err(|_| format!("{description} path is not UTF-8"))?
        .trim();
    if value.is_empty() || value.as_bytes().contains(&0) {
        return Err(format!("{description} path is invalid"));
    }
    let path = PathBuf::from(value);
    Ok(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
}

fn candidate_argument_batches(candidates: &BTreeSet<String>) -> Vec<Vec<&str>> {
    let mut batches = Vec::new();
    let mut batch = Vec::new();
    let mut bytes = 0_usize;
    for candidate in candidates {
        if !batch.is_empty()
            && (batch.len() >= GIT_ATTRIBUTE_BATCH_PATHS
                || bytes.saturating_add(candidate.len()) > 64 * 1024)
        {
            batches.push(batch);
            batch = Vec::new();
            bytes = 0;
        }
        batch.push(candidate.as_str());
        bytes = bytes.saturating_add(candidate.len() + 1);
    }
    if !batch.is_empty() {
        batches.push(batch);
    }
    batches
}

fn literal_candidate_git_args(repo_prefix: &str, prefix: &[&str], batch: &[&str]) -> Vec<String> {
    let mut args = Vec::with_capacity(prefix.len() + batch.len() + 1);
    args.extend(prefix.iter().map(|value| (*value).to_string()));
    args.push("--".into());
    for path in batch {
        args.push(format!(":(top,literal){repo_prefix}{path}"));
    }
    args
}

fn borrowed_git_args(args: &[String]) -> Vec<&str> {
    args.iter().map(String::as_str).collect()
}

fn record_git_stage_zero_entry(
    entries: &mut BTreeMap<String, (u32, String)>,
    path: String,
    mode: u32,
    object: String,
) -> Result<(), String> {
    if let Some((existing_mode, existing_object)) = entries.get(&path) {
        if *existing_mode == mode && existing_object == &object {
            return Ok(());
        }
        return Err(format!(
            "conflicting duplicate Git index stage-zero entry for `{path}`"
        ));
    }
    entries.insert(path, (mode, object));
    Ok(())
}

/// Is `path` inside the requested candidate scope?
///
/// A candidate naming a directory expands under `:(top,literal)` to every tracked
/// file beneath it, so those files were requested just as much as an exact
/// candidate was. Rejecting them made `finalize` fail in any repository that had
/// ever archived a change — every project past its first — while passing on the
/// fresh fixtures the suite is built on.
///
/// Compares at the path separator so `a/b` cannot be admitted by `a/bc`.
fn candidate_scope_admits(candidates: &BTreeSet<String>, path: &str) -> bool {
    candidates.contains(path)
        || candidates.iter().any(|candidate| {
            path.len() > candidate.len()
                && path.as_bytes()[candidate.len()] == b'/'
                && path.starts_with(candidate.as_str())
        })
}

fn inspect_git_candidates(
    root: &Path,
    candidates: &BTreeSet<String>,
    regular_files_only: bool,
    identify_clean_blobs: bool,
) -> Result<InspectedGitCandidates, String> {
    let mut stage_entries = BTreeMap::new();
    let batches = candidate_argument_batches(candidates);
    let repo_prefix = git_repo_prefix(root)?;
    let mut stage_output_bytes = 0_usize;
    for batch in &batches {
        let args = literal_candidate_git_args(&repo_prefix, &["ls-files", "--stage", "-z"], batch);
        let args = borrowed_git_args(&args);
        let output = run_git_required(root, &args, None, MAX_GIT_INDEX_BYTES)?;
        stage_output_bytes = stage_output_bytes
            .checked_add(output.len())
            .ok_or_else(|| "scoped Git index output exceeds deterministic bounds".to_string())?;
        if stage_output_bytes > MAX_GIT_INDEX_BYTES {
            return Err("scoped Git index output exceeds deterministic bounds".into());
        }
        for record in output
            .split(|byte| *byte == 0)
            .filter(|entry| !entry.is_empty())
        {
            let tab = record
                .iter()
                .position(|byte| *byte == b'\t')
                .ok_or_else(|| "invalid `git ls-files --stage` record".to_string())?;
            let metadata = std::str::from_utf8(&record[..tab])
                .map_err(|_| "non-UTF-8 Git index metadata".to_string())?;
            let mut fields = metadata.split_whitespace();
            let mode_text = fields
                .next()
                .ok_or_else(|| "Git index entry is missing a mode".to_string())?;
            let object = fields
                .next()
                .ok_or_else(|| "Git index entry is missing an object ID".to_string())?;
            let stage = fields
                .next()
                .ok_or_else(|| "Git index entry is missing a stage".to_string())?;
            if fields.next().is_some()
                || object.is_empty()
                || !object.bytes().all(|byte| byte.is_ascii_hexdigit())
                || !matches!(stage, "0" | "1" | "2" | "3")
            {
                return Err("invalid Git index metadata".into());
            }
            let mode = u32::from_str_radix(mode_text, 8)
                .map_err(|_| format!("invalid Git file mode `{mode_text}`"))?;
            let path = std::str::from_utf8(&record[tab + 1..])
                .map_err(|_| "non-UTF-8 scoped Git index path".to_string())?;
            let path = strict_portable_relative_path(path)?;
            if !candidate_scope_admits(candidates, &path) {
                return Err(format!("Git returned an out-of-scope index path `{path}`"));
            }
            if stage != "0" {
                return Err(format!(
                    "cannot hash relevant path `{path}` with unresolved Git index stages"
                ));
            }
            let object = object.to_ascii_lowercase();
            record_git_stage_zero_entry(&mut stage_entries, path, mode, object)?;
        }
    }
    let mut modes = BTreeMap::new();
    let mut objects = BTreeMap::new();
    for (path, (mode, object)) in stage_entries {
        modes.insert(path.clone(), mode);
        objects.insert(path, object);
    }

    let fsmonitor = run_git_bounded(
        root,
        &["config", "--get", "core.fsmonitor"],
        None,
        16 * 1024,
    )?;
    let fsmonitor_value = std::str::from_utf8(&fsmonitor.stdout)
        .map_err(|_| "Git core.fsmonitor value is not UTF-8".to_string())?
        .trim();
    if !fsmonitor.status.success() && (!fsmonitor.stdout.is_empty() || !fsmonitor.stderr.is_empty())
    {
        return Err(format!(
            "failed to inspect Git fsmonitor configuration: {}",
            String::from_utf8_lossy(&fsmonitor.stderr).trim()
        ));
    }
    let fsmonitor_active = !fsmonitor_value.is_empty()
        && !matches!(
            fsmonitor_value.to_ascii_lowercase().as_str(),
            "false" | "no" | "off" | "0"
        );
    if fsmonitor_active
        && candidates
            .iter()
            .any(|path| fs::symlink_metadata(root.join(path)).is_ok())
    {
        return Err("Git core.fsmonitor cannot supply canonical working-tree evidence".into());
    }

    let checkout_overrides = effective_checkout_overrides(root)?;
    let mut modified = BTreeSet::new();
    for batch in &batches {
        let mut prefix = Vec::new();
        for setting in &checkout_overrides {
            prefix.push("-c".to_string());
            prefix.push(setting.clone());
        }
        prefix.extend(
            ["diff-files", "--name-only", "-z"]
                .iter()
                .map(|value| (*value).to_string()),
        );
        let prefix = borrowed_git_args(&prefix);
        let args = literal_candidate_git_args(&repo_prefix, &prefix, batch);
        let args = borrowed_git_args(&args);
        let output = run_git_required(root, &args, None, MAX_GIT_COMMAND_OUTPUT_BYTES)?;
        for path in output
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
        {
            let path =
                std::str::from_utf8(path).map_err(|_| "non-UTF-8 Git diff path".to_string())?;
            let path = strict_portable_relative_path(path)?;
            if !candidate_scope_admits(candidates, &path) {
                return Err(format!(
                    "Git returned an out-of-scope modified path `{path}`"
                ));
            }
            modified.insert(path);
        }
    }

    let mut sparse_absent = BTreeSet::new();
    for batch in &batches {
        let args = literal_candidate_git_args(&repo_prefix, &["ls-files", "-v", "-z"], batch);
        let args = borrowed_git_args(&args);
        let output = run_git_required(root, &args, None, MAX_GIT_COMMAND_OUTPUT_BYTES)?;
        for record in output
            .split(|byte| *byte == 0)
            .filter(|entry| !entry.is_empty())
        {
            if record.len() < 3 || record[1] != b' ' {
                return Err("invalid `git ls-files -v` output".into());
            }
            let tag = record[0];
            let path = std::str::from_utf8(&record[2..])
                .map_err(|_| "non-UTF-8 Git visibility path".to_string())?;
            let path = strict_portable_relative_path(path)?;
            if !candidate_scope_admits(candidates, &path) {
                return Err(format!(
                    "Git returned an out-of-scope visibility path `{path}`"
                ));
            }
            if tag.is_ascii_lowercase() {
                return Err(format!(
                    "Git assume-unchanged hides working-tree evidence for `{path}`"
                ));
            }
            if tag == b'S' {
                match fs::symlink_metadata(root.join(&path)) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        sparse_absent.insert(path);
                    }
                    Ok(_) => {
                        return Err(format!(
                            "materialized skip-worktree path cannot use canonical index evidence: `{path}`"
                        ));
                    }
                    Err(error) => {
                        return Err(format!("failed to inspect sparse path `{path}`: {error}"));
                    }
                }
            }
        }
    }

    for batch in &batches {
        let args = literal_candidate_git_args(&repo_prefix, &["ls-files", "-f", "-z"], batch);
        let args = borrowed_git_args(&args);
        let output = run_git_required(root, &args, None, MAX_GIT_COMMAND_OUTPUT_BYTES)?;
        for record in output
            .split(|byte| *byte == 0)
            .filter(|entry| !entry.is_empty())
        {
            if record.len() < 3 || record[1] != b' ' {
                return Err("invalid `git ls-files -f` output".into());
            }
            let path = std::str::from_utf8(&record[2..])
                .map_err(|_| "non-UTF-8 Git fsmonitor path".to_string())?;
            let path = strict_portable_relative_path(path)?;
            if !candidate_scope_admits(candidates, &path) {
                return Err(format!(
                    "Git returned an out-of-scope fsmonitor path `{path}`"
                ));
            }
            if record[0].is_ascii_lowercase() && fs::symlink_metadata(root.join(&path)).is_ok() {
                return Err(format!(
                    "Git fsmonitor-valid hides working-tree evidence for `{path}`"
                ));
            }
        }
    }

    let regular_for_attributes = candidates
        .iter()
        .filter(|path| {
            matches!(modes.get(*path), Some(0o100644 | 0o100755))
                && !modified.contains(*path)
                && !sparse_absent.contains(*path)
                && fs::symlink_metadata(root.join(*path)).is_ok_and(|metadata| metadata.is_file())
        })
        .cloned()
        .collect();
    let lfs_paths = validate_canonical_git_attributes(root, &regular_for_attributes)?;

    let worktree = GitWorktreeState {
        modified,
        sparse_absent,
    };
    let clean_objects: BTreeSet<_> = candidates
        .iter()
        .filter_map(|path| {
            let mode = modes.get(path)?;
            let object = objects.get(path)?;
            let clean =
                !worktree.modified.contains(path) && matches!(mode, 0o100644 | 0o100755 | 0o120000);
            if !clean {
                return None;
            }
            // Regular-file bytes are not needed when the caller names the object.
            // An LFS path is the exception: its pointer blob is read so a
            // `filter=lfs` attribute cannot name an arbitrary object. Symlinks
            // stay materialized so their targets are still checked.
            if identify_clean_blobs
                && matches!(mode, 0o100644 | 0o100755)
                && !lfs_paths.contains(path)
            {
                return None;
            }
            Some(object.as_str())
        })
        .collect();
    let clean_object_refs: Vec<_> = clean_objects.iter().copied().collect();
    let clean_payloads = git_blob_bytes_batch(root, &clean_object_refs)?;
    let prefetched_blobs: BTreeMap<_, _> = clean_objects
        .into_iter()
        .map(str::to_string)
        .zip(clean_payloads)
        .collect();
    let mut entries = BTreeMap::new();
    let mut payload_bytes = 0_usize;
    for path in candidates {
        if regular_files_only {
            let working = fs::symlink_metadata(root.join(path));
            let non_regular = matches!(modes.get(path), Some(0o120000 | 0o160000))
                || working.as_ref().is_ok_and(|metadata| !metadata.is_file());
            if non_regular {
                return Err(format!(
                    "selected definition artifact is not a regular file: {path}"
                ));
            }
        }
        let entry = capture_git_candidate(
            root,
            path,
            modes.get(path).copied(),
            objects.get(path),
            &worktree,
            Some(&prefetched_blobs),
            GitCaptureOptions {
                identify_clean_blobs,
                lfs: lfs_paths.contains(path),
            },
        )?;
        payload_bytes = payload_bytes
            .checked_add(entry.payload.len())
            .ok_or_else(|| "Git evidence payload exceeds deterministic bounds".to_string())?;
        if payload_bytes > MAX_GIT_EVIDENCE_PAYLOAD_BYTES {
            return Err("Git evidence payload exceeds deterministic bounds".into());
        }
        entries.insert(path.clone(), entry);
    }
    Ok(InspectedGitCandidates {
        modes,
        objects,
        worktree,
        entries,
    })
}

#[derive(Default)]
struct GitCaptureOptions {
    identify_clean_blobs: bool,
    lfs: bool,
}

fn capture_git_candidate(
    root: &Path,
    relative: &str,
    index_mode: Option<u32>,
    object: Option<&String>,
    worktree: &GitWorktreeState,
    prefetched_blobs: Option<&BTreeMap<String, Vec<u8>>>,
    options: GitCaptureOptions,
) -> Result<GitCapturedEntry, String> {
    if index_mode == Some(0o160000) {
        let object = object
            .ok_or_else(|| format!("gitlink `{relative}` has no exact index object ID"))?
            .clone();
        return Ok(GitCapturedEntry {
            kind: AcceptanceInputKind::Gitlink,
            mode: 0o160000,
            payload: object.as_bytes().to_vec(),
            object: Some(object),
            object_identity: false,
        });
    }
    let clean = object.is_some() && !worktree.modified.contains(relative);
    if clean {
        let mode = index_mode.ok_or_else(|| format!("tracked path `{relative}` has no mode"))?;
        if worktree.sparse_absent.contains(relative)
            || matches!(mode, 0o100644 | 0o100755 | 0o120000)
        {
            let object = object.expect("clean tracked object").clone();
            if options.identify_clean_blobs && matches!(mode, 0o100644 | 0o100755) {
                if options.lfs {
                    let pointer = match prefetched_blobs.and_then(|blobs| blobs.get(&object)) {
                        Some(payload) => payload.clone(),
                        None => git_blob_bytes(root, &object)?,
                    };
                    if !is_git_lfs_pointer(&pointer) {
                        return Err(format!(
                            "Git `filter=lfs` blob is not a Git LFS pointer: `{relative}`"
                        ));
                    }
                }
                return Ok(GitCapturedEntry {
                    kind: acceptance_kind_for_mode(mode),
                    mode,
                    payload: object.as_bytes().to_vec(),
                    object: Some(object),
                    object_identity: true,
                });
            }
            let payload = match prefetched_blobs.and_then(|blobs| blobs.get(&object)) {
                Some(payload) => payload.clone(),
                None => git_blob_bytes(root, &object)?,
            };
            if mode == 0o120000 {
                let target = std::str::from_utf8(&payload)
                    .map_err(|_| format!("symlink target is not UTF-8: `{relative}`"))?;
                validate_portable_symlink_target(target)?;
            }
            return Ok(GitCapturedEntry {
                kind: acceptance_kind_for_mode(mode),
                mode,
                object: Some(object),
                payload,
                object_identity: false,
            });
        }
        return Err(format!(
            "unsupported Git index mode `{mode:o}` for `{relative}`"
        ));
    }
    capture_working_candidate(root, relative, object.cloned())
}

fn capture_working_candidate(
    root: &Path,
    relative: &str,
    object: Option<String>,
) -> Result<GitCapturedEntry, String> {
    let path = root.join(relative);
    let (kind, mode, payload) = match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let target = fs::read_link(&path)
                .map_err(|error| format!("failed to read symlink {}: {error}", path.display()))?;
            let target = target.to_str().ok_or_else(|| {
                format!(
                    "non-UTF-8 symlink target cannot be hashed portably: {}",
                    path.display()
                )
            })?;
            validate_portable_symlink_target(target)?;
            (
                AcceptanceInputKind::Symlink,
                0o120000,
                target.as_bytes().to_vec(),
            )
        }
        Ok(metadata) if metadata.is_file() => (
            AcceptanceInputKind::File,
            working_file_mode(&metadata),
            fs::read(&path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?,
        ),
        Ok(_) => (AcceptanceInputKind::NonFile, 0, Vec::new()),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidFilename
            ) =>
        {
            (AcceptanceInputKind::Missing, 0, Vec::new())
        }
        Err(error) => return Err(format!("failed to inspect {}: {error}", path.display())),
    };
    Ok(GitCapturedEntry {
        kind,
        mode,
        object,
        payload,
        object_identity: false,
    })
}

fn capture_non_git_candidates(
    root: &Path,
    candidates: &BTreeSet<String>,
    regular_files_only: bool,
) -> Result<BTreeMap<String, GitCapturedEntry>, String> {
    let mut entries = BTreeMap::new();
    let mut payload_bytes = 0_usize;
    for path in candidates {
        if regular_files_only
            && fs::symlink_metadata(root.join(path)).is_ok_and(|metadata| !metadata.is_file())
        {
            return Err(format!(
                "selected definition artifact is not a regular file: {path}"
            ));
        }
        let entry = capture_working_candidate(root, path, None)?;
        payload_bytes = payload_bytes
            .checked_add(entry.payload.len())
            .ok_or_else(|| {
                "filesystem evidence payload exceeds deterministic bounds".to_string()
            })?;
        if payload_bytes > MAX_GIT_EVIDENCE_PAYLOAD_BYTES {
            return Err("filesystem evidence payload exceeds deterministic bounds".into());
        }
        entries.insert(path.clone(), entry);
    }
    Ok(entries)
}

#[cfg(test)]
fn git_worktree_state(root: &Path) -> Result<Option<GitWorktreeState>, String> {
    let candidates = git_project_paths(root)?.unwrap_or_default();
    let candidates = candidates.into_iter().collect::<BTreeSet<_>>();
    if !git_repository_present(root)? {
        return Ok(None);
    }
    Ok(Some(
        inspect_git_candidates(root, &candidates, false, false)?.worktree,
    ))
}

fn validate_canonical_git_attributes(
    root: &Path,
    paths: &BTreeSet<String>,
) -> Result<BTreeSet<String>, String> {
    if paths.is_empty() {
        return Ok(BTreeSet::new());
    }
    let paths = paths.iter().collect::<Vec<_>>();
    let mut lfs_paths = BTreeSet::new();
    for batch in paths.chunks(GIT_ATTRIBUTE_BATCH_PATHS) {
        let mut input = Vec::new();
        for path in batch {
            input.extend_from_slice(path.as_bytes());
            input.push(0);
        }
        let output = run_git_required(
            root,
            &[
                "check-attr",
                "-z",
                "--stdin",
                "filter",
                "working-tree-encoding",
                "ident",
            ],
            Some(input),
            MAX_GIT_ATTRIBUTE_OUTPUT_BYTES,
        )
        .map_err(|error| format!("failed to inspect Git content attributes: {error}"))?;
        lfs_paths.extend(validate_git_attribute_output(batch, &output)?);
    }
    if !lfs_paths.is_empty() {
        ensure_official_git_lfs_driver(root)?;
    }
    Ok(lfs_paths)
}

fn validate_git_attribute_output(
    paths: &[&String],
    output: &[u8],
) -> Result<BTreeSet<String>, String> {
    if !output.ends_with(&[0]) {
        return Err("invalid unterminated NUL-delimited `git check-attr` output".into());
    }
    let fields = output[..output.len() - 1]
        .split(|byte| *byte == 0)
        .collect::<Vec<_>>();
    if fields.iter().any(|field| field.is_empty()) || fields.len() % 3 != 0 {
        return Err("invalid NUL-delimited `git check-attr` output".into());
    }
    let attributes = ["filter", "working-tree-encoding", "ident"];
    let expected = paths
        .iter()
        .flat_map(|path| {
            attributes
                .iter()
                .map(move |attribute| (path.as_str(), *attribute))
        })
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut lfs_paths = BTreeSet::new();
    for record in fields.chunks_exact(3) {
        let path = record[0];
        let attribute = record[1];
        let value = record[2];
        let path =
            std::str::from_utf8(path).map_err(|_| "non-UTF-8 Git attribute path".to_string())?;
        let attribute = std::str::from_utf8(attribute)
            .map_err(|_| "non-UTF-8 Git attribute name".to_string())?;
        let value =
            std::str::from_utf8(value).map_err(|_| "non-UTF-8 Git attribute value".to_string())?;
        if !expected.contains(&(path, attribute)) {
            return Err(format!(
                "Git returned an unrequested attribute pair `{path}` / `{attribute}`"
            ));
        }
        if !seen.insert((path, attribute)) {
            return Err(format!(
                "Git returned a duplicate attribute pair `{path}` / `{attribute}`"
            ));
        }
        if value != "unspecified" && value != "unset" {
            // `filter=lfs` names a driver. The driver commands and the pointer
            // blob are checked before that object id is used as evidence.
            if attribute == "filter" && value == "lfs" {
                lfs_paths.insert(path.to_string());
                continue;
            }
            return Err(format!(
                "Git `{attribute}` attribute is not supported for canonical evidence: `{path}`"
            ));
        }
    }
    if seen != expected {
        return Err("Git attribute output is missing a requested path/attribute pair".into());
    }
    Ok(lfs_paths)
}

/// `git lfs install` writes these commands. Anything else is some other
/// program using the name `lfs`, and a clean filter can hide a dirty worktree.
fn ensure_official_git_lfs_driver(root: &Path) -> Result<(), String> {
    let clean = local_git_config(root, "filter.lfs.clean")?;
    let smudge = local_git_config(root, "filter.lfs.smudge")?;
    let process = local_git_config(root, "filter.lfs.process")?;
    let clean_ok = clean.as_deref().is_some_and(is_official_lfs_clean);
    let smudge_ok = smudge.as_deref().is_some_and(is_official_lfs_smudge);
    let process_ok = match process.as_deref() {
        None => true,
        Some(value) => is_official_lfs_process(value),
    };
    if clean_ok && smudge_ok && process_ok {
        return Ok(());
    }
    Err(
        "Git `filter.lfs` is not the Git LFS driver; canonical evidence cannot treat `filter=lfs` as a pointer"
            .into(),
    )
}

fn local_git_config(root: &Path, key: &str) -> Result<Option<String>, String> {
    let output = run_git_bounded(root, &["config", "--local", "--get", key], None, 4096)?;
    if output.status.success() {
        let value = std::str::from_utf8(&output.stdout)
            .map_err(|_| format!("Git config `{key}` is not UTF-8"))?
            .trim()
            .to_string();
        return Ok((!value.is_empty()).then_some(value));
    }
    if output.stdout.is_empty() && output.stderr.is_empty() {
        return Ok(None);
    }
    Err(format!(
        "failed to read Git config `{key}`: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}

fn is_official_lfs_clean(value: &str) -> bool {
    matches!(value, "git-lfs clean -- %f" | "git lfs clean -- %f")
}

fn is_official_lfs_smudge(value: &str) -> bool {
    matches!(value, "git-lfs smudge -- %f" | "git lfs smudge -- %f")
}

fn is_official_lfs_process(value: &str) -> bool {
    matches!(value, "git-lfs filter-process" | "git lfs filter-process")
}

/// A Git LFS pointer is a short text blob. The smudged object is not.
fn is_git_lfs_pointer(bytes: &[u8]) -> bool {
    if bytes.is_empty() || bytes.len() > 1024 || !bytes.ends_with(b"\n") {
        return false;
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let mut lines = text.lines();
    if lines.next() != Some("version https://git-lfs.github.com/spec/v1") {
        return false;
    }
    let mut saw_oid = false;
    let mut saw_size = false;
    for line in lines {
        let Some((key, value)) = line.split_once(' ') else {
            return false;
        };
        if key.is_empty() || value.is_empty() || value.contains(' ') {
            return false;
        }
        match key {
            "oid" => {
                let Some(hex) = value.strip_prefix("sha256:") else {
                    return false;
                };
                if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return false;
                }
                saw_oid = true;
            }
            "size" => {
                if value.len() > 20
                    || !value.bytes().all(|byte| byte.is_ascii_digit())
                    || value.starts_with('0') && value != "0"
                {
                    return false;
                }
                saw_size = true;
            }
            "version" => return false,
            _ => {}
        }
    }
    saw_oid && saw_size
}

fn git_blob_bytes(root: &Path, object: &str) -> Result<Vec<u8>, String> {
    if let Some(bytes) = GIT_BLOB_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .map_err(|_| "Git blob cache lock is poisoned".to_string())?
        .get(object)
        .cloned()
    {
        return Ok(bytes);
    }
    let stdout = run_git_required(
        root,
        &["cat-file", "blob", object],
        None,
        MAX_GIT_EVIDENCE_PAYLOAD_BYTES,
    )?;
    if stdout.len() <= MAX_CHANGE_ARTIFACT_BYTES as usize {
        let mut cache = GIT_BLOB_CACHE
            .get_or_init(|| Mutex::new(BTreeMap::new()))
            .lock()
            .map_err(|_| "Git blob cache lock is poisoned".to_string())?;
        if cache.len() >= 4_096 {
            cache.clear();
        }
        cache.insert(object.to_string(), stdout.clone());
    }
    Ok(stdout)
}

fn working_file_mode(metadata: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            0o100644
        } else {
            0o100755
        }
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        0o100644
    }
}

fn resolved_acceptance_manifest(
    root: &Path,
    record: &ChangeRecord,
) -> Result<AcceptanceManifestV1, String> {
    let verification = load_verification(root, record)?;
    let signed = verification
        .acceptance_input_digest
        .clone()
        .ok_or_else(|| {
            format!(
                "accepted change `{}` has no acceptance input digest",
                record.id
            )
        })?;
    if let Some(manifest) = &verification.acceptance_manifest {
        if !verification.passed {
            return Err(format!(
                "accepted change `{}` has failed verification",
                record.id
            ));
        }
        ensure_definition_approval_valid(root, record)?;
        if !definition_digest_matches(root, record, &verification.contract_digest)? {
            return Err(format!(
                "accepted change `{}` verification contract is stale",
                record.id
            ));
        }
        validate_verification_execution_digest(root, record, &verification)?;
        validate_acceptance_manifest(manifest)?;
        if acceptance_manifest_digest(manifest)? != signed {
            return Err(format!(
                "accepted change `{}` manifest does not reproduce its signed aggregate",
                record.id
            ));
        }
        if let Some(evidence) = &verification.semantic_succession {
            validate_semantic_succession(record, evidence)?;
        } else if !record.supersedes.is_empty() {
            return Err(format!(
                "accepted change `{}` is missing semantic succession evidence",
                record.id
            ));
        }
        if record.workflow_version >= 2 {
            validate_finalization_evidence(root, record, &verification)?;
        } else {
            let ledger = load_approvals(root, record)?;
            let approval = latest_terminal_approval(&ledger).ok_or_else(|| {
                format!("accepted change `{}` has no closing approval", record.id)
            })?;
            if approval.digest != closing_digest(record, &verification) {
                return Err(format!(
                    "accepted change `{}` closing approval does not authenticate its manifest",
                    record.id
                ));
            }
        }
        return Ok(manifest.clone());
    }
    reconstruct_legacy_acceptance_manifest(root, record, &signed)
}

fn reconstruct_legacy_acceptance_manifest(
    root: &Path,
    record: &ChangeRecord,
    signed_legacy_digest: &str,
) -> Result<AcceptanceManifestV1, String> {
    let anchors = accepted_transition_anchors(root, record)?;
    let mut reconstructions: BTreeMap<Vec<u8>, AcceptanceManifestV1> = BTreeMap::new();
    let mut first_failure = None;
    for anchor in anchors {
        match reconstruct_legacy_at_anchor(root, record, signed_legacy_digest, &anchor) {
            Ok((key, manifest)) => {
                reconstructions.entry(key).or_insert(manifest);
            }
            Err(error) => {
                first_failure.get_or_insert(error);
            }
        }
    }
    if reconstructions.len() != 1 {
        let mut error = format!(
            "legacy accepted change `{}` requires exactly one distinct valid historical reconstruction, found {}",
            record.id,
            reconstructions.len()
        );
        if let Some(first_failure) = first_failure {
            error.push_str(&format!("; first reconstruction failure: {first_failure}"));
        }
        return Err(error);
    }
    reconstructions
        .into_values()
        .next()
        .ok_or_else(|| "legacy reconstruction disappeared unexpectedly".to_string())
}

fn reconstruct_legacy_at_anchor(
    root: &Path,
    record: &ChangeRecord,
    signed_legacy_digest: &str,
    anchor: &str,
) -> Result<(Vec<u8>, AcceptanceManifestV1), String> {
    let temporary = create_effective_contract_workspace()?;
    let tree = temporary.join("tree");
    let added = Command::new("git")
        .args([
            "worktree",
            "add",
            "--detach",
            tree.to_string_lossy().as_ref(),
            anchor,
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to reconstruct legacy acceptance tree: {error}"))?;
    if !added.status.success() {
        let _ = fs::remove_dir_all(&temporary);
        return Err(format!(
            "failed to reconstruct legacy acceptance tree: {}",
            String::from_utf8_lossy(&added.stderr).trim()
        ));
    }
    let result = (|| {
        let historical = load_change(&tree, &record.id)?;
        if historical.state != ChangeState::Accepted {
            return Err(format!(
                "trusted transition `{anchor}` does not contain accepted state for `{}`",
                record.id
            ));
        }
        ensure_definition_approval_valid(&tree, &historical)?;
        let verification = load_verification(&tree, &historical)?;
        if !verification.passed {
            return Err("historical accepted verification did not pass".into());
        }
        let ledger = load_approvals(&tree, &historical)?;
        let closing = latest_terminal_approval(&ledger)
            .ok_or_else(|| "historical accepted state has no closing approval".to_string())?;
        if closing.digest != closing_digest(&historical, &verification) {
            return Err("historical accepted closing approval is invalid".into());
        }
        let aggregate = acceptance_input_digest(&tree, &historical, &[])?;
        if aggregate != signed_legacy_digest {
            return Err(format!(
                "legacy accepted change `{}` cannot reproduce its signed raw-content aggregate",
                record.id
            ));
        }
        let manifest = acceptance_manifest_legacy(&tree, &historical, &[])?;
        let key = serde_json::to_vec(&serde_json::json!({
            "manifest": manifest,
            "verification": verification,
            "closing": closing,
        }))
        .map_err(|error| format!("failed to canonicalize historical evidence: {error}"))?;
        Ok((key, manifest))
    })();
    // Best-effort cleanup of the disposable scratch worktree. A successful
    // reconstruction must never be discarded because hygiene cleanup failed
    // (product #511 / flaky CI under worktree contention).
    let force_remove_failure = {
        #[cfg(test)]
        {
            FORCE_LEGACY_WORKTREE_REMOVE_FAILURE.get()
        }
        #[cfg(not(test))]
        {
            false
        }
    };
    let removed = if force_remove_failure {
        None
    } else {
        Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                tree.to_string_lossy().as_ref(),
            ])
            .current_dir(root)
            .output()
            .ok()
    };
    if !removed.is_some_and(|output| output.status.success()) {
        // Reclaim the registration if remove failed; ignore further hygiene errors.
        let _ = Command::new("git")
            .args(["worktree", "prune"])
            .current_dir(root)
            .output();
    }
    let _ = fs::remove_dir_all(&temporary);
    result
}

fn accepted_transition_anchors(root: &Path, record: &ChangeRecord) -> Result<Vec<String>, String> {
    let state = format!("{CHANGES_PATH}/{}/state.json", record.id);
    let state = git_repo_relative_path(root, &state)?;
    let mut references = vec!["HEAD".to_string()];
    if let Some(remote_default) = remote_default_ref(root)
        && !references.contains(&remote_default)
    {
        references.push(remote_default);
    }
    let mut anchors = Vec::new();
    for reference in references {
        let state_pathspec = format!(":(top,literal){state}");
        let max_count = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
        let output = run_git_bounded(
            root,
            &[
                "log",
                "--format=%H",
                &max_count,
                &reference,
                "--",
                state_pathspec.as_str(),
            ],
            None,
            MAX_GIT_COMMAND_OUTPUT_BYTES,
        )
        .map_err(|error| format!("failed to inspect accepted history: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "failed to inspect accepted history at `{reference}`"
            ));
        }
        let history = String::from_utf8_lossy(&output.stdout);
        let commits: Vec<&str> = history
            .lines()
            .filter(|commit| !commit.is_empty())
            .collect();
        if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
            return Err(format!(
                "accepted history exceeds the deterministic {}-commit bound",
                MAX_TRUSTED_HISTORY_COMMITS
            ));
        }
        for commit in commits {
            let current = git_change_record_at(root, commit, &state);
            if !current.is_some_and(|current| {
                current.id == record.id && current.state == ChangeState::Accepted
            }) {
                continue;
            }
            let parents =
                git_output(root, &["rev-list", "--parents", "-n", "1", commit]).unwrap_or_default();
            let parent_accepted = parents.split_whitespace().skip(1).any(|parent| {
                git_change_record_at(root, parent, &state).is_some_and(|parent| {
                    parent.id == record.id && parent.state == ChangeState::Accepted
                })
            });
            if !parent_accepted {
                anchors.push(commit.to_string());
            }
        }
    }
    anchors.sort();
    anchors.dedup();
    Ok(anchors)
}

/// Lists every commit reachable from `HEAD` or the remote default whose `state.json` records
/// `record` as accepted, regardless of the parent state. Unlike
/// [`accepted_transition_anchors`], this also matches commits that refresh accepted evidence
/// while the change is already accepted, which is the shape squash-merged re-verification
/// produces on the default branch.
fn accepted_recording_anchors(root: &Path, record: &ChangeRecord) -> Result<Vec<String>, String> {
    let state = format!("{CHANGES_PATH}/{}/state.json", record.id);
    let state = git_repo_relative_path(root, &state)?;
    let mut references = vec!["HEAD".to_string()];
    if let Some(remote_default) = remote_default_ref(root)
        && !references.contains(&remote_default)
    {
        references.push(remote_default);
    }
    let mut anchors = Vec::new();
    for reference in references {
        let state_pathspec = format!(":(top,literal){state}");
        let max_count = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
        let output = run_git_bounded(
            root,
            &[
                "log",
                "--format=%H",
                &max_count,
                &reference,
                "--",
                state_pathspec.as_str(),
            ],
            None,
            MAX_GIT_COMMAND_OUTPUT_BYTES,
        )
        .map_err(|error| format!("failed to inspect accepted recording history: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "failed to inspect accepted recording history at `{reference}`"
            ));
        }
        let history = String::from_utf8_lossy(&output.stdout);
        let commits: Vec<&str> = history
            .lines()
            .filter(|commit| !commit.is_empty())
            .collect();
        if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
            return Err(format!(
                "accepted recording history exceeds the deterministic {}-commit bound",
                MAX_TRUSTED_HISTORY_COMMITS
            ));
        }
        for commit in commits {
            if git_change_record_at(root, commit, &state).is_some_and(|current| {
                current.id == record.id && current.state == ChangeState::Accepted
            }) {
                anchors.push(commit.to_string());
            }
        }
    }
    anchors.sort();
    anchors.dedup();
    Ok(anchors)
}

/// Evaluates one candidate anchor commit against the current evidence, recording it in
/// `eligible` when the committed state, verification, and approvals authenticate the change's
/// accepted record.
#[allow(clippy::too_many_arguments)]
fn consider_accepted_evidence_anchor(
    root: &Path,
    record: &ChangeRecord,
    anchor: &str,
    state_path: &str,
    verification_path: &str,
    approvals_path: &str,
    current_verification: &[u8],
    current_approvals: &[u8],
    eligible: &mut BTreeMap<String, (String, Vec<u8>, ChangeRecord)>,
) {
    let Some(state_bytes) = git_object_bytes(root, anchor, state_path) else {
        return;
    };
    let Some(verification_bytes) = git_object_bytes(root, anchor, verification_path) else {
        return;
    };
    let Some(approval_bytes) = git_object_bytes(root, anchor, approvals_path) else {
        return;
    };
    if verification_bytes != current_verification || approval_bytes != current_approvals {
        return;
    }
    let Ok(accepted) = serde_json::from_slice::<ChangeRecord>(&state_bytes) else {
        return;
    };
    if accepted.id != record.id || accepted.state != ChangeState::Accepted {
        return;
    }
    let mut projection = record.clone();
    if record.state == ChangeState::Archived {
        projection.state = ChangeState::Accepted;
        projection.updated_at = accepted.updated_at;
    }
    if projection == accepted {
        let key = accepted_evidence_key(&state_bytes, &verification_bytes, &approval_bytes);
        eligible
            .entry(key)
            .or_insert((anchor.to_string(), state_bytes, accepted));
    }
}

/// One reachable commit that introduced a change's archived package into history, together with
/// the repository-relative package directory that commit created and the approval ledger the
/// package carried there.
///
/// The ledger is carried whole rather than reduced to its reopen count. The count is a number the
/// attacker writes next to the evidence it is supposed to qualify; the ledger is evidence already
/// in history, which a later generation must be shown to CONTAIN. See `ledger_succession`.
#[derive(Clone, Debug)]
struct ArchiveIntroduction {
    commit: String,
    directory: String,
    approvals: Option<Vec<u8>>,
}

type ArchiveIntroductionIndex = BTreeMap<String, Vec<ArchiveIntroduction>>;
static ARCHIVE_INTRODUCTION_CACHE: OnceLock<
    Mutex<BTreeMap<String, Arc<ArchiveIntroductionIndex>>>,
> = OnceLock::new();

/// Indexes every reachable commit that ADDS an archived change package, keyed by the change ID
/// recorded inside the committed `state.json` rather than by the directory the package occupies.
///
/// `find_change_dir` resolves an archived package by parsing every `state.json` under the archive
/// root and matching `record.id`, so the directory name is not part of a package's identity. An
/// anchor search keyed by path therefore accepts a commit that merely re-adds an already committed
/// package under a different name, which is what lets a `git mv` mint a fresh anchor for evidence
/// rewritten after archiving. Keying this index the way `find_change_dir` keys its lookup takes the
/// directory name out of the trust decision entirely.
///
/// Rename detection is disabled deliberately. With `diff.renames` on -- the default since Git 2.9
/// -- a relocation is reported as `R` and disappears from `--diff-filter=A`, which would make the
/// hole look closed while resting on a similarity heuristic that an attacker controls. Forcing
/// every first appearance of a path to surface as an addition puts the ordering rule in
/// `admissible_archive_introductions`, not Git's guesswork, in charge of the decision.
fn archive_introduction_index(root: &Path) -> Result<Arc<ArchiveIntroductionIndex>, String> {
    let repo_prefix = git_repo_prefix(root)?;
    let archive_root = format!("{repo_prefix}{ARCHIVE_PATH}");
    let history_pathspec = format!(":(top,literal){archive_root}");
    let directory_prefix = format!("{archive_root}/");
    let mut references = vec!["HEAD".to_string()];
    if let Some(remote_default) = remote_default_ref(root)
        && !references.contains(&remote_default)
    {
        references.push(remote_default);
    }
    let resolved: Vec<String> = references
        .iter()
        .map(|reference| {
            git_output(
                root,
                &["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
            )
            .unwrap_or_default()
        })
        .collect();
    let cache_key = format!("{}|{archive_root}|{}", root.display(), resolved.join(","));
    if let Some(cached) = ARCHIVE_INTRODUCTION_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .ok()
        .and_then(|cache| cache.get(&cache_key).cloned())
    {
        return Ok(cached);
    }
    let max_count = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
    let mut args = vec![
        "log".to_string(),
        "--format=%H".to_string(),
        "--diff-filter=A".to_string(),
        "--no-renames".to_string(),
        max_count,
    ];
    args.extend(references.iter().cloned());
    args.extend(["--".to_string(), history_pathspec.clone()]);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_git_bounded(root, &arg_refs, None, MAX_GIT_COMMAND_OUTPUT_BYTES)
        .map_err(|error| format!("failed to inspect archived acceptance history: {error}"))?;
    if !output.status.success() {
        return Err("failed to inspect archived acceptance history".into());
    }
    let history = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<&str> = history.lines().filter(|line| !line.is_empty()).collect();
    if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
        return Err(format!(
            "archived acceptance history exceeds the deterministic {}-commit bound",
            MAX_TRUSTED_HISTORY_COMMITS
        ));
    }
    let mut index: ArchiveIntroductionIndex = BTreeMap::new();
    for commit in commits {
        let output = run_git_bounded(
            root,
            &[
                "diff-tree",
                "--root",
                "-m",
                "--no-commit-id",
                "--name-only",
                "-r",
                "-z",
                "--no-renames",
                "--diff-filter=A",
                commit,
                "--",
                history_pathspec.as_str(),
            ],
            None,
            MAX_GIT_COMMAND_OUTPUT_BYTES,
        )
        .map_err(|error| {
            format!("failed to inspect archived acceptance introduction `{commit}`: {error}")
        })?;
        if !output.status.success() {
            return Err(format!(
                "failed to inspect archived acceptance introduction `{commit}`"
            ));
        }
        let names = String::from_utf8_lossy(&output.stdout);
        for path in names.split('\0').filter(|path| !path.is_empty()) {
            let Some(directory) = path.strip_suffix("/state.json") else {
                continue;
            };
            let Some(package) = directory.strip_prefix(&directory_prefix) else {
                continue;
            };
            if package.is_empty() || package.contains('/') {
                continue;
            }
            let Some(archived) = git_change_record_at(root, commit, path) else {
                continue;
            };
            let approvals = git_object_bytes(root, commit, &format!("{directory}/approvals.json"));
            let introductions = index.entry(archived.id).or_default();
            if introductions
                .iter()
                .any(|existing| existing.commit == commit && existing.directory == directory)
            {
                continue;
            }
            introductions.push(ArchiveIntroduction {
                commit: commit.to_string(),
                directory: directory.to_string(),
                approvals,
            });
        }
    }
    for introductions in index.values_mut() {
        introductions.sort_by(|left, right| {
            (&left.commit, &left.directory).cmp(&(&right.commit, &right.directory))
        });
    }
    let index = Arc::new(index);
    if let Ok(mut cache) = ARCHIVE_INTRODUCTION_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
    {
        cache.insert(cache_key, Arc::clone(&index));
    }
    Ok(index)
}

/// Whether a package's current approval ledger lawfully continues one already in history.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LedgerSuccession {
    /// The current ledger does not extend the committed one: it is the same generation, it is
    /// shorter, or it rewrote something the committed ledger already recorded.
    NotASuccessor,
    /// The current ledger keeps every approval and every reopen event the committed one recorded,
    /// adds at least one reopen event, and that event supersedes exactly the approval the
    /// committed package closed on.
    LawfulSuccessor,
}

/// The array named `field` in an approval-ledger JSON document, compared as raw JSON.
///
/// Raw values rather than `ApprovalLedger` structs on purpose: round-tripping through the typed
/// form silently drops fields the struct does not know, which is exactly where an attacker would
/// hide a difference between the ledger history committed and the one being presented.
fn ledger_entries(ledger: &serde_json::Value, field: &str) -> Vec<serde_json::Value> {
    ledger
        .get(field)
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// Decide whether `current` lawfully continues the ledger an earlier introduction committed.
///
/// `reopen` is append-only: `reopen_unarchived_change` leaves `approvals` untouched and pushes one
/// `ReopenRecord` whose `superseded_approval` is the package's own terminal approval, and the
/// `finalize` that follows only appends. A genuine later generation therefore CONTAINS the earlier
/// ledger verbatim. A relocation or a delete-and-re-add of the same package does not grow at all,
/// and a package whose committed approvals were rewritten -- the #660 laundering, whose payload is
/// an approval `actor` that no digest covers -- fails the prefix.
///
/// This is what the reopen count was asked to prove and could not. `reopenings.len()` is a number
/// written next to the evidence it is supposed to qualify, so appending one hand-made record used
/// to promote a rewritten package past the introduction that contradicts it. Extension is instead
/// checked against bytes that are already committed, which cannot be changed without rewriting
/// history itself.
///
/// `scope_adoptions` is deliberately not covered: `append_approval` clears it whenever a renewed
/// definition approval lands, so it legitimately shrinks across a reopen. It carries no actor and
/// `validate_scope_adoption` admits it for one hard-coded legacy change only.
fn ledger_succession(earlier: Option<&[u8]>, current: &[u8]) -> LedgerSuccession {
    let Some(earlier) = earlier else {
        return LedgerSuccession::NotASuccessor;
    };
    let (Ok(earlier), Ok(current)) = (
        serde_json::from_slice::<serde_json::Value>(earlier),
        serde_json::from_slice::<serde_json::Value>(current),
    ) else {
        return LedgerSuccession::NotASuccessor;
    };
    let earlier_approvals = ledger_entries(&earlier, "approvals");
    let current_approvals = ledger_entries(&current, "approvals");
    let earlier_reopenings = ledger_entries(&earlier, "reopenings");
    let current_reopenings = ledger_entries(&current, "reopenings");
    if current_reopenings.len() <= earlier_reopenings.len()
        || current_approvals.len() < earlier_approvals.len()
        || current_approvals[..earlier_approvals.len()] != earlier_approvals[..]
        || current_reopenings[..earlier_reopenings.len()] != earlier_reopenings[..]
    {
        return LedgerSuccession::NotASuccessor;
    }
    // The first reopen event the successor adds must supersede exactly the approval the earlier
    // package closed on, so that a new generation names the package it supersedes instead of
    // merely counting past it.
    let Some(superseded) = current_reopenings[earlier_reopenings.len()].get("superseded_approval")
    else {
        return LedgerSuccession::NotASuccessor;
    };
    let earlier_terminal = earlier_approvals.iter().rev().find(|approval| {
        approval
            .get("gate")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|gate| matches!(gate, "acceptance" | "finalization"))
    });
    if earlier_terminal != Some(superseded) {
        return LedgerSuccession::NotASuccessor;
    }
    LedgerSuccession::LawfulSuccessor
}

/// Selects the archived-package introductions that may anchor `record`'s acceptance.
///
/// A change's package can legitimately enter history more than once: `reopen` un-archives an
/// accepted package and a later `finalize` archives it again, so history holds an earlier
/// introduction of superseded evidence and a later introduction of the evidence in the tree.
/// What an attacker produces instead is a re-introduction of the SAME evidence -- a relocation, or
/// a delete-and-re-add -- carrying bytes rewritten after archiving.
///
/// The two are told apart by whether the package being authenticated CONTAINS the ledger the
/// earlier introduction committed. An introduction is admissible only when no strictly earlier
/// introduction of the same change committed a ledger the current package fails to extend. On a
/// package that was never reopened this reduces to "the earliest introduction wins", the property
/// a `git mv` cannot mint; on a reopened one it admits the new generation without ever consulting
/// a count the attacker controls.
fn admissible_archive_introductions(
    root: &Path,
    record: &ChangeRecord,
    current_approvals: &[u8],
) -> Result<Vec<ArchiveIntroduction>, String> {
    let index = archive_introduction_index(root)?;
    let Some(candidates) = index.get(&record.id) else {
        return Ok(Vec::new());
    };
    let mut admissible = Vec::new();
    for candidate in candidates {
        let superseded = candidates.iter().any(|earlier| {
            earlier.commit != candidate.commit
                && ledger_succession(earlier.approvals.as_deref(), current_approvals)
                    == LedgerSuccession::NotASuccessor
                && ensure_git_ancestor(
                    root,
                    &earlier.commit,
                    &candidate.commit,
                    "archived acceptance introduction",
                )
                .is_ok()
        });
        if !superseded {
            admissible.push(candidate.clone());
        }
    }
    Ok(admissible)
}

/// True when `current_approvals` lawfully extends every ledger history holds for this change.
///
/// Used only by the closing-evidence fallback, where the package being authenticated is not in
/// history yet because the commit that would introduce it has not been made. Every generation
/// already committed must be contained in the one about to be, so a reopen may add evidence and
/// may never rewrite what an earlier archive of the same change recorded.
fn closing_ledger_extends_all_introductions(
    root: &Path,
    record: &ChangeRecord,
    current_approvals: &[u8],
) -> Result<bool, String> {
    let index = archive_introduction_index(root)?;
    Ok(index.get(&record.id).is_some_and(|introductions| {
        !introductions.is_empty()
            && introductions.iter().all(|introduction| {
                ledger_succession(introduction.approvals.as_deref(), current_approvals)
                    == LedgerSuccession::LawfulSuccessor
            })
    }))
}

/// True when `anchor` lies in the history that produced one of `introductions`.
///
/// The active-workspace stages authenticate an archived change from `.specsync/changes/<id>/`,
/// where a genuine acceptance transition was recorded before the package was archived -- and where
/// a forged `reopen`/`archive` pair writes the same shape afterwards. Requiring the anchor to
/// precede the package's introduction keeps the first and rejects the second without depending on
/// the directory name, on a diff being empty, or on Git's rename detection.
fn anchor_precedes_an_introduction(
    root: &Path,
    anchor: &str,
    introductions: &[ArchiveIntroduction],
) -> bool {
    introductions.iter().any(|introduction| {
        introduction.commit == anchor
            || ensure_git_ancestor(
                root,
                anchor,
                &introduction.commit,
                "archived acceptance introduction",
            )
            .is_ok()
    })
}

/// The archive this process is creating right now, named by the package directory it is closing.
///
/// `archive_change_with_options` both writes a package and validates it. Only the writing process
/// knows that the closing evidence under `package` is evidence it authenticated out of the
/// change's own ACTIVE workspace moments ago, and that the commit which would introduce the
/// package is the one this command is preparing. Every reading path -- `status`, `audit`, `list`,
/// `ship-status`, the corpus census, the legacy-baseline check -- passes `None` and is judged
/// entirely by history, which is what #660 requires. The successor-coverage walk forwards whatever
/// its caller holds: `None` from every reader, and the token from the archive preflight, where the
/// package being closed is the only successor that can cover the changes it supersedes and would
/// otherwise be judged by a history its own commit has not yet written.
///
/// The token is deliberately NOT minted for a post-move resume whose package was found already
/// sitting in the archive. That shape cannot be told from an attacker flipping a committed
/// package's `state.json` back to `accepted` and re-running `finalize`, so it keeps HEAD's rule.
struct PendingArchiveClose<'a> {
    package: &'a Path,
}

impl PendingArchiveClose<'_> {
    /// True when `workspace` is the very package directory this process is closing.
    fn is_closing(&self, workspace: &Path) -> bool {
        let resolve = |path: &Path| path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        resolve(self.package) == resolve(workspace)
    }
}

fn authenticated_accepted_transition(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(String, Vec<u8>, ChangeRecord), String> {
    authenticated_accepted_transition_for(root, record, None)
}

fn authenticated_accepted_transition_for(
    root: &Path,
    record: &ChangeRecord,
    pending: Option<&PendingArchiveClose<'_>>,
) -> Result<(String, Vec<u8>, ChangeRecord), String> {
    let workspace = find_change_dir(root, &record.id)?;
    let workspace_relative = strict_portable_project_path(root, &workspace)?;
    let current_verification_path = format!("{workspace_relative}/verification.json");
    let current_approvals_path = format!("{workspace_relative}/approvals.json");
    let candidates = BTreeSet::from([
        current_verification_path.clone(),
        current_approvals_path.clone(),
    ]);
    let evidence = git_evidence(root, &candidates)?;
    let current_verification = evidence.entry(&current_verification_path)?.payload.clone();
    let current_approvals = evidence.entry(&current_approvals_path)?.payload.clone();
    let active = format!("{CHANGES_PATH}/{}", record.id);
    let state_path = git_repo_relative_path(root, &format!("{active}/state.json"))?;
    let verification_path = git_repo_relative_path(root, &format!("{active}/verification.json"))?;
    let approvals_path = git_repo_relative_path(root, &format!("{active}/approvals.json"))?;
    let mut eligible = BTreeMap::new();
    // Once this change's archived package exists in reachable history, that history is the
    // authority for its acceptance. The active-workspace stages accept any commit that merely
    // carries the bytes being checked, which is how a forged `reopen`/`archive` pair mints an
    // anchor without ever touching the archive directory. They are therefore admitted only for
    // commits that PRECEDE the package's introduction -- the genuine pre-archive acceptance
    // transition -- and not at all once history holds the package. The working-tree fallback below
    // is bounded separately, because the one package history cannot hold is the one this command
    // is about to create.
    let introductions = if record.state == ChangeState::Archived {
        admissible_archive_introductions(root, record, &current_approvals)?
    } else {
        Vec::new()
    };
    let archived_package_is_in_history = record.state == ChangeState::Archived
        && archive_introduction_index(root)?.contains_key(&record.id);
    for anchor in accepted_transition_anchors(root, record)? {
        if archived_package_is_in_history
            && !anchor_precedes_an_introduction(root, &anchor, &introductions)
        {
            continue;
        }
        consider_accepted_evidence_anchor(
            root,
            record,
            &anchor,
            &state_path,
            &verification_path,
            &approvals_path,
            &current_verification,
            &current_approvals,
            &mut eligible,
        );
    }
    if record.state == ChangeState::Archived {
        for introduction in &introductions {
            let accepted_state_path = format!("{}/accepted-state.json", introduction.directory);
            let archived_verification_path =
                format!("{}/verification.json", introduction.directory);
            let archived_approvals_path = format!("{}/approvals.json", introduction.directory);
            consider_accepted_evidence_anchor(
                root,
                record,
                &introduction.commit,
                &accepted_state_path,
                &archived_verification_path,
                &archived_approvals_path,
                &current_verification,
                &current_approvals,
                &mut eligible,
            );
        }
    }
    if eligible.is_empty() {
        // Squash merges discard the original acceptance-transition commits while preserving
        // the accepted evidence bytes in later default-branch commits. Trust any in-history
        // commit that records this change as accepted when it carries byte-identical
        // evidence; the per-anchor checks, the introduction bound above and the exactly-one
        // rule still fail closed.
        for anchor in accepted_recording_anchors(root, record)? {
            if archived_package_is_in_history
                && !anchor_precedes_an_introduction(root, &anchor, &introductions)
            {
                continue;
            }
            consider_accepted_evidence_anchor(
                root,
                record,
                &anchor,
                &state_path,
                &verification_path,
                &approvals_path,
                &current_verification,
                &current_approvals,
                &mut eligible,
            );
        }
    }
    // A reopened change's SECOND package is not in history at the moment it is created: the
    // commit that would introduce it is the one this `finalize` is preparing. History holds only
    // the superseded generation, whose bytes stage B refuses exactly as it should, so #663's
    // "once the package is in history, history is the only authority" left a genuine re-finalize
    // with nothing to authenticate against (#540). The fallback is reopened for that case alone,
    // under two conditions no reader of a repository can satisfy together:
    //
    //   * this process is the one WRITING the package (`is_closing`), never true on a read path;
    //   * the ledger it is about to commit contains, unrewritten, every ledger history already
    //     holds for this change -- so a reopen may append evidence and may never restate what an
    //     earlier archive of the same change recorded.
    //
    // Both are required. The first alone would let `finalize` bless a package it merely found in
    // the archive; the second alone would let any working tree speak for a committed package.
    let closing_evidence_may_speak = !archived_package_is_in_history
        || (pending.is_some_and(|pending| pending.is_closing(&workspace))
            && closing_ledger_extends_all_introductions(root, record, &current_approvals)?);
    if eligible.is_empty()
        && closing_evidence_may_speak
        && let Ok(state_bytes) = fs::read(workspace.join("accepted-state.json"))
        && let Ok(accepted) = serde_json::from_slice::<ChangeRecord>(&state_bytes)
    {
        let mut projection = record.clone();
        if record.state == ChangeState::Archived {
            projection.state = ChangeState::Accepted;
            projection.updated_at = accepted.updated_at;
        }
        if projection == accepted
            && accepted.state == ChangeState::Accepted
            && staged_accepted_snapshot_is_closing_authenticated(
                root,
                &accepted,
                &current_verification,
                &current_approvals,
            )?
        {
            let key =
                accepted_evidence_key(&state_bytes, &current_verification, &current_approvals);
            eligible.insert(
                key,
                (
                    WORKING_TREE_CLOSING_EVIDENCE_ANCHOR.into(),
                    state_bytes,
                    accepted,
                ),
            );
        }
    }
    if eligible.len() != 1 {
        return Err(format!(
            "accepted change `{}` requires exactly one trusted transition matching its state, verification, and closing evidence; found {}",
            record.id,
            eligible.len()
        ));
    }
    eligible
        .into_values()
        .next()
        .ok_or_else(|| "authenticated accepted transition disappeared".to_string())
}

fn staged_accepted_snapshot_is_closing_authenticated(
    root: &Path,
    accepted: &ChangeRecord,
    verification_bytes: &[u8],
    approval_bytes: &[u8],
) -> Result<bool, String> {
    let verification: VerificationRecord = serde_json::from_slice(verification_bytes)
        .map_err(|error| format!("invalid staged accepted verification: {error}"))?;
    let approvals: ApprovalLedger = serde_json::from_slice(approval_bytes)
        .map_err(|error| format!("invalid staged accepted approvals: {error}"))?;
    if !verification.passed
        || !definition_digest_matches(root, accepted, &verification.contract_digest)?
    {
        return Ok(false);
    }
    if validate_verification_execution_digest(root, accepted, &verification).is_err() {
        return Ok(false);
    }
    let Some(signed_inputs) = verification.acceptance_input_digest.as_ref() else {
        return Ok(false);
    };
    if let Some(manifest) = &verification.acceptance_manifest {
        if acceptance_manifest_digest(manifest)? != *signed_inputs {
            return Ok(false);
        }
        match (&accepted.supersedes[..], &verification.semantic_succession) {
            ([], None) => {}
            (_, Some(evidence)) => validate_semantic_succession(accepted, evidence)?,
            _ => return Ok(false),
        }
    } else if verification.semantic_succession.is_some() {
        return Ok(false);
    }
    if accepted.workflow_version >= 2 {
        return Ok(
            validate_finalization_evidence(root, accepted, &verification).is_ok()
                && verification_commit_is_accepted_current(root, &verification),
        );
    }
    let closing_matches = latest_terminal_approval(&approvals)
        .is_some_and(|approval| approval.digest == closing_digest(accepted, &verification));
    Ok(closing_matches && verification_commit_is_accepted_current(root, &verification))
}

fn accepted_evidence_key(state: &[u8], verification: &[u8], approvals: &[u8]) -> String {
    let mut digest = FramedDigest::new(b"specsync.accepted-evidence-key.v1");
    digest.frame(b"state", state);
    digest.frame(b"verification", verification);
    digest.frame(b"approvals", approvals);
    digest.finish()
}

fn git_object_bytes(root: &Path, commit: &str, path: &str) -> Option<Vec<u8>> {
    let object = format!("{commit}:{path}");
    let output = run_git_bounded(
        root,
        &["show", object.as_str()],
        None,
        MAX_CHANGE_ARTIFACT_BYTES as usize,
    )
    .ok()?;
    output.status.success().then_some(output.stdout)
}

fn git_change_record_at(root: &Path, commit: &str, path: &str) -> Option<ChangeRecord> {
    let object = format!("{commit}:{path}");
    let output = run_git_bounded(
        root,
        &["show", object.as_str()],
        None,
        MAX_CHANGE_ARTIFACT_BYTES as usize,
    )
    .ok()?;
    output
        .status
        .success()
        .then(|| serde_json::from_slice(&output.stdout).ok())
        .flatten()
}

fn acceptance_input_digest(
    root: &Path,
    record: &ChangeRecord,
    overrides: &[(PathBuf, String)],
) -> Result<String, String> {
    let discovery_scopes = acceptance_discovery_scopes(root, record)?;
    let override_content: BTreeMap<String, &[u8]> = overrides
        .iter()
        .map(|(path, content)| {
            Ok((
                strict_portable_project_path(root, path)?,
                content.as_bytes(),
            ))
        })
        .collect::<Result<_, String>>()?;
    let extra_candidates = override_content.keys().cloned().collect();
    let (mut paths, evidence) = stable_discovered_evidence(
        root,
        Some(&discovery_scopes),
        &extra_candidates,
        false,
        false,
    )?;
    paths.retain(|path| {
        !project_input_is_volatile(path) && record_covers_project_path(root, record, path)
    });
    let historical_sequence_ledger = if record_covers_project_path(root, record, SEQUENCE_PATH) {
        historical_sequence_ledger_acceptance_content(root, record)?
    } else {
        None
    };
    let mut digest = FramedDigest::new(ACCEPTANCE_DIGEST_DOMAIN);
    for relative in paths {
        if let Some(content) = override_content.get(&relative) {
            let mode = evidence.generated_file_mode(&relative)?;
            let kind: &[u8] = match mode {
                0o120000 => b"symlink",
                0o160000 => b"gitlink",
                _ => b"file",
            };
            digest.entry(&relative, kind, mode, content);
        } else if relative == SEQUENCE_PATH
            && let Some(content) = &historical_sequence_ledger
        {
            let mode = evidence.generated_file_mode(&relative)?;
            digest.entry(&relative, b"file", mode, content);
        } else {
            let entry = evidence.entry(&relative)?;
            digest.entry(
                &relative,
                acceptance_kind_bytes(&entry.kind),
                entry.mode,
                &entry.payload,
            );
        }
    }
    Ok(digest.finish())
}

fn historical_sequence_ledger_acceptance_content(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Option<Vec<u8>>, String> {
    validate_change_sequences(root)?;
    let Some(ledger) = load_change_sequence_ledger(root)? else {
        return Ok(None);
    };
    // A slug-only change has no earlier ledger revision to reconstruct: it never claimed a
    // sequence, so the frozen file it may have scoped is the same file it signed.
    let Some(sequence) = located_change_ordinal(&record.id)? else {
        return Ok(None);
    };
    if ledger.sequence <= sequence {
        return Ok(None);
    }
    if let Some(content) =
        historical_sequence_ledger_from_history(root, sequence, record.id.as_str())?
    {
        return Ok(Some(content));
    }
    let historical = ChangeSequenceLedger {
        schema_version: ledger.schema_version,
        sequence,
        id: record.id.clone(),
        acknowledged_collisions: ledger
            .acknowledged_collisions
            .into_iter()
            .filter(|collision| collision.sequence <= sequence)
            .collect(),
    };
    Ok(Some(json_content(&historical)?.into_bytes()))
}

fn historical_sequence_ledger_from_history(
    root: &Path,
    sequence: u64,
    record_id: &str,
) -> Result<Option<Vec<u8>>, String> {
    let candidates = if let Some(cached) = read_scope_value(root, |scope| {
        scope.historical_sequence_ledgers.get(&sequence).cloned()
    }) {
        cached?
    } else {
        let result = (|| {
            let mut candidates = Vec::new();
            if !git_repository_present(root)? {
                return Ok(candidates);
            }
            let path = git_repo_relative_path(root, SEQUENCE_PATH)?;
            let limits = lifecycle_validation_limits();
            let max_count = format!("--max-count={}", limits.scoped_review_max_descendants + 1);
            let history = scoped_review_git_text(
                root,
                &["rev-list", max_count.as_str(), "HEAD", "--", path.as_str()],
            )
            .map_err(|_| "failed to inspect historical change-sequence ledgers".to_string())?;
            let commits: Vec<&str> = history.lines().filter(|line| !line.is_empty()).collect();
            if commits.len() > limits.scoped_review_max_descendants {
                return Err(format!(
                    "change-sequence history exceeds the deterministic {}-commit bound",
                    limits.scoped_review_max_descendants
                ));
            }
            for commit in commits {
                let Some(bytes) = scoped_review_file_at_commit(root, commit, &path)? else {
                    continue;
                };
                let Ok(candidate) = serde_json::from_slice::<ChangeSequenceLedger>(&bytes) else {
                    continue;
                };
                if candidate.schema_version == 1
                    && candidate.sequence == sequence
                    && change_sequence(&candidate.id) == Some(sequence)
                    && json_content(&candidate)?.as_bytes() == bytes
                {
                    candidates.push(bytes);
                }
            }
            Ok(candidates)
        })();
        update_read_scope(root, |scope| {
            scope
                .historical_sequence_ledgers
                .insert(sequence, result.clone());
        });
        result?
    };
    for bytes in candidates {
        let candidate: ChangeSequenceLedger =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if candidate.acknowledged_collisions.iter().any(|collision| {
            collision.sequence == sequence && collision.ids.iter().any(|id| id == record_id)
        }) {
            return Ok(Some(bytes));
        }
    }
    Ok(None)
}

fn resolve_definition_approval_event<'a>(
    record: &ChangeRecord,
    ledger: &'a ApprovalLedger,
    expected_current_digest: &str,
    expected_legacy_digest: Option<&str>,
    expected_correction_prefix: &str,
) -> Result<&'a ApprovalRecord, String> {
    let index = ledger
        .approvals
        .iter()
        .rposition(|approval| approval.gate == "definition")
        .ok_or_else(|| "definition approval is missing".to_string())?;
    let terminal = &ledger.approvals[index];
    let Some(legacy_metadata) = terminal.definition_pair.as_ref() else {
        if terminal.digest != expected_current_digest {
            return Err(
                "definition approval is stale; approve the current artifact digest again".into(),
            );
        }
        return Ok(terminal);
    };
    if legacy_metadata.schema_version != 1
        || legacy_metadata.projection != PORTABLE_DEFINITION_PROJECTION_V501
        || legacy_metadata.role != DefinitionApprovalPairRole::Legacy
        || legacy_metadata.change_id != record.id
        || legacy_metadata.correction_count != record.correction_count
        || legacy_metadata.correction_prefix_digest != expected_correction_prefix
        || legacy_metadata.event_index.checked_add(1) != Some(index as u64)
    {
        return Err("portable definition approval has invalid terminal metadata".into());
    }
    let current_index = index
        .checked_sub(1)
        .ok_or_else(|| "portable definition approval is missing its current member".to_string())?;
    let current = &ledger.approvals[current_index];
    let current_metadata = current
        .definition_pair
        .as_ref()
        .ok_or_else(|| "portable definition approval current member is unmarked".to_string())?;
    if current.gate != "definition"
        || current_metadata.schema_version != legacy_metadata.schema_version
        || current_metadata.projection != legacy_metadata.projection
        || current_metadata.pair_id != legacy_metadata.pair_id
        || current_metadata.role != DefinitionApprovalPairRole::Current
        || current_metadata.change_id != legacy_metadata.change_id
        || current_metadata.correction_count != legacy_metadata.correction_count
        || current_metadata.correction_prefix_digest != legacy_metadata.correction_prefix_digest
        || current_metadata.current_digest != legacy_metadata.current_digest
        || current_metadata.legacy_digest != legacy_metadata.legacy_digest
        || current_metadata.event_index != legacy_metadata.event_index
        || current_metadata.event_index != current_index as u64
        || current.actor != terminal.actor
        || current.timestamp != terminal.timestamp
        || current.digest == terminal.digest
        || current.digest != expected_current_digest
        || current.digest != current_metadata.current_digest
        || terminal.digest != legacy_metadata.legacy_digest
        || expected_legacy_digest.is_some_and(|expected| terminal.digest != expected)
    {
        return Err("portable definition approval pair is malformed or stale".into());
    }
    let expected_pair_id = definition_approval_pair_id(
        record,
        current_index as u64,
        &current.actor,
        current.timestamp,
        expected_correction_prefix,
        &current.digest,
        &terminal.digest,
    );
    if legacy_metadata.pair_id != expected_pair_id
        || ledger
            .approvals
            .iter()
            .filter(|approval| {
                approval
                    .definition_pair
                    .as_ref()
                    .is_some_and(|metadata| metadata.pair_id == legacy_metadata.pair_id)
            })
            .count()
            != 2
    {
        return Err("portable definition approval pair is duplicated or replayed".into());
    }
    Ok(current)
}

fn validate_scope_adoption<'a>(
    root: &Path,
    record: &ChangeRecord,
    ledger: &'a ApprovalLedger,
    approval_index: usize,
    approval: &ApprovalRecord,
) -> Result<&'a ApprovedScopeV1, String> {
    let matches: Vec<&ScopeAdoptionV1> = ledger
        .scope_adoptions
        .iter()
        .filter(|adoption| adoption.source_approval_index == approval_index as u64)
        .collect();
    if matches.len() != 1 || ledger.scope_adoptions.len() != 1 {
        return Err("scope adoption must contain one unambiguous allowlisted event".into());
    }
    let adoption = matches[0];
    let immutable_event_matches = record.id == CHG_0068_ID
        && approval_index == 0
        && approval.gate == "definition"
        && approval.actor == "0xLeif"
        && approval.timestamp == 1_785_369_606
        && approval.digest == CHG_0068_LEGACY_APPROVAL_DIGEST
        && approval.note.is_none()
        && approval.definition_pair.is_none()
        && approval.approved_scope.is_none()
        && approval.scope_migration.is_none()
        // The allowlist pins the ENTIRE shape of the one trusted historical event; leaving a
        // field unpinned would let a later rewrite add evidence the allowlist never authorized.
        && approval.approved_delta_digests.is_none();
    if !immutable_event_matches {
        return Err(
            "scope adoption source approval is outside the trusted CHG-0068 allowlist".into(),
        );
    }
    let adoption_shape_matches = adoption.schema_version == 1
        && adoption.change_id == CHG_0068_ID
        && adoption.source_approval_index == 0
        && adoption.legacy_approval_digest == CHG_0068_LEGACY_APPROVAL_DIGEST
        && adoption.source_preimage_status == ScopeAdoptionSourcePreimageStatus::Unavailable
        && adoption.equivalence_claim == ScopeAdoptionEquivalenceClaim::None
        && adoption.adopted_scope_digest == CHG_0068_ADOPTED_SCOPE_DIGEST
        && adoption.anchor.base_commit == CHG_0068_ADOPTION_BASE_COMMIT
        && adoption.anchor.commit == CHG_0068_ADOPTION_ANCHOR_COMMIT
        && adoption.anchor.approval_index == 0
        && adoption.anchor.approvals_blob_sha256 == CHG_0068_ADOPTION_ANCHOR_BLOB
        && adoption.authorization.actor == "0xLeif"
        && adoption.authorization.recorded_at == 1_785_381_022
        && adoption.authorization.reason == CHG_0068_ADOPTION_REASON
        && !adoption.changes.is_empty()
        && adoption.changes.iter().all(|change| {
            !change.path.trim().is_empty()
                && change.path.trim() == change.path
                && !change.summary.trim().is_empty()
                && change.summary.trim() == change.summary
                && strict_portable_relative_path(&change.path).is_ok()
        });
    if !adoption_shape_matches {
        return Err("scope adoption differs from the trusted CHG-0068 allowlist".into());
    }
    if scope_digest_from_approved(&adoption.adopted_scope)? != CHG_0068_ADOPTED_SCOPE_DIGEST {
        return Err("scope adoption projection does not match its allowlisted digest".into());
    }
    let changes = serde_json::to_vec(&adoption.changes)
        .map_err(|error| format!("failed to hash scope adoption classifications: {error}"))?;
    if sha256_hex(&changes) != CHG_0068_ADOPTION_CHANGES_DIGEST {
        return Err("scope adoption classifications differ from the trusted allowlist".into());
    }

    let anchor_expression = format!("{}^{{commit}}", adoption.anchor.commit);
    let resolved_anchor = git_output(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &anchor_expression,
        ],
    )
    .ok_or_else(|| {
        "scope adoption anchor is unavailable; fetch full trusted history".to_string()
    })?;
    if resolved_anchor != adoption.anchor.commit {
        return Err("scope adoption anchor is not canonical".into());
    }
    let resolved_base = git_output(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{}^{{commit}}", adoption.anchor.base_commit),
        ],
    )
    .ok_or_else(|| "scope adoption base commit is unavailable".to_string())?;
    let anchor_parent = git_output(
        root,
        &["rev-parse", &format!("{}^", adoption.anchor.commit)],
    )
    .ok_or_else(|| "scope adoption anchor parent is unavailable".to_string())?;
    if resolved_base != adoption.anchor.base_commit || anchor_parent != resolved_base {
        return Err("scope adoption anchor is outside its allowlisted base lineage".into());
    }
    let path = git_repo_relative_path(
        root,
        &format!("{CHANGES_PATH}/{}/approvals.json", record.id),
    )?;
    let bytes = git_file_at_commit(root, &resolved_anchor, &path)?
        .ok_or_else(|| "scope adoption anchor approval blob is unavailable".to_string())?;
    if sha256_hex(&bytes) != CHG_0068_ADOPTION_ANCHOR_BLOB {
        return Err("scope adoption anchor approval blob is not allowlisted".into());
    }
    let anchor: ApprovalLedger = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid scope adoption anchor approval ledger: {error}"))?;
    let anchored_approval = anchor
        .approvals
        .get(adoption.anchor.approval_index as usize)
        .ok_or_else(|| "scope adoption anchor approval index is missing".to_string())?;
    let anchored_migration = anchored_approval
        .scope_migration
        .as_ref()
        .ok_or_else(|| "scope adoption anchor migration projection is missing".to_string())?;
    if anchored_approval.gate != approval.gate
        || anchored_approval.actor != approval.actor
        || anchored_approval.timestamp != approval.timestamp
        || anchored_approval.digest != approval.digest
        || anchored_approval.note != approval.note
        || anchored_approval.approved_scope.as_ref() != Some(&adoption.adopted_scope)
        || anchored_migration.source_definition_digest != adoption.legacy_approval_digest
        || anchored_migration.scope_digest != adoption.adopted_scope_digest
        || anchored_migration.changes != adoption.changes
    {
        return Err("scope adoption does not match its immutable anchor projection".into());
    }
    Ok(&adoption.adopted_scope)
}

fn effective_definition_approval<'a>(
    root: &Path,
    record: &ChangeRecord,
    ledger: &'a ApprovalLedger,
) -> Result<&'a ApprovalRecord, String> {
    let latest_index = ledger
        .approvals
        .iter()
        .rposition(|approval| approval.gate == "definition")
        .ok_or_else(|| "definition approval is missing".to_string())?;
    let latest = &ledger.approvals[latest_index];
    if record.workflow_version >= 2 {
        let approved = if let Some(approved) = latest.approved_scope.as_ref() {
            if latest.scope_migration.is_some() || !ledger.scope_adoptions.is_empty() {
                return Err(
                    "scope approval combines incompatible direct and adopted projections".into(),
                );
            }
            approved
        } else {
            validate_scope_adoption(root, record, ledger, latest_index, latest)?
        };
        if approved.schema_version != 1 || approved.change_id != record.id {
            return Err("scope approval projection has the wrong schema or change identity".into());
        }
        let approved_digest = scope_digest_from_approved(approved)?;
        if latest.approved_scope.is_some() && latest.digest != approved_digest {
            return Err("scope approval digest does not match its approved projection".into());
        }
        let current = approved_scope(root, record)?;
        let expansion = scope_expansion(approved, &current);
        if !expansion.is_empty() {
            return Err(format!(
                "scope approval requires renewal because {}",
                expansion.join("; ")
            ));
        }
        return Ok(latest);
    }
    let corrections = load_correction_ledger(root, record)?;
    let prefix = correction_prefix_digest(record, &corrections.corrections)?;
    if latest.definition_pair.is_some() {
        let (current, legacy, portable_prefix) =
            portable_definition_digest_pair_v501(root, record)?;
        if prefix != portable_prefix {
            return Err("portable definition approval correction prefix is stale".into());
        }
        if let Ok(approval) =
            resolve_definition_approval_event(record, ledger, &current, Some(&legacy), &prefix)
        {
            return Ok(approval);
        }
        let (legacy_current, legacy_projected, legacy_prefix) =
            portable_definition_digest_pair_v501_with_task_mode(root, record, false)?;
        if prefix != legacy_prefix {
            return Err("portable definition approval correction prefix is stale".into());
        }
        return resolve_definition_approval_event(
            record,
            ledger,
            &legacy_current,
            Some(&legacy_projected),
            &prefix,
        );
    }
    let current = definition_digest(root, record)?;
    match resolve_definition_approval_event(record, ledger, &current, None, &prefix) {
        Ok(approval) => Ok(approval),
        Err(_) => {
            let transitional = definition_digest_with_explicit_false(root, record)?;
            if let Ok(approval) =
                resolve_definition_approval_event(record, ledger, &transitional, None, &prefix)
            {
                return Ok(approval);
            }
            for explicit_false in [false, true] {
                let legacy = legacy_task_definition_digest_for_correction_count(
                    root,
                    record,
                    record.correction_count,
                    explicit_false,
                )?;
                if let Ok(approval) =
                    resolve_definition_approval_event(record, ledger, &legacy, None, &prefix)
                {
                    return Ok(approval);
                }
            }
            Err("definition approval is stale; approve the current artifact digest again".into())
        }
    }
}

fn ensure_definition_approval_valid(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let ledger = load_approvals(root, record)?;
    effective_definition_approval(root, record, &ledger).map(|_| ())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AcceptedInputValidity {
    Exact,
    SuccessorCovered,
}

fn terminal_evidence_summary(root: &Path, record: &ChangeRecord) -> TerminalEvidenceSummary {
    if let Some(cached) = read_scope_value(root, |scope| scope.terminal_evidence.clone()) {
        return terminal_summary_from_cache(record, cached);
    }
    if read_scope_value(root, |_| Some(())).is_some() {
        let result = list_all_changes_checked(root).map(|records| {
            let (results, _) = terminal_evidence_results_with_records(root, &records, &records);
            results
                .into_iter()
                .map(|result| (result.id, result.evidence))
                .collect()
        });
        update_read_scope(root, |scope| {
            scope.terminal_evidence = Some(result.clone());
        });
        return terminal_summary_from_cache(record, result);
    }
    let result = list_all_changes_checked(root)
        .map(|records| terminal_evidence_summary_with_records(root, record, &records));
    match result {
        Ok(summary) => summary,
        Err(reason) => TerminalEvidenceSummary {
            validity: if record.state == ChangeState::Archived {
                TerminalEvidenceValidity::CorruptHistory
            } else {
                TerminalEvidenceValidity::Stale
            },
            reason: Some(reason),
        },
    }
}

fn terminal_summary_from_cache(
    record: &ChangeRecord,
    cached: Result<BTreeMap<String, TerminalEvidenceSummary>, String>,
) -> TerminalEvidenceSummary {
    match cached {
        Ok(summaries) => {
            summaries
                .get(&record.id)
                .cloned()
                .unwrap_or_else(|| TerminalEvidenceSummary {
                    validity: if record.state == ChangeState::Archived {
                        TerminalEvidenceValidity::CorruptHistory
                    } else {
                        TerminalEvidenceValidity::Stale
                    },
                    reason: Some(format!(
                        "terminal evidence snapshot is missing change `{}`",
                        record.id
                    )),
                })
        }
        Err(reason) => TerminalEvidenceSummary {
            validity: if record.state == ChangeState::Archived {
                TerminalEvidenceValidity::CorruptHistory
            } else {
                TerminalEvidenceValidity::Stale
            },
            reason: Some(reason),
        },
    }
}

fn terminal_evidence_summary_with_records(
    root: &Path,
    record: &ChangeRecord,
    records: &BTreeMap<String, ChangeRecord>,
) -> TerminalEvidenceSummary {
    terminal_evidence_summary_with_validation_state(
        root,
        record,
        records,
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
        &mut ArchivedIntegrityCache::default(),
    )
}

/// Terminal evidence for every accepted or archived record in `records`, each judged against
/// `successors` as the universe of changes that may cover its changed inputs.
///
/// The two maps are distinct on purpose. The active-only audit evaluates active terminal records
/// alone, but a finalized successor is an archive, so a legacy accepted change superseded by the
/// v2 change that had just finalized over it was reported as uncovered by `audit` while the
/// archive preflight and the full walk both saw the cover. Evaluating fewer records never means
/// offering fewer successors.
fn terminal_evidence_results_with_records(
    root: &Path,
    records: &BTreeMap<String, ChangeRecord>,
    successors: &BTreeMap<String, ChangeRecord>,
) -> (Vec<TerminalEvidenceResult>, ArchivedIntegrityCache) {
    let mut visiting = BTreeSet::new();
    let mut memo = BTreeMap::new();
    let mut archived_cache = ArchivedIntegrityCache::default();
    let results = records
        .values()
        .filter(|record| matches!(record.state, ChangeState::Accepted | ChangeState::Archived))
        .map(|record| TerminalEvidenceResult {
            id: record.id.clone(),
            evidence: terminal_evidence_summary_with_validation_state(
                root,
                record,
                successors,
                &mut visiting,
                &mut memo,
                &mut archived_cache,
            ),
        })
        .collect();
    (results, archived_cache)
}

fn terminal_evidence_summary_with_validation_state(
    root: &Path,
    record: &ChangeRecord,
    records: &BTreeMap<String, ChangeRecord>,
    visiting: &mut BTreeSet<String>,
    memo: &mut BTreeMap<String, Result<AcceptedInputValidity, String>>,
    archived_cache: &mut ArchivedIntegrityCache,
) -> TerminalEvidenceSummary {
    if record.state == ChangeState::Archived {
        return match validate_archived_integrity_with_cache(root, record, archived_cache) {
            Ok(()) => TerminalEvidenceSummary {
                validity: TerminalEvidenceValidity::AuthenticatedHistory,
                reason: None,
            },
            Err(reason) => TerminalEvidenceSummary {
                validity: TerminalEvidenceValidity::CorruptHistory,
                reason: Some(reason),
            },
        };
    }
    match validate_accepted_inputs_recursive(root, record, records, visiting, memo, None) {
        Ok(AcceptedInputValidity::Exact) => TerminalEvidenceSummary {
            validity: TerminalEvidenceValidity::Exact,
            reason: None,
        },
        Ok(AcceptedInputValidity::SuccessorCovered) => TerminalEvidenceSummary {
            validity: TerminalEvidenceValidity::SuccessorCovered,
            reason: None,
        },
        Err(reason) => TerminalEvidenceSummary {
            validity: TerminalEvidenceValidity::Stale,
            reason: Some(reason),
        },
    }
}

fn ensure_closing_approval_valid(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let records = list_all_changes_checked(root)?;
    let mut visiting = BTreeSet::new();
    let mut memo = BTreeMap::new();
    validate_accepted_inputs_recursive(root, record, &records, &mut visiting, &mut memo, None)
        .map(|_| ())
}

/// `pending` is the closing token of `archive_change_with_options`, forwarded so the package that
/// command is closing can authenticate as the successor of a change it supersedes. Every reading
/// path passes `None`. See `PendingArchiveClose`.
fn authenticate_accepted_evidence_with_anchor(
    root: &Path,
    record: &ChangeRecord,
    pending: Option<&PendingArchiveClose<'_>>,
) -> Result<(VerificationRecord, bool), String> {
    validate_acceptance_owner_correction_records(record)?;
    if record.state == ChangeState::Archived {
        validate_archived_accepted_snapshot(root, record, pending)?;
    }
    let verification = load_verification(root, record)?;
    if !verification.passed {
        return Err("accepted change has failed verification evidence".into());
    }
    ensure_definition_approval_valid(root, record)?;
    if !definition_digest_matches(root, record, &verification.contract_digest)? {
        return Err("accepted change verification contract is stale".into());
    }
    validate_verification_execution_digest(root, record, &verification)?;
    let expected_inputs = verification
        .acceptance_input_digest
        .as_ref()
        .ok_or_else(|| "accepted change is missing current delivery-input evidence".to_string())?;
    if let Some(manifest) = &verification.acceptance_manifest {
        validate_acceptance_manifest(manifest)?;
        if acceptance_manifest_digest(manifest)? != *expected_inputs {
            return Err(
                "accepted change manifest does not reproduce signed delivery inputs".into(),
            );
        }
        match (&record.supersedes[..], &verification.semantic_succession) {
            ([], None) => {}
            (_, Some(evidence)) => validate_semantic_succession(record, evidence)?,
            _ => {
                return Err(
                    "accepted change is missing semantic succession evidence for approved obligations"
                        .into(),
                );
            }
        }
    } else if verification.semantic_succession.is_some() {
        return Err("legacy acceptance cannot carry succession evidence without a manifest".into());
    }
    if record.workflow_version >= 2 {
        validate_finalization_evidence(root, record, &verification)?;
    } else {
        let expected = closing_digest(record, &verification);
        let ledger = load_approvals(root, record)?;
        let approval = latest_terminal_approval(&ledger)
            .ok_or_else(|| "accepted change is missing closing approval".to_string())?;
        if approval.digest != expected {
            return Err(
                "accepted change closing approval does not match verification evidence".into(),
            );
        }
    }
    let anchored = accepted_evidence_is_anchored(root, record, &verification);
    Ok((verification, anchored))
}

/// True when accepted evidence is still tied to history somebody can reach.
fn accepted_evidence_is_anchored(
    root: &Path,
    record: &ChangeRecord,
    evidence: &VerificationRecord,
) -> bool {
    verification_commit_is_accepted_current(root, evidence)
        || accepted_workspace_is_integrated(root, record)
        || accepted_change_is_recorded_on_remote_default(root, record)
}

fn authenticate_accepted_evidence(
    root: &Path,
    record: &ChangeRecord,
    pending: Option<&PendingArchiveClose<'_>>,
) -> Result<VerificationRecord, String> {
    match authenticate_accepted_evidence_with_anchor(root, record, pending)? {
        (verification, true) => Ok(verification),
        (_, false) => Err("accepted change verification commit is not in current history and canonical acceptance is not recorded on the remote default branch".into()),
    }
}

fn validate_archived_accepted_snapshot(
    root: &Path,
    archived: &ChangeRecord,
    pending: Option<&PendingArchiveClose<'_>>,
) -> Result<(), String> {
    let workspace = find_change_dir(root, &archived.id)?;
    let path = workspace.join("accepted-state.json");
    let (_, historical_bytes, historical) =
        authenticated_accepted_transition_for(root, archived, pending)?;
    let accepted = match fs::read(&path) {
        Ok(content) => {
            if content != historical_bytes {
                return Err(format!(
                    "archived change `{}` accepted-state snapshot does not match its trusted transition",
                    archived.id
                ));
            }
            serde_json::from_slice(&content).map_err(|error| {
                format!(
                    "invalid accepted-state snapshot {}: {error}",
                    path.display()
                )
            })?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let verification = load_verification(root, archived)?;
            if verification.acceptance_manifest.is_some() {
                return Err(format!(
                    "archived change `{}` is missing its authenticated accepted-state snapshot",
                    archived.id
                ));
            }
            historical
        }
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };
    if accepted.id != archived.id || accepted.state != ChangeState::Accepted {
        return Err(format!(
            "archived change `{}` has an invalid accepted-state snapshot identity",
            archived.id
        ));
    }
    let mut projection = archived.clone();
    projection.state = ChangeState::Accepted;
    projection.updated_at = accepted.updated_at;
    if projection != accepted {
        return Err(format!(
            "archived change `{}` does not match its authenticated accepted-state snapshot",
            archived.id
        ));
    }
    Ok(())
}

#[derive(Debug)]
struct LegacyArchiveBaselineContext {
    baseline: LegacyArchiveBaselineV1,
    snapshots: BTreeMap<String, BTreeMap<String, (u32, Vec<u8>)>>,
}

#[derive(Debug, Default)]
struct ArchivedIntegrityCache {
    legacy: Option<Result<LegacyArchiveBaselineContext, String>>,
}

fn validate_archived_integrity(root: &Path, archived: &ChangeRecord) -> Result<(), String> {
    validate_archived_integrity_with_cache(root, archived, &mut ArchivedIntegrityCache::default())
}

fn validate_archived_integrity_with_cache(
    root: &Path,
    archived: &ChangeRecord,
    cache: &mut ArchivedIntegrityCache,
) -> Result<(), String> {
    validate_archived_integrity_inner(root, archived, cache, None)
}

/// The archive preflight run by the command that is creating the package, which is the only caller
/// allowed to present closing evidence history has not seen yet. See `PendingArchiveClose`.
fn validate_archived_integrity_closing(
    root: &Path,
    archived: &ChangeRecord,
    pending: &PendingArchiveClose<'_>,
) -> Result<(), String> {
    validate_archived_integrity_inner(
        root,
        archived,
        &mut ArchivedIntegrityCache::default(),
        Some(pending),
    )
}

fn validate_archived_integrity_inner(
    root: &Path,
    archived: &ChangeRecord,
    cache: &mut ArchivedIntegrityCache,
    pending: Option<&PendingArchiveClose<'_>>,
) -> Result<(), String> {
    validate_acceptance_owner_correction_records(archived)?;
    let workspace = find_change_dir(root, &archived.id)?;
    if !workspace.join("accepted-state.json").exists() {
        return authenticate_legacy_archive_baseline(root, archived, &workspace, cache);
    }
    validate_archived_accepted_snapshot(root, archived, pending)?;
    let verification = load_verification(root, archived)?;
    if !verification.passed {
        return Err("archived change has failed verification evidence".into());
    }
    ensure_definition_approval_valid(root, archived)?;
    if !definition_digest_matches(root, archived, &verification.contract_digest)? {
        return Err("archived change verification contract is stale".into());
    }
    validate_verification_execution_digest(root, archived, &verification)?;
    let expected_inputs = verification
        .acceptance_input_digest
        .as_ref()
        .ok_or_else(|| "archived change is missing delivery-input evidence".to_string())?;
    if let Some(manifest) = &verification.acceptance_manifest {
        validate_acceptance_manifest(manifest)?;
        if acceptance_manifest_digest(manifest)? != *expected_inputs {
            return Err(
                "archived change manifest does not reproduce signed delivery inputs".into(),
            );
        }
        match (&archived.supersedes[..], &verification.semantic_succession) {
            ([], None) => {}
            (_, Some(evidence)) => validate_semantic_succession(archived, evidence)?,
            _ => {
                return Err(
                    "archived change is missing semantic succession evidence for approved obligations"
                        .into(),
                );
            }
        }
    } else {
        if verification.semantic_succession.is_some() {
            return Err(
                "legacy archive cannot carry succession evidence without a manifest".into(),
            );
        }
        reconstruct_legacy_acceptance_manifest(root, archived, expected_inputs)?;
    }
    if archived.workflow_version >= 2 {
        validate_finalization_evidence(root, archived, &verification)?;
    } else {
        let ledger = load_approvals(root, archived)?;
        let approval = latest_terminal_approval(&ledger)
            .ok_or_else(|| "archived change is missing closing approval".to_string())?;
        if approval.digest != closing_digest(archived, &verification) {
            return Err(
                "archived change closing approval does not match verification evidence".into(),
            );
        }
    }
    Ok(())
}

fn authenticate_legacy_archive_baseline(
    root: &Path,
    archived: &ChangeRecord,
    workspace: &Path,
    cache: &mut ArchivedIntegrityCache,
) -> Result<(), String> {
    let verification = load_verification(root, archived)?;
    if verification.acceptance_manifest.is_some() || verification.semantic_succession.is_some() {
        return Err(format!(
            "archived change `{}` is missing its authenticated accepted-state snapshot",
            archived.id
        ));
    }
    if !verification.passed {
        return Err("legacy archive has no passed verification evidence".into());
    }
    let ledger = load_approvals(root, archived)?;
    if !ledger
        .approvals
        .iter()
        .any(|approval| approval.gate == "definition")
    {
        return Err("legacy archive is missing stored definition approval".into());
    }
    ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "acceptance")
        .ok_or_else(|| "legacy archive is missing stored closing approval".to_string())?;

    if cache.legacy.is_none() {
        cache.legacy = Some(load_legacy_archive_baseline_context(root));
    }
    let context = cache
        .legacy
        .as_ref()
        .expect("legacy archive cache was initialized")
        .as_ref()
        .map_err(Clone::clone)?;
    let baseline = &context.baseline;
    let archive_root = root.join(ARCHIVE_PATH);
    let relative_workspace = workspace.strip_prefix(&archive_root).map_err(|_| {
        format!(
            "legacy archive workspace is outside {}",
            archive_root.display()
        )
    })?;
    let project_subtree = format!(
        "{}/{}",
        ARCHIVE_PATH,
        strict_portable_relative_path(
            relative_workspace
                .to_str()
                .ok_or_else(|| "legacy archive path is not UTF-8".to_string())?
        )?
    );
    let baseline_entry = legacy_baseline_entry(baseline, &archived.id)?;
    if baseline_entry.archive_path != project_subtree {
        return Err(format!(
            "legacy archive `{}` baseline path does not match its unique workspace",
            archived.id
        ));
    }
    let current = context.snapshots.get(&archived.id).ok_or_else(|| {
        format!(
            "legacy archive `{}` has no captured baseline subtree",
            archived.id
        )
    })?;
    if legacy_archive_subtree_digest(current)? != baseline_entry.subtree_digest {
        return Err(format!(
            "legacy archive `{}` subtree does not match its baseline digest",
            archived.id
        ));
    }
    Ok(())
}

fn load_legacy_archive_baseline_context(
    root: &Path,
) -> Result<LegacyArchiveBaselineContext, String> {
    let baseline_path = root.join(LEGACY_BASELINE_PATH);
    let baseline_bytes = fs::read(&baseline_path)
        .map_err(|error| format!("failed to read legacy archive baseline: {error}"))?;
    let (baseline, baseline_digest) = validate_legacy_archive_baseline_bytes(&baseline_bytes)?;

    let records = list_all_changes_checked(root)?;
    let authority = records.get(&baseline.authority_change_id).ok_or_else(|| {
        format!(
            "legacy archive baseline authority `{}` is unavailable",
            baseline.authority_change_id
        )
    })?;
    if authority.legacy_archive_baseline_digest.as_deref() != Some(&baseline_digest) {
        return Err(
            "legacy archive baseline bytes do not match its definition-bound authority".into(),
        );
    }
    ensure_definition_approval_valid(root, authority)
        .map_err(|error| format!("legacy archive baseline authority is not approved: {error}"))?;

    validate_legacy_baseline_authority_cutoff(root, authority, &baseline.cutoff_commit)?;
    let cutoff = baseline.cutoff_commit.clone();
    match authority.state {
        ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying => {}
        ChangeState::Accepted | ChangeState::Archived => {
            let authority_verification = authenticate_accepted_evidence(root, authority, None)
                .map_err(|error| {
                    format!("legacy archive baseline authority is not closing-valid: {error}")
                })?;
            let manifest = authority_verification
                .acceptance_manifest
                .as_ref()
                .ok_or_else(|| {
                    "accepted legacy archive baseline authority must be manifest-backed".to_string()
                })?;
            let ledger_entry = manifest
                .entries
                .iter()
                .find(|entry| entry.path == LEGACY_BASELINE_PATH)
                .ok_or_else(|| {
                    "accepted legacy archive baseline authority did not sign the ledger path"
                        .to_string()
                })?;
            if ledger_entry.kind != AcceptanceInputKind::File
                || !ledger_entry
                    .owners
                    .iter()
                    .any(|owner| owner == EXACT_DELIVERY_OWNER)
                || ledger_entry.payload_digest != sha256_hex(&baseline_bytes)
                || ledger_entry.entry_digest
                    != acceptance_entry_digest(
                        LEGACY_BASELINE_PATH,
                        &AcceptanceInputKind::File,
                        ledger_entry.mode,
                        &ledger_entry.payload_digest,
                    )
            {
                return Err(
                    "accepted legacy archive baseline authority has invalid exact ledger evidence"
                        .into(),
                );
            }
            let (anchor, _, _) = authenticated_accepted_transition(root, authority)?;
            ensure_git_ancestor(root, &cutoff, &anchor, "authority acceptance anchor")?;
        }
        ChangeState::Draft => {
            return Err("legacy archive baseline authority is still draft".into());
        }
    }

    let scopes = baseline
        .entries
        .iter()
        .map(|entry| entry.archive_path.clone())
        .collect();
    let (project_paths, evidence) =
        stable_discovered_evidence(root, Some(&scopes), &BTreeSet::new(), false, false)?;
    let snapshots = baseline
        .entries
        .iter()
        .map(|entry| {
            archive_snapshot_from_evidence(&project_paths, &evidence, &entry.archive_path)
                .map(|snapshot| (entry.id.clone(), snapshot))
        })
        .collect::<Result<_, _>>()?;
    validate_legacy_archive_introductions(root, &baseline, &snapshots)?;

    Ok(LegacyArchiveBaselineContext {
        baseline,
        snapshots,
    })
}

fn validate_legacy_archive_introductions(
    root: &Path,
    baseline: &LegacyArchiveBaselineV1,
    snapshots: &BTreeMap<String, BTreeMap<String, (u32, Vec<u8>)>>,
) -> Result<(), String> {
    let cutoff = baseline.cutoff_commit.as_str();
    let mut repo_subtrees = BTreeMap::new();
    let mut introductions = BTreeSet::new();
    let repo_prefix = git_repo_prefix(root)?;
    for entry in &baseline.entries {
        let repo_subtree = format!("{repo_prefix}{}", entry.archive_path);
        repo_subtrees.insert(entry.id.clone(), repo_subtree);
        introductions.insert(entry.introduction_commit.as_str());
    }
    let archive_root = format!("{repo_prefix}{ARCHIVE_PATH}");
    let history_pathspec = format!(":(top,literal){archive_root}");
    for introduction in introductions {
        let resolved = git_output(
            root,
            &[
                "rev-parse",
                "--verify",
                &format!("{introduction}^{{commit}}"),
            ],
        )
        .ok_or_else(|| format!("legacy archive introduction `{introduction}` is unavailable"))?;
        if resolved != introduction {
            return Err(format!(
                "legacy archive introduction `{introduction}` must be a canonical commit ID"
            ));
        }
        ensure_git_ancestor(root, introduction, cutoff, "legacy archive cutoff")?;
    }

    let max_count = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
    let output = run_git_bounded(
        root,
        &[
            "log",
            "--format=%H",
            "--diff-filter=A",
            &max_count,
            cutoff,
            "--",
            &history_pathspec,
        ],
        None,
        MAX_GIT_COMMAND_OUTPUT_BYTES,
    )
    .map_err(|error| format!("failed to inspect legacy archive introductions: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect legacy archive introductions at cutoff `{cutoff}`"
        ));
    }
    let commits: BTreeSet<_> = String::from_utf8(output.stdout)
        .map_err(|_| "legacy archive introduction history is not UTF-8".to_string())?
        .lines()
        .map(str::to_string)
        .collect();
    if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
        return Err(format!(
            "legacy archive introduction history exceeds the deterministic {}-commit bound",
            MAX_TRUSTED_HISTORY_COMMITS
        ));
    }
    let mut candidates: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut parents = BTreeMap::new();
    for commit in commits {
        let output = run_git_bounded(
            root,
            &[
                "diff-tree",
                "--root",
                "-m",
                "--no-commit-id",
                "--name-only",
                "-r",
                "-z",
                "--diff-filter=A",
                &commit,
                "--",
                &history_pathspec,
            ],
            None,
            MAX_GIT_COMMAND_OUTPUT_BYTES,
        )
        .map_err(|error| {
            format!("failed to inspect legacy archive introduction `{commit}`: {error}")
        })?;
        if !output.status.success() {
            return Err(format!(
                "failed to inspect legacy archive introduction `{commit}`"
            ));
        }
        let added = output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|raw| !raw.is_empty())
            .map(|raw| {
                std::str::from_utf8(raw)
                    .map(str::to_string)
                    .map_err(|_| "legacy archive introduction path is not UTF-8".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (id, subtree) in &repo_subtrees {
            let prefix = format!("{subtree}/");
            if added
                .iter()
                .any(|path| path == subtree || path.starts_with(&prefix))
            {
                candidates
                    .entry(id.clone())
                    .or_default()
                    .insert(commit.clone());
            }
        }
        let commit_parents: Vec<String> =
            git_output(root, &["rev-list", "--parents", "-n", "1", &commit])
                .unwrap_or_default()
                .split_whitespace()
                .skip(1)
                .map(str::to_string)
                .collect();
        parents.insert(commit, commit_parents);
    }

    for entry in &baseline.entries {
        let current = snapshots.get(&entry.id).ok_or_else(|| {
            format!(
                "legacy archive `{}` has no captured baseline subtree",
                entry.id
            )
        })?;
        let repo_subtree = repo_subtrees
            .get(&entry.id)
            .ok_or_else(|| format!("legacy archive `{}` has no repository subtree", entry.id))?;
        let mut anchors = BTreeSet::new();
        for commit in candidates.get(&entry.id).into_iter().flatten() {
            let parent_has_subtree = parents.get(commit).into_iter().flatten().any(|parent| {
                !git_tree_snapshot(root, parent, repo_subtree)
                    .unwrap_or_default()
                    .is_empty()
            });
            if parent_has_subtree {
                continue;
            }
            if git_tree_snapshot(root, commit, repo_subtree).is_ok_and(|tree| &tree == current) {
                anchors.insert(commit.clone());
            }
        }
        if anchors.len() != 1 || !anchors.contains(&entry.introduction_commit) {
            return Err(format!(
                "legacy archive `{}` requires its one baseline-bound pre-cutoff introduction anchor, found {}",
                entry.id,
                anchors.len()
            ));
        }
    }
    Ok(())
}

fn legacy_baseline_entry<'a>(
    baseline: &'a LegacyArchiveBaselineV1,
    archived_id: &str,
) -> Result<&'a LegacyArchiveBaselineEntryV1, String> {
    baseline
        .entries
        .iter()
        .find(|entry| entry.id == archived_id)
        .ok_or_else(|| format!("legacy archive `{archived_id}` is not enumerated by the baseline"))
}

fn validate_legacy_archive_baseline_bytes(
    baseline_bytes: &[u8],
) -> Result<(LegacyArchiveBaselineV1, String), String> {
    if baseline_bytes.len() as u64 > MAX_CHANGE_ARTIFACT_BYTES {
        return Err("legacy archive baseline exceeds the change artifact byte limit".into());
    }
    let baseline: LegacyArchiveBaselineV1 = serde_json::from_slice(baseline_bytes)
        .map_err(|error| format!("invalid legacy archive baseline: {error}"))?;
    if baseline.schema_version != 1 || baseline.domain != "specsync.legacy-archive-baseline.v1" {
        return Err("unsupported legacy archive baseline schema or domain".into());
    }
    if baseline.entries.len() > MAX_ACCEPTANCE_ENTRIES {
        return Err("legacy archive baseline exceeds the entry limit".into());
    }
    let canonical = json_content(&baseline)?;
    if !bytes_match_canonical_json(baseline_bytes, canonical.as_bytes()) {
        return Err("legacy archive baseline must use canonical persisted JSON bytes".into());
    }
    let mut previous: Option<(&str, &str)> = None;
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for entry in &baseline.entries {
        let key = (entry.id.as_str(), entry.archive_path.as_str());
        if previous.is_some_and(|prior| prior >= key) {
            return Err(
                "legacy archive baseline entries must be strictly sorted and unique".into(),
            );
        }
        previous = Some(key);
        if !ids.insert(entry.id.as_str()) || !paths.insert(entry.archive_path.as_str()) {
            return Err("legacy archive baseline IDs and archive paths must each be unique".into());
        }
        validate_sha256_digest(&entry.subtree_digest, "legacy archive subtree digest")?;
        let portable = strict_portable_relative_path(&entry.archive_path)?;
        if portable != entry.archive_path || !portable.starts_with(&format!("{ARCHIVE_PATH}/")) {
            return Err(format!(
                "legacy archive baseline path is not canonical: `{}`",
                entry.archive_path
            ));
        }
    }
    let mut digest = FramedDigest::new(LEGACY_BASELINE_DOMAIN);
    digest.frame(b"ledger", baseline_bytes);
    let baseline_digest = digest.finish();
    Ok((baseline, baseline_digest))
}

fn ensure_git_ancestor(
    root: &Path,
    ancestor: &str,
    descendant: &str,
    label: &str,
) -> Result<(), String> {
    let status = Command::new("git")
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .current_dir(root)
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to inspect {label} ancestry: {error}"))?;
    if !status.success() {
        return Err(format!(
            "`{ancestor}` is not an ancestor of {label} `{descendant}`"
        ));
    }
    Ok(())
}

fn legacy_archive_subtree_digest(
    snapshot: &BTreeMap<String, (u32, Vec<u8>)>,
) -> Result<String, String> {
    if snapshot.is_empty() {
        return Err("legacy archive baseline subtree must not be empty".into());
    }
    let mut digest = FramedDigest::new(LEGACY_SUBTREE_DOMAIN);
    digest.frame(b"schema-version", &1_u32.to_be_bytes());
    for (path, (mode, payload)) in snapshot {
        let kind: &[u8] = match mode {
            0o100644 | 0o100755 => b"file",
            0o120000 => {
                let target = std::str::from_utf8(payload)
                    .map_err(|_| format!("legacy archive symlink `{path}` is not UTF-8"))?;
                validate_portable_symlink_target(target)?;
                b"symlink"
            }
            _ => {
                return Err(format!(
                    "legacy archive subtree contains unsupported mode `{mode:o}` at `{path}`"
                ));
            }
        };
        digest.frame(b"entry", b"");
        digest.frame(b"path", path.as_bytes());
        digest.frame(b"kind", kind);
        digest.frame(b"mode", &mode.to_be_bytes());
        digest.frame(b"payload", payload);
    }
    Ok(digest.finish())
}

#[cfg(test)]
fn archive_workspace_snapshot(
    root: &Path,
    workspace: &Path,
    project_subtree: &str,
) -> Result<BTreeMap<String, (u32, Vec<u8>)>, String> {
    let mut workspace_paths = BTreeSet::new();
    for entry in walkdir::WalkDir::new(workspace).follow_links(false) {
        let entry = entry.map_err(|error| format!("failed to inspect legacy archive: {error}"))?;
        if entry.path() != workspace && !entry.file_type().is_dir() {
            workspace_paths.insert(format!(
                "{project_subtree}/{}",
                strict_portable_project_path(workspace, entry.path())?
            ));
        }
    }
    let scopes = BTreeSet::from([project_subtree.to_string()]);
    let (project_paths, evidence) =
        stable_discovered_evidence(root, Some(&scopes), &workspace_paths, false, false)?;
    archive_snapshot_from_evidence(&project_paths, &evidence, project_subtree)
}

fn archive_snapshot_from_evidence(
    project_paths: &[String],
    evidence: &GitEvidence,
    project_subtree: &str,
) -> Result<BTreeMap<String, (u32, Vec<u8>)>, String> {
    let prefix = format!("{project_subtree}/");
    let mut snapshot = BTreeMap::new();
    for project_path in project_paths {
        let Some(relative) = project_path.strip_prefix(&prefix) else {
            continue;
        };
        let entry = evidence.entry(project_path)?;
        match entry.kind {
            AcceptanceInputKind::File | AcceptanceInputKind::Symlink => {
                snapshot.insert(relative.to_string(), (entry.mode, entry.payload.clone()));
            }
            AcceptanceInputKind::Missing => {}
            AcceptanceInputKind::Gitlink | AcceptanceInputKind::NonFile => {
                return Err(format!(
                    "legacy archive contains unsupported non-file entry {project_path}"
                ));
            }
        }
    }
    Ok(snapshot)
}

fn git_tree_snapshot(
    root: &Path,
    commit: &str,
    repo_subtree: &str,
) -> Result<BTreeMap<String, (u32, Vec<u8>)>, String> {
    let pathspec = format!(":(top,literal){repo_subtree}");
    let output = Command::new("git")
        .args([
            "ls-tree",
            "-r",
            "-z",
            "--full-name",
            commit,
            "--",
            &pathspec,
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to inspect legacy archive tree: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect legacy archive tree at `{commit}`"
        ));
    }
    let prefix = format!("{repo_subtree}/");
    let mut entries = Vec::new();
    for raw in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|raw| !raw.is_empty())
    {
        let raw = std::str::from_utf8(raw)
            .map_err(|_| "legacy archive tree contains non-UTF-8 metadata".to_string())?;
        let (metadata, path) = raw
            .split_once('\t')
            .ok_or_else(|| format!("invalid legacy archive tree entry `{raw}`"))?;
        let mut fields = metadata.split_whitespace();
        let mode = fields
            .next()
            .ok_or_else(|| "legacy archive tree entry has no mode".to_string())?;
        let kind = fields
            .next()
            .ok_or_else(|| "legacy archive tree entry has no kind".to_string())?;
        let object = fields
            .next()
            .ok_or_else(|| "legacy archive tree entry has no object".to_string())?;
        let mode = u32::from_str_radix(mode, 8)
            .map_err(|_| "legacy archive tree entry has invalid mode".to_string())?;
        if kind != "blob" || !matches!(mode, 0o100644 | 0o100755 | 0o120000) {
            return Err(format!(
                "legacy archive tree contains unsupported `{kind}` mode `{mode:o}`"
            ));
        }
        let relative = path
            .strip_prefix(&prefix)
            .ok_or_else(|| format!("legacy archive tree path `{path}` escaped its subtree"))?;
        entries.push((
            strict_portable_relative_path(relative)?,
            mode,
            object.to_string(),
        ));
    }
    let objects: Vec<_> = entries
        .iter()
        .map(|(_, _, object)| object.as_str())
        .collect();
    let blobs = git_blob_bytes_batch(root, &objects)?;
    let mut snapshot = BTreeMap::new();
    for ((relative, mode, _object), bytes) in entries.into_iter().zip(blobs) {
        snapshot.insert(relative, (mode, bytes));
    }
    Ok(snapshot)
}

fn git_blob_bytes_batch(root: &Path, objects: &[&str]) -> Result<Vec<Vec<u8>>, String> {
    if objects.is_empty() {
        return Ok(Vec::new());
    }
    let mut input = Vec::new();
    for object in objects {
        if object.is_empty() || object.bytes().any(|byte| byte.is_ascii_whitespace()) {
            return Err(format!("invalid legacy archive blob object `{object}`"));
        }
        input.extend_from_slice(object.as_bytes());
        input.push(b'\n');
    }
    let output = run_git_required(
        root,
        &["cat-file", "--batch"],
        Some(input),
        MAX_GIT_EVIDENCE_PAYLOAD_BYTES + MAX_GIT_EVIDENCE_PATH_BYTES,
    )?;
    let mut cursor = 0;
    let mut blobs = Vec::with_capacity(objects.len());
    for expected in objects {
        let header_end = output[cursor..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|offset| cursor + offset)
            .ok_or_else(|| "legacy archive batch output has no header terminator".to_string())?;
        let header = std::str::from_utf8(&output[cursor..header_end])
            .map_err(|_| "legacy archive batch output has non-UTF-8 metadata".to_string())?;
        let mut fields = header.split_whitespace();
        let object = fields
            .next()
            .ok_or_else(|| "legacy archive batch output has no object".to_string())?;
        let kind = fields
            .next()
            .ok_or_else(|| "legacy archive batch output has no kind".to_string())?;
        let size = fields
            .next()
            .ok_or_else(|| "legacy archive batch output has no size".to_string())?
            .parse::<usize>()
            .map_err(|_| "legacy archive batch output has an invalid size".to_string())?;
        if fields.next().is_some() || object != *expected || kind != "blob" {
            return Err(format!(
                "unexpected legacy archive batch header `{header}` for `{expected}`"
            ));
        }
        let payload_start = header_end + 1;
        let payload_end = payload_start
            .checked_add(size)
            .ok_or_else(|| "legacy archive blob size overflowed".to_string())?;
        if payload_end >= output.len() || output[payload_end] != b'\n' {
            return Err(format!(
                "legacy archive batch output is truncated for blob `{expected}`"
            ));
        }
        blobs.push(output[payload_start..payload_end].to_vec());
        cursor = payload_end + 1;
    }
    if cursor != output.len() {
        return Err("legacy archive batch output has unexpected trailing bytes".into());
    }
    Ok(blobs)
}

/// `pending` is the closing token of the archive preflight, forwarded to every successor lookup
/// the walk makes; readers pass `None`. See `PendingArchiveClose`.
fn validate_accepted_inputs_recursive(
    root: &Path,
    record: &ChangeRecord,
    records: &BTreeMap<String, ChangeRecord>,
    visiting: &mut BTreeSet<String>,
    memo: &mut BTreeMap<String, Result<AcceptedInputValidity, String>>,
    pending: Option<&PendingArchiveClose<'_>>,
) -> Result<AcceptedInputValidity, String> {
    if let Some(result) = memo.get(&record.id) {
        return result.clone();
    }
    if !matches!(record.state, ChangeState::Accepted | ChangeState::Archived) {
        return Err(format!(
            "successor `{}` is not accepted or archived",
            record.id
        ));
    }
    if !visiting.insert(record.id.clone()) {
        return Err(format!(
            "semantic succession cycle detected at `{}`",
            record.id
        ));
    }
    let result = (|| {
        let verification = authenticate_accepted_evidence(root, record, pending)?;
        let expected_digest = verification
            .acceptance_input_digest
            .as_deref()
            .ok_or_else(|| {
                format!(
                    "accepted change is missing current delivery-input evidence; run `specsync change reopen {}` to record fresh acceptance evidence",
                    record.id
                )
            })?;
        let signed = if let Some(manifest) = &verification.acceptance_manifest {
            manifest.clone()
        } else {
            let legacy_current =
                acceptance_input_digest(root, &terminal_delivery_projection(record), &[])?;
            if legacy_current == expected_digest {
                return Ok(AcceptedInputValidity::Exact);
            }
            reconstruct_legacy_acceptance_manifest(root, record, expected_digest)?
        };
        let current = acceptance_manifest_with_signed_owners(
            root,
            &terminal_delivery_projection(record),
            &[],
            &signed,
        )?;
        let current_by_path: BTreeMap<_, _> = current
            .entries
            .iter()
            .map(|entry| (entry.path.as_str(), entry))
            .collect();
        let mut successor_covered = false;
        for expected in &signed.entries {
            let current = current_by_path.get(expected.path.as_str()).ok_or_else(|| {
                format!(
                    "delivery input `{}` disappeared from the current inventory; restore the file or run `specsync change reopen {}` to re-verify the accepted change",
                    expected.path, record.id
                )
            })?;
            if expected.kind == current.kind
                && expected.mode == current.mode
                && expected.payload_digest == current.payload_digest
                && expected.entry_digest == current.entry_digest
            {
                continue;
            }
            if expected
                .owners
                .iter()
                .any(|owner| owner.starts_with("@exact:"))
            {
                if expected.path == SEQUENCE_PATH
                    && later_sequence_owner_covers_historical_input(
                        root, record, records, visiting, memo, pending,
                    )?
                {
                    successor_covered = true;
                    continue;
                }
                // A reserved exact owner records that no module owned the input when it was
                // signed, so the entry names nobody to look for. The module that owns the path
                // now may have superseded it: its obligation names the module and its signed
                // manifest carries the input under that module, so the claimants are read from
                // the successors, and each is judged exactly as a signed owner's successor is.
                let claimants = succession_claimants(records, record, expected);
                if claimants.is_empty() {
                    return Err(exact_only_input_remediation_reason(record, &expected.path));
                }
                let mut rejected_successors = BTreeMap::new();
                if !claimants.iter().any(|module| {
                    successor_covers_input(
                        root,
                        record,
                        records,
                        visiting,
                        memo,
                        pending,
                        expected,
                        module,
                        &mut rejected_successors,
                    )
                }) {
                    return Err(stale_input_remediation_reason(
                        record,
                        &expected.path,
                        &expected.owners.join("`, `"),
                        &rejected_successors,
                    ));
                }
                successor_covered = true;
                continue;
            }
            for owner in &expected.owners {
                let mut rejected_successors = BTreeMap::new();
                if !successor_covers_input(
                    root,
                    record,
                    records,
                    visiting,
                    memo,
                    pending,
                    expected,
                    owner,
                    &mut rejected_successors,
                ) {
                    return Err(stale_input_remediation_reason(
                        record,
                        &expected.path,
                        owner,
                        &rejected_successors,
                    ));
                }
                successor_covered = true;
            }
        }
        Ok(if successor_covered {
            AcceptedInputValidity::SuccessorCovered
        } else {
            AcceptedInputValidity::Exact
        })
    })();
    visiting.remove(&record.id);
    memo.insert(record.id.clone(), result.clone());
    result
}
