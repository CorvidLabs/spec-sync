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

/// Where the branch's requirements are compared from, or `None` when there is nothing to compare
/// with: no default branch, and no `base_commit` in HEAD's history. `Err` when a default branch
/// exists and Git cannot give its merge-base with HEAD (fail closed, as REQ-change-142).
fn lost_requirements_reference(root: &Path, record: &ChangeRecord) -> Result<Option<String>, String> {
    if let Some(default) = remote_default_ref(root) {
        return git_output_allow_empty(root, &["merge-base", "HEAD", default.as_str()])
            .map(|text| text.trim().to_string())
            .filter(|commit| is_canonical_commit_id(commit))
            .map(Some)
            .ok_or_else(|| {
                format!(
                    "cannot tell whether this branch lost canonical requirements: Git cannot read the merge-base of HEAD and {default}"
                )
            });
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

/// The project-relative directories that hold requirements files: the specs directory and the
/// directory of every registered spec.
fn requirements_directories(root: &Path) -> Vec<String> {
    let specs_dir = crate::config::load_config(root).specs_dir;
    let mut directories = vec![specs_dir.trim_end_matches('/').to_string()];
    if let Ok(Some(registry)) = crate::registry::load_local_registry(root) {
        for (_, path) in &registry.specs {
            if let Some((directory, _)) = path.rsplit_once('/')
                && !directories.iter().any(|known| known == directory)
            {
                directories.push(directory.to_string());
            }
        }
    }
    directories
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

/// Every requirements file on disk under `directories`, project-relative, with its blocks.
fn working_tree_requirement_blocks(
    root: &Path,
    directories: &[String],
) -> Vec<(String, Vec<(String, String)>)> {
    fn walk(directory: &Path, depth: usize, found: &mut Vec<PathBuf>) {
        if depth > 16 {
            return;
        }
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if kind.is_dir() {
                walk(&path, depth + 1, found);
            } else if kind.is_file()
                && is_requirements_file(&entry.file_name().to_string_lossy())
            {
                found.push(path);
            }
        }
    }
    let mut files = Vec::new();
    for directory in directories {
        walk(&root.join(directory), 0, &mut files);
    }
    files.sort();
    files.dedup();
    files
        .into_iter()
        .filter_map(|path| {
            let text = fs::read_to_string(&path).ok()?;
            Some((portable_project_path(root, &path), requirement_blocks(&text)))
        })
        .collect()
}

/// The requirement ids approved `## REMOVED` items name, in the deltas of every change on the
/// branch: each active change whose definition approval binds its delta, and each change archived
/// on the branch after `reference`.
fn removed_requirement_ids_on_branch(root: &Path, reference: &str) -> Result<BTreeSet<String>, String> {
    let mut removed = BTreeSet::new();
    let mut collect = |delta: &str| {
        if let Ok(items) = parse_delta(delta) {
            for item in items {
                if item.operation == DeltaOperation::Removed
                    && item.target == DeltaTarget::Requirement
                {
                    removed.insert(item.key.clone());
                }
            }
        }
    };
    for change in list_changes_checked(root)? {
        if change.state == ChangeState::Archived || change.state == ChangeState::Draft {
            continue;
        }
        let Ok(ledger) = load_approvals(root, &change) else {
            continue;
        };
        let Ok(approval) = effective_definition_approval(root, &change, &ledger) else {
            continue;
        };
        for module in &change.affected_specs {
            let Ok(path) = delta_path_checked(root, &change, module) else {
                continue;
            };
            let Ok(delta) = read_bounded_change_text(&path, "semantic delta") else {
                continue;
            };
            let approved = approval
                .approved_delta_digests
                .as_ref()
                .is_none_or(|digests| digests.get(module) == Some(&delta_body_digest(module, &delta)));
            if approved {
                collect(&delta);
            }
        }
    }
    let archive = format!(":(literal){ARCHIVE_PATH}");
    let added = git_output_allow_empty(
        root,
        &[
            "diff",
            "--relative",
            "--name-only",
            "--no-renames",
            "--diff-filter=A",
            reference,
            "--",
            archive.as_str(),
        ],
    )
    .ok_or_else(|| "cannot read the changes this branch archived".to_string())?;
    for path in added.lines().map(str::trim).filter(|line| line.contains("/deltas/")) {
        if let Ok(delta) = fs::read_to_string(root.join(path)) {
            collect(&delta);
        }
    }
    Ok(removed)
}

/// REQ-change-169: refuse a change on a branch that lost a canonical requirement that no approved
/// `## REMOVED` delta names. `Ok(None)` when nothing is lost or every loss is accounted for;
/// `Ok(Some(note))` when there is nothing to compare with; `Err` with the refusal otherwise.
fn ensure_no_lost_requirements(root: &Path, record: &ChangeRecord) -> Result<Option<String>, String> {
    let Some(reference) = lost_requirements_reference(root, record)? else {
        return Ok(Some(
            "note: there is no default branch and no change base commit in HEAD's history, so lost canonical requirements were not checked (REQ-change-169)".to_string(),
        ));
    };
    let short = &reference[..reference.len().min(12)];
    let directories = requirements_directories(root);
    let pathspecs: Vec<String> = directories
        .iter()
        .map(|directory| format!(":(literal){directory}"))
        .collect();
    let mut args = vec![
        "diff",
        "--relative",
        "--name-only",
        "--no-renames",
        reference.as_str(),
        "--",
    ];
    args.extend(pathspecs.iter().map(String::as_str));
    let changed = git_output_allow_empty(root, &args).ok_or_else(|| {
        format!("cannot tell whether this branch lost canonical requirements: Git cannot compare it with {short}")
    })?;
    let changed: Vec<&str> = changed
        .lines()
        .map(str::trim)
        .filter(|path| is_requirements_file(path))
        .collect();
    if changed.is_empty() {
        return Ok(None);
    }
    let prefix = git_repo_prefix(root).map_err(|error| format!("cannot read Git: {error}"))?;
    // Every id at the reference in a file that differs now, with its file and block body.
    let mut at_reference: Vec<(String, String, String)> = Vec::new();
    for path in &changed {
        let literal = format!(":(literal){path}");
        let listed = git_output_allow_empty(
            root,
            &["ls-tree", "--name-only", "--end-of-options", reference.as_str(), "--", literal.as_str()],
        )
        .ok_or_else(|| format!("cannot read the tree of {short}"))?;
        if listed.trim().is_empty() {
            continue;
        }
        let object = format!("{reference}:{prefix}{path}");
        let text = git_output_allow_empty(root, &["show", object.as_str()])
            .ok_or_else(|| format!("cannot read {path} at {short}"))?;
        for (id, body) in requirement_blocks(&text) {
            at_reference.push((id, path.to_string(), body));
        }
    }
    let working = working_tree_requirement_blocks(root, &directories);
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
    unaccounted.sort();
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
    Err(format!(
        "this branch lost canonical requirements that no approved `## REMOVED` delta names, compared with {short}: {listed}. Restore each block from {short} ({restore} prints it), or remove it on purpose by naming it in a `## REMOVED` delta and approving the change again with `specsync change approve {id}`, then `specsync change check {id}`{renumber_hint}"
    ))
}

/// The refusal REQ-change-169 gives `record`, for `change ship-status` (REQ-cmd-change-042).
pub fn lost_requirements_refusal(root: &Path, record: &ChangeRecord) -> Option<String> {
    ensure_no_lost_requirements(root, record).err()
}
