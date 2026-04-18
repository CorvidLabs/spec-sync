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
}
