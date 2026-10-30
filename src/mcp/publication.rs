//! Identity-bound staging, atomic publication, and rollback of MCP writes.

use super::confinement::{
    DirectoryIdentity, FileIdentity, directory_identity, file_identity, validate_confined_relative,
};
use super::snapshot::open_snapshot_configuration_directory;
use super::tools::MAX_GENERATED_SPECS;
use cap_std::fs::{Dir, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static STAGED_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn write_new_confined(
    directory: &Dir,
    relative: &Path,
    content: &[u8],
    label: &str,
) -> Result<(), String> {
    let mut staged = stage_new_confined(directory, relative, content, label)?;
    if let Err(error) = publish_staged_file(&mut staged, label) {
        return Err(rollback_staged_batch(&[staged], error));
    }
    Ok(())
}

pub(super) fn write_generated_specs_transactionally(
    directory: &Dir,
    generated_specs: Vec<(PathBuf, String)>,
) -> Result<(), String> {
    if generated_specs.len() > MAX_GENERATED_SPECS {
        return Err(format!(
            "MCP generation exceeds the {MAX_GENERATED_SPECS}-spec output limit"
        ));
    }
    let mut staged = Vec::with_capacity(generated_specs.len());
    for (relative, content) in generated_specs {
        match stage_new_confined(directory, &relative, content.as_bytes(), "generated spec") {
            Ok(file) => staged.push(file),
            Err(error) => {
                return Err(rollback_staged_batch(&staged, error));
            }
        }
    }

    for index in 0..staged.len() {
        if let Err(error) = publish_staged_file(&mut staged[index], "generated spec") {
            return Err(rollback_staged_batch(&staged, error));
        }
    }
    Ok(())
}

struct StagedFile<'root> {
    root: &'root Dir,
    parent: Dir,
    parent_path: PathBuf,
    parent_identity: DirectoryIdentity,
    destination_name: PathBuf,
    destination_display: PathBuf,
    temporary_name: PathBuf,
    temporary_display: PathBuf,
    identity: FileIdentity,
    temporary_present: bool,
    published: bool,
}

fn stage_new_confined<'root>(
    directory: &'root Dir,
    destination: &Path,
    content: &[u8],
    label: &str,
) -> Result<StagedFile<'root>, String> {
    stage_new_confined_with_hook(directory, destination, content, label, |_, _| {})
}

fn stage_new_confined_with_hook<'root, Hook>(
    directory: &'root Dir,
    destination: &Path,
    content: &[u8],
    label: &str,
    after_identity: Hook,
) -> Result<StagedFile<'root>, String>
where
    Hook: FnOnce(&Dir, &Path),
{
    validate_confined_relative(destination, label)?;
    let parent_path = destination.parent().unwrap_or_else(|| Path::new(""));
    let destination_name = destination
        .file_name()
        .map(PathBuf::from)
        .ok_or_else(|| format!("MCP {label} must name a destination file"))?;
    let parent = create_confined_directories(directory, parent_path, label)?;
    let parent_identity = directory_identity(&parent).map_err(|error| {
        format!(
            "Cannot identify confined MCP {label} parent {}: {error}",
            parent_path.display()
        )
    })?;

    for _ in 0..128 {
        let sequence = STAGED_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary_name = PathBuf::from(format!(
            ".specsync-mcp-stage-{}-{sequence}",
            std::process::id()
        ));
        let temporary_display = parent_path.join(&temporary_name);
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        let mut file = match parent.open_with(&temporary_name, &options) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                let failure = format!(
                    "Cannot create confined MCP staged {label} {}: {error}",
                    temporary_display.display()
                );
                return Err(failure);
            }
        };
        let write_result = file
            .write_all(content)
            .and_then(|_| file.flush())
            .and_then(|_| file.sync_all());
        let (identity, identity_error) = match file_identity(&file) {
            Ok(identity) => (identity, None),
            Err(error) => (invalid_file_identity(), Some(error)),
        };
        drop(file);
        after_identity(&parent, &temporary_name);
        let staged = StagedFile {
            root: directory,
            parent,
            parent_path: parent_path.to_path_buf(),
            parent_identity,
            destination_name,
            destination_display: destination.to_path_buf(),
            temporary_name,
            temporary_display,
            identity,
            temporary_present: true,
            published: false,
        };
        if let Err(error) = write_result {
            let failure = format!(
                "Cannot stage confined MCP {label} {}: {error}",
                destination.display()
            );
            return Err(rollback_staged_batch(&[staged], failure));
        }
        if let Some(error) = identity_error {
            let failure = format!(
                "Cannot identify confined MCP staged {label} {}: {error}",
                destination.display()
            );
            return Err(rollback_staged_batch(&[staged], failure));
        }
        return Ok(staged);
    }

    let failure = format!(
        "Cannot reserve a unique staged MCP {label} beside {}",
        destination.display()
    );
    Err(failure)
}

#[cfg(unix)]
fn invalid_file_identity() -> FileIdentity {
    (u64::MAX, u64::MAX, [0_u8; 32])
}

#[cfg(windows)]
fn invalid_file_identity() -> FileIdentity {
    (u32::MAX, u64::MAX, [0_u8; 32])
}

#[cfg(not(any(unix, windows)))]
fn invalid_file_identity() -> FileIdentity {
    (u64::MAX, None, [0_u8; 32])
}

fn create_confined_directories(directory: &Dir, parent: &Path, label: &str) -> Result<Dir, String> {
    // Parent directory creation has no portable create-and-open primitive. Retain empty
    // parents after a failed batch rather than risk claiming and deleting a concurrently
    // replaced directory.
    let mut current_directory = directory
        .try_clone()
        .map_err(|error| format!("Cannot clone MCP root capability: {error}"))?;
    let mut current_display = PathBuf::new();
    for component in parent.components() {
        let name = PathBuf::from(component.as_os_str());
        current_display.push(&name);
        let exists = match current_directory.try_exists(&name) {
            Ok(exists) => exists,
            Err(error) => {
                return Err(format!(
                    "Cannot inspect confined MCP {label} directory {}: {error}",
                    current_display.display()
                ));
            }
        };
        if !exists {
            match current_directory.create_dir(&name) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(format!(
                        "Cannot create confined MCP {label} directory {}: {error}",
                        current_display.display()
                    ));
                }
            }
        }
        let next = match current_directory.open_dir(&name) {
            Ok(next) => next,
            Err(error) => {
                return Err(format!(
                    "Cannot open confined MCP {label} directory {}: {error}",
                    current_display.display()
                ));
            }
        };
        current_directory = next;
    }
    Ok(current_directory)
}

fn publish_staged_file(staged: &mut StagedFile<'_>, label: &str) -> Result<(), String> {
    publish_staged_file_with_hooks(staged, label, |_, _| {}, |_, _| {})
}

#[cfg(test)]
fn publish_staged_file_with_hook<Hook>(
    staged: &mut StagedFile<'_>,
    label: &str,
    before_quarantine: Hook,
) -> Result<(), String>
where
    Hook: FnOnce(&Dir, &Path),
{
    publish_staged_file_with_hooks(staged, label, before_quarantine, |_, _| {})
}

fn publish_staged_file_with_hooks<BeforeQuarantine, AfterHardLink>(
    staged: &mut StagedFile<'_>,
    label: &str,
    before_quarantine: BeforeQuarantine,
    after_hard_link: AfterHardLink,
) -> Result<(), String>
where
    BeforeQuarantine: FnOnce(&Dir, &Path),
    AfterHardLink: FnOnce(&Dir, &Path),
{
    publish_staged_file_with_verification_hooks(
        staged,
        label,
        before_quarantine,
        after_hard_link,
        file_identity,
    )
}

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

fn remove_identity_bound_file(
    parent: &Dir,
    name: &Path,
    display: &Path,
    expected: FileIdentity,
) -> Result<(), String> {
    remove_identity_bound_file_with_hook(parent, name, display, expected, |_, _| {})
}

pub(super) fn remove_identity_bound_file_with_hook<Hook>(
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

#[cfg(test)]
mod tests;
