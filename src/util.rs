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

/// Confine a user-supplied path (e.g. a `depends_on` entry) to the project
/// root. Returns the joined path when `rel` is a relative path with no `..`
/// escape and no absolute/prefix component; `None` otherwise. Use this to
/// keep dependency declarations from validating (or reading) files outside
/// the project — `/etc/passwd` must never pass as a spec dependency.
pub fn confine_path_to_root(root: &std::path::Path, rel: &str) -> Option<std::path::PathBuf> {
    use std::path::Component;
    let path = std::path::Path::new(rel);
    if rel.is_empty() || path.is_absolute() {
        return None;
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            // ParentDir could escape the root; RootDir/Prefix are absolute.
            _ => return None,
        }
    }
    Some(root.join(path))
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
    fn test_confine_path_to_root_accepts_relative() {
        let root = std::path::Path::new("/proj");
        assert_eq!(
            confine_path_to_root(root, "specs/a/a.spec.md"),
            Some(root.join("specs/a/a.spec.md"))
        );
        assert_eq!(
            confine_path_to_root(root, "./specs/a.spec.md"),
            Some(root.join("./specs/a.spec.md"))
        );
    }

    #[test]
    fn test_confine_path_to_root_rejects_escapes() {
        let root = std::path::Path::new("/proj");
        // Absolute paths and `..` traversal must never validate (#444).
        assert_eq!(confine_path_to_root(root, "/etc/passwd"), None);
        assert_eq!(confine_path_to_root(root, "../outside.spec.md"), None);
        assert_eq!(confine_path_to_root(root, "specs/../../etc/passwd"), None);
        assert_eq!(confine_path_to_root(root, ""), None);
    }
}
