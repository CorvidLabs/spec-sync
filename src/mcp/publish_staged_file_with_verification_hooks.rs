
fn publish_staged_file_with_verification_hooks<
    BeforeQuarantine,
    AfterHardLink,
    DestinationIdentity,
>(
    staged: &mut StagedFile<'_>,
    label: &str,
    before_quarantine: BeforeQuarantine,
    after_hard_link: AfterHardLink,
    identify_destination: DestinationIdentity,
) -> Result<(), String>
where
    BeforeQuarantine: FnOnce(&Dir, &Path),
    AfterHardLink: FnOnce(&Dir, &Path),
    DestinationIdentity: FnOnce(&cap_std::fs::File) -> io::Result<FileIdentity>,
{
    let temporary = staged
        .parent
        .open(&staged.temporary_name)
        .map_err(|error| {
            format!(
                "Cannot open staged MCP {label} {} before publication: {error}",
                staged.temporary_display.display()
            )
        })?;
    let observed = file_identity(&temporary).map_err(|error| {
        format!(
            "Cannot identify staged MCP {label} {} before publication: {error}",
            staged.temporary_display.display()
        )
    })?;
    if observed != staged.identity {
        return Err(format!(
            "Refusing to publish replacement staged MCP {label} {}",
            staged.temporary_display.display()
        ));
    }
    drop(temporary);
    before_quarantine(&staged.parent, &staged.temporary_name);
    verify_staged_public_parent(staged, label)?;

    let quarantined = quarantine_entry(
        &staged.parent,
        &staged.temporary_name,
        &staged.temporary_display,
        "staged file",
    )?;
    staged.temporary_present = false;
    let quarantined_file = quarantined.directory.open("entry").map_err(|error| {
        format!(
            "Cannot open quarantined staged MCP {label} {}: {error}",
            staged.temporary_display.display()
        )
    })?;
    let quarantined_identity = file_identity(&quarantined_file).map_err(|error| {
        format!(
            "Cannot identify quarantined staged MCP {label} {}: {error}",
            staged.temporary_display.display()
        )
    })?;
    if quarantined_identity != staged.identity {
        return Err(format!(
            "Refusing to publish replaced staged MCP {label} {}; quarantined replacement was preserved",
            staged.temporary_display.display()
        ));
    }

    drop(quarantined_file);

    if let Err(error) =
        quarantined
            .directory
            .hard_link("entry", &staged.parent, &staged.destination_name)
    {
        let failure = format!(
            "Cannot atomically publish confined MCP {label} {}: {error}",
            staged.destination_display.display()
        );
        return fail_after_hard_link(
            quarantined,
            staged.identity,
            &staged.temporary_display,
            failure,
        );
    }
    staged.published = true;
    after_hard_link(&quarantined.directory, Path::new("entry"));
    let destination = match staged.parent.open(&staged.destination_name) {
        Ok(destination) => destination,
        Err(error) => {
            let failure = format!(
                "Cannot verify published MCP {label} {}: {error}",
                staged.destination_display.display()
            );
            return fail_after_hard_link(
                quarantined,
                staged.identity,
                &staged.temporary_display,
                failure,
            );
        }
    };
    let destination_identity = match identify_destination(&destination) {
        Ok(identity) => identity,
        Err(error) => {
            drop(destination);
            let failure = format!(
                "Cannot identify published MCP {label} {}: {error}",
                staged.destination_display.display()
            );
            return fail_after_hard_link(
                quarantined,
                staged.identity,
                &staged.temporary_display,
                failure,
            );
        }
    };
    if destination_identity != staged.identity {
        drop(destination);
        let failure = format!(
            "Published MCP {label} {} was replaced before identity verification",
            staged.destination_display.display()
        );
        return fail_after_hard_link(
            quarantined,
            staged.identity,
            &staged.temporary_display,
            failure,
        );
    }
    drop(destination);
    if let Err(error) = verify_staged_public_parent(staged, label) {
        return fail_after_hard_link(
            quarantined,
            staged.identity,
            &staged.temporary_display,
            error,
        );
    }
    cleanup_quarantined_file(quarantined, staged.identity, &staged.temporary_display)
}

fn fail_after_hard_link(
    quarantined: QuarantinedEntry,
    expected: FileIdentity,
    display: &Path,
    failure: String,
) -> Result<(), String> {
    match cleanup_quarantined_file(quarantined, expected, display) {
        Ok(()) => Err(failure),
        Err(cleanup) => Err(format!("{failure}; {cleanup}")),
    }
}

struct ConfinedReadRoot<'root> {
    root: &'root Dir,
    root_identity: DirectoryIdentity,
    directory: Dir,
    components: Vec<(PathBuf, SnapshotEntryIdentity)>,
}

impl ConfinedReadRoot<'_> {
    fn directory(&self) -> &Dir {
        &self.directory
    }

    fn revalidate_before_success(&self) -> Result<(), String> {
        #[cfg(debug_assertions)]
        wait_for_mcp_read_root_revalidation_test_barrier()?;

        let observed_root = directory_identity(self.root)
            .map_err(|error| format!("Cannot reidentify MCP read-root authority: {error}"))?;
        if observed_root != self.root_identity {
            return Err("MCP read-root authority changed before operation success".to_string());
        }

        let retained_identity = directory_identity(&self.directory)
            .map_err(|error| format!("Cannot reidentify retained MCP read root: {error}"))?;
        let expected_final = self
            .components
            .last()
            .map(|(_, identity)| *identity)
            .unwrap_or(self.root_identity);
        if retained_identity != expected_final {
            return Err("Retained MCP read root changed before operation success".to_string());
        }

        let mut directory = self
            .root
            .try_clone()
            .map_err(|error| format!("Cannot clone MCP read-root authority: {error}"))?;
        for (relative, expected) in &self.components {
            let Some(name) = relative.file_name() else {
                return Err(format!(
                    "MCP read-root route is invalid before operation success: {}",
                    relative.display()
                ));
            };
            let Some((next, observed)) = open_snapshot_configuration_directory(
                &directory, name, relative,
            )
            .map_err(|error| {
                format!(
                    "MCP read-root route changed before operation success at {}: {error}",
                    relative.display()
                )
            })?
            else {
                return Err(format!(
                    "MCP read-root route changed before operation success at {}",
                    relative.display()
                ));
            };
            if observed != *expected {
                return Err(format!(
                    "MCP read-root route changed before operation success at {}",
                    relative.display()
                ));
            }
            directory = next;
        }
        Ok(())
    }
}

fn open_confined_read_root<'root>(
    server_directory: &'root Dir,
    relative: &Path,
) -> Result<ConfinedReadRoot<'root>, String> {
    let root_identity = directory_identity(server_directory)
        .map_err(|error| format!("Cannot identify MCP read-root authority: {error}"))?;
    let mut directory = server_directory
        .try_clone()
        .map_err(|error| format!("Cannot clone MCP server root capability: {error}"))?;
    let mut display = PathBuf::new();
    let mut components = Vec::new();
    for component in relative.components() {
        match component {
            Component::CurDir => continue,
            Component::Normal(name) => {
                display.push(name);
                let Some((next, identity)) =
                    open_snapshot_configuration_directory(&directory, name, &display)
                        .map_err(|error| {
                            format!(
                                "Read root override does not resolve through regular confined directories {}: {error}",
                                display.display()
                            )
                        })?
                else {
                    return Err(format!(
                        "Read root override does not resolve to a confined existing directory {}",
                        display.display()
                    ));
                };
                directory = next;
                components.push((display.clone(), identity));
            }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "Read root override must remain beneath the MCP server root: {}",
                    relative.display()
                ));
            }
        }
    }
    Ok(ConfinedReadRoot {
        root: server_directory,
        root_identity,
        directory,
        components,
    })
}

fn verify_staged_public_parent(staged: &StagedFile<'_>, label: &str) -> Result<(), String> {
    let mut public_parent = staged
        .root
        .try_clone()
        .map_err(|error| format!("Cannot clone MCP server root capability: {error}"))?;
    let mut display = PathBuf::new();
    for component in staged.parent_path.components() {
        match component {
            Component::CurDir => continue,
            Component::Normal(name) => {
                display.push(name);
                let Some((next, _)) = open_snapshot_configuration_directory(
                    &public_parent,
                    name,
                    &display,
                )
                .map_err(|error| {
                    format!(
                        "Refusing to publish MCP {label}; public parent {} changed: {error}",
                        display.display()
                    )
                })?
                else {
                    return Err(format!(
                        "Refusing to publish MCP {label}; public parent {} no longer exists",
                        display.display()
                    ));
                };
                public_parent = next;
            }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "Refusing to publish MCP {label}; public parent path is not confined: {}",
                    staged.parent_path.display()
                ));
            }
        }
    }
    let public_identity = directory_identity(&public_parent).map_err(|error| {
        format!(
            "Cannot identify public MCP {label} parent {}: {error}",
            staged.parent_path.display()
        )
    })?;
    let retained_identity = directory_identity(&staged.parent).map_err(|error| {
        format!(
            "Cannot identify retained MCP {label} parent {}: {error}",
            staged.parent_path.display()
        )
    })?;
    if public_identity != staged.parent_identity || retained_identity != staged.parent_identity {
        return Err(format!(
            "Refusing to publish MCP {label}; public parent {} changed after staging",
            staged.parent_path.display()
        ));
    }
    Ok(())
}

fn validate_confined_relative(relative: &Path, label: &str) -> Result<(), String> {
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "MCP {label} must be a project-relative path without traversal: {}",
            relative.display()
        ));
    }
    Ok(())
}

fn remove_identity_bound_file(
    parent: &Dir,
    name: &Path,
    display: &Path,
    expected: FileIdentity,
) -> Result<(), String> {
    remove_identity_bound_file_with_hook(parent, name, display, expected, |_, _| {})
}

fn remove_identity_bound_file_with_hook<Hook>(
    parent: &Dir,
    name: &Path,
    display: &Path,
    expected: FileIdentity,
    before_quarantine: Hook,
) -> Result<(), String>
where
    Hook: FnOnce(&Dir, &Path),
{
    let file = match parent.open(name) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{}: {error}", display.display())),
    };
    let observed = file_identity(&file).map_err(|error| {
        format!(
            "Cannot identify {} during rollback: {error}",
            display.display()
        )
    })?;
    if observed != expected {
        return Err(format!(
            "Refusing to remove replacement file {} during rollback",
            display.display()
        ));
    }
    drop(file);
    before_quarantine(parent, name);
    let quarantined = match quarantine_entry(parent, name, display, "file") {
        Ok(quarantined) => quarantined,
        Err(error) if error.contains("not found") => return Ok(()),
        Err(error) => return Err(error),
    };
    let file = quarantined.directory.open("entry").map_err(|error| {
        format!(
            "Cannot open quarantined {} during rollback: {error}",
            display.display()
        )
    })?;
    let observed = file_identity(&file).map_err(|error| {
        format!(
            "Cannot identify quarantined {} during rollback: {error}",
            display.display()
        )
    })?;
    if observed != expected {
        return Err(format!(
            "Refusing to remove replacement file {} during rollback; quarantined replacement was preserved",
            display.display()
        ));
    }
    drop(file);
    cleanup_quarantined_file(quarantined, expected, display)
}

struct QuarantinedEntry {
    directory: Dir,
    directory_identity: DirectoryIdentity,
}

fn quarantine_entry(
    parent: &Dir,
    name: &Path,
    display: &Path,
    kind: &str,
) -> Result<QuarantinedEntry, String> {
    for _ in 0..128 {
        let sequence = STAGED_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory_name = PathBuf::from(format!(
            ".specsync-mcp-quarantine-{}-{sequence}",
            std::process::id()
        ));
        match parent.create_dir(&directory_name) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "Cannot reserve rollback quarantine for {}: {error}",
                    display.display()
                ));
            }
        }
        let directory = parent.open_dir(&directory_name).map_err(|error| {
            format!(
                "Cannot open rollback quarantine for {}: {error}",
                display.display()
            )
        })?;
        let directory_identity = match directory_identity(&directory) {
            Ok(identity) => identity,
            Err(error) => {
                let failure = format!(
                    "Cannot identify rollback quarantine for {}: {error}",
                    display.display()
                );
                return match directory.remove_open_dir() {
                    Ok(()) => Err(failure),
                    Err(cleanup) => Err(format!(
                        "{failure}; cannot remove unidentifiable rollback quarantine: {cleanup}"
                    )),
                };
            }
        };
        if let Err(error) = parent.rename(name, &directory, "entry") {
            let failure = format!(
                "Cannot atomically quarantine {kind} {} before deletion: {error}",
                display.display()
            );
            return match directory.remove_open_dir() {
                Ok(()) => Err(failure),
                Err(cleanup) => Err(format!(
                    "{failure}; cannot remove unused rollback quarantine: {cleanup}"
                )),
            };
        }
        return Ok(QuarantinedEntry {
            directory,
            directory_identity,
        });
    }
    Err(format!(
        "Cannot reserve a unique rollback quarantine for {}",
        display.display()
    ))
}

fn cleanup_quarantined_file(
    quarantined: QuarantinedEntry,
    expected: FileIdentity,
    display: &Path,
) -> Result<(), String> {
    let file = quarantined
        .directory
        .open("entry")
        .map_err(|error| format!("Cannot reopen quarantined {}: {error}", display.display()))?;
    if file_identity(&file).map_err(|error| {
        format!(
            "Cannot reidentify quarantined {}: {error}",
            display.display()
        )
    })? != expected
    {
        return Err(format!(
            "Refusing to delete a replacement in the private quarantine for {}",
            display.display()
        ));
    }
    drop(file);
    quarantined
        .directory
        .remove_file("entry")
        .map_err(|error| format!("Cannot remove quarantined {}: {error}", display.display()))?;
    cleanup_quarantine_directory(quarantined, display)
}

fn cleanup_quarantine_directory(
    quarantined: QuarantinedEntry,
    display: &Path,
) -> Result<(), String> {
    let observed = directory_identity(&quarantined.directory).map_err(|error| {
        format!(
            "Cannot reidentify rollback quarantine for {}: {error}",
            display.display()
        )
    })?;
    if observed != quarantined.directory_identity {
        return Err(format!(
            "Refusing to remove a replaced rollback quarantine for {}",
            display.display()
        ));
    }
    quarantined.directory.remove_open_dir().map_err(|error| {
        format!(
            "Cannot remove rollback quarantine for {}: {error}",
            display.display()
        )
    })
}

fn rollback_staged_batch(staged: &[StagedFile<'_>], error: String) -> String {
    let mut cleanup_failures = Vec::new();
    for file in staged.iter().rev() {
        if file.published
            && let Err(cleanup_error) = remove_identity_bound_file(
                &file.parent,
                &file.destination_name,
                &file.destination_display,
                file.identity,
            )
        {
            cleanup_failures.push(cleanup_error);
        }
        if file.temporary_present
            && let Err(cleanup_error) = remove_identity_bound_file(
                &file.parent,
                &file.temporary_name,
                &file.temporary_display,
                file.identity,
            )
        {
            cleanup_failures.push(cleanup_error);
        }
    }
    append_rollback_failures(error, cleanup_failures)
}

fn append_rollback_failures(error: String, cleanup_failures: Vec<String>) -> String {
    if cleanup_failures.is_empty() {
        error
    } else {
        format!(
            "{error}; failed to roll back generated outputs: {}",
            cleanup_failures.join(", ")
        )
    }
}

fn tool_score(root: &Path, git_freshness_available: bool) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, false)?;

    let scores: Vec<scoring::SpecScore> = spec_files
        .iter()
        .map(|f| score_spec_for_mcp(f, root, &config, git_freshness_available))
        .collect();
    let project = scoring::compute_project_score(scores);

    let specs_json: Vec<Value> = project
        .spec_scores
        .iter()
        .map(|s| {
            json!({
                "spec": s.spec_path,
                "total": s.total,
                "grade": s.grade,
                "frontmatter": s.frontmatter_score,
                "sections": s.sections_score,
                "api": s.api_score,
                "depth": s.depth_score,
                "freshness": s.freshness_score,
                "git_freshness_available": git_freshness_available,
                "suggestions": s.suggestions,
            })
        })
        .collect();

    Ok(json!({
        "average_score": (project.average_score * 10.0).round() / 10.0,
        "grade": project.grade,
        "total_specs": project.total_specs,
        "git_freshness_available": git_freshness_available,
        "distribution": {
            "A": project.grade_distribution[0],
            "B": project.grade_distribution[1],
            "C": project.grade_distribution[2],
            "D": project.grade_distribution[3],
            "F": project.grade_distribution[4],
        },
        "specs": specs_json,
    }))
}

fn score_spec_for_mcp(
    spec_path: &Path,
    root: &Path,
    config: &crate::types::SpecSyncConfig,
    git_freshness_available: bool,
) -> scoring::SpecScore {
    let mut score = scoring::score_spec(spec_path, root, config);
    if git_freshness_available {
        return score;
    }

    const UNAVAILABLE_GIT_FRESHNESS_POINTS: u32 = 5;
    // `score_spec` now withholds these points itself whenever there is no
    // history to measure against (#572), which is always true of the confined
    // snapshot — it is a temp copy with no `.git`. Deducting again would charge
    // the spec twice for one absence, so only deduct when the points are still
    // on the board. The relabel below always runs: "intentionally unavailable
    // here" is a more useful thing to read than "not a git repository".
    let penalty = match score.git_freshness {
        scoring::GitFreshness::Withheld => 0,
        _ => score.freshness_score.min(UNAVAILABLE_GIT_FRESHNESS_POINTS),
    };
    score.freshness_score = score.freshness_score.saturating_sub(penalty);
    score.total = score.total.saturating_sub(penalty);
    score.grade = scoring::letter_grade(score.total);
    score
        .suggestions
        .retain(|suggestion| !suggestion.contains("drift is unverifiable"));
    score.suggestions.push(
        "Freshness (-5pts): Git history is intentionally unavailable in the confined MCP snapshot"
            .to_string(),
    );
    if let Some(freshness) = score
        .explain
        .iter_mut()
        .find(|detail| detail.dimension == "Freshness")
    {
        freshness.score = score.freshness_score;
        if let Some(criterion) = freshness
            .criteria
            .iter_mut()
            .find(|criterion| criterion.name == "git_freshness")
        {
            criterion.passed = false;
            criterion.points = 0;
            criterion.detail = Some("unavailable in confined MCP snapshot".to_string());
        }
    }
    score
}

fn tool_issues(root: &Path) -> Result<Value, String> {
    use crate::github;

    let (config, references) = discover_issue_references(root)?;

    let mut results: Vec<Value> = Vec::new();
    let mut total_valid = 0usize;
    let mut total_closed = 0usize;
    let mut total_not_found = 0usize;

    if references.is_empty() {
        return Ok(json!({
            "repo": null,
            "total_valid": 0,
            "total_closed": 0,
            "total_not_found": 0,
            "specs": results,
        }));
    }

    let repo = config
        .github
        .as_ref()
        .and_then(|github| github.repo.as_deref())
        .filter(|repo| !repo.trim().is_empty())
        .ok_or_else(|| {
            "MCP specsync_issues requires an explicit `github.repo`; Git metadata auto-detection is disabled by the server security boundary".to_string()
        })?
        .to_string();

    for verification in github::verify_issue_batch(&repo, &references) {
        let rel_path = verification.spec_path.clone();
        ensure_issue_verification_complete(&rel_path, &verification)?;

        total_valid += verification.valid.len();
        total_closed += verification.closed.len();
        total_not_found += verification.not_found.len();

        results.push(json!({
            "spec": rel_path,
            "valid": verification.valid.iter().map(|i| json!({
                "number": i.number,
                "title": i.title,
                "state": i.state,
            })).collect::<Vec<_>>(),
            "closed": verification.closed.iter().map(|i| json!({
                "number": i.number,
                "title": i.title,
            })).collect::<Vec<_>>(),
            "not_found": verification.not_found,
        }));
    }

    Ok(json!({
        "repo": repo,
        "total_valid": total_valid,
        "total_closed": total_closed,
        "total_not_found": total_not_found,
        "specs": results,
    }))
}

type IssueReferences = (String, Vec<u64>, Vec<u64>);

fn is_issue_spec_file_name(name: &OsStr) -> Result<bool, ()> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let bytes = name.as_bytes();
        if !bytes.ends_with(b".spec.md") {
            return Ok(false);
        }
        if name.to_str().is_none() {
            return Err(());
        }
        Ok(!bytes.starts_with(b"_"))
    }

    #[cfg(not(unix))]
    {
        Ok(name
            .to_str()
            .is_some_and(|name| name.ends_with(".spec.md") && !name.starts_with('_')))
    }
}

fn find_issue_spec_files_checked(root: &Path, specs_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut spec_files = Vec::new();
    for result in WalkDir::new(specs_dir)
        .follow_links(false)
        .sort_by_file_name()
    {
        let entry = result.map_err(|error| {
            let rel_path = issue_relative_spec_path(root, error.path().unwrap_or(specs_dir));
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec directory entry could not be inspected"
            )
        })?;
        let is_spec = is_issue_spec_file_name(entry.file_name()).map_err(|()| {
            let rel_path = issue_relative_spec_path(root, entry.path());
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec filename is not valid UTF-8"
            )
        })?;
        if !is_spec {
            continue;
        }
        if !entry.file_type().is_file() {
            let rel_path = issue_relative_spec_path(root, entry.path());
            return Err(format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec path is not a regular file"
            ));
        }
        spec_files.push(entry.into_path());
    }
    spec_files.sort();
    Ok(spec_files)
}

fn discover_issue_references(
    root: &Path,
) -> Result<(SpecSyncConfig, Vec<IssueReferences>), String> {
    let config = load_confined_config(root).map_err(|error| {
        format!(
            "MCP specsync_issues issue discovery is inconclusive: {}",
            issue_project_error_reason(&error)
        )
    })?;
    let specs_dir = root.join(&config.specs_dir);
    let spec_files = find_issue_spec_files_checked(root, &specs_dir)?;

    if spec_files.is_empty() {
        return Err(
            "MCP specsync_issues issue discovery is inconclusive: no spec files were found"
                .to_string(),
        );
    }

    let mut references = Vec::new();
    for spec_path in &spec_files {
        let rel_path = issue_relative_spec_path(root, spec_path);
        let content = read_file_bounded(root, spec_path, "spec file").map_err(|error| {
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: {}",
                issue_spec_read_error_reason(&error)
            )
        })?;
        let (implements, tracks) = crate::parser::parse_checked_issue_references(&content)
            .map_err(|reason| {
                format!(
                    "MCP specsync_issues issue discovery is inconclusive for {rel_path}: {reason}"
                )
            })?;

        if !implements.is_empty() || !tracks.is_empty() {
            references.push((rel_path, implements, tracks));
        }
    }

    validate_spec_file_mappings(root, &spec_files, &config.exclude_dirs).map_err(|error| {
        format!(
            "MCP specsync_issues issue discovery is inconclusive: {}",
            issue_project_error_reason(&error)
        )
    })?;

    Ok((config, references))
}

fn is_unsafe_issue_diagnostic_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}

fn issue_relative_spec_path(root: &Path, spec_path: &Path) -> String {
    let Some(relative) = spec_path.strip_prefix(root).ok() else {
        return "<spec>".to_string();
    };
    let relative = relative.to_string_lossy();
    let mut sanitized = String::with_capacity(relative.len());
    for character in relative.chars() {
        if is_unsafe_issue_diagnostic_character(character) {
            sanitized.push_str(&format!("\\u{{{:04X}}}", character as u32));
        } else if cfg!(windows) && character == '\\' {
            sanitized.push('/');
        } else {
            sanitized.push(character);
        }
    }
    if sanitized.chars().count() <= MAX_ISSUE_DIAGNOSTIC_PATH_CHARS {
        return sanitized;
    }

    let mut bounded: String = sanitized
        .chars()
        .take(MAX_ISSUE_DIAGNOSTIC_PATH_CHARS.saturating_sub(3))
        .collect();
    bounded.push_str("...");
    bounded
}

fn issue_spec_read_error_reason(error: &str) -> &'static str {
    if error.contains("not valid UTF-8") {
        "spec file is not valid UTF-8"
    } else if error.contains("exceeds") {
        "spec file exceeds the configured size limit"
    } else if error.contains("not a file") {
        "spec path is not a regular file"
    } else if error.contains("Cannot inspect") {
        "spec file metadata could not be inspected"
    } else if error.contains("Cannot read") {
        "spec file could not be read"
    } else {
        "spec path failed safety validation"
    }
}

fn issue_project_error_reason(error: &str) -> &'static str {
    if error.contains("exceeds") || error.contains("limit") {
        "project input exceeds a configured safety limit"
    } else if error.contains("Cannot read") || error.contains("not valid UTF-8") {
        "project metadata could not be read"
    } else if error.contains("Cannot inspect") {
        "project metadata could not be inspected"
    } else if error.contains("escape")
        || error.contains("outside")
        || error.contains("symlink")
        || error.contains("junction")
        || error.contains("relative path")
    {
        "project input failed confinement validation"
    } else {
        "project discovery failed safety validation"
    }
}

fn ensure_issue_verification_complete(
    spec_path: &str,
    verification: &crate::github::IssueVerification,
) -> Result<(), String> {
    if verification.errors.is_empty() {
        return Ok(());
    }
    Err(format!(
        "GitHub issue verification was inconclusive for {spec_path}: {}",
        verification.errors.join("; ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn open_test_directory(path: &Path) -> Dir {
        Dir::open_ambient_dir(path, ambient_authority()).unwrap()
    }

    fn assert_no_mcp_transaction_debris(root: &Path) {
        assert!(
            WalkDir::new(root)
                .into_iter()
                .filter_map(Result::ok)
                .all(|entry| {
                    let name = entry.file_name().to_string_lossy();
                    !name.starts_with(".specsync-mcp-stage-")
                        && !name.starts_with(".specsync-mcp-quarantine-")
                }),
            "successful MCP publication must remove private staging and quarantine entries"
        );
    }

    fn setup_project() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let config = json!({
            "specsDir": "specs",
            "sourceDirs": ["src"],
            "requiredSections": ["Purpose", "Public API"]
        });
        std::fs::write(
            tmp.path().join("specsync.json"),
            serde_json::to_string_pretty(&config).unwrap(),
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join("specs")).unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        tmp
    }

    fn setup_project_with_spec(spec_name: &str, spec_content: &str) -> TempDir {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join(spec_name);
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join(format!("{spec_name}.spec.md")), spec_content).unwrap();
        tmp
    }

    fn git_in(root: &Path, args: &[&str]) {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("git command should run");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn init_git_repo(root: &Path) {
        git_in(root, &["init"]);
        git_in(root, &["config", "user.email", "test@test.com"]);
        git_in(root, &["config", "user.name", "Test"]);
        git_in(root, &["config", "commit.gpgsign", "false"]);
        git_in(root, &["add", "-A"]);
        git_in(root, &["commit", "-m", "init"]);
    }

    #[test]
    fn issue_verification_errors_fail_closed_instead_of_returning_empty_success() {
        let verification = crate::github::IssueVerification {
            spec_path: "specs/mcp/mcp.spec.md".to_string(),
            valid: Vec::new(),
            closed: Vec::new(),
            not_found: Vec::new(),
            errors: vec!["#414: authentication failed".to_string()],
        };

        let error =
            ensure_issue_verification_complete("specs/mcp/mcp.spec.md", &verification).unwrap_err();
        assert!(error.contains("inconclusive"));
        assert!(error.contains("#414"));
    }

    #[test]
    fn issue_tool_enforces_one_deduplicated_invocation_cap_across_specs() {
        let tmp = setup_project();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"],"github":{"repo":"CorvidLabs/spec-sync"}}"#,
        )
        .unwrap();
        let first: Vec<String> = (1..=60).map(|number| number.to_string()).collect();
        let second: Vec<String> = (41..=101).map(|number| number.to_string()).collect();
        for (module, issues) in [("one", first), ("two", second)] {
            let directory = tmp.path().join("specs").join(module);
            fs::create_dir_all(&directory).unwrap();
            fs::write(
                directory.join(format!("{module}.spec.md")),
                format!(
                    "---\nmodule: {module}\nversion: 1\nstatus: draft\nfiles: []\ntracks: [{}]\n---\n\n# Purpose\nTest\n",
                    issues.join(", ")
                ),
            )
            .unwrap();
        }

        let error = tool_issues(tmp.path())
            .expect_err("the global issue cap must fail before provider selection");

        assert!(error.contains("100-issue invocation limit"));
    }

    #[test]
    fn issue_tool_fails_inconclusive_for_malformed_frontmatter() {
        let tmp = setup_project_with_spec("malformed", "# Missing frontmatter\n");

        let error = tool_issues(tmp.path())
            .expect_err("malformed frontmatter must not produce a zero-reference success");

        assert_eq!(
            error,
            "MCP specsync_issues issue discovery is inconclusive for specs/malformed/malformed.spec.md: missing or malformed YAML frontmatter"
        );
    }

    #[test]
    fn issue_tool_fails_inconclusive_for_unreadable_spec_text() {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("unreadable");
        fs::create_dir_all(&spec_dir).unwrap();
        let spec_path = spec_dir.join("unreadable.spec.md");
        fs::write(&spec_path, [0xff]).unwrap();

        let error = tool_issues(tmp.path())
            .expect_err("unreadable spec text must not produce a zero-reference success");

        assert_eq!(
            error,
            "MCP specsync_issues issue discovery is inconclusive for specs/unreadable/unreadable.spec.md: spec file is not valid UTF-8"
        );
        assert!(!error.contains(&tmp.path().to_string_lossy().to_string()));
    }

    #[test]
    fn issue_tool_fails_inconclusive_for_malformed_known_issue_fields() {
        let cases = [
            ("implements:", "`implements` must be a list"),
            ("implements: null", "`implements` must be a list"),
            ("implements: nope", "`implements` must be a list"),
            ("tracks: 42", "`tracks` must be a list"),
            ("implements : [nope]", "`implements` contains an invalid"),
            ("implements: [0]", "`implements` contains an invalid"),
            ("implements: [-1]", "`implements` contains an invalid"),
            (
                "implements: [18446744073709551616]",
                "`implements` contains an invalid",
            ),
            (
                "implements: [41, not-a-number, 42]",
                "`implements` contains an invalid unsigned issue number",
            ),
            (
                "tracks:\n  - 41\n  - nope\n  - 42",
                "`tracks` contains an invalid unsigned issue number",
            ),
            (
                "tracks:\n- 41\n- nope",
                "`tracks` contains an invalid unsigned issue number",
            ),
        ];

        for (field, expected_reason) in cases {
            let spec = format!(
                "---\nmodule: malformed\nversion: 1\nstatus: draft\nfiles: []\n{field}\n---\n\n# Purpose\nTest\n"
            );
            let tmp = setup_project_with_spec("malformed", &spec);

            let error = tool_issues(tmp.path()).expect_err(
                "malformed known issue fields must not produce a zero-reference success",
            );

            assert!(
                error.contains(expected_reason),
                "unexpected error for {field}: {error}"
            );
            assert!(!error.contains(&tmp.path().to_string_lossy().to_string()));
        }
    }

    #[test]
    fn issue_reference_field_validation_accepts_supported_list_forms() {
        let content = "---\nmodule: valid\nimplements: [41, 42]\ntracks:\n  - 43\n  - 44 # retained comment\n---\n\n# Purpose\nTest\n";

        assert_eq!(
            crate::parser::parse_checked_issue_references(content).unwrap(),
            (vec![41, 42], vec![43, 44])
        );
    }

    #[test]
    fn issue_discovery_accepts_yaml_trailing_comma_and_inline_comment() {
        let content = "---\nmodule: valid\nversion: 1\nstatus: draft\nfiles: []\nimplements: [42,] # valid YAML\n---\n\n# Purpose\nTest\n";
        let tmp = setup_project_with_spec("valid", content);

        let (_, references) = discover_issue_references(tmp.path()).unwrap();

        assert_eq!(
            references,
            vec![(
                "specs/valid/valid.spec.md".to_string(),
                vec![42],
                Vec::new()
            )]
        );
    }

    #[test]
    fn issue_discovery_accepts_crlf_frontmatter_and_retains_references() {
        let content = "---\r\nmodule: crlf\r\nversion: 1\r\nstatus: draft\r\nfiles: []\r\nimplements: [41, 42]\r\ntracks:\r\n  - 43\r\n---\r\n\r\n# Purpose\r\nTest\r\n";
        let tmp = setup_project_with_spec("crlf", content);

        let (_, references) = discover_issue_references(tmp.path()).unwrap();

        assert_eq!(
            references,
            vec![(
                "specs/crlf/crlf.spec.md".to_string(),
                vec![41, 42],
                vec![43],
            )]
        );
    }

    #[test]
    fn issue_reference_field_validation_ignores_nested_extensions_and_block_scalars() {
        let content = "\
---
module: valid
extensions:
  implements: [900]
  nested:
    tracks: invalid
extension_sequence:
  - implements: [901]
    tracks: [902]
notes: |
  implements: invalid
  tracks:
    - 903
folded: >
  tracks: [904]
implements: [41, 42]
tracks:
  - 43
  - 44
---

# Purpose
Test
";

        assert_eq!(
            crate::parser::parse_checked_issue_references(content).unwrap(),
            (vec![41, 42], vec![43, 44])
        );
    }

    #[test]
    fn issue_reference_field_validation_keeps_top_level_known_fields_strict() {
        let content = "\
---
module: invalid
extensions:
  implements: [900]
implements:
  nested: invalid
---

# Purpose
Test
";

        assert_eq!(
            crate::parser::parse_checked_issue_references(content).unwrap_err(),
            "`implements` must be a list of unsigned issue numbers"
        );
    }

    #[test]
    fn issue_tool_rejects_duplicate_issue_reference_keys() {
        let content = "---\nmodule: invalid\nversion: 1\nstatus: draft\nfiles: []\nimplements: [41]\nimplements: [42]\n---\n\n# Purpose\nTest\n";
        let tmp = setup_project_with_spec("invalid", content);

        let error = tool_issues(tmp.path()).unwrap_err();

        assert_eq!(
            error,
            "MCP specsync_issues issue discovery is inconclusive for specs/invalid/invalid.spec.md: duplicate `implements` issue-reference field"
        );
    }

    #[test]
    fn issue_tool_rejects_reviewer_reproducer_without_leaking_content() {
        let content = "\
---
module: invalid
version: 1
status: draft
files: []
implements:
private_extension:
  secret: [reviewer-reproducer
---

# Purpose
Test
";
        let tmp = setup_project_with_spec("invalid", content);

        let error = tool_issues(tmp.path()).unwrap_err();

        assert!(error.contains("issue discovery is inconclusive"));
        assert!(
            error.contains("invalid YAML frontmatter")
                || error.contains("`implements` must be a list")
        );
        assert!(!error.contains("reviewer-reproducer"));
        assert!(!error.contains("private_extension"));
    }

    #[cfg(unix)]
    #[test]
    fn issue_spec_file_name_rejects_non_utf8_spec_suffix() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let mut file_name = b"hidden-".to_vec();
        file_name.push(0xff);
        file_name.extend_from_slice(b".spec.md");

        assert_eq!(
            is_issue_spec_file_name(&OsString::from_vec(file_name)),
            Err(())
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn issue_tool_rejects_non_utf8_spec_filename_after_snapshot_copy() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("hostile");
        fs::create_dir_all(&spec_dir).unwrap();
        let mut file_name = b"hidden-".to_vec();
        file_name.push(0xff);
        file_name.extend_from_slice(b".spec.md");
        let spec_path = spec_dir.join(OsString::from_vec(file_name));
        fs::write(
            &spec_path,
            "---\nmodule: hostile\nversion: 1\nstatus: draft\nfiles: []\n---\n\n# Purpose\nTest\n",
        )
        .unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        let error = tool_issues(snapshot.root())
            .expect_err("a non-UTF-8 spec filename must make issue discovery inconclusive");

        assert!(error.contains("spec filename is not valid UTF-8"));
        assert!(error.contains("specs/hostile/hidden-"));
        assert!(!error.contains(&tmp.path().to_string_lossy().to_string()));
        assert!(!error.contains("# Purpose"));
        assert!(error.chars().count() < 400);
    }

    #[test]
    fn issue_read_diagnostics_are_bounded_relative_and_content_free() {
        let tmp = setup_project();
        let long_name = "x".repeat(MAX_ISSUE_DIAGNOSTIC_PATH_CHARS + 80);
        let path = tmp.path().join("specs").join(long_name);

        let relative = issue_relative_spec_path(tmp.path(), &path);
        let reason = issue_spec_read_error_reason(&format!(
            "Cannot read MCP spec file {}: secret operating system detail",
            path.display()
        ));

        assert!(relative.chars().count() <= MAX_ISSUE_DIAGNOSTIC_PATH_CHARS);
        assert!(!relative.contains(&tmp.path().to_string_lossy().to_string()));
        assert_eq!(reason, "spec file could not be read");
        assert!(!reason.contains("secret"));
    }

    #[test]
    fn issue_relative_spec_path_escapes_every_unsafe_display_character() {
        let root = Path::new("project");
        let dangerous_code_points = (0x0000u32..=0x001f)
            .chain(0x007f..=0x009f)
            .chain([0x061c, 0x200e, 0x200f, 0x2028, 0x2029])
            .chain(0x202a..=0x202e)
            .chain(0x2066..=0x2069);

        for code_point in dangerous_code_points {
            let character =
                char::from_u32(code_point).expect("test code point must be valid Unicode");
            let path = root
                .join("specs")
                .join(format!("before{character}after.spec.md"));
            let escaped = format!("\\u{{{code_point:04X}}}");

            assert_eq!(
                issue_relative_spec_path(root, &path),
                format!("specs/before{escaped}after.spec.md"),
                "unsafe diagnostic character U+{code_point:04X} was not escaped"
            );
        }

        let path = root.join(r"specs\windows\path.spec.md");
        #[cfg(windows)]
        assert_eq!(
            issue_relative_spec_path(root, &path),
            "specs/windows/path.spec.md"
        );
        #[cfg(not(windows))]
        assert_eq!(
            issue_relative_spec_path(root, &path),
            r"specs\windows\path.spec.md"
        );
    }

    #[cfg(unix)]
    #[test]
    fn issue_tool_escapes_adversarial_filename_formatting_in_diagnostic() {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("hostile");
        fs::create_dir_all(&spec_dir).unwrap();
        let dangerous_characters = [
            '\n', '\u{001b}', '\u{009b}', '\u{061c}', '\u{200e}', '\u{200f}', '\u{2028}',
            '\u{2029}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}', '\u{2066}',
            '\u{2067}', '\u{2068}', '\u{2069}',
        ];
        let hostile: String = dangerous_characters.iter().copied().collect();
        let filename = format!(r"segment\before{hostile}after.spec.md");
        fs::write(spec_dir.join(filename), "# Missing frontmatter\n").unwrap();

        let error = tool_issues(tmp.path())
            .expect_err("unsafe filename formatting must not produce a successful issue result");

        assert!(error.contains(r"specs/hostile/segment\before"));
        for character in dangerous_characters {
            assert!(
                error.contains(&format!("\\u{{{:04X}}}", character as u32)),
                "diagnostic omitted escape for U+{:04X}",
                character as u32
            );
            assert!(
                !error.contains(character),
                "diagnostic retained unsafe character U+{:04X}",
                character as u32
            );
        }
        assert!(error.ends_with("after.spec.md: missing or malformed YAML frontmatter"));
    }

    #[cfg(unix)]
    #[test]
    fn server_root_capability_rejects_a_root_replaced_before_canonicalization() {
        use std::os::unix::fs::symlink;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("server");
        let moved = tmp.path().join("moved-server");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let error = open_server_root_capability(&root, || {
            fs::rename(&root, &moved).unwrap();
            symlink(&outside, &root).unwrap();
            Ok(())
        })
        .expect_err("a swapped root must be rejected");

        assert!(error.contains("root") || error.contains("capability"));
    }

    #[test]
    fn windows_read_root_suffix_normalizes_drive_extended_unc_and_case() {
        assert_eq!(
            windows_relative_suffix_text(r"C:\Project", r"c:\PROJECT\Specs\Auth"),
            Some(PathBuf::from("Specs").join("Auth"))
        );
        assert_eq!(
            windows_relative_suffix_text(r"\\?\C:\Project", r"C:\Project\src"),
            Some(PathBuf::from("src"))
        );
        assert_eq!(
            windows_relative_suffix_text(
                r"\\server\share\Project",
                r"\\?\UNC\SERVER\SHARE\project\specs"
            ),
            Some(PathBuf::from("specs"))
        );
        assert_eq!(
            windows_relative_suffix_text(r"C:\Project", r"C:\ProjectElsewhere\src"),
            None
        );
        assert_eq!(
            windows_relative_suffix_text(r"\\server\share\Project", r"\\server\other\Project"),
            None
        );
        assert_eq!(
            windows_relative_suffix_text(r"C:\Projekt\Ärger", r"c:\PROJEKT\ärger\Specs"),
            Some(PathBuf::from("Specs"))
        );
    }

    #[test]
    fn read_root_suffix_accepts_the_identity_bound_startup_alias_only_as_a_prefix() {
        #[cfg(windows)]
        let canonical_root = Path::new(r"C:\Users\runneradmin\AppData\Local\Temp\server");
        #[cfg(windows)]
        let requested_root = Path::new(r"C:\Users\RUNNER~1\AppData\Local\Temp\server");
        #[cfg(windows)]
        let child = r"C:\Users\RUNNER~1\AppData\Local\Temp\server\child";
        #[cfg(windows)]
        let sibling = r"C:\Users\RUNNER~1\AppData\Local\Temp\server-sibling\child";

        #[cfg(not(windows))]
        let canonical_root = Path::new("/resolved/temp/server");
        #[cfg(not(windows))]
        let requested_root = Path::new("/startup-alias/temp/server");
        #[cfg(not(windows))]
        let child = "/startup-alias/temp/server/child";
        #[cfg(not(windows))]
        let sibling = "/startup-alias/temp/server-sibling/child";

        assert_eq!(
            resolve_read_root(canonical_root, requested_root, Some(child)).unwrap(),
            PathBuf::from("child")
        );
        assert_eq!(
            resolve_read_root(canonical_root, requested_root, Some(sibling)).unwrap_err(),
            "Read root override escapes the configured server root"
        );
    }

    #[test]
    fn read_root_rejects_git_metadata_components_case_insensitively() {
        #[cfg(not(windows))]
        let canonical_root = Path::new("/resolved/temp/server");
        #[cfg(not(windows))]
        let requested_root = Path::new("/startup-alias/temp/server");
        #[cfg(not(windows))]
        let absolute_git_root = "/startup-alias/temp/server/.gIt";

        #[cfg(windows)]
        let canonical_root = Path::new(r"C:\resolved\temp\server");
        #[cfg(windows)]
        let requested_root = Path::new(r"C:\startup-alias\temp\server");
        #[cfg(windows)]
        let absolute_git_root = r"C:\startup-alias\temp\server\.gIt";

        for candidate in [".git", ".GIT", "child/.GiT", absolute_git_root] {
            assert_eq!(
                resolve_read_root(canonical_root, requested_root, Some(candidate)).unwrap_err(),
                "Read root override must not select Git metadata"
            );
        }
        assert_eq!(
            resolve_read_root(canonical_root, requested_root, Some("child")).unwrap(),
            PathBuf::from("child")
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_read_root_accepts_runtime_absolute_variants() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("Child")).unwrap();
        let canonical = tmp.path().canonicalize().unwrap();
        let child = canonical.join("Child");
        let ordinary = child.to_string_lossy().replace(r"\\?\", "");
        let case_varied = ordinary.to_ascii_uppercase();

        assert_eq!(
            resolve_read_root(&canonical, &canonical, Some(&ordinary)).unwrap(),
            PathBuf::from("Child")
        );
        assert_eq!(
            resolve_read_root(&canonical, &canonical, Some(&case_varied)).unwrap(),
            PathBuf::from("CHILD")
        );

        let unicode_root = canonical.join("Ärger");
        fs::create_dir_all(unicode_root.join("Child")).unwrap();
        let unicode_candidate = unicode_root
            .join("Child")
            .to_string_lossy()
            .replace("Ärger", "ärger");
        assert_eq!(
            resolve_read_root(&unicode_root, &unicode_root, Some(&unicode_candidate)).unwrap(),
            PathBuf::from("Child")
        );
    }

    #[test]
    fn snapshot_root_source_dir_includes_otherwise_ignored_inputs() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("target")).unwrap();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["."]}"#,
        )
        .unwrap();
        fs::File::create(tmp.path().join("target/oversized.rs"))
            .unwrap()
            .set_len(MAX_PROJECT_FILE_BYTES + 1)
            .unwrap();

        let error = ProjectSnapshot::create(tmp.path())
            .err()
            .expect("explicit root inputs must not disappear from the snapshot");
        assert!(error.contains("per-file limit"));
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_skips_ignored_and_configured_exclusion_symlinks_before_following() {
        use std::os::unix::fs::symlink;

        let source_temp = TempDir::new().unwrap();
        let destination_temp = TempDir::new().unwrap();
        let explicit_destination_temp = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        fs::write(outside.path().join("secret.rs"), "outside\n").unwrap();
        fs::create_dir_all(source_temp.path().join("kept")).unwrap();
        fs::write(source_temp.path().join("kept/module.rs"), "inside\n").unwrap();
        symlink(outside.path(), source_temp.path().join("target")).unwrap();
        symlink("kept", source_temp.path().join("generated")).unwrap();
        let source = open_test_directory(source_temp.path());
        let destination = open_test_directory(destination_temp.path());
        let mut budget = SnapshotBudget::default();

        copy_snapshot_directory(
            &source,
            &destination,
            Path::new("."),
            &HashSet::from(["generated".to_string()]),
            &[],
            &mut budget,
        )
        .expect("excluded symlink names must be skipped without following their targets");

        assert!(!destination_temp.path().join("target").exists());
        assert!(!destination_temp.path().join("generated").exists());

        let explicit_destination = open_test_directory(explicit_destination_temp.path());
        let mut explicit_budget = SnapshotBudget::default();
        let error = copy_snapshot_directory(
            &source,
            &explicit_destination,
            Path::new("."),
            &HashSet::from(["generated".to_string()]),
            &[PathBuf::from("generated")],
            &mut explicit_budget,
        )
        .expect_err("an explicit input must inspect and reject the excluded symlink");
        assert!(
            error.contains("regular file or directory"),
            "unexpected explicit symlink rejection: {error}"
        );
    }

    #[test]
    fn snapshot_includes_manifest_derived_member_under_ignored_directory() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"vendor/member\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("vendor/member/Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("vendor/member/src/lib.rs"),
            "pub fn member() {}\n",
        )
        .unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        assert!(snapshot.root().join("vendor/member/src/lib.rs").is_file());
    }

    #[test]
    fn snapshot_normalizes_confined_cargo_sibling_dependency() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
        fs::create_dir_all(tmp.path().join("crates/b/src")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/a\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/a/Cargo.toml"),
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = \"../b\" }\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/b/Cargo.toml"),
            "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(tmp.path().join("crates/a/src/lib.rs"), "pub fn a() {}\n").unwrap();
        fs::write(tmp.path().join("crates/b/src/lib.rs"), "pub fn b() {}\n").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

        assert!(snapshot.root().join("crates/b/src/lib.rs").is_file());
    }

    #[test]
    fn snapshot_normalizes_confined_windows_native_cargo_paths() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
        fs::create_dir_all(tmp.path().join("crates/b/src")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = ['crates\\a']\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/a/Cargo.toml"),
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = '..\\b' }\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/b/Cargo.toml"),
            "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(tmp.path().join("crates/a/src/lib.rs"), "pub fn a() {}\n").unwrap();
        fs::write(tmp.path().join("crates/b/src/lib.rs"), "pub fn b() {}\n").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

        assert!(snapshot.root().join("crates/a/src/lib.rs").is_file());
        assert!(snapshot.root().join("crates/b/src/lib.rs").is_file());
    }

    #[test]
    fn snapshot_ignores_nonsemantic_cargo_metadata_paths() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join("vendor")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"metadata-path\"\nversion = \"0.1.0\"\n\n[lib]\npath = \"vendor/lib.rs\"\n\n[package.metadata.example]\npath = \"/benign/absolute/metadata/value\"\n",
        )
        .unwrap();
        fs::write(tmp.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
        fs::write(
            tmp.path().join("vendor/lib.rs"),
            "pub fn custom_target() {}\n",
        )
        .unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path())
            .expect("non-semantic Cargo metadata paths must not affect snapshot discovery");

        assert!(snapshot.root().join("src/lib.rs").is_file());
        assert!(snapshot.root().join("vendor/lib.rs").is_file());
    }

    #[test]
    fn snapshot_manifest_input_rejects_true_root_and_cross_platform_lexical_escapes() {
        let error = snapshot_manifest_input(
            Path::new("crates/a"),
            "../../../outside",
            "Cargo target path",
        )
        .expect_err("parent traversal beyond the retained root must be rejected");
        assert!(error.contains("escapes the configured server root"));

        for configured in [
            "/outside",
            "C:/outside",
            "C:outside",
            r"\\server\share\outside",
            r"\outside",
        ] {
            let error =
                snapshot_manifest_input(Path::new("crates/a"), configured, "Cargo target path")
                    .expect_err(
                        "absolute, drive, UNC, and backslash paths must be rejected portably",
                    );
            assert!(error.contains("safe project-relative path"));
        }

        let error = snapshot_manifest_input(
            Path::new("crates/a"),
            r"..\..\..\outside",
            "Cargo target path",
        )
        .expect_err("Windows-native traversal beyond the retained root must be rejected");
        assert!(error.contains("escapes the configured server root"));
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_rejects_confined_sibling_dependency_symlink_escape() {
        use std::os::unix::fs::symlink;

        let tmp = setup_project();
        let outside = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
        fs::create_dir_all(outside.path().join("src")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/a\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/a/Cargo.toml"),
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = '..\\b' }\n",
        )
        .unwrap();
        fs::write(
            outside.path().join("Cargo.toml"),
            "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(outside.path().join("src/lib.rs"), "pub fn outside() {}\n").unwrap();
        symlink(outside.path(), tmp.path().join("crates/b")).unwrap();

        ProjectSnapshot::create(tmp.path())
            .err()
            .expect("a normalized sibling dependency must not follow a symlink outside root");
    }

    #[test]
    fn snapshot_includes_multiline_cargo_members_beneath_ignored_directories() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\n  \"vendor/member\", # kept\n]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("vendor/member/Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::File::create(tmp.path().join("vendor/member/src/lib.rs"))
            .unwrap()
            .set_len(MAX_PROJECT_FILE_BYTES + 1)
            .unwrap();

        let error = ProjectSnapshot::create(tmp.path())
            .err()
            .expect("a multiline workspace member must remain visible and bounded");

        assert!(error.contains("per-file limit"));
        let expected_path = PathBuf::from("vendor")
            .join("member")
            .join("src")
            .join("lib.rs")
            .display()
            .to_string();
        assert!(error.contains(&expected_path));
    }

    #[test]
    fn snapshot_uses_toml_structure_when_comments_precede_workspace_header() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "# [workspace]\n# members = [\"ignored\"]\n\n[workspace]\nmembers = [\"vendor/member\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("vendor/member/Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("vendor/member/src/lib.rs"),
            "pub fn member() {}\n",
        )
        .unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

        assert!(snapshot.root().join("vendor/member/src/lib.rs").is_file());
    }

    #[test]
    fn snapshot_rejects_malformed_cargo_toml_without_partial_discovery() {
        let tmp = setup_project();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"vendor/member\"\n",
        )
        .unwrap();

        let error = ProjectSnapshot::create(tmp.path())
            .err()
            .expect("malformed Cargo TOML must make snapshot discovery inconclusive");

        assert!(error.contains("Cannot parse MCP Cargo workspace manifest Cargo.toml as TOML"));
    }

    #[test]
    fn manifest_discovery_charges_the_shared_cumulative_snapshot_budget() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("vendor/one")).unwrap();
        fs::create_dir_all(tmp.path().join("vendor/two")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"vendor/one\", \"vendor/two\"]\n",
        )
        .unwrap();
        fs::write(tmp.path().join("vendor/one/Cargo.toml"), vec![b' '; 300]).unwrap();
        fs::write(tmp.path().join("vendor/two/Cargo.toml"), vec![b' '; 300]).unwrap();
        let source = open_test_directory(tmp.path());
        let mut inputs = Vec::new();
        let mut budget = SnapshotBudget {
            max_bytes: 512,
            ..SnapshotBudget::default()
        };

        let error = collect_snapshot_manifest_inputs(&source, &mut inputs, &mut budget)
            .expect_err("manifest parsing must not bypass the cumulative byte budget");

        assert!(error.contains("512 bytes cumulative limit"));
    }

    #[test]
    fn snapshot_workspace_declarations_are_charged_before_deduplication() {
        let cargo = TempDir::new().unwrap();
        fs::create_dir_all(cargo.path().join("crates/member")).unwrap();
        fs::write(
            cargo.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/member\", \"./crates/member\"]\n",
        )
        .unwrap();
        fs::write(
            cargo.path().join("crates/member/Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let cargo_source = open_test_directory(cargo.path());
        let mut cargo_inputs = Vec::new();
        let mut cargo_budget = SnapshotBudget {
            entries: MAX_CONFINEMENT_ENTRIES - 2,
            ..SnapshotBudget::default()
        };

        collect_snapshot_manifest_inputs(&cargo_source, &mut cargo_inputs, &mut cargo_budget)
            .expect("two duplicate Cargo declarations must fit exactly at the traversal limit");

        assert_eq!(cargo_budget.entries, MAX_CONFINEMENT_ENTRIES);
        assert_eq!(
            cargo_inputs
                .iter()
                .filter(|path| path.as_path() == Path::new("crates/member"))
                .count(),
            1
        );

        let node = TempDir::new().unwrap();
        fs::write(
            node.path().join("package.json"),
            r#"{"workspaces":["packages/*","packages/*","packages/**"]}"#,
        )
        .unwrap();
        let node_source = open_test_directory(node.path());
        let mut node_inputs = Vec::new();
        let mut node_budget = SnapshotBudget {
            entries: MAX_CONFINEMENT_ENTRIES - 2,
            ..SnapshotBudget::default()
        };

        let error =
            collect_snapshot_manifest_inputs(&node_source, &mut node_inputs, &mut node_budget)
                .expect_err(
                    "a duplicate Node declaration beyond the traversal limit must be rejected",
                );

        assert!(error.contains("exceeds 100000 entries"), "{error}");
        assert_eq!(node_budget.entries, MAX_CONFINEMENT_ENTRIES + 1);
        assert_eq!(
            node_inputs
                .iter()
                .filter(|path| path.as_path() == Path::new("packages"))
                .count(),
            0
        );
    }

    #[test]
    fn snapshot_package_json_fails_closed_and_charges_entries_before_type_validation() {
        for (content, expected) in [
            ("{", "Cannot parse MCP package.json as JSON"),
            ("[]", "root must be a JSON object"),
            (
                r#"{"workspaces":7}"#,
                "`workspaces` must be an array or an object",
            ),
            (
                r#"{"workspaces":{"packages":7}}"#,
                "`workspaces.packages` must be an array",
            ),
            (
                r#"{"workspaces":{}}"#,
                "`workspaces` object must contain a `packages` array",
            ),
        ] {
            let tmp = TempDir::new().unwrap();
            fs::write(tmp.path().join("package.json"), content).unwrap();
            let error = ProjectSnapshot::create(tmp.path())
                .err()
                .expect("invalid package.json must make snapshot discovery inconclusive");
            assert!(error.contains(expected), "{content}: {error}");
        }

        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"workspaces":[7,"packages/*"]}"#,
        )
        .unwrap();
        let source = open_test_directory(tmp.path());
        let mut inputs = Vec::new();
        let mut budget = SnapshotBudget {
            entries: MAX_CONFINEMENT_ENTRIES - 2,
            ..SnapshotBudget::default()
        };
        let error = collect_snapshot_manifest_inputs(&source, &mut inputs, &mut budget)
            .expect_err("wrong-typed workspace entries must fail after declaration charging");

        assert!(
            error.contains("workspace entries must be strings"),
            "{error}"
        );
        assert_eq!(budget.entries, MAX_CONFINEMENT_ENTRIES);
    }

    #[test]
    fn snapshot_copies_the_exact_manifest_bytes_charged_during_discovery() {
        let source_temp = TempDir::new().unwrap();
        let destination_temp = TempDir::new().unwrap();
        let original = b"[workspace]\nmembers = []\n";
        fs::write(source_temp.path().join("Cargo.toml"), original).unwrap();
        let source = open_test_directory(source_temp.path());
        let destination = open_test_directory(destination_temp.path());
        let mut budget = SnapshotBudget {
            max_bytes: original.len() as u64,
            ..SnapshotBudget::default()
        };

        let loaded =
            read_capability_text_if_exists(&source, Path::new("Cargo.toml"), &mut budget).unwrap();
        assert!(loaded.is_some());
        fs::write(
            source_temp.path().join("Cargo.toml"),
            vec![b'x'; MAX_PROJECT_FILE_BYTES as usize],
        )
        .unwrap();

        copy_preloaded_snapshot_files(&destination, &mut budget).unwrap();
        copy_snapshot_directory(
            &source,
            &destination,
            Path::new("."),
            &HashSet::new(),
            &[PathBuf::from("Cargo.toml")],
            &mut budget,
        )
        .unwrap();

        assert_eq!(
            fs::read(destination_temp.path().join("Cargo.toml")).unwrap(),
            original
        );
        assert_eq!(budget.bytes, original.len() as u64);
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_manifest_reader_rejects_special_files_without_blocking() {
        use std::process::Command;
        use std::time::{Duration, Instant};

        let tmp = TempDir::new().unwrap();
        assert!(
            Command::new("mkfifo")
                .arg(tmp.path().join("Cargo.toml"))
                .status()
                .unwrap()
                .success()
        );
        let source = open_test_directory(tmp.path());
        let mut budget = SnapshotBudget::default();
        let started = Instant::now();

        let error = read_capability_text_if_exists(&source, Path::new("Cargo.toml"), &mut budget)
            .expect_err("special-file manifests must fail before a blocking open");

        assert!(error.contains("regular file"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[cfg(unix)]
    #[test]
    fn generic_snapshot_rejects_fifo_and_socket_sources_without_blocking() {
        use std::os::unix::net::UnixListener;
        use std::process::Command;
        use std::time::{Duration, Instant};

        let fifo = setup_project();
        assert!(
            Command::new("mkfifo")
                .arg(fifo.path().join("src/special.rs"))
                .status()
                .unwrap()
                .success()
        );
        let started = Instant::now();
        let fifo_error = ProjectSnapshot::create(fifo.path())
            .err()
            .expect("a configured FIFO source must fail the generic snapshot");
        assert!(
            fifo_error.contains("regular file or directory"),
            "{fifo_error}"
        );
        assert!(started.elapsed() < Duration::from_secs(2));

        let socket = setup_project();
        match UnixListener::bind(socket.path().join("src/special.rs")) {
            Ok(_listener) => {
                let socket_error = ProjectSnapshot::create(socket.path())
                    .err()
                    .expect("a configured socket source must fail the generic snapshot");
                assert!(
                    socket_error.contains("regular file or directory"),
                    "{socket_error}"
                );
            }
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {}
            Err(error) => panic!("cannot create generic snapshot socket fixture: {error}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn generic_snapshot_reader_rejects_fifo_symlink_and_regular_replacements() {
        use std::os::unix::fs::symlink;
        use std::process::Command;
        use std::time::{Duration, Instant};

        const ATTACKER_BYTES: &str = "GENERIC_SNAPSHOT_ATTACKER_BYTES";
        for replacement in ["fifo", "symlink", "regular"] {
            let tmp = setup_project();
            let root = tmp.path();
            let source_path = root.join("src/lib.rs");
            let attacker_path = root.join("attacker.rs");
            fs::write(&source_path, "pub fn retained() {}\n").unwrap();
            fs::write(&attacker_path, ATTACKER_BYTES).unwrap();
            let source = open_test_directory(root);
            let started = Instant::now();

            let result =
                read_snapshot_project_file_with_hook(&source, Path::new("src/lib.rs"), || {
                    fs::rename(&source_path, root.join("src/original.rs")).unwrap();
                    match replacement {
                        "fifo" => {
                            assert!(
                                Command::new("mkfifo")
                                    .arg(&source_path)
                                    .status()
                                    .unwrap()
                                    .success()
                            );
                        }
                        "symlink" => symlink(&attacker_path, &source_path).unwrap(),
                        "regular" => fs::rename(&attacker_path, &source_path).unwrap(),
                        _ => unreachable!(),
                    }
                });

            let error = result.expect_err("every post-open path replacement must fail closed");
            assert!(
                error.contains("changed during inspection"),
                "{replacement}: {error}"
            );
            assert!(!error.contains(ATTACKER_BYTES), "{replacement}: {error}");
            assert!(
                started.elapsed() < Duration::from_secs(2),
                "{replacement} replacement blocked the retained reader"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_rejects_directory_replacement_after_enumeration_before_recursion() {
        let source_temp = setup_project();
        let destination_temp = TempDir::new().unwrap();
        let source_root = source_temp.path();
        let original = source_root.join("src");
        let retained = source_root.join("retained-src");
        let destination = open_test_directory(destination_temp.path());
        let source = open_test_directory(source_root);
        let mut budget = SnapshotBudget::default();
        let mut replaced = false;

        let error = copy_snapshot_directory_with_hook(
            &source,
            &destination,
            Path::new("."),
            &HashSet::new(),
            &[PathBuf::from("src"), PathBuf::from("specs")],
            &mut budget,
            |relative| {
                if relative == Path::new(".") && !replaced {
                    fs::rename(&original, &retained).unwrap();
                    fs::create_dir(&original).unwrap();
                    fs::write(original.join("attacker.rs"), "pub fn attacker() {}\n").unwrap();
                    replaced = true;
                }
                Ok(())
            },
        )
        .expect_err("a post-enumeration directory replacement must fail closed");

        assert!(
            error.contains("changed during snapshot traversal"),
            "{error}"
        );
        assert!(error.contains("src"), "{error}");
        assert!(!destination_temp.path().join("src/attacker.rs").exists());
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_preflight_rejects_special_and_linked_gradle_manifests_without_blocking() {
        use std::os::unix::fs::symlink;
        use std::os::unix::net::UnixListener;
        use std::process::Command;
        use std::time::{Duration, Instant};

        let fifo = TempDir::new().unwrap();
        assert!(
            Command::new("mkfifo")
                .arg(fifo.path().join("build.gradle.kts"))
                .status()
                .unwrap()
                .success()
        );
        let started = Instant::now();
        let error = ProjectSnapshot::create(fifo.path())
            .err()
            .expect("a Gradle FIFO must fail before a blocking snapshot open");
        assert!(error.contains("regular file"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(2));

        let socket = TempDir::new().unwrap();
        match UnixListener::bind(socket.path().join("settings.gradle.kts")) {
            Ok(_listener) => {
                let error = ProjectSnapshot::create(socket.path())
                    .err()
                    .expect("a Gradle socket must fail snapshot preflight");
                assert!(error.contains("regular file"), "{error}");
            }
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {}
            Err(error) => panic!("cannot create Gradle socket fixture: {error}"),
        }

        for manifest in ["build.gradle.kts", "settings.gradle.kts"] {
            let linked = TempDir::new().unwrap();
            fs::write(linked.path().join("real.gradle"), "plugins {}\n").unwrap();
            symlink(
                linked.path().join("real.gradle"),
                linked.path().join(manifest),
            )
            .unwrap();
            let error = ProjectSnapshot::create(linked.path())
                .err()
                .expect("a linked Gradle manifest must fail snapshot preflight");
            assert!(error.contains("symlink or reparse point"), "{error}");
        }
    }

    #[test]
    fn snapshot_preflight_applies_the_gradle_limit_before_discovery() {
        for manifest in [
            "build.gradle.kts",
            "build.gradle",
            "settings.gradle.kts",
            "settings.gradle",
        ] {
            let tmp = TempDir::new().unwrap();
            fs::write(
                tmp.path().join(manifest),
                vec![b' '; MAX_GRADLE_MANIFEST_BYTES as usize + 1],
            )
            .unwrap();

            let error = ProjectSnapshot::create(tmp.path())
                .err()
                .expect("an oversized Gradle manifest must fail snapshot preflight");
            assert!(error.contains("4 MiB"), "{manifest}: {error}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn gradle_manifest_path_replacement_cannot_block_or_change_preloaded_bytes() {
        use std::process::Command;
        use std::time::{Duration, Instant};

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
        let source = open_test_directory(root);
        let mut budget = SnapshotBudget::default();
        let started = Instant::now();

        let result = read_capability_text_if_exists_with_limit_and_hook(
            &source,
            Path::new("build.gradle.kts"),
            &mut budget,
            MAX_GRADLE_MANIFEST_BYTES,
            || {
                fs::rename(
                    root.join("build.gradle.kts"),
                    root.join("original.gradle.kts"),
                )
                .unwrap();
                assert!(
                    Command::new("mkfifo")
                        .arg(root.join("build.gradle.kts"))
                        .status()
                        .unwrap()
                        .success()
                );
            },
        );

        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn snapshot_manifest_reader_binds_the_retained_file_identity() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        fs::write(
            root.join("replacement.toml"),
            "[workspace]\nmembers = [\"outside\"]\n",
        )
        .unwrap();
        let source = open_test_directory(root);
        let mut budget = SnapshotBudget::default();

        let result = read_capability_text_if_exists_with_hook(
            &source,
            Path::new("Cargo.toml"),
            &mut budget,
            || {
                fs::rename(root.join("Cargo.toml"), root.join("original.toml")).unwrap();
                fs::rename(root.join("replacement.toml"), root.join("Cargo.toml")).unwrap();
            },
        );

        assert!(
            result.is_err(),
            "a manifest path replaced after the retained open must be rejected"
        );
    }

    #[test]
    fn selected_config_directory_identity_stays_bound_to_the_pre_open_observation() {
        assert!(snapshot_configuration_identities_match(
            &1_u64,
            &[Some(1), Some(1), Some(1)]
        ));
        assert!(!snapshot_configuration_identities_match(
            &1_u64,
            &[Some(2), Some(2), Some(2)]
        ));
        assert!(!snapshot_configuration_identities_match(
            &1_u64,
            &[Some(1), None, Some(1)]
        ));
    }

    #[test]
    fn selected_config_parent_open_binds_the_pre_open_identity() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("selected")).unwrap();
        fs::create_dir(root.join("replacement")).unwrap();
        let source = open_test_directory(root);

        let result = open_snapshot_configuration_directory_with_hook(
            &source,
            OsStr::new("selected"),
            Path::new("selected"),
            || {
                fs::rename(root.join("selected"), root.join("original")).unwrap();
                fs::rename(root.join("replacement"), root.join("selected")).unwrap();
            },
        );

        let error =
            result.expect_err("a replacement opened after metadata inspection must be rejected");
        assert!(error.contains("changed during inspection"), "{error}");
    }

    #[test]
    fn selected_config_open_binds_the_retained_file_identity() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::write(
            root.join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();
        fs::write(
            root.join("other.json"),
            r#"{"specsDir":"other","sourceDirs":["other"]}"#,
        )
        .unwrap();
        let source = open_test_directory(root);

        let result =
            read_snapshot_configuration_with_hook(&source, Path::new("specsync.json"), || {
                fs::rename(root.join("specsync.json"), root.join("original.json")).unwrap();
                fs::rename(root.join("other.json"), root.join("specsync.json")).unwrap();
            });

        assert!(
            result.is_err(),
            "a regular selected config replaced after discovery must not be read"
        );
    }

    #[test]
    fn selected_config_read_revalidates_each_parent_identity() {
        const ERROR_ACCESS_DENIED: i32 = 5;
        const ERROR_SHARING_VIOLATION: i32 = 32;

        for replaced_parent in ["selected", "selected/nested"] {
            let tmp = TempDir::new().unwrap();
            let root = tmp.path();
            fs::create_dir_all(root.join("selected/nested")).unwrap();
            fs::write(
                root.join("selected/nested/specsync.json"),
                r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
            )
            .unwrap();
            let replacement = root.join("replacement");
            if replaced_parent == "selected" {
                fs::create_dir_all(replacement.join("nested")).unwrap();
                fs::write(
                    replacement.join("nested/specsync.json"),
                    r#"{"specsDir":"outside","sourceDirs":["outside"]}"#,
                )
                .unwrap();
            } else {
                fs::create_dir(&replacement).unwrap();
                fs::write(
                    replacement.join("specsync.json"),
                    r#"{"specsDir":"outside","sourceDirs":["outside"]}"#,
                )
                .unwrap();
            }
            let selected_parent = root.join(replaced_parent);
            let original_identity =
                directory_identity(&open_test_directory(&selected_parent)).unwrap();
            let source = open_test_directory(root);
            let replacement_denied = std::cell::Cell::new(false);

            let result = read_snapshot_configuration_with_hook(
                &source,
                Path::new("selected/nested/specsync.json"),
                || {
                    let original_parent = if replaced_parent == "selected" {
                        root.join("original-selected")
                    } else {
                        root.join("selected/original-nested")
                    };
                    match fs::rename(&selected_parent, &original_parent) {
                        Ok(()) => fs::rename(&replacement, &selected_parent).unwrap(),
                        Err(error)
                            if cfg!(windows)
                                && matches!(
                                    error.raw_os_error(),
                                    Some(ERROR_ACCESS_DENIED | ERROR_SHARING_VIOLATION)
                                ) =>
                        {
                            replacement_denied.set(true);
                        }
                        Err(error) => panic!(
                            "cannot rename retained selected-config parent {replaced_parent}: {error}"
                        ),
                    }
                },
            );

            if replacement_denied.get() {
                let bytes = result
                    .expect("Windows sharing protection must preserve the retained parent identity")
                    .expect("the retained configuration must remain present");
                assert_eq!(
                    bytes,
                    r#"{"specsDir":"specs","sourceDirs":["src"]}"#.as_bytes(),
                    "{replaced_parent}: retained capability read replacement bytes"
                );
                let original_parent = if replaced_parent == "selected" {
                    root.join("original-selected")
                } else {
                    root.join("selected/original-nested")
                };
                assert!(selected_parent.is_dir());
                assert!(!original_parent.exists());
                assert!(replacement.is_dir());
                assert_eq!(
                    directory_identity(&open_test_directory(&selected_parent)).unwrap(),
                    original_identity,
                    "{replaced_parent}: refused replacement changed the original identity"
                );
                drop(source);
                fs::rename(&selected_parent, &original_parent).expect(
                    "the same rename must succeed after selected-config capabilities are released",
                );
                fs::rename(&original_parent, &selected_parent)
                    .expect("the original selected-config parent must restore");
                assert_eq!(
                    directory_identity(&open_test_directory(&selected_parent)).unwrap(),
                    original_identity,
                    "{replaced_parent}: negative-control rename changed the original identity"
                );
                assert!(replacement.is_dir());
                continue;
            }

            let error = result
                .expect_err("every selected-config parent replacement must fail after the read");
            assert!(
                error.contains("changed while the configuration was being read"),
                "{replaced_parent}: {error}"
            );
        }
    }

    #[test]
    fn snapshot_includes_python_package_derived_under_ignored_directory() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("vendor")).unwrap();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("pyproject.toml"),
            "[project]\nname = \"vendor\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(tmp.path().join("vendor/module.py"), "def public(): pass\n").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        assert!(snapshot.root().join("vendor/module.py").is_file());
    }

    #[test]
    fn snapshot_includes_all_standard_gradle_module_forms_under_ignored_directories() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("build.gradle"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle"),
            r#"
include ':vendor:groovy',
        ':nested:second'
include(
    ":kotlin:member",
    ':override'
)
project(':override').projectDir = file('vendor/custom')
"#,
        )
        .unwrap();
        for path in [
            "vendor/groovy/src/main/kotlin/one.kt",
            "nested/second/src/main/java/Two.java",
            "kotlin/member/src/main/kotlin/three.kt",
            "vendor/custom/src/main/kotlin/four.kt",
        ] {
            let path = tmp.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "public class Module {}\n").unwrap();
        }

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

        assert!(
            snapshot
                .root()
                .join("vendor/groovy/src/main/kotlin/one.kt")
                .is_file()
        );
        assert!(
            snapshot
                .root()
                .join("nested/second/src/main/java/Two.java")
                .is_file()
        );
        assert!(
            snapshot
                .root()
                .join("kotlin/member/src/main/kotlin/three.kt")
                .is_file()
        );
        assert!(
            snapshot
                .root()
                .join("vendor/custom/src/main/kotlin/four.kt")
                .is_file()
        );
    }

    // --- handle_initialize ---

    #[test]
    fn test_handle_initialize_response_format() {
        let resp = handle_initialize(Some(json!(1)));
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(resp["result"]["serverInfo"]["name"], "specsync");
        assert!(resp["result"]["capabilities"]["tools"].is_object());
        assert!(resp["result"]["capabilities"]["resources"].is_object());
    }

    #[test]
    fn test_handle_initialize_null_id() {
        let resp = handle_initialize(None);
        assert_eq!(resp["jsonrpc"], "2.0");
        assert!(resp["result"]["protocolVersion"].is_string());
    }

    #[test]
    fn test_handle_initialize_string_id() {
        let resp = handle_initialize(Some(json!("req-42")));
        assert_eq!(resp["id"], "req-42");
    }

    // --- handle_tools_list ---

    #[test]
    fn test_handle_tools_list_returns_all_tools() {
        let resp = handle_tools_list(Some(json!(2)), true);
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn test_handle_tools_list_defaults_to_read_only_tools() {
        let resp = handle_tools_list(Some(json!(2)), false);
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 5);
        assert!(
            tools
                .iter()
                .all(|tool| tool["name"] != "specsync_generate" && tool["name"] != "specsync_init")
        );
    }

    #[test]
    fn test_handle_tools_list_tool_names() {
        let resp = handle_tools_list(Some(json!(1)), true);
        let tools = resp["result"]["tools"].as_array().unwrap();
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"specsync_check"));
        assert!(names.contains(&"specsync_coverage"));
        assert!(names.contains(&"specsync_generate"));
        assert!(names.contains(&"specsync_list_specs"));
        assert!(names.contains(&"specsync_init"));
        assert!(names.contains(&"specsync_score"));
        assert!(names.contains(&"specsync_issues"));
    }

    #[test]
    fn test_handle_tools_list_all_have_schemas() {
        let resp = handle_tools_list(Some(json!(1)), true);
        let tools = resp["result"]["tools"].as_array().unwrap();
        for tool in tools {
            assert!(
                tool["inputSchema"].is_object(),
                "Tool {} missing inputSchema",
                tool["name"]
            );
            assert_eq!(tool["inputSchema"]["type"], "object");
            assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        }
    }

    // --- handle_tools_call ---

    #[test]
    fn test_handle_tools_call_unknown_tool() {
        let tmp = setup_project();
        let params = json!({ "name": "nonexistent_tool", "arguments": {} });
        let resp = handle_tools_call(Some(json!(1)), &params, tmp.path(), false);
        assert_eq!(resp["result"]["isError"], true);
        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Unknown tool"));
    }

    #[test]
    fn test_handle_tools_call_custom_root() {
        let tmp = setup_project();
        let server_root = tmp.path().canonicalize().unwrap();
        // coverage tool should work with empty specs
        let params = json!({
            "name": "specsync_coverage",
            "arguments": { "root": server_root.to_string_lossy() }
        });
        let resp = handle_tools_call(Some(json!(1)), &params, &server_root, false);
        assert!(!resp["result"]["isError"].as_bool().unwrap_or(false));
    }

    // --- load_and_discover ---

    #[test]
    fn test_load_and_discover_empty_allowed() {
        let tmp = setup_project();
        let result = load_and_discover(tmp.path(), true);
        assert!(result.is_ok());
        let (config, specs) = result.unwrap();
        assert_eq!(config.specs_dir, "specs");
        assert!(specs.is_empty());
    }

    #[test]
    fn test_load_and_discover_empty_not_allowed() {
        let tmp = setup_project();
        let result = load_and_discover(tmp.path(), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No spec files found"));
    }

    #[test]
    fn test_load_and_discover_filters_private_specs() {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("_private");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("_private.spec.md"),
            "---\nmodule: private\n---",
        )
        .unwrap();

        let result = load_and_discover(tmp.path(), true);
        assert!(result.is_ok());
        let (_config, specs) = result.unwrap();
        // Private specs (starting with _) should be filtered out
        assert!(specs.is_empty());
    }

    #[test]
    fn test_load_and_discover_finds_real_specs() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let result = load_and_discover(tmp.path(), false);
        assert!(result.is_ok());
        let (_config, specs) = result.unwrap();
        assert_eq!(specs.len(), 1);
    }

    // --- tool_init ---

    #[test]
    fn test_tool_init_creates_config() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();

        let directory = open_test_directory(tmp.path());
        let result = tool_init(tmp.path(), &directory);
        assert!(
            result.is_ok(),
            "unexpected initialization error: {result:?}"
        );
        let val = result.unwrap();
        assert_eq!(val["created"], true);
        assert!(tmp.path().join("specsync.json").exists());
        assert_no_mcp_transaction_debris(tmp.path());
    }

    #[test]
    fn test_tool_init_already_exists() {
        let tmp = setup_project();
        let directory = open_test_directory(tmp.path());
        let result = tool_init(tmp.path(), &directory);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["created"], false);
        assert!(val["message"].as_str().unwrap().contains("already exists"));
    }

    #[test]
    fn test_generation_outcome_rejects_missing_required_output() {
        let tmp = TempDir::new().unwrap();
        let expected = vec![(
            "missing".to_string(),
            tmp.path().join("specs/missing/missing.spec.md"),
        )];
        let outcome = crate::generator::GenerationOutcome {
            generated: 1,
            generated_paths: vec!["specs/missing/missing.spec.md".to_string()],
            skipped_no_files: Vec::new(),
        };

        let error = validate_generation_outcome(tmp.path(), &expected, &outcome).unwrap_err();
        assert!(error.contains("did not create the required spec"));
    }

    #[test]
    fn test_generation_outcome_rejects_incomplete_count() {
        let tmp = TempDir::new().unwrap();
        let expected = vec![(
            "missing".to_string(),
            tmp.path().join("specs/missing/missing.spec.md"),
        )];
        let outcome = crate::generator::GenerationOutcome::default();

        let error = validate_generation_outcome(tmp.path(), &expected, &outcome).unwrap_err();
        assert!(error.contains("created 0 of 1 required specs"));
    }

    // --- tool_coverage ---

    #[test]
    fn test_tool_coverage_empty_project() {
        let tmp = setup_project();
        let result = tool_coverage(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        // Same correction as `test_resource_coverage_empty`: zero files
        // measured is `null`, not full coverage (#582).
        assert!(val["file_coverage"].is_null(), "{val}");
        assert!(val["loc_coverage"].is_null(), "{val}");
        assert_eq!(val["files_total"], 0);
    }

    #[test]
    fn test_tool_coverage_with_unspecced_files() {
        let tmp = setup_project();
        // Create a source file without a spec
        std::fs::write(tmp.path().join("src").join("main.rs"), "fn main() {}").unwrap();

        let result = tool_coverage(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val["files_total"].as_u64().unwrap() > 0);
    }

    #[test]
    fn malformed_gradle_makes_every_mcp_coverage_consumer_inconclusive() {
        let tmp = setup_project();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            "include(\"member\"\n",
        )
        .unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        let directory = open_test_directory(tmp.path());

        let errors = [
            resource_coverage(tmp.path()).unwrap_err(),
            tool_check(tmp.path(), &json!({})).unwrap_err(),
            tool_coverage(tmp.path()).unwrap_err(),
            tool_generate(tmp.path(), &directory, &json!({})).unwrap_err(),
        ];

        for error in errors {
            assert!(error.contains("Gradle"), "unexpected error: {error}");
            assert!(
                error.contains("inconclusive") || error.contains("Cannot parse"),
                "malformed Gradle must not become a compatibility coverage report: {error}"
            );
        }
    }

    // --- tool_list_specs ---

    #[test]
    fn test_tool_list_specs_empty() {
        let tmp = setup_project();
        let result = tool_list_specs(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["count"], 0);
        assert!(val["specs"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_tool_list_specs_with_frontmatter() {
        let spec_content = "---\nmodule: auth\nversion: 2.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let result = tool_list_specs(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["count"], 1);
        let spec = &val["specs"][0];
        assert_eq!(spec["module"], "auth");
        assert_eq!(spec["version"], "2.0.0");
        assert_eq!(spec["status"], "stable");
    }

    #[test]
    fn test_tool_list_specs_malformed_frontmatter() {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("bad");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join("bad.spec.md"), "no frontmatter here").unwrap();

        let result = tool_list_specs(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["count"], 1);
        let spec = &val["specs"][0];
        assert!(spec["module"].is_null());
    }

    // --- tool_check ---

    #[test]
    fn test_tool_check_strict_mode() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n";
        let tmp = setup_project_with_spec("auth", spec_content);
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let result_normal = tool_check(tmp.path(), &json!({ "strict": false }));
        assert!(result_normal.is_ok());

        let result_strict = tool_check(tmp.path(), &json!({ "strict": true }));
        assert!(result_strict.is_ok());
    }

    #[test]
    fn test_tool_check_no_specs_error() {
        let tmp = setup_project();
        let result = tool_check(tmp.path(), &json!({}));
        assert!(result.is_err());
    }

    // --- tool_score ---

    #[test]
    fn test_tool_score_no_specs_error() {
        let tmp = setup_project();
        let result = tool_score(tmp.path(), true);
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_score_with_spec() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n\n# Invariants\nNone\n\n# Behavioral Examples\nNone\n\n# Error Cases\nNone\n\n# Dependencies\nNone\n\n# Change Log\nNone\n";
        let tmp = setup_project_with_spec("auth", spec_content);
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let result = tool_score(tmp.path(), true);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val["average_score"].as_f64().unwrap() >= 0.0);
        assert!(val["grade"].is_string());
        assert_eq!(val["total_specs"], 1);
        assert!(val["distribution"].is_object());
    }

    // --- tool_generate ---

    #[test]
    fn test_tool_generate_no_uncovered() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
        let tmp = setup_project_with_spec("auth", spec_content);
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        let directory = open_test_directory(tmp.path());
        let result = tool_generate(snapshot.root(), &directory, &json!({}));
        assert!(result.is_ok());
    }

    #[test]
    fn test_tool_generate_creates_spec() {
        let tmp = setup_project();
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        let directory = open_test_directory(tmp.path());
        let result = tool_generate(snapshot.root(), &directory, &json!({}));
        assert!(result.is_ok(), "unexpected generation error: {result:?}");
        let val = result.unwrap();
        assert_eq!(val["count"].as_u64(), Some(1));
        let generated = val["generated"].as_array().unwrap();
        assert_eq!(generated.len(), 1);
        let generated_path = std::path::Path::new(generated[0].as_str().unwrap());
        assert!(!generated_path.is_absolute());
        assert!(tmp.path().join(generated_path).is_file());
        assert!(
            generated_path.ends_with(
                std::path::Path::new("specs")
                    .join("auth")
                    .join("auth.spec.md")
            )
        );
        assert_no_mcp_transaction_debris(tmp.path());
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

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum PostLinkDestinationFailure {
        Open,
        IdentityRead,
        IdentityMismatch,
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

    #[test]
    fn file_rollback_quarantines_a_replacement_swapped_after_verification() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("generated.spec.md"), "generated\n").unwrap();
        let parent = open_test_directory(tmp.path());
        let expected = file_identity(&parent.open("generated.spec.md").unwrap()).unwrap();

        let error = remove_identity_bound_file_with_hook(
            &parent,
            Path::new("generated.spec.md"),
            Path::new("generated.spec.md"),
            expected,
            |parent, name| {
                parent.remove_file(name).unwrap();
                parent.write(name, b"replacement\n").unwrap();
            },
        )
        .unwrap_err();

        assert!(error.contains("quarantined replacement was preserved"));
        assert!(!tmp.path().join("generated.spec.md").exists());
        assert!(quarantine_contains(&parent, b"replacement\n"));
    }

    #[test]
    fn file_identity_rejects_changed_bytes_on_the_same_filesystem_entry() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("generated.spec.md"), "generated\n").unwrap();
        let parent = open_test_directory(tmp.path());
        let original = file_identity(&parent.open("generated.spec.md").unwrap()).unwrap();

        fs::write(tmp.path().join("generated.spec.md"), "replacement\n").unwrap();
        let replacement = file_identity(&parent.open("generated.spec.md").unwrap()).unwrap();

        assert_ne!(original, replacement);
    }

    #[test]
    fn file_identity_digest_fails_closed_at_the_generated_output_bound() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("oversized"), b"123456789").unwrap();
        let parent = open_test_directory(tmp.path());
        let file = parent.open("oversized").unwrap();

        let error = file_content_digest_with_limit(&file, 8).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("8-byte limit"));
    }

    fn quarantine_contains(parent: &Dir, expected: &[u8]) -> bool {
        parent
            .read_dir(".")
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".specsync-mcp-quarantine-")
            })
            .any(|entry| {
                let Ok(directory) = entry.open_dir() else {
                    return false;
                };
                if directory.read("entry").is_ok_and(|bytes| bytes == expected) {
                    return true;
                }
                let Ok(nested) = directory.open_dir("entry") else {
                    return false;
                };
                nested
                    .read("replacement.txt")
                    .is_ok_and(|bytes| bytes == expected)
            })
    }

    #[test]
    fn generation_rejects_an_unbounded_output_count_before_reading_files() {
        let tmp = TempDir::new().unwrap();
        let expected = (0..=MAX_GENERATED_SPECS)
            .map(|index| {
                (
                    format!("module-{index}"),
                    tmp.path()
                        .join(format!("specs/module-{index}/module-{index}.spec.md")),
                )
            })
            .collect::<Vec<_>>();
        let outcome = crate::generator::GenerationOutcome {
            generated: expected.len(),
            generated_paths: Vec::new(),
            skipped_no_files: Vec::new(),
        };

        let error = validate_generation_outcome(tmp.path(), &expected, &outcome).unwrap_err();
        assert!(error.contains("1000-spec output limit"));
    }

    #[test]
    fn generation_rejects_cumulative_output_bytes_before_publication() {
        let tmp = TempDir::new().unwrap();
        let first = tmp.path().join("specs/first/first.spec.md");
        let second = tmp.path().join("specs/second/second.spec.md");
        fs::create_dir_all(first.parent().unwrap()).unwrap();
        fs::create_dir_all(second.parent().unwrap()).unwrap();
        fs::write(&first, "12345").unwrap();
        fs::write(&second, "67890").unwrap();
        let expected = vec![("first".to_string(), first), ("second".to_string(), second)];
        let outcome = crate::generator::GenerationOutcome {
            generated: 2,
            generated_paths: vec![
                "specs/first/first.spec.md".to_string(),
                "specs/second/second.spec.md".to_string(),
            ],
            skipped_no_files: Vec::new(),
        };

        assert_eq!(MAX_GENERATED_OUTPUT_BYTES, 64 * 1024 * 1024);
        let error = validate_generation_outcome_with_limits(
            tmp.path(),
            &expected,
            &outcome,
            MAX_GENERATED_SPECS,
            8,
        )
        .unwrap_err();
        assert!(error.contains("cumulative output limit"));
    }

    #[test]
    fn generation_rejects_an_oversized_result_during_response_preflight() {
        let result = json!({
            "generated": ["specs/a/very-long-generated-spec-name.spec.md"],
            "count": 1,
        });

        let error = validate_tool_content_response_size_with_limit(&result, 64).unwrap_err();
        assert!(error.contains("bounded response limit"));
    }

    #[test]
    fn test_tool_generate_rejects_retired_ai_arguments_without_echoing_values() {
        let tmp = setup_project();
        let secret = "sk-do-not-echo";
        let directory = open_test_directory(tmp.path());
        let error =
            tool_generate(tmp.path(), &directory, &json!({ "apiKey": secret })).unwrap_err();
        assert!(error.contains("removed in spec-sync 5.0"));
        assert!(!error.contains(secret));
    }

    #[test]
    fn test_tool_generate_rejects_unknown_arguments() {
        let tmp = setup_project();
        let directory = open_test_directory(tmp.path());
        let error =
            tool_generate(tmp.path(), &directory, &json!({ "unexpected": true })).unwrap_err();
        assert!(error.contains("Unknown MCP generate argument `unexpected`"));
    }

    // --- JSONRPC response structure ---

    #[test]
    fn test_tools_call_success_response_structure() {
        let tmp = setup_project();
        let params = json!({ "name": "specsync_coverage", "arguments": {} });
        let resp = handle_tools_call(Some(json!(42)), &params, tmp.path(), false);

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 42);
        assert!(resp["result"]["content"].is_array());
        assert_eq!(resp["result"]["content"][0]["type"], "text");
    }

    #[test]
    fn test_tools_call_error_response_structure() {
        let tmp = setup_project();
        let params = json!({ "name": "bogus", "arguments": {} });
        let resp = handle_tools_call(Some(json!(99)), &params, tmp.path(), false);

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 99);
        assert_eq!(resp["result"]["isError"], true);
    }

    // --- handle_resources_list ---

    #[test]
    fn test_handle_resources_list_returns_resources() {
        let resp = handle_resources_list(Some(json!(1)));
        assert_eq!(resp["jsonrpc"], "2.0");
        let resources = resp["result"]["resources"].as_array().unwrap();
        assert_eq!(resources.len(), 4);
        let uris: Vec<&str> = resources
            .iter()
            .map(|r| r["uri"].as_str().unwrap())
            .collect();
        assert!(uris.contains(&"specsync:///specs"));
        assert!(uris.contains(&"specsync:///graph"));
        assert!(uris.contains(&"specsync:///config"));
        assert!(uris.contains(&"specsync:///coverage"));
    }

    #[test]
    fn test_handle_resources_list_has_templates() {
        let resp = handle_resources_list(Some(json!(1)));
        let templates = resp["result"]["resourceTemplates"].as_array().unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(
            templates[0]["uriTemplate"].as_str().unwrap(),
            "specsync:///specs/{module}"
        );
    }

    // --- handle_resources_read ---

    #[test]
    fn test_resource_specs_list_empty() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///specs" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        assert_eq!(resp["jsonrpc"], "2.0");
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["count"], 0);
    }

    #[test]
    fn test_resource_specs_list_with_spec() {
        let spec_content = "---\nmodule: auth\nversion: 2.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let params = json!({ "uri": "specsync:///specs" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["count"], 1);
        assert_eq!(parsed["specs"][0]["module"], "auth");
        assert!(parsed["specs"][0]["score"].is_number());
        assert_eq!(parsed["git_freshness_available"], false);
        assert_eq!(parsed["specs"][0]["git_freshness_available"], false);
    }

    #[test]
    fn test_resource_spec_by_module() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let params = json!({ "uri": "specsync:///specs/auth" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        assert!(text.contains("module: auth"));
        assert_eq!(
            resp["result"]["contents"][0]["mimeType"].as_str().unwrap(),
            "text/markdown"
        );
    }

    #[test]
    fn test_resource_spec_by_module_not_found() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///specs/nonexistent" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("No spec found")
        );
    }

    #[test]
    fn test_resource_graph_empty() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///graph" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["module_count"], 0);
        assert_eq!(parsed["edge_count"], 0);
    }

    #[test]
    fn test_resource_config() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///config" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["specs_dir"], "specs");
    }

    #[test]
    fn test_resource_coverage_empty() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///coverage" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        // This test used to assert the bug: an empty project has no source
        // file, so there is no percentage to report. `100.0` told an agent the
        // project was fully specced when nothing had been looked at (#582).
        assert!(parsed["file_coverage_percent"].is_null(), "{parsed}");
        assert!(parsed["loc_coverage_percent"].is_null(), "{parsed}");
        assert_eq!(parsed["files_total"], 0, "{parsed}");
    }

    #[test]
    fn test_resource_unknown_uri() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///bogus" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Unknown resource URI")
        );
    }

    #[test]
    fn test_run_mcp_server_rejects_a_non_directory_root() {
        let tmp = TempDir::new().unwrap();
        let file_root = tmp.path().join("not-a-directory");
        fs::write(&file_root, "not a project root").unwrap();

        let error = run_mcp_server(&file_root, false).unwrap_err();
        assert!(error.contains("not a directory"));
    }

    #[cfg(unix)]
    #[test]
    fn test_retained_server_capability_survives_root_path_swap() {
        use std::os::unix::fs::symlink;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("server");
        let moved = tmp.path().join("server-moved");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn local() {}\n").unwrap();
        fs::write(outside.join("victim"), "outside\n").unwrap();
        let directory = open_test_directory(&root);
        fs::rename(&root, &moved).unwrap();
        symlink(&outside, &root).unwrap();

        let response = handle_tools_call_with_directory(
            Some(json!(1)),
            &json!({
                "name": "specsync_init",
                "arguments": {}
            }),
            &root,
            &root,
            &directory,
            true,
        );

        assert_eq!(response["result"]["isError"].as_bool(), None);
        assert!(moved.join("specsync.json").is_file());
        assert!(!outside.join("specsync.json").exists());
        assert_eq!(
            fs::read_to_string(outside.join("victim")).unwrap(),
            "outside\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_read_root_resolution_ignores_a_replacement_server_path() {
        use std::os::unix::fs::symlink;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("server");
        let moved = tmp.path().join("server-moved");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(root.join("child/src")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(
            root.join("child/specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();
        fs::write(root.join("child/src/lib.rs"), "pub fn retained() {}\n").unwrap();
        let directory = open_test_directory(&root);
        fs::rename(&root, &moved).unwrap();
        symlink(&outside, &root).unwrap();

        let retained = handle_tools_call_with_directory(
            Some(json!(1)),
            &json!({
                "name": "specsync_coverage",
                "arguments": { "root": "child" }
            }),
            &root,
            &root,
            &directory,
            false,
        );
        assert_eq!(retained["result"]["isError"], Value::Null);

        let missing_before = handle_tools_call_with_directory(
            Some(json!(2)),
            &json!({
                "name": "specsync_coverage",
                "arguments": { "root": "probe" }
            }),
            &root,
            &root,
            &directory,
            false,
        );
        fs::create_dir_all(outside.join("probe/src")).unwrap();
        fs::write(outside.join("probe/src/outside.rs"), "outside\n").unwrap();
        let missing_after = handle_tools_call_with_directory(
            Some(json!(3)),
            &json!({
                "name": "specsync_coverage",
                "arguments": { "root": "probe" }
            }),
            &root,
            &root,
            &directory,
            false,
        );
        assert_eq!(missing_before["result"]["isError"], true);
        assert_eq!(missing_after["result"]["isError"], true);
        assert_eq!(
            missing_before["result"]["content"][0]["text"],
            missing_after["result"]["content"][0]["text"]
        );
    }

    #[test]
    fn test_snapshot_excludes_mixed_case_git_metadata() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join(".GIT")).unwrap();
        fs::write(tmp.path().join(".GIT/config"), "gitdir = outside\n").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

        assert!(!snapshot.root().join(".GIT").exists());
        assert!(!snapshot.root().join(".git").exists());
        assert!(!snapshot.git_freshness_available());
    }

    #[test]
    fn test_read_mcp_line_rejects_and_drains_an_oversized_request() {
        let mut input = vec![b'x'; MAX_JSON_RPC_LINE_BYTES + 1];
        input.extend_from_slice(b"\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n");
        let mut reader = std::io::Cursor::new(input);

        let first = read_mcp_line(&mut reader).unwrap().unwrap().unwrap_err();
        assert!(first.contains("1 MiB"));
        let second = read_mcp_line(&mut reader).unwrap().unwrap().unwrap();
        assert!(second.contains("\"id\":2"));
        assert!(read_mcp_line(&mut reader).unwrap().is_none());
    }

    #[test]
    fn test_read_mcp_line_propagates_reader_failures() {
        struct FailingReader;

        impl Read for FailingReader {
            fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("reader failed"))
            }
        }

        impl BufRead for FailingReader {
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                Err(io::Error::other("reader failed"))
            }

            fn consume(&mut self, _amount: usize) {}
        }

        let error = read_mcp_line(&mut FailingReader).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(error.to_string(), "reader failed");
    }

    #[test]
    fn test_mcp_score_always_marks_git_freshness_unavailable_and_fails_closed() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuthentication behavior.\n\n# Requirements\n- Authenticate users.\n\n# Public API\n| Name | Kind | Description |\n| --- | --- | --- |\n| `login` | Function | Authenticates a user. |\n\n# Invariants\n- Credentials are validated.\n\n# Behavioral Examples\nCalling `login` validates credentials.\n\n# Error Cases\nInvalid credentials return an error.\n\n# Dependencies\nNone.\n\n# Change Log\n- 1.0.0: Initial specification.\n";
        let tmp = setup_project_with_spec("auth", spec_content);
        fs::write(tmp.path().join("src/auth.rs"), "pub fn login() {}\n").unwrap();
        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        assert!(!snapshot.git_freshness_available());
        let config = load_config(snapshot.root());
        let spec_path = snapshot.root().join("specs/auth/auth.spec.md");
        let baseline = scoring::score_spec(&spec_path, snapshot.root(), &config);
        let confined = score_spec_for_mcp(&spec_path, snapshot.root(), &config, false);

        // The snapshot is a temp copy with no `.git`, so `score_spec` itself
        // now withholds the git-freshness budget (#572) — it did not before,
        // and this wrapper was the only thing standing between an agent and 5
        // points for a question git could not be asked. The invariant that
        // matters is unchanged and now holds twice over: NOTHING in the
        // confined path awards those points. What must not happen is charging
        // for the same absence twice, so `confined` equals `baseline` here.
        assert_eq!(baseline.git_freshness, scoring::GitFreshness::Withheld);
        assert_eq!(
            confined.total, baseline.total,
            "the confined score must not be penalized a second time for one absence"
        );
        assert_eq!(confined.freshness_score, baseline.freshness_score);
        let git_criterion = confined
            .explain
            .iter()
            .find(|detail| detail.dimension == "Freshness")
            .and_then(|detail| {
                detail
                    .criteria
                    .iter()
                    .find(|criterion| criterion.name == "git_freshness")
            })
            .expect("freshness/git_freshness criterion");
        assert_eq!(
            git_criterion.points, 0,
            "the confined snapshot must award no git-freshness points"
        );
        assert!(!git_criterion.passed);
        assert!(
            confined
                .suggestions
                .iter()
                .any(|suggestion| suggestion.contains("Git history is intentionally unavailable"))
        );
        assert!(
            !confined
                .suggestions
                .iter()
                .any(|suggestion| suggestion.contains("drift is unverifiable")),
            "the confined wording replaces the generic one rather than stacking with it: {:?}",
            confined.suggestions
        );
        let output = tool_score(snapshot.root(), snapshot.git_freshness_available()).unwrap();
        assert_eq!(output["git_freshness_available"], false);
        assert_eq!(output["specs"][0]["git_freshness_available"], false);
    }

    #[test]
    fn mcp_snapshot_inside_host_worktree_does_not_discover_host_git_or_double_penalize() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuthentication behavior.\n\n# Requirements\n- Authenticate users.\n\n# Public API\n| Name | Kind | Description |\n| --- | --- | --- |\n| `login` | Function | Authenticates a user. |\n\n# Invariants\n- Credentials are validated.\n\n# Behavioral Examples\nCalling `login` validates credentials.\n\n# Error Cases\nInvalid credentials return an error.\n\n# Dependencies\nNone.\n\n# Change Log\n- 1.0.0: Initial specification.\n";
        let host = TempDir::new().unwrap();
        let project = host.path().join("project");
        fs::create_dir_all(project.join("src")).unwrap();
        fs::create_dir_all(project.join("specs/auth")).unwrap();
        let config = json!({
            "specsDir": "specs",
            "sourceDirs": ["src"],
            "requiredSections": ["Purpose", "Public API"]
        });
        fs::write(
            project.join("specsync.json"),
            serde_json::to_string_pretty(&config).unwrap(),
        )
        .unwrap();
        fs::write(project.join("src/auth.rs"), "pub fn login() {}\n").unwrap();
        fs::write(project.join("specs/auth/auth.spec.md"), spec_content).unwrap();
        init_git_repo(host.path());

        let temp_parent = host.path().join("tmp");
        fs::create_dir_all(&temp_parent).unwrap();
        let snapshot = ProjectSnapshot::create_in(&project, &temp_parent).unwrap();
        assert!(
            crate::git_utils::is_git_repo(snapshot.root()),
            "without isolation, a snapshot whose tempfile sits in the host worktree discovers that work tree"
        );
        assert!(!snapshot.git_freshness_available());

        let config = load_config(snapshot.root());
        let spec_path = snapshot.root().join("specs/auth/auth.spec.md");
        let (baseline, confined, output) = with_isolated_snapshot_git(&snapshot, || {
            let baseline = scoring::score_spec(&spec_path, snapshot.root(), &config);
            let confined = score_spec_for_mcp(&spec_path, snapshot.root(), &config, false);
            let output = tool_score(snapshot.root(), snapshot.git_freshness_available()).unwrap();
            (baseline, confined, output)
        });

        assert_eq!(baseline.git_freshness, scoring::GitFreshness::Withheld);
        assert_eq!(
            confined.total, baseline.total,
            "host walk-up must not award git freshness that the confined path then deducts again"
        );
        assert_eq!(confined.freshness_score, baseline.freshness_score);
        assert_eq!(output["git_freshness_available"], false);
        assert_eq!(output["specs"][0]["git_freshness_available"], false);
        assert!(
            confined
                .suggestions
                .iter()
                .any(|suggestion| suggestion.contains("Git history is intentionally unavailable"))
        );
    }

    #[test]
    fn test_invalid_json_rpc_envelopes_fail_before_dispatch() {
        for request in [
            json!([]),
            json!({"id": 1, "method": "ping"}),
            json!({"jsonrpc": "1.0", "id": 1, "method": "ping"}),
            json!({"jsonrpc": "2.0", "id": 1, "method": 7}),
            json!({"jsonrpc": "2.0", "id": true, "method": "ping"}),
            json!({"jsonrpc": "2.0", "id": 1, "method": "ping", "params": true}),
        ] {
            let response = validate_request_envelope(&request).unwrap_err();
            assert_eq!(response["error"]["code"], -32600);
            assert!(response["id"].is_null());
        }

        let notification = json!({"jsonrpc": "2.0", "method": "ping"});
        assert_eq!(validate_request_envelope(&notification).unwrap(), None);
        let request = json!({"jsonrpc": "2.0", "id": "safe", "method": "ping"});
        assert_eq!(
            validate_request_envelope(&request).unwrap(),
            Some(json!("safe"))
        );
    }

    #[test]
    fn test_oversized_response_is_replaced_with_a_bounded_error() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 19,
            "result": "x".repeat(MAX_JSON_RPC_RESPONSE_BYTES + 1)
        });
        let mut output = Vec::new();

        write_mcp_response(&mut output, &response).unwrap();

        assert!(output.len() < MAX_JSON_RPC_RESPONSE_BYTES);
        let bounded: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(bounded["id"], 19);
        assert_eq!(bounded["error"]["code"], -32603);
    }

    #[test]
    fn test_response_writer_failures_are_propagated() {
        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("intentional writer failure"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let error = write_mcp_response(
            &mut FailingWriter,
            &json!({"jsonrpc": "2.0", "id": 1, "result": {}}),
        )
        .unwrap_err();
        assert!(error.contains("intentional writer failure"));
    }

    #[test]
    fn test_resources_read_rejects_non_exact_arguments() {
        let tmp = setup_project();
        for params in [
            json!([]),
            json!({}),
            json!({"uri": 7}),
            json!({"uri": "specsync:///config", "extra": true}),
        ] {
            let response = handle_resources_read(Some(json!(1)), &params, tmp.path());
            assert_eq!(response["error"]["code"], -32602);
        }
    }

    #[test]
    fn test_repeated_tree_scans_share_one_confinement_budget() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
        let exclusions = HashSet::new();
        let mut entries_seen = MAX_CONFINEMENT_ENTRIES - 2;

        validate_configured_path_with_budget(
            tmp.path(),
            "src",
            "source_dirs entry",
            Some(&exclusions),
            &mut entries_seen,
        )
        .unwrap();
        let error = validate_configured_path_with_budget(
            tmp.path(),
            "src",
            "source_dirs entry",
            Some(&exclusions),
            &mut entries_seen,
        )
        .unwrap_err();

        assert!(error.contains("confinement scan exceeds"));
    }

    #[test]
    fn test_cargo_manifest_preflight_enforces_manifest_bound() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        let mut visiting = HashSet::new();
        let mut validated = HashSet::new();
        let mut manifests_seen = MAX_MANIFEST_PREFLIGHTS;
        let mut traversal_entries_seen = 0;

        let error = validate_cargo_workspace_manifest(
            tmp.path(),
            tmp.path(),
            &mut visiting,
            &mut validated,
            &mut manifests_seen,
            &mut traversal_entries_seen,
        )
        .unwrap_err();
        assert!(error.contains("preflight exceeds"));
    }

    #[test]
    fn cargo_preflight_charges_duplicate_members_without_replaying_manifests() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("crates/member")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/member\", \"./crates/member\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/member/Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let mut visiting = HashSet::new();
        let mut validated = HashSet::new();
        let mut manifests_seen = 0;
        let mut traversal_entries_seen = MAX_CONFINEMENT_ENTRIES - 2;

        validate_cargo_workspace_manifest(
            tmp.path(),
            tmp.path(),
            &mut visiting,
            &mut validated,
            &mut manifests_seen,
            &mut traversal_entries_seen,
        )
        .expect("duplicate members must fit exactly at the declaration limit");

        assert_eq!(traversal_entries_seen, MAX_CONFINEMENT_ENTRIES);
        assert_eq!(manifests_seen, 2);

        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/member\", \"./crates/member\", \"crates/member\"]\n",
        )
        .unwrap();
        let mut visiting = HashSet::new();
        let mut validated = HashSet::new();
        let mut manifests_seen = 0;
        let mut traversal_entries_seen = MAX_CONFINEMENT_ENTRIES - 2;
        let error = validate_cargo_workspace_manifest(
            tmp.path(),
            tmp.path(),
            &mut visiting,
            &mut validated,
            &mut manifests_seen,
            &mut traversal_entries_seen,
        )
        .expect_err("the limit-plus-one duplicate declaration must be rejected");

        assert!(error.contains("exceeds 100000 entries"), "{error}");
        assert_eq!(manifests_seen, 2);
    }

    #[test]
    fn node_preflight_deduplicates_patterns_bases_and_workspace_paths() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("packages/member")).unwrap();
        fs::write(
            tmp.path().join("packages/member/package.json"),
            r#"{"name":"member"}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"workspaces":["packages/*","packages/*","packages/**"]}"#,
        )
        .unwrap();
        let mut exact_entries_seen = MAX_CONFINEMENT_ENTRIES - 4;

        validate_package_workspaces(tmp.path(), &mut exact_entries_seen)
            .expect("three declarations plus one unique workspace must fit exactly");

        assert_eq!(exact_entries_seen, MAX_CONFINEMENT_ENTRIES);

        let mut overflow_entries_seen = MAX_CONFINEMENT_ENTRIES - 3;
        let error = validate_package_workspaces(tmp.path(), &mut overflow_entries_seen)
            .expect_err("the limit-plus-one Node declaration must be rejected");

        assert!(error.contains("exceeds 100000 entries"), "{error}");
    }

    #[test]
    fn node_preflight_fails_closed_and_charges_entries_before_type_validation() {
        for (content, expected) in [
            ("{", "Cannot parse MCP package.json as JSON"),
            ("false", "root must be a JSON object"),
            (
                r#"{"workspaces":"packages/*"}"#,
                "`workspaces` must be an array or an object",
            ),
            (
                r#"{"workspaces":{"packages":{}}}"#,
                "`workspaces.packages` must be an array",
            ),
            (
                r#"{"workspaces":{}}"#,
                "`workspaces` object must contain a `packages` array",
            ),
        ] {
            let tmp = TempDir::new().unwrap();
            fs::write(tmp.path().join("package.json"), content).unwrap();
            let mut entries_seen = 0;
            let error = validate_package_workspaces(tmp.path(), &mut entries_seen)
                .expect_err("invalid package.json must make MCP preflight inconclusive");
            assert!(error.contains(expected), "{content}: {error}");
        }

        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"workspaces":[false,"packages/*"]}"#,
        )
        .unwrap();
        let mut entries_seen = MAX_CONFINEMENT_ENTRIES - 2;
        let error = validate_package_workspaces(tmp.path(), &mut entries_seen)
            .expect_err("wrong-typed workspace entries must fail after declaration charging");

        assert!(
            error.contains("workspace entries must be strings"),
            "{error}"
        );
        assert_eq!(entries_seen, MAX_CONFINEMENT_ENTRIES);
    }
}
