use regex::RegexBuilder;

/// Levenshtein edit distance between two strings.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            curr[j] = if a[i - 1] == b[j - 1] {
                prev[j - 1]
            } else {
                1 + prev[j - 1].min(prev[j]).min(curr[j - 1])
            };
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Maximum allowed size for user-provided regex patterns (in bytes of the compiled DFA).
/// Prevents ReDoS from crafted patterns in config files.
const MAX_REGEX_SIZE: usize = 1 << 16; // 64 KB

/// Compile a user-provided regex pattern with size limits to prevent ReDoS.
/// Returns None if the pattern is invalid or exceeds the size limit.
pub fn safe_regex(pattern: &str) -> Option<regex::Regex> {
    RegexBuilder::new(pattern)
        .size_limit(MAX_REGEX_SIZE)
        .dfa_size_limit(MAX_REGEX_SIZE)
        .build()
        .ok()
}

/// Confine a caller-supplied path to a server/project root.
///
/// Canonicalizes (resolving symlinks, `.`/`..`, and platform prefixes) both
/// `root` and `candidate`, then accepts `candidate` only when it equals the
/// canonical root or is a descendant of it. Returns the canonical candidate on
/// success.
///
/// Use this before honoring any caller-provided filesystem path (e.g. an MCP
/// tool `root` argument) so a hostile or prompt-injected caller cannot make
/// the process read or write outside its configured root. Both paths must
/// exist — canonicalization of a non-existent path fails and is reported as an
/// error rather than silently allowed.
pub fn confine_path_to_root(
    root: &std::path::Path,
    candidate: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("cannot resolve server root {}: {e}", root.display()))?;
    let canonical_candidate = candidate.canonicalize().map_err(|e| {
        format!(
            "path {} cannot be resolved (must exist): {e}",
            candidate.display()
        )
    })?;
    if canonical_candidate == canonical_root || canonical_candidate.starts_with(&canonical_root) {
        Ok(canonical_candidate)
    } else {
        Err(format!(
            "path {} is outside the server root {}",
            canonical_candidate.display(),
            canonical_root.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("config.ts", "confg.ts"), 1);
    }

    #[test]
    fn test_safe_regex_valid() {
        assert!(safe_regex(r"\bfoo\b").is_some());
        assert!(safe_regex(r"^## \w+").is_some());
    }

    #[test]
    fn test_safe_regex_invalid() {
        assert!(safe_regex(r"[invalid").is_none());
    }

    #[test]
    fn test_confine_path_to_root_accepts_root_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let confined = confine_path_to_root(&root, &root).unwrap();
        assert_eq!(confined, root);
    }

    #[test]
    fn test_confine_path_to_root_accepts_child() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let child = root.join("sub");
        std::fs::create_dir_all(&child).unwrap();
        // Also accepts non-canonical spellings like `sub/../sub`.
        let spelled = root.join("sub").join("..").join("sub");
        let confined = confine_path_to_root(&root, &spelled).unwrap();
        assert_eq!(confined, child.canonicalize().unwrap());
    }

    #[test]
    fn test_confine_path_to_root_rejects_sibling() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("root");
        let outside = tmp.path().join("outside");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let err = confine_path_to_root(&root, &outside).unwrap_err();
        assert!(err.contains("outside the server root"));
    }

    #[test]
    fn test_confine_path_to_root_rejects_dotdot_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("root");
        std::fs::create_dir_all(&root).unwrap();
        let escape = root.join("..");
        assert!(confine_path_to_root(&root, &escape).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn test_confine_path_to_root_rejects_symlink_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("root");
        let outside = tmp.path().join("outside");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let link = root.join("link");
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        assert!(confine_path_to_root(&root, &link).is_err());
    }

    #[test]
    fn test_confine_path_to_root_rejects_nonexistent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        assert!(confine_path_to_root(&root, &root.join("missing")).is_err());
    }
}
