// A change never deletes a canonical requirement that no approved `## REMOVED` delta names
// (#1081, REQ-change-169, CHANGE-17).
//
// In CorvidLabs/starling a setup script truncated `requirements.md` before appending a change's
// one new block, and every gate passed: an approval binds only the blocks its delta names, and the
// strict check compares specs with code, not canonical requirements with their history. Here the
// branch's requirement ids are compared with those at a reference commit (the merge-base of HEAD
// and the default branch, or the change's `base_commit` when there is no default branch), and an
// id that is gone from every requirements file and that no approved REMOVED item of a change on
// the branch names is refused, by name.

/// The default branch this check compares with ("the default branch (`origin/HEAD`, else
/// `origin/main` or `origin/master`)"): `origin/HEAD` only when it resolves to a commit and is not
/// the remote-tracking ref of the branch HEAD is on, else the first of `origin/main` and
/// `origin/master` that is usable the same way. An `origin/HEAD` left dangling, by a default
/// branch renamed and then pruned, names no default branch; nor does one a clone of a single
/// branch (`git clone --branch <b>`) points at `origin/<b>`, since comparing a branch with its own
/// pushed tip finds nothing lost (corvid-agent on #1108). It is local to this check, so the other
/// callers of `remote_default_ref` do not change. Its Git queries keep the argument shapes the
/// rest of SpecSync asks, which `git_utils` knows read nothing but facts: a `symbolic-ref` with
/// one more option counts as a possible write and drops every memoized repository fact.
fn lost_requirements_default_ref(root: &Path) -> Option<String> {
    let own = git_output(root, &["symbolic-ref", "--short", "HEAD"])
        .map(|branch| format!("origin/{branch}"));
    let usable = |candidate: &str| {
        let commit = format!("{candidate}^{{commit}}");
        own.as_deref() != Some(candidate)
            && git_output(root, &["rev-parse", "--verify", commit.as_str()]).is_some()
    };
    git_output(root, &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])
        .filter(|candidate| usable(candidate))
        .or_else(|| {
            ["origin/main", "origin/master"]
                .into_iter()
                .find(|candidate| usable(candidate))
                .map(str::to_string)
        })
}

/// Where the branch's requirements are compared from, or `None` when there is nothing to compare
/// with: no default branch, and no `base_commit` in HEAD's history. `Err` when a default branch
/// exists and Git cannot give its merge-base with HEAD (fail closed, as REQ-change-142).
fn lost_requirements_reference(root: &Path, record: &ChangeRecord) -> Result<Option<String>, String> {
    if let Some(default) = lost_requirements_default_ref(root) {
        return git_output_allow_empty(root, &["merge-base", "HEAD", default.as_str()])
            .map(|text| text.trim().to_string())
            .filter(|commit| is_canonical_commit_id(commit))
            .map(Some)
            .ok_or_else(|| format!("Git cannot read the merge-base of HEAD and {default}"));
    }
    Ok(record
        .base_commit
        .as_deref()
        .filter(|base| is_canonical_commit_id(base))
        .filter(|base| commit_is_ancestor(root, base, "HEAD") == Some(true))
        .map(str::to_string))
}

/// Whether `path` (project-relative) is a requirements file: `requirements.md`, or a legacy
/// `<module>.req.md`, as `hash_cache.rs` and the merge driver already read them.
fn is_requirements_file(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    name == "requirements.md" || name.ends_with(".req.md")
}

/// The names in a NUL-separated Git listing (`-z`), which Git never quotes; a plain listing
/// C-quotes a non-ASCII path under the default `core.quotePath` (Codex on #1108).
fn nul_separated(listing: &str) -> impl Iterator<Item = &str> {
    listing.split('\0').filter(|name| !name.is_empty())
}

/// Read bounded Git output without altering its path or blob bytes.
fn git_raw_utf8(root: &Path, args: &[&str]) -> Option<String> {
    let output = run_git_bounded(root, args, None, MAX_GIT_COMMAND_OUTPUT_BYTES).ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

/// Git terminates its prefix with a newline. A space can be part of the directory name.
fn requirement_repo_prefix(root: &Path) -> Result<String, String> {
    let mut prefix = git_raw_utf8(root, &["rev-parse", "--show-prefix"])
        .ok_or_else(|| "unable to determine project path within Git repository".to_string())?;
    if prefix.ends_with('\n') {
        prefix.pop();
    }
    Ok(prefix)
}

/// Where canonical requirements files live ("under the specs directory or beside a registered
/// spec"). `directories`, the specs directories, are read to any depth. `companions`, the
/// directories of the registered specs, are read without descending, each for its own
/// `requirements.md` and `<module>.req.md`; the project root is `""` (Codex on #1108, both
/// rounds; ss-lead's guard). A companion inside a specs directory adds nothing.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct RequirementScope {
    directories: Vec<String>,
    companions: Vec<String>,
}

impl RequirementScope {
    /// Adds the specs directory `directory`. One that leaves the project adds nothing, so Git is
    /// never handed a pathspec outside the repository; `specsync check` names that configuration
    /// itself (the tester's run on #1108 at 4ced5a26).
    fn add_directory(&mut self, directory: &str) {
        let directory = directory.trim_end_matches('/');
        if !matches!(directory, "" | ".") && normalize_project_path(directory).is_err() {
            return;
        }
        if !self.directories.iter().any(|known| known == directory) {
            self.directories.push(directory.to_string());
        }
    }

    /// Adds the directory of each spec `registry` maps. A path that leaves the project adds
    /// nothing, as `canonical_module_paths` refuses it (Codex's second round). The working tree's
    /// registry is judged on disk (`safe_project_path`); one read at a reference is judged as the
    /// path it was there, not by today's tree (Codex's fourth round).
    fn add_registry(&mut self, root: &Path, registry: &crate::types::RegistryEntry, on_disk: bool) {
        for (_, path) in &registry.specs {
            let Ok(path) = normalize_project_path(path) else {
                continue;
            };
            if on_disk && safe_project_path(root, &path).is_err() {
                continue;
            }
            let directory = path
                .rsplit_once('/')
                .map_or("", |(directory, _)| directory)
                .trim_end_matches('/');
            if !self.companions.iter().any(|known| known == directory) {
                self.companions.push(directory.to_string());
            }
        }
    }

    /// The companion directories that no specs directory already covers.
    fn own_companions(&self) -> impl Iterator<Item = &str> {
        self.companions.iter().map(String::as_str).filter(|companion| {
            !self.directories.iter().any(|directory| {
                directory == "."
                    || *companion == directory.as_str()
                    || companion.starts_with(&format!("{directory}/"))
            })
        })
    }

    /// The Git pathspecs naming the same files, relative to the project root.
    fn pathspecs(&self) -> Vec<String> {
        let mut pathspecs: Vec<String> = self
            .directories
            .iter()
            .map(|directory| format!(":(literal){directory}"))
            .collect();
        for companion in self.own_companions() {
            let (literal, glob) = if companion.is_empty() {
                (String::new(), String::new())
            } else {
                (format!("{companion}/"), format!("{}/", glob_escaped(companion)))
            };
            pathspecs.push(format!(":(literal){literal}requirements.md"));
            pathspecs.push(format!(":(glob){glob}*.req.md"));
        }
        pathspecs
    }
}

/// `text` with every character a `:(glob)` pathspec reads as a wildcard escaped.
fn glob_escaped(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        if matches!(character, '*' | '?' | '[' | ']' | '\\') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

/// The scope the working tree's configuration and registry give.
fn requirement_scope(root: &Path) -> RequirementScope {
    let mut scope = RequirementScope::default();
    scope.add_directory(&crate::config::load_config(root).specs_dir);
    if let Ok(Some(registry)) = crate::registry::load_local_registry(root) {
        scope.add_registry(root, &registry, true);
    }
    scope
}

/// The registry files `load_local_registry` reads, in the order it tries them.
const REGISTRY_PATH_CANDIDATES: [&str; 2] = [".specsync/registry.toml", "specsync-registry.toml"];

/// Adds to `scope` what was canonical at `reference` ("in such a file at the reference commit";
/// Codex on #1108): the specs directory its configuration named and the specs its registry held,
/// read through Git in the order the loaders read them on disk. Returns the directory names the
/// reference excluded, by which a file canonical then is judged, so a directory the branch newly
/// excludes still counts there (Codex's second round). With no configuration file there, the
/// defaults apply; one that does not parse adds no specs directory and excludes only Git's and
/// SpecSync's directories. A registry that is not there or does not parse adds nothing, as on
/// disk. `Err` when Git cannot read the tree or an object.
fn add_reference_requirement_scope(
    root: &Path,
    reference: &str,
    prefix: &str,
    scope: &mut RequirementScope,
) -> Result<BTreeSet<String>, String> {
    let short = &reference[..reference.len().min(12)];
    let candidates: Vec<String> = crate::config::CONFIG_PATH_CANDIDATES
        .iter()
        .chain(REGISTRY_PATH_CANDIDATES.iter())
        .map(|path| format!(":(literal){path}"))
        .collect();
    let mut args = vec!["ls-tree", "-z", "--name-only", "--end-of-options", reference, "--"];
    args.extend(candidates.iter().map(String::as_str));
    let listed = git_raw_utf8(root, &args)
        .ok_or_else(|| format!("cannot read the tree of {short}"))?;
    let present: BTreeSet<&str> = nul_separated(&listed).collect();
    let read = |path: &str| {
        let object = format!("{reference}:{prefix}{path}");
        git_raw_utf8(root, &["show", object.as_str()])
            .ok_or_else(|| format!("cannot read {path} at {short}"))
    };
    let mut excluded: BTreeSet<String> = ALWAYS_EXCLUDED_DIRECTORIES.map(String::from).into();
    let config = match crate::config::CONFIG_PATH_CANDIDATES
        .iter()
        .find(|path| present.contains(**path))
    {
        Some(&path) => crate::config::parse_config_content_checked_with_source_dirs(
            Path::new(path),
            &read(path)?,
            root,
            Some(Vec::new()),
        )
        .ok(),
        None => Some(crate::types::SpecSyncConfig::default()),
    };
    if let Some(config) = config {
        scope.add_directory(&config.specs_dir);
        excluded.extend(config.exclude_dirs);
    }
    if let Some(&path) = REGISTRY_PATH_CANDIDATES
        .iter()
        .find(|path| present.contains(**path))
    {
        let content = read(path)?;
        if let Some(registry) = reference_registry(path, &content)? {
            scope.add_registry(root, &registry, false);
        }
    }
    Ok(excluded)
}

/// The registry `content`, read at a reference commit from `path`, parsed by the loader the
/// working tree's registry goes through (`load_local_registry`), from a private copy at the same
/// relative path; `None` when it is inert or does not parse, as on disk.
fn reference_registry(
    path: &str,
    content: &str,
) -> Result<Option<crate::types::RegistryEntry>, String> {
    let unreadable = |error: std::io::Error| format!("cannot copy {path} to read it: {error}");
    let copy = tempfile::tempdir().map_err(unreadable)?;
    // The private copy is its own root, so the write stays confined to it (REQ-util-002).
    let file = copy.path().join(path);
    crate::util::write_project_file(copy.path(), &file, content.as_bytes()).map_err(unreadable)?;
    Ok(crate::registry::load_local_registry(copy.path()).ok().flatten())
}

/// How two requirement ids sort for a person: runs of digits compare as numbers, so
/// `REQ-a-2` comes before `REQ-a-10` (ss-lead's review of #1081). Equal numbers with different
/// zero padding fall back to the text, so the order is total.
fn natural_order(left: &str, right: &str) -> std::cmp::Ordering {
    fn runs(text: &str) -> Vec<(bool, &str)> {
        let mut runs = Vec::new();
        let mut start = 0;
        let mut digits = None;
        for (index, character) in text.char_indices() {
            let digit = character.is_ascii_digit();
            if digits.is_some_and(|previous| previous != digit) {
                runs.push((digits.unwrap_or(false), &text[start..index]));
                start = index;
            }
            digits = Some(digit);
        }
        if start < text.len() {
            runs.push((digits.unwrap_or(false), &text[start..]));
        }
        runs
    }
    let (left_runs, right_runs) = (runs(left), runs(right));
    for (a, b) in left_runs.iter().zip(&right_runs) {
        let order = match (a, b) {
            ((true, a), (true, b)) => {
                let (a, b) = (a.trim_start_matches('0'), b.trim_start_matches('0'));
                a.len().cmp(&b.len()).then_with(|| a.cmp(b))
            }
            ((_, a), (_, b)) => a.cmp(b),
        };
        if order != std::cmp::Ordering::Equal {
            return order;
        }
    }
    left_runs
        .len()
        .cmp(&right_runs.len())
        .then_with(|| left.cmp(right))
}

/// The `### REQ-` heading ids of `text`, outside fenced blocks, each with its block body
/// (everything after the heading line up to the next heading, trimmed).
fn requirement_blocks(text: &str) -> Vec<(String, String)> {
    let mut blocks: Vec<(String, String)> = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut current: Option<(String, Vec<&str>)> = None;
    for line in text.lines() {
        if let Some(mark) = fence_mark(line) {
            match fence {
                None => fence = Some((mark.marker, mark.length)),
                Some((marker, length)) if mark.marker == marker && mark.length >= length && mark.bare => {
                    fence = None
                }
                _ => {}
            }
            if let Some((_, body)) = current.as_mut() {
                body.push(line);
            }
            continue;
        }
        if fence.is_none() && line.trim_start().starts_with('#') {
            if let Some((id, body)) = current.take() {
                blocks.push((id, body.join("\n").trim().to_string()));
            }
            if let Some(id) = crate::validator::requirement_heading_id(line.trim_end()) {
                current = Some((id.to_string(), Vec::new()));
            }
            continue;
        }
        if let Some((_, body)) = current.as_mut() {
            body.push(line);
        }
    }
    if let Some((id, body)) = current.take() {
        blocks.push((id, body.join("\n").trim().to_string()));
    }
    blocks
}

/// Git's directory and SpecSync's, which no configuration can make canonical.
const ALWAYS_EXCLUDED_DIRECTORIES: [&str; 2] = [".git", ".specsync"];

/// The directory names the check never reads requirements from, wherever they sit: Git's own
/// directory, SpecSync's (whose change workspaces and archives hold `requirements.md` artifacts,
/// not canonical requirements), and the project's `exclude_dirs` (ss-lead's review of #1081).
fn excluded_requirement_directories(root: &Path) -> BTreeSet<String> {
    let mut excluded: BTreeSet<String> = ALWAYS_EXCLUDED_DIRECTORIES.map(String::from).into();
    excluded.extend(crate::config::load_config(root).exclude_dirs);
    excluded
}

/// Whether the project-relative `path` lies under none of the `excluded` directory names.
fn is_scanned_requirements_path(path: &str, excluded: &BTreeSet<String>) -> bool {
    let mut components: Vec<&str> = path.split('/').collect();
    components.pop();
    components.iter().all(|component| !excluded.contains(*component))
}

/// Whether the scope directory `directory` is read: a directory that leads outside the project, or
/// is itself a link, is not read (Codex on #1108, third round). The project root always is.
fn readable_scope_directory(root: &Path, directory: &str) -> bool {
    if directory.is_empty() || directory == "." {
        return true;
    }
    safe_project_path(root, directory).is_ok()
        && fs::symlink_metadata(root.join(directory))
            .is_ok_and(|metadata| !metadata.file_type().is_symlink())
}

/// Every requirements file on disk in `scope`, project-relative, with its blocks. Its directories
/// are read to any depth, as Git lists them (Codex on #1108); symlinks are not followed, and no
/// directory named in `excluded` is entered.
fn working_tree_requirement_blocks(
    root: &Path,
    scope: &RequirementScope,
    excluded: &BTreeSet<String>,
) -> Vec<(String, Vec<(String, String)>)> {
    let mut files = Vec::new();
    let mut pending: Vec<PathBuf> = scope
        .directories
        .iter()
        .filter(|directory| readable_scope_directory(root, directory))
        .map(|directory| root.join(directory))
        .collect();
    while let Some(directory) = pending.pop() {
        let Ok(entries) = crate::util::read_project_dir_entries(root, &directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if kind.is_dir() {
                if !excluded.contains(name.as_ref()) {
                    pending.push(entry.path());
                }
            } else if kind.is_file() && is_requirements_file(&name) {
                files.push(entry.path());
            }
        }
    }
    for companion in scope
        .own_companions()
        .filter(|companion| readable_scope_directory(root, companion))
    {
        let Ok(entries) = crate::util::read_project_dir_entries(root, &root.join(companion)) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.file_type().is_ok_and(|kind| kind.is_file())
                && is_requirements_file(&entry.file_name().to_string_lossy())
            {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    files.dedup();
    files
        .into_iter()
        .filter_map(|path| {
            let relative = portable_project_path(root, &path);
            if !is_scanned_requirements_path(&relative, excluded) {
                return None;
            }
            let text =
                crate::util::read_project_string(root, &path, crate::util::MAX_SPEC_FILE_BYTES).ok()?;
            Some((relative, requirement_blocks(&text)))
        })
        .collect()
}

/// A lifecycle commit owns its selected workspace. A sibling authorizes a removal only
/// when Git's index already contains the definition and delta that authorize it.
fn sibling_removal_is_staged(
    root: &Path,
    change: &ChangeRecord,
    approval: &ApprovalRecord,
    delta: &Path,
) -> Result<bool, String> {
    let package = strict_portable_project_path(root, &find_change_dir(root, &change.id)?)?;
    let corrections = format!("{package}/{CORRECTIONS_FILE}");
    let mut paths = BTreeSet::from([
        format!("{package}/state.json"),
        format!("{package}/approvals.json"),
        strict_portable_project_path(root, delta)?,
        corrections.clone(),
    ]);
    // Legacy definition approvals bind the selected artifacts as well as the scope.
    if change.workflow_version < 2 || approval.approved_scope.is_none() {
        let ledger = load_correction_ledger(root, change)?;
        let definition = definition_artifact_snapshot(root, change, &ledger.corrections)?;
        // Approval digests keep logical change paths when a package moves to the archive.
        let original = format!("{CHANGES_PATH}/{}/", change.id);
        for (path, _, _, _) in definition.entries {
            let indexed = match path.strip_prefix(&original) {
                Some(relative) => format!("{package}/{relative}"),
                None => path,
            };
            paths.insert(indexed);
        }
    }
    let staged = inspect_git_candidates(root, &paths, true, true)?;
    Ok(staged.entries.iter().all(|(path, entry)| {
        entry.object_identity
            || (path == &corrections
                && entry.kind == AcceptanceInputKind::Missing
                && entry.object.is_none())
    }))
}

/// Adds to `removed` the requirement ids the approved `## REMOVED` items of `change` name: the
/// items of each delta its valid definition approval binds by digest. An approval that records
/// no delta digests, as one made before they existed, binds no wording, so its change adds
/// nothing until it is approved again (Codex on #1108, third round). A change whose approval or
/// delta cannot be read adds nothing either.
fn add_approved_removals(
    root: &Path,
    change: &ChangeRecord,
    is_sibling: bool,
    removed: &mut BTreeSet<String>,
) {
    let Ok(ledger) = load_approvals(root, change) else {
        return;
    };
    let Ok(approval) = effective_definition_approval(root, change, &ledger) else {
        return;
    };
    for module in &change.affected_specs {
        let Ok(path) = delta_path_checked(root, change, module) else {
            continue;
        };
        let Ok(delta) = read_bounded_change_text(root, &path, "semantic delta") else {
            continue;
        };
        let approved = approval
            .approved_delta_digests
            .as_ref()
            .is_some_and(|digests| digests.get(module) == Some(&delta_body_digest(module, &delta)));
        if !approved {
            continue;
        }
        let removals: Vec<_> = parse_delta(&delta)
            .unwrap_or_default()
            .into_iter()
            .filter(|item| {
                item.operation == DeltaOperation::Removed && item.target == DeltaTarget::Requirement
            })
            .map(|item| item.key)
            .collect();
        if removals.is_empty() {
            continue;
        }
        if is_sibling && sibling_removal_is_staged(root, change, approval, &path) != Ok(true) {
            continue;
        }
        removed.extend(removals);
    }
}

/// The requirement ids approved `## REMOVED` items name, in the deltas of every change on the
/// branch that may account for a loss: each active change past its draft, and each change the
/// branch archived after `reference`. Only the selected workspace can supply untracked removal
/// authority. A retired change accounts for nothing, in the active roster or in the archive, and neither does a delta file under the
/// archive path with no archived record around it (Codex on #1108; one mechanism for both).
fn removed_requirement_ids_on_branch(
    root: &Path,
    reference: &str,
    selected_id: Option<&str>,
) -> Result<BTreeSet<String>, String> {
    let mut removed = BTreeSet::new();
    // A change that cannot be read accounts for no removal, and never turns this refusal into its
    // own error (ss-lead's review of #1081, #1104's pattern); so is every change when the changes
    // directory itself cannot be read. Either way the lost ids are still named.
    let readable = list_changes(root).map(|roster| roster.records).unwrap_or_default();
    for change in readable.iter().filter(|change| {
        matches!(
            change.state,
            ChangeState::Approved
                | ChangeState::Implementing
                | ChangeState::Verifying
                | ChangeState::Accepted
        )
    }) {
        add_approved_removals(root, change, Some(change.id.as_str()) != selected_id, &mut removed);
    }
    let archive = format!(":(literal){ARCHIVE_PATH}");
    let added = git_raw_utf8(
        root,
        &[
            "diff",
            "--relative",
            "--name-only",
            "-z",
            "--no-renames",
            "--diff-filter=A",
            reference,
            "--",
            archive.as_str(),
        ],
    )
    .ok_or_else(|| "cannot read the changes this branch archived".to_string())?;
    // `change finalize` moves a change into the archive before its tip is committed, and Git's diff
    // does not list a file it does not track yet (corvid-agent on #1108).
    let untracked = git_raw_utf8(
        root,
        &["ls-files", "-z", "--others", "--exclude-standard", "--", archive.as_str()],
    )
    .ok_or_else(|| "cannot read the changes this branch archived".to_string())?;
    for path in nul_separated(&added).chain(nul_separated(&untracked)) {
        // The archived record of a change: `<archive>/<package>/state.json`, one level down.
        let Some(package) = path
            .strip_prefix(ARCHIVE_PATH)
            .and_then(|rest| rest.strip_prefix('/'))
            .and_then(|rest| rest.strip_suffix("/state.json"))
            .filter(|package| !package.is_empty() && !package.contains('/'))
        else {
            continue;
        };
        let directory = root.join(ARCHIVE_PATH).join(package);
        let Ok(change) = crate::util::read_project_bytes(
            root,
            &directory.join("state.json"),
            crate::util::MAX_STATE_FILE_BYTES,
        )
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                serde_json::from_slice::<ChangeRecord>(&bytes).map_err(|error| error.to_string())
            })
        else {
            continue;
        };
        // Only an archived change, and only the package SpecSync resolves its id to, so that its
        // approval and deltas are read from this record.
        let resolved = find_change_dir(root, &change.id)
            .ok()
            .and_then(|found| fs::canonicalize(found).ok());
        let Ok(here) = fs::canonicalize(&directory) else {
            continue;
        };
        if change.state != ChangeState::Archived || resolved.as_ref() != Some(&here) {
            continue;
        }
        // Archived after the reference: a package the reference already held, renamed since,
        // authorizes nothing new (Codex on #1108, fourth round).
        if archived_at(root, reference, &change.id)? {
            continue;
        }
        add_approved_removals(root, &change, Some(change.id.as_str()) != selected_id, &mut removed);
    }
    Ok(removed)
}

/// Whether `commit`'s tree already holds an archive package for change `id`: a directory named as
/// `change finalize` names it (`<date>-<id>`, or `<id>`) whose `state.json` there names `id`.
fn archived_at(root: &Path, commit: &str, id: &str) -> Result<bool, String> {
    let archive = format!(":(literal){ARCHIVE_PATH}/");
    let listed = git_raw_utf8(
        root,
        &["ls-tree", "-z", "--name-only", "--end-of-options", commit, "--", archive.as_str()],
    )
    .ok_or_else(|| "cannot read the changes this branch archived".to_string())?;
    let prefix = requirement_repo_prefix(root).map_err(|error| format!("cannot read Git: {error}"))?;
    let suffix = format!("-{id}");
    for package in nul_separated(&listed) {
        let name = package.rsplit('/').next().unwrap_or(package);
        if name != id && !name.ends_with(&suffix) {
            continue;
        }
        let object = format!("{commit}:{prefix}{package}/state.json");
        let named = git_raw_utf8(root, &["show", object.as_str()])
            .and_then(|text| serde_json::from_str::<ChangeRecord>(&text).ok())
            .is_some_and(|record| record.id == id);
        if named {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The way out of a refusal that could not read Git (Leif's rule 3): repair the repository or
/// fetch the default branch, unshallowing a shallow clone, then check again.
fn lost_requirements_way_out(root: &Path, record: &ChangeRecord) -> String {
    let shallow = git_output_allow_empty(root, &["rev-parse", "--is-shallow-repository"])
        .is_some_and(|text| text.trim() == "true");
    let fetch = if shallow {
        "`git fetch --unshallow origin`"
    } else {
        "`git fetch origin`"
    };
    format!(
        "repair the repository or fetch the default branch ({fetch}), then run `specsync change check {}` again",
        shell_word(&record.id)
    )
}

/// REQ-change-169: refuse a change on a branch that lost a canonical requirement that no approved
/// `## REMOVED` delta names. `Ok(None)` when nothing is lost or every loss is accounted for;
/// `Ok(Some(note))` when there is nothing to compare with; `Err` with the refusal otherwise.
fn ensure_no_lost_requirements(root: &Path, record: &ChangeRecord) -> Result<Option<String>, String> {
    lost_requirements_check(root, record).map_err(|refusal| match refusal {
        LostRequirementsRefusal::Lost(message) | LostRequirementsRefusal::Misconfigured(message) => {
            message
        }
        LostRequirementsRefusal::Unreadable(what) => format!(
            "cannot tell whether this branch lost canonical requirements: {what}; {}",
            lost_requirements_way_out(root, record)
        ),
    })
}

/// Mutating lifecycle callers hold the project lock when recording a refusal.
fn ensure_no_lost_requirements_recording_refusal(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Option<String>, String> {
    let result = ensure_no_lost_requirements(root, record);
    if let Err(error) = &result {
        record_refused_attempt(root, record, error).map_err(|write_error| {
            format!("{error}\ncould not record refused attempt ({write_error})")
        })?;
    }
    result
}

/// Why `lost_requirements_check` refuses: requirements were lost, the configuration names a
/// specs directory outside the project, or Git could not be read.
enum LostRequirementsRefusal {
    Lost(String),
    Misconfigured(String),
    Unreadable(String),
}

impl From<String> for LostRequirementsRefusal {
    fn from(what: String) -> Self {
        Self::Unreadable(what)
    }
}

fn lost_requirements_check(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Option<String>, LostRequirementsRefusal> {
    // A specs directory outside the project is a configuration to fix, not a repository Git
    // cannot read: say so in the words `specsync check` uses, and send Git no such pathspec (the
    // tester's run on #1108 at 4ced5a26).
    let config = crate::config::load_config(root);
    let specs_dir = config.specs_dir.trim_end_matches('/');
    if !matches!(specs_dir, "" | ".") && normalize_project_path(specs_dir).is_err() {
        let file = config
            .config_path
            .as_deref()
            .map_or_else(|| ".specsync/config.toml".to_string(), |path| portable_project_path(root, path));
        return Err(LostRequirementsRefusal::Misconfigured(format!(
            "Coverage specs directory must remain beneath the project root: {}; set `specs_dir` in {file} to a directory inside the project, then run `specsync change check {}` again",
            config.specs_dir,
            shell_word(&record.id)
        )));
    }
    let Some(reference) = lost_requirements_reference(root, record)? else {
        return Ok(Some(
            "note: there is no default branch and no change base commit in HEAD's history, so lost canonical requirements were not checked (REQ-change-169)".to_string(),
        ));
    };
    let short = &reference[..reference.len().min(12)];
    let prefix = requirement_repo_prefix(root).map_err(|error| format!("cannot read Git: {error}"))?;
    // What is canonical now, and what was canonical at the reference (Codex on #1108).
    let mut scope = requirement_scope(root);
    let excluded_at_reference =
        add_reference_requirement_scope(root, &reference, &prefix, &mut scope)?;
    let pathspecs = scope.pathspecs();
    let mut args = vec![
        "diff",
        "--relative",
        "--name-only",
        "-z",
        "--no-renames",
        reference.as_str(),
        "--",
    ];
    args.extend(pathspecs.iter().map(String::as_str));
    let changed = git_raw_utf8(root, &args)
        .ok_or_else(|| format!("Git cannot compare it with {short}"))?;
    // The files that differ, as the reference judged them: its blocks are what can be lost.
    let changed: Vec<&str> = nul_separated(&changed)
        .filter(|path| {
            is_requirements_file(path) && is_scanned_requirements_path(path, &excluded_at_reference)
        })
        .collect();
    if changed.is_empty() {
        return Ok(None);
    }
    // Every id at the reference in a file that differs now, with its file and block body.
    let mut at_reference: Vec<(String, String, String)> = Vec::new();
    for path in &changed {
        let literal = format!(":(literal){path}");
        let listed = git_raw_utf8(
            root,
            &["ls-tree", "-z", "--name-only", "--end-of-options", reference.as_str(), "--", literal.as_str()],
        )
        .ok_or_else(|| format!("cannot read the tree of {short}"))?;
        if nul_separated(&listed).next().is_none() {
            continue;
        }
        let object = format!("{reference}:{prefix}{path}");
        let text = git_raw_utf8(root, &["show", object.as_str()])
            .ok_or_else(|| format!("cannot read {path} at {short}"))?;
        for (id, body) in requirement_blocks(&text) {
            at_reference.push((id, path.to_string(), body));
        }
    }
    // A file canonical now or at the reference counts as present, so a directory is skipped only
    // when both exclude it: one the branch newly excludes keeps the ids it still holds (Codex on
    // #1108, third round).
    let excluded_both: BTreeSet<String> = excluded_requirement_directories(root)
        .intersection(&excluded_at_reference)
        .cloned()
        .collect();
    let working = working_tree_requirement_blocks(root, &scope, &excluded_both);
    // Only the tracked ones count ("in none of the tracked ones in the working tree"): `change
    // check --commit` commits no file Git does not track, so a copy there keeps nothing (Codex on
    // #1108, fourth round; Leif's 'continue with 95% confidence').
    let mut args = vec!["ls-files", "-z", "--cached", "--"];
    args.extend(pathspecs.iter().map(String::as_str));
    let tracked_listing = git_raw_utf8(root, &args)
        .ok_or_else(|| "Git cannot list the files it tracks".to_string())?;
    let tracked: BTreeSet<&str> = nul_separated(&tracked_listing).collect();
    let present: BTreeSet<&str> = working
        .iter()
        .filter(|(path, _)| tracked.contains(path.as_str()))
        .flat_map(|(_, blocks)| blocks.iter().map(|(id, _)| id.as_str()))
        .collect();
    let lost: Vec<&(String, String, String)> = at_reference
        .iter()
        .filter(|(id, _, _)| !present.contains(id.as_str()))
        .collect();
    if lost.is_empty() {
        return Ok(None);
    }
    let removed = removed_requirement_ids_on_branch(root, &reference, Some(&record.id))?;
    let mut unaccounted: Vec<&(String, String, String)> = lost
        .into_iter()
        .filter(|(id, _, _)| !removed.contains(id))
        .collect();
    if unaccounted.is_empty() {
        return Ok(None);
    }
    unaccounted.sort_by(|a, b| natural_order(&a.0, &b.0).then_with(|| a.1.cmp(&b.1)));
    unaccounted.dedup_by(|a, b| a.0 == b.0);
    let mut by_file: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (id, path, _) in &unaccounted {
        by_file.entry(path.as_str()).or_default().push(id.as_str());
    }
    let listed = by_file
        .iter()
        .map(|(path, ids)| format!("{} ({path})", ids.join(", ")))
        .collect::<Vec<_>>()
        .join("; ");
    let restore = by_file
        .keys()
        .map(|path| format!("`git show {}`", shell_word(&format!("{short}:{prefix}{path}"))))
        .collect::<Vec<_>>()
        .join(", ");
    let renumbered = unaccounted
        .iter()
        .filter(|(_, _, body)| !body.is_empty())
        .filter_map(|(id, _, body)| {
            working
                .iter()
                .flat_map(|(_, blocks)| blocks.iter())
                .find(|(other, text)| other != id && text == body)
                .map(|(other, _)| format!("{id} reads as renumbered to {other}"))
        })
        .collect::<Vec<_>>();
    // A lost id kept only in a file Git does not track: the way out is to track it.
    let untracked = unaccounted
        .iter()
        .filter_map(|(id, _, _)| {
            working
                .iter()
                .filter(|(path, _)| !tracked.contains(path.as_str()))
                .find(|(_, blocks)| blocks.iter().any(|(other, _)| other == id))
                .map(|(path, _)| {
                    format!(
                        "{id} is in {path}, which Git does not track: `git add {}`",
                        shell_word(path)
                    )
                })
        })
        .collect::<Vec<_>>();
    let untracked_hint = if untracked.is_empty() {
        String::new()
    } else {
        format!(
            ". {}, then run `specsync change check {}` again",
            untracked.join("; "),
            shell_word(&record.id)
        )
    };
    let renumber_hint = if renumbered.is_empty() {
        String::new()
    } else {
        format!(
            ". {}; a requirement id is permanent, so a renumbered requirement needs a `## REMOVED` item for its old id",
            renumbered.join("; ")
        )
    };
    let id = shell_word(&record.id);
    // A lost id the default branch's tip no longer holds is a removal the default branch made,
    // brought in some other way than a merge (a cherry-pick, or the same edit made by hand). The
    // way out is to merge it, not to restore the block or approve a REMOVED delta (Leif's rule 3;
    // the tester's L2b). When the tip cannot be read, the general way out stands.
    let restore_or_remove = format!(
        "restore each block from {short} ({restore} prints it), or remove it on purpose by naming it in a `## REMOVED` delta and approving the change again with `specsync change approve {id}`"
    );
    let default_remediation = default_branch_requirement_ids(root, &scope).and_then(|(default, held)| {
        let (main_removed, others): (Vec<&str>, Vec<&str>) = unaccounted
            .iter()
            .map(|(lost, _, _)| lost.as_str())
            .partition(|lost| !held.contains(*lost));
        if main_removed.is_empty() {
            return None;
        }
        let merge_ref = shell_word(&default);
        let merge = format!(
            "{default} has removed {} too, so merge {default} into this branch (`git merge {merge_ref}`)",
            main_removed.join(", ")
        );
        let followup = if others.is_empty() {
            ": a removal merged from the default branch is not this branch's loss".to_string()
        } else {
            format!(". For {}, {restore_or_remove}", others.join(", "))
        };
        Some(format!("{merge}{followup}. Then run `specsync change check {id}`"))
    });
    let way_out = default_remediation.unwrap_or_else(|| {
        let mut way = restore_or_remove;
        if let Some(first) = way.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        format!("{way}, then `specsync change check {id}`")
    });
    Err(LostRequirementsRefusal::Lost(format!(
        "this branch lost canonical requirements that no approved `## REMOVED` delta names, compared with {short}: {listed}. {way_out}{untracked_hint}{renumber_hint}"
    )))
}

/// The default branch and the requirement ids its tip holds in the requirements files of `scope`
/// (outside the excluded directories); `None` when there is no default branch or Git cannot read
/// its tip.
fn default_branch_requirement_ids(
    root: &Path,
    scope: &RequirementScope,
) -> Option<(String, BTreeSet<String>)> {
    let default = lost_requirements_default_ref(root)?;
    let prefix = requirement_repo_prefix(root).ok()?;
    // The default branch's ids are read as it is configured: an id in a directory the branch newly
    // excludes is still the default branch's, so the way out never names a merge that changes
    // nothing (the tester's excl2 on #1108).
    let mut scope = scope.clone();
    let excluded = add_reference_requirement_scope(root, &default, &prefix, &mut scope).ok()?;
    let scope = &scope;
    let pathspecs: Vec<String> = scope
        .directories
        .iter()
        .map(|directory| format!(":(literal){directory}"))
        .collect();
    let mut args = vec!["ls-tree", "-r", "-z", "--name-only", "--end-of-options", default.as_str(), "--"];
    args.extend(pathspecs.iter().map(String::as_str));
    let mut listed = git_raw_utf8(root, &args)?;
    // `ls-tree` takes no glob, so a companion's own files come from its directory's listing.
    for companion in scope.own_companions() {
        let directory = format!(":(literal){companion}/");
        let mut args = vec!["ls-tree", "-z", "--name-only", "--end-of-options", default.as_str()];
        if !companion.is_empty() {
            args.extend(["--", directory.as_str()]);
        }
        listed.push('\0');
        listed.push_str(&git_raw_utf8(root, &args)?);
    }
    let mut held = BTreeSet::new();
    for path in nul_separated(&listed)
        .filter(|path| is_requirements_file(path) && is_scanned_requirements_path(path, &excluded))
    {
        let object = format!("{default}:{prefix}{path}");
        let text = git_raw_utf8(root, &["show", object.as_str()])?;
        held.extend(requirement_blocks(&text).into_iter().map(|(id, _)| id));
    }
    Some((default, held))
}

/// The refusal REQ-change-169 gives `record`, for `change ship-status` (REQ-cmd-change-042).
pub fn lost_requirements_refusal(root: &Path, record: &ChangeRecord) -> Option<String> {
    ensure_no_lost_requirements(root, record).err()
}

/// Real `ship` reports a preflight blocker before it calls finalize. Recheck that refusal under
/// the project lock so a completed or restored change cannot acquire a stale attempt.
pub(crate) fn record_ship_lost_requirements_refusal(
    root: &Path,
    id: &str,
    refusal: &str,
) -> Result<(), String> {
    let _lock = acquire_project_lock(root)?;
    crate::git_utils::invalidate_repository_facts();
    let record = load_change(root, id)?;
    if !matches!(record.state, ChangeState::Archived | ChangeState::Retired)
        && change_dir(root, id).is_dir()
        && lost_requirements_refusal(root, &record).as_deref() == Some(refusal)
    {
        record_refused_attempt(root, &record, refusal)?;
    }
    Ok(())
}
