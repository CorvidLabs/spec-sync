// ─── `change close` (REQ-change-159, REQ-cmd-change-037, #1070) ───

/// Where `change close` puts a package: under `.specsync/archive/`, which every SpecSync 6.x
/// treats as a non-input, and outside `.specsync/archive/changes/`, the only archive directory
/// 6.x enumerates, as a late close does (REQ-change-143). A closed change is therefore never part
/// of the archive preflight, the successor walk, the sequence accounting or path coverage, and
/// SpecSync 6.0 to 6.3 never read it.
const CLOSED_PATH: &str = ".specsync/archive/closed";
const CLOSE_RECORD: &str = "close.json";

const CLOSE_KIND: PackageCloseKind = PackageCloseKind {
    folder: CLOSED_PATH,
    record_file: CLOSE_RECORD,
    destination_name: "close destination",
    done: "closed",
    record_name: "close record",
};

/// What `change close` records beside the package it moved (REQ-change-159).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloseRecord {
    pub schema_version: u32,
    pub change_id: String,
    pub actor: String,
    pub reason: String,
    pub timestamp: u64,
    /// The state the change's own `state.json` still holds, byte for byte.
    pub from_state: ChangeState,
    pub workflow_version: u32,
    /// The project path the package was moved from.
    pub from_path: String,
    /// The change that replaces this one, when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<String>,
    /// Always false: a closed change was never reviewed, accepted or archived.
    pub reviewed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relayed_by: Option<RelayedDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dropped_collision_acknowledgement: Option<LateCloseCollision>,
}

impl PackageCloseRecord for CloseRecord {
    fn change_id(&self) -> &str {
        &self.change_id
    }

    fn set_dropped_collision(&mut self, collision: LateCloseCollision) {
        self.dropped_collision_acknowledgement = Some(collision);
    }
}

/// What a person asks `change close` to do.
#[derive(Debug, Clone)]
pub struct CloseRequest {
    pub ids: Vec<String>,
    pub actor: String,
    pub reason: String,
    pub replaced_by: Option<String>,
    pub relayed_by: Option<RelayedDecision>,
}

/// One change `change close` closed: its record and where its package now is.
#[derive(Debug, Clone, Serialize)]
pub struct ClosedChange {
    pub record: CloseRecord,
    pub package: String,
}

/// The closed package for `id`, if there is one: `YYYY-MM-DD-<id>` under the closed folder.
fn closed_package(root: &Path, id: &str) -> Option<PathBuf> {
    fs::read_dir(root.join(CLOSED_PATH))
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| is_closed_late_name_for(name, id))
        })
}

/// Whether `id` is held only by a closed package: no active workspace and no archive package
/// records it, so status reports the close record only then.
pub fn is_only_closed(root: &Path, id: &str) -> bool {
    !change_dir(root, id).is_dir()
        && archived_package_for(root, id).is_none()
        && closed_package(root, id).is_some()
}

/// The closed package for `id` and its record, read from `close.json`.
pub fn closed_record(root: &Path, id: &str) -> Option<(PathBuf, Result<CloseRecord, String>)> {
    let package = closed_package(root, id)?;
    let path = package.join(CLOSE_RECORD);
    let record = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))
        .and_then(|content| {
            serde_json::from_str(&content)
                .map_err(|error| format!("invalid {}: {error}", path.display()))
        });
    Some((package, record))
}

/// The one sentence every verb gives for a closed change: who closed it, when and why, what
/// replaces it, and that it was never reviewed.
pub fn closed_sentence(root: &Path, id: &str) -> Option<String> {
    let (package, record) = closed_record(root, id)?;
    let location = portable_project_path(root, &package);
    Some(match record {
        Ok(record) => format!(
            "change `{id}` was closed on {} by {} ({}){}; it was never reviewed{}; its package is `{location}`",
            utc_date(record.timestamp),
            printable_record_text(&record.actor),
            printable_record_text(&record.reason),
            record
                .replaced_by
                .as_deref()
                .map(|other| format!("; `{}` replaces it", printable_record_text(other)))
                .unwrap_or_default(),
            if record.from_state == ChangeState::Draft {
                " and never approved"
            } else {
                ""
            },
        ),
        Err(error) => format!(
            "change `{id}` was closed, but its record cannot be read ({}); it was never reviewed; its package is `{location}`",
            printable_record_text(&error)
        ),
    })
}

/// Check one change for closing and plan it; nothing is written.
fn plan_close(
    root: &Path,
    id: &str,
    request: &CloseRequest,
    date: &str,
) -> Result<PackageClosePlan<CloseRecord>, String> {
    validate_change_id(id)?;
    if let Some(sentence) = closed_sentence(root, id) {
        return Err(sentence);
    }
    let source = change_dir(root, id);
    if !source.is_dir() {
        if archived_package_for(root, id).is_some() {
            return Err(format!(
                "change `{id}` is archived; an archived change is reopened, retired or kept, not closed"
            ));
        }
        return Err(closed_late_sentence(root, id)
            .unwrap_or_else(|| format!("change `{id}` was not found")));
    }
    if let Some(archived) = archived_package_for(root, id) {
        return Err(format!(
            "change `{id}` is ambiguous: both `{}` and `{}` hold it; decide which package is the change before closing it",
            portable_project_path(root, &source),
            portable_project_path(root, &archived)
        ));
    }
    let destination = root.join(CLOSED_PATH).join(format!("{date}-{id}"));
    validate_package_close_destination(root, &destination, &CLOSE_KIND)?;
    let state_path = source.join("state.json");
    let record: ChangeRecord = fs::read(&state_path)
        .map_err(|error| format!("failed to read {}: {error}", state_path.display()))
        .and_then(|content| {
            serde_json::from_slice(&content)
                .map_err(|error| format!("invalid change state {}: {error}", state_path.display()))
        })?;
    if record.id != id {
        return Err(format!(
            "{} names change `{}`, not `{id}`",
            state_path.display(),
            record.id
        ));
    }
    // A record this binary cannot validate is not moved: its lifecycle format is unknown here.
    validate_loaded_change(&record, id, &state_path)
        .map_err(|error| format!("change `{id}` cannot be closed: {error}"))?;
    if !matches!(
        record.state,
        ChangeState::Draft
            | ChangeState::Approved
            | ChangeState::Implementing
            | ChangeState::Verifying
    ) {
        return Err(format!(
            "change `{id}` is {}; only a draft, approved, implementing or verifying change is closed. An accepted change is finalized, reopened or retired",
            record.state.as_str()
        ));
    }
    if let Some(replacement) = &request.replaced_by {
        if replacement == id {
            return Err(format!("change `{id}` cannot be replaced by itself"));
        }
        find_change_dir(root, replacement)
            .map_err(|error| format!("--replaced-by `{replacement}`: {error}"))?;
    }
    let existing_record = source.join(CLOSE_RECORD);
    if fs::symlink_metadata(&existing_record).is_ok() {
        return Err(format!(
            "change `{id}` cannot be closed: its package already holds `{}`",
            portable_project_path(root, &existing_record)
        ));
    }
    if record.canonical_applied {
        ensure_materialized_text_merged(root, &record)?;
    }
    Ok(PackageClosePlan {
        source: source.clone(),
        destination,
        record: CloseRecord {
            schema_version: 1,
            change_id: id.to_string(),
            actor: request.actor.clone(),
            reason: request.reason.clone(),
            timestamp: now(),
            from_state: record.state,
            workflow_version: record.workflow_version,
            from_path: portable_project_path(root, &source),
            replaced_by: request.replaced_by.clone(),
            reviewed: false,
            relayed_by: request.relayed_by.clone(),
            dropped_collision_acknowledgement: None,
        },
    })
}

/// A change that materialized its deltas leaves spec text no change owns once it closes, unless
/// that text already merged. Every canonical file of its affected modules must therefore match the
/// remote default branch, committed and in the working tree; otherwise the close is refused with
/// the command that restores them (#1058).
fn ensure_materialized_text_merged(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let id = &record.id;
    let specs_dir = crate::config::load_config(root).specs_dir;
    let mut files = Vec::new();
    for module in &record.affected_specs {
        let (spec, requirements) = canonical_module_paths(root, &specs_dir, module)?;
        for path in [spec, requirements] {
            files.push(portable_project_path(root, &path));
        }
    }
    let Some(default_ref) = remote_default_ref(root) else {
        return Err(format!(
            "change `{id}` materialized spec text, and no remote default branch (`origin/HEAD`, `origin/main` or `origin/master`) resolves, so SpecSync cannot tell whether that text merged; fetch the remote, or restore {} from the commit before the change, then close it",
            files.join(", ")
        ));
    };
    let mut differing = Vec::new();
    for file in &files {
        let repo_path = git_repo_relative_path(root, file)?;
        let pathspec = format!(":(top,literal){repo_path}");
        let same = git_output_allow_empty(root, &["diff", "--quiet", &default_ref, "--", &pathspec])
            .is_some()
            && git_output_allow_empty(
                root,
                &["ls-files", "--others", "--exclude-standard", "--", &pathspec],
            )
            .is_some_and(|untracked| untracked.trim().is_empty());
        if !same {
            differing.push(file.clone());
        }
    }
    if differing.is_empty() {
        return Ok(());
    }
    // A file the default branch holds is restored from it; one it does not hold was created by
    // the change and is removed.
    let (restore, remove): (Vec<String>, Vec<String>) = differing
        .iter()
        .cloned()
        .partition(|file| package_is_on_reference(root, file, &default_ref));
    let mut steps = Vec::new();
    if !restore.is_empty() {
        steps.push(format!(
            "restore {} with `git checkout {default_ref} -- {}`",
            restore.join(", "),
            restore.join(" ")
        ));
    }
    if !remove.is_empty() {
        steps.push(format!(
            "remove {}, which `{default_ref}` does not have",
            remove.join(", ")
        ));
    }
    Err(format!(
        "change `{id}` materialized spec text that has not merged: {} differ from `{default_ref}`. Closing it would leave that text with no change owning it; {}, then close it",
        differing.join(", "),
        steps.join(", and ")
    ))
}

/// Close every change in `request.ids`, or none: each active package moves byte for byte into
/// `.specsync/archive/closed/<date>-<id>/` with a `close.json` record, and nothing is reviewed,
/// accepted or written into a spec (REQ-change-159).
pub fn close_changes(root: &Path, request: &CloseRequest) -> Result<Vec<ClosedChange>, String> {
    if request.ids.is_empty() {
        return Err("change close needs at least one change ID".into());
    }
    let actor = normalize_actor(&request.actor, "change close")?;
    let reason = request.reason.trim();
    if reason.is_empty() {
        return Err("change close requires a non-empty reason passed with --reason".into());
    }
    let request = CloseRequest {
        actor,
        reason: reason.to_string(),
        ..request.clone()
    };
    let mut seen = BTreeSet::new();
    for id in &request.ids {
        if !seen.insert(id.as_str()) {
            return Err(format!("change `{id}` is named twice"));
        }
        if request.replaced_by.as_deref() == Some(id.as_str()) {
            return Err(format!(
                "change `{id}` cannot be closed and named as the replacement in one command"
            ));
        }
    }
    let date = today();
    let mut plans = request
        .ids
        .iter()
        .map(|id| plan_close(root, id, &request, &date))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{error}; nothing was closed"))?;
    write_package_closes(root, &mut plans, &CLOSE_KIND)?;
    Ok(plans
        .into_iter()
        .map(|plan| ClosedChange {
            package: portable_project_path(root, &plan.destination),
            record: plan.record,
        })
        .collect())
}
