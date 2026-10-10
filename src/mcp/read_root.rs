use super::capabilities::DirectoryIdentity;
use super::capabilities::SnapshotEntryIdentity;
use super::capabilities::directory_identity;
use super::capabilities::open_snapshot_configuration_directory;
#[cfg(debug_assertions)]
use super::test_barriers::wait_for_mcp_read_root_revalidation_test_barrier;
use cap_std::fs::Dir;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

pub(super) fn resolve_read_root(
    server_root: &Path,
    requested_server_root: &Path,
    requested_root: Option<&str>,
) -> Result<PathBuf, String> {
    let Some(requested_root) = requested_root else {
        return Ok(PathBuf::from("."));
    };
    let requested_path = Path::new(requested_root);
    let components: Vec<Component<'_>> = requested_path.components().collect();
    if components.contains(&Component::ParentDir) {
        return Err("Read root override must not contain parent traversal".to_string());
    }
    if !requested_path.is_absolute()
        && components
            .iter()
            .any(|component| matches!(*component, Component::RootDir | Component::Prefix(_)))
    {
        return Err("Read root override must not use a rooted or drive-relative path".to_string());
    }

    let relative = if requested_path.is_absolute() {
        relative_read_root_suffix_from_aliases(server_root, requested_server_root, requested_path)
            .ok_or_else(|| "Read root override escapes the configured server root".to_string())?
    } else {
        requested_path.to_path_buf()
    };
    if relative.components().any(|component| {
        matches!(
            component,
            Component::Normal(name)
                if name
                    .to_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
        )
    }) {
        return Err("Read root override must not select Git metadata".to_string());
    }
    Ok(if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    })
}

fn relative_read_root_suffix_from_aliases(
    canonical_root: &Path,
    requested_root: &Path,
    candidate: &Path,
) -> Option<PathBuf> {
    // Windows canonicalization can expand an operator-supplied DOS 8.3 root spelling. Accept the
    // candidate beneath either startup spelling, but never canonicalize the candidate before
    // authorization. The derived suffix is still opened only through the retained root capability.
    relative_read_root_suffix(canonical_root, candidate)
        .or_else(|| relative_read_root_suffix(requested_root, candidate))
}

#[cfg(not(windows))]
fn relative_read_root_suffix(root: &Path, candidate: &Path) -> Option<PathBuf> {
    candidate.strip_prefix(root).ok().map(Path::to_path_buf)
}

#[cfg(windows)]
fn relative_read_root_suffix(root: &Path, candidate: &Path) -> Option<PathBuf> {
    let (root_prefix, root_components) = parse_windows_absolute_path_native(root)?;
    let (candidate_prefix, candidate_components) = parse_windows_absolute_path_native(candidate)?;
    if !windows_prefixes_equal(&root_prefix, &candidate_prefix)
        || candidate_components.len() < root_components.len()
        || !root_components
            .iter()
            .zip(&candidate_components)
            .all(|(root, candidate)| windows_ordinal_ignore_case(root, candidate))
    {
        return None;
    }
    Some(
        candidate_components[root_components.len()..]
            .iter()
            .collect(),
    )
}

#[cfg(windows)]
enum WindowsPathPrefix {
    Disk(u8),
    Unc(std::ffi::OsString, std::ffi::OsString),
}

#[cfg(windows)]
fn parse_windows_absolute_path_native(
    path: &Path,
) -> Option<(WindowsPathPrefix, Vec<std::ffi::OsString>)> {
    use std::path::Prefix;

    let mut components = path.components();
    let Component::Prefix(prefix) = components.next()? else {
        return None;
    };
    let prefix = match prefix.kind() {
        Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => WindowsPathPrefix::Disk(drive),
        Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => {
            WindowsPathPrefix::Unc(server.to_os_string(), share.to_os_string())
        }
        Prefix::Verbatim(_) | Prefix::DeviceNS(_) => return None,
    };
    if !matches!(components.next(), Some(Component::RootDir)) {
        return None;
    }
    let mut relative = Vec::new();
    for component in components {
        match component {
            Component::Normal(component) => relative.push(component.to_os_string()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some((prefix, relative))
}

#[cfg(windows)]
fn windows_prefixes_equal(left: &WindowsPathPrefix, right: &WindowsPathPrefix) -> bool {
    match (left, right) {
        (WindowsPathPrefix::Disk(left), WindowsPathPrefix::Disk(right)) => {
            left.eq_ignore_ascii_case(right)
        }
        (
            WindowsPathPrefix::Unc(left_server, left_share),
            WindowsPathPrefix::Unc(right_server, right_share),
        ) => {
            windows_ordinal_ignore_case(left_server, right_server)
                && windows_ordinal_ignore_case(left_share, right_share)
        }
        _ => false,
    }
}

#[cfg(windows)]
fn windows_ordinal_ignore_case(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt;

    unsafe extern "system" {
        fn CompareStringOrdinal(
            left: *const u16,
            left_len: i32,
            right: *const u16,
            right_len: i32,
            ignore_case: i32,
        ) -> i32;
    }

    let left: Vec<u16> = left.encode_wide().collect();
    let right: Vec<u16> = right.encode_wide().collect();
    let (Ok(left_len), Ok(right_len)) = (i32::try_from(left.len()), i32::try_from(right.len()))
    else {
        return false;
    };
    // SAFETY: both pointers reference initialized UTF-16 buffers for the exact lengths supplied;
    // CompareStringOrdinal does not retain or mutate either buffer.
    unsafe { CompareStringOrdinal(left.as_ptr(), left_len, right.as_ptr(), right_len, 1) == 2 }
}

#[cfg(test)]
fn windows_relative_suffix_text(root: &str, candidate: &str) -> Option<PathBuf> {
    let root = parse_windows_absolute_path(root)?;
    let candidate = parse_windows_absolute_path(candidate)?;
    if !windows_text_ignore_case(&root.prefix, &candidate.prefix)
        || candidate.components.len() < root.components.len()
        || !root
            .components
            .iter()
            .zip(&candidate.components)
            .all(|(root, candidate)| windows_text_ignore_case(root, candidate))
    {
        return None;
    }
    Some(
        candidate.components[root.components.len()..]
            .iter()
            .collect(),
    )
}

#[cfg(test)]
fn windows_text_ignore_case(left: &str, right: &str) -> bool {
    left.to_lowercase() == right.to_lowercase()
}

#[cfg(test)]
struct WindowsAbsolutePath {
    prefix: String,
    components: Vec<String>,
}

#[cfg(test)]
fn parse_windows_absolute_path(path: &str) -> Option<WindowsAbsolutePath> {
    let mut path = path.replace('/', "\\");
    if let Some(without_prefix) = strip_ascii_case_prefix(&path, r"\\?\UNC\") {
        path = format!(r"\\{without_prefix}");
    } else if let Some(without_prefix) = strip_ascii_case_prefix(&path, r"\\?\") {
        path = without_prefix.to_string();
    }

    if let Some(unc) = path.strip_prefix(r"\\") {
        let mut components = unc.split('\\').filter(|component| !component.is_empty());
        let server = components.next()?;
        let share = components.next()?;
        return Some(WindowsAbsolutePath {
            prefix: format!(r"\\{server}\{share}"),
            components: components.map(str::to_string).collect(),
        });
    }

    let bytes = path.as_bytes();
    if bytes.len() < 3 || bytes[1] != b':' || bytes[2] != b'\\' {
        return None;
    }
    Some(WindowsAbsolutePath {
        prefix: path[..2].to_string(),
        components: path[3..]
            .split('\\')
            .filter(|component| !component.is_empty())
            .map(str::to_string)
            .collect(),
    })
}

#[cfg(test)]
fn strip_ascii_case_prefix<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let candidate = value.get(..prefix.len())?;
    candidate
        .eq_ignore_ascii_case(prefix)
        .then(|| &value[prefix.len()..])
}

pub(super) struct ConfinedReadRoot<'root> {
    root: &'root Dir,
    root_identity: DirectoryIdentity,
    directory: Dir,
    components: Vec<(PathBuf, SnapshotEntryIdentity)>,
}

impl ConfinedReadRoot<'_> {
    pub(super) fn directory(&self) -> &Dir {
        &self.directory
    }

    pub(super) fn revalidate_before_success(&self) -> Result<(), String> {
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

pub(super) fn open_confined_read_root<'root>(
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

#[cfg(test)]
mod tests;
