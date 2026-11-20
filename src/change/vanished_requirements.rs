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
/// `origin/main` or `origin/master`)"): `origin/HEAD` only when it resolves to a commit, else the
/// first of `origin/main` and `origin/master` that does. An `origin/HEAD` left dangling, by a
/// default branch renamed and then pruned, names no default branch (corvid-agent on #1108). It is
/// local to this check, so the other callers of `remote_default_ref` do not change.
fn lost_requirements_default_ref(root: &Path) -> Option<String> {
    let resolves = |candidate: &str| {
        let commit = format!("{candidate}^{{commit}}");
        git_output(root, &["rev-parse", "--verify", "--quiet", commit.as_str()]).is_some()
    };
    git_output(root, &["symbolic-ref", "--short", "-q", "refs/remotes/origin/HEAD"])
        .filter(|candidate| resolves(candidate))
        .or_else(|| {
            ["origin/main", "origin/master"]
                .into_iter()
                .find(|candidate| resolves(candidate))
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

/// Where canonical requirements files live ("under the specs directory or beside a registered
/// spec"): `directories`, read to any depth, are the specs directory and the directory of every
/// registered spec; `root_files` is set when a spec is registered at the project root, whose
/// requirements are the root directory's own `requirements.md` and `<module>.req.md`, read
/// without descending into the rest of the tree (Codex on #1108; ss-lead's guard).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct RequirementScope {
    directories: Vec<String>,
    root_files: bool,
}

impl RequirementScope {
    fn add_directory(&mut self, directory: &str) {
        let directory = directory.trim_end_matches('/');
        if !self.directories.iter().any(|known| known == directory) {
            self.directories.push(directory.to_string());
        }
    }

    fn add_registry(&mut self, registry: &crate::types::RegistryEntry) {
        for (_, path) in &registry.specs {
            match path.rsplit_once('/') {
                Some((directory, _)) => self.add_directory(directory),
                None => self.root_files = true,
            }
        }
    }

    /// The Git pathspecs naming the same files, relative to the project root.
    fn pathspecs(&self) -> Vec<String> {
        let mut pathspecs: Vec<String> = self
            .directories
            .iter()
            .map(|directory| format!(":(literal){directory}"))
            .collect();
        if self.root_files {
            pathspecs.push(":(literal)requirements.md".to_string());
            pathspecs.push(":(glob)*.req.md".to_string());
        }
        pathspecs
    }
}

/// The scope the working tree's configuration and registry give.
fn requirement_scope(root: &Path) -> RequirementScope {
    let mut scope = RequirementScope::default();
    scope.add_directory(&crate::config::load_config(root).specs_dir);
    if let Ok(Some(registry)) = crate::registry::load_local_registry(root) {
        scope.add_registry(&registry);
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
    let listed = git_output_allow_empty(root, &args)
        .ok_or_else(|| format!("cannot read the tree of {short}"))?;
    let present: BTreeSet<&str> = nul_separated(&listed).collect();
    let read = |path: &str| {
        let object = format!("{reference}:{prefix}{path}");
        git_output_allow_empty(root, &["show", object.as_str()])
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
            scope.add_registry(&registry);
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
    let file = copy.path().join(path);
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent).map_err(unreadable)?;
    }
    fs::write(&file, content).map_err(unreadable)?;
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

/// Every requirements file on disk in `scope`, project-relative, with its blocks. Its directories
/// are read to any depth, as Git lists them (Codex on #1108); symlinks are not followed, and no
/// directory named in `excluded_requirement_directories` is entered.
fn working_tree_requirement_blocks(
    root: &Path,
    scope: &RequirementScope,
) -> Vec<(String, Vec<(String, String)>)> {
    let excluded = excluded_requirement_directories(root);
    let mut files = Vec::new();
    let mut pending: Vec<PathBuf> = scope
        .directories
        .iter()
        .map(|directory| root.join(directory))
        .collect();
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(&directory) else {
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
    if scope.root_files
        && let Ok(entries) = fs::read_dir(root)
    {
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
            if !is_scanned_requirements_path(&relative, &excluded) {
                return None;
            }
            let text = fs::read_to_string(&path).ok()?;
            Some((relative, requirement_blocks(&text)))
        })
        .collect()
}

/// Adds to `removed` the requirement ids the approved `## REMOVED` items of `change` name: the
/// items of each delta its valid definition approval binds. A change whose approval or delta
/// cannot be read adds nothing.
fn add_approved_removals(root: &Path, change: &ChangeRecord, removed: &mut BTreeSet<String>) {
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
        let Ok(delta) = read_bounded_change_text(&path, "semantic delta") else {
            continue;
        };
        let approved = approval
            .approved_delta_digests
            .as_ref()
            .is_none_or(|digests| digests.get(module) == Some(&delta_body_digest(module, &delta)));
        if !approved {
            continue;
        }
        for item in parse_delta(&delta).unwrap_or_default() {
            if item.operation == DeltaOperation::Removed && item.target == DeltaTarget::Requirement {
                removed.insert(item.key.clone());
            }
        }
    }
}

/// The requirement ids approved `## REMOVED` items name, in the deltas of every change on the
/// branch that may account for a loss: each active change past its draft, and each change the
/// branch archived after `reference`, committed or not yet, taken from its archived record. A retired change accounts
/// for nothing, in the active roster or in the archive, and neither does a delta file under the
/// archive path with no archived record around it (Codex on #1108; one mechanism for both).
fn removed_requirement_ids_on_branch(root: &Path, reference: &str) -> Result<BTreeSet<String>, String> {
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
        add_approved_removals(root, change, &mut removed);
    }
    let archive = format!(":(literal){ARCHIVE_PATH}");
    let added = git_output_allow_empty(
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
    let untracked = git_output_allow_empty(
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
        let Ok(change) = fs::read(directory.join("state.json"))
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
        add_approved_removals(root, &change, &mut removed);
    }
    Ok(removed)
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
        LostRequirementsRefusal::Lost(message) => message,
        LostRequirementsRefusal::Unreadable(what) => format!(
            "cannot tell whether this branch lost canonical requirements: {what}; {}",
            lost_requirements_way_out(root, record)
        ),
    })
}

/// Why `lost_requirements_check` refuses: requirements were lost, or Git could not be read.
enum LostRequirementsRefusal {
    Lost(String),
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
    let Some(reference) = lost_requirements_reference(root, record)? else {
        return Ok(Some(
            "note: there is no default branch and no change base commit in HEAD's history, so lost canonical requirements were not checked (REQ-change-169)".to_string(),
        ));
    };
    let short = &reference[..reference.len().min(12)];
    let prefix = git_repo_prefix(root).map_err(|error| format!("cannot read Git: {error}"))?;
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
    let changed = git_output_allow_empty(root, &args)
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
        let listed = git_output_allow_empty(
            root,
            &["ls-tree", "-z", "--name-only", "--end-of-options", reference.as_str(), "--", literal.as_str()],
        )
        .ok_or_else(|| format!("cannot read the tree of {short}"))?;
        if nul_separated(&listed).next().is_none() {
            continue;
        }
        let object = format!("{reference}:{prefix}{path}");
        let text = git_output_allow_empty(root, &["show", object.as_str()])
            .ok_or_else(|| format!("cannot read {path} at {short}"))?;
        for (id, body) in requirement_blocks(&text) {
            at_reference.push((id, path.to_string(), body));
        }
    }
    let working = working_tree_requirement_blocks(root, &scope);
    let present: BTreeSet<&str> = working
        .iter()
        .flat_map(|(_, blocks)| blocks.iter().map(|(id, _)| id.as_str()))
        .collect();
    let lost: Vec<&(String, String, String)> = at_reference
        .iter()
        .filter(|(id, _, _)| !present.contains(id.as_str()))
        .collect();
    if lost.is_empty() {
        return Ok(None);
    }
    let removed = removed_requirement_ids_on_branch(root, &reference)?;
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
        .map(|path| format!("`git show {short}:{prefix}{path}`"))
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
    let (main_removed, others): (Vec<&str>, Vec<&str>) = match default_branch_requirement_ids(root, &scope) {
        Some((_, held)) => unaccounted
            .iter()
            .map(|(lost, _, _)| lost.as_str())
            .partition(|lost| !held.contains(*lost)),
        None => (Vec::new(), unaccounted.iter().map(|(lost, _, _)| lost.as_str()).collect()),
    };
    let restore_or_remove = format!(
        "restore each block from {short} ({restore} prints it), or remove it on purpose by naming it in a `## REMOVED` delta and approving the change again with `specsync change approve {id}`"
    );
    let way_out = match (main_removed.is_empty(), lost_requirements_default_ref(root)) {
        (false, Some(default)) if others.is_empty() => format!(
            "{default} has removed {} too, so merge {default} into this branch (`git merge {default}`): a removal merged from the default branch is not this branch's loss. Then run `specsync change check {id}`",
            main_removed.join(", ")
        ),
        (false, Some(default)) => format!(
            "{default} has removed {} too, so merge {default} into this branch (`git merge {default}`). For {}, {restore_or_remove}. Then run `specsync change check {id}`",
            main_removed.join(", "),
            others.join(", ")
        ),
        _ => {
            let mut way = restore_or_remove;
            if let Some(first) = way.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            format!("{way}, then `specsync change check {id}`")
        }
    };
    Err(LostRequirementsRefusal::Lost(format!(
        "this branch lost canonical requirements that no approved `## REMOVED` delta names, compared with {short}: {listed}. {way_out}{renumber_hint}"
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
    let prefix = git_repo_prefix(root).ok()?;
    let excluded = excluded_requirement_directories(root);
    let pathspecs: Vec<String> = scope
        .directories
        .iter()
        .map(|directory| format!(":(literal){directory}"))
        .collect();
    let mut args = vec!["ls-tree", "-r", "-z", "--name-only", "--end-of-options", default.as_str(), "--"];
    args.extend(pathspecs.iter().map(String::as_str));
    let mut listed = git_output_allow_empty(root, &args)?;
    // `ls-tree` takes no glob, so the root's own files come from its top-level listing.
    if scope.root_files {
        listed.push('\0');
        listed.push_str(&git_output_allow_empty(
            root,
            &["ls-tree", "-z", "--name-only", "--end-of-options", default.as_str()],
        )?);
    }
    let mut held = BTreeSet::new();
    for path in nul_separated(&listed)
        .filter(|path| is_requirements_file(path) && is_scanned_requirements_path(path, &excluded))
    {
        let object = format!("{default}:{prefix}{path}");
        let text = git_output_allow_empty(root, &["show", object.as_str()])?;
        held.extend(requirement_blocks(&text).into_iter().map(|(id, _)| id));
    }
    Some((default, held))
}

/// The refusal REQ-change-169 gives `record`, for `change ship-status` (REQ-cmd-change-042).
pub fn lost_requirements_refusal(root: &Path, record: &ChangeRecord) -> Option<String> {
    ensure_no_lost_requirements(root, record).err()
}
