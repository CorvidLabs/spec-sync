use super::*;
#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use tempfile::TempDir;

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
