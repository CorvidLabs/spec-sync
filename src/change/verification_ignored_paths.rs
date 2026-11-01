// `.specsync/sdd.json` `verification_ignored_paths` (issue #825, REQ-change-114, REQ-change-115).
//
// The list names directory prefixes whose files another tool owns (Chough writes `.chough/`).
// Files under them are left out of the verification project-input digest and NOTHING else:
// coverage (`ignored_paths`, `meaningful_paths`), spec scanning (`config.toml` `exclude_dirs`),
// acceptance-input digests, lifecycle commit staging and the scoped-review descendant walk all
// keep their meaning. Because the list hides inputs from the one digest every currency gate
// compares, it is bounded hard, validated by every lifecycle verb that loads policy, and refuses
// any entry that could cover something `change check` reads in-process: specs, sources, policy,
// principles, custom-artifact templates, the configured `schema_dir` (replayed to judge spec
// `db_tables`), spec `files:`, path-form spec `depends_on` entries, the files a spec's
// `Consumed By` table names, and every `meaningful_paths` scope. Root-level files (`config.toml`
// alternatives, `.specsyncignore`) cannot be covered, because every entry is a directory prefix.
// `ignored_paths` is never consulted, so it keeps its single meaning (#676).

const MAX_VERIFICATION_IGNORED_PATHS: usize = 8;
const MAX_VERIFICATION_IGNORED_PATH_BYTES: usize = 128;
const VERIFICATION_IGNORED_PATTERN_CHARACTERS: [char; 7] = ['*', '?', '[', ']', '{', '}', '!'];
const VERIFICATION_IGNORED_REFUSAL: &str = "invalid SDD policy: verification_ignored_paths";

/// The validated list, sorted, or the refusal naming the first offending entry and its rule.
///
/// A missing policy is an empty list and an unparseable one is today's invalid-policy error.
/// Memoized in the same read scope as `project_input_digest`, so status and audit validate once
/// per invocation rather than once per change.
fn validated_verification_ignored_paths(root: &Path) -> Result<Vec<String>, String> {
    if let Some(result) = read_scope_value(root, |scope| scope.verification_ignored_paths.clone())
    {
        return result;
    }
    let result = load_policy_checked(root).and_then(|policy| match policy {
        Some(policy) => validate_verification_ignored_paths(root, &policy),
        None => Ok(Vec::new()),
    });
    update_read_scope(root, |scope| {
        scope.verification_ignored_paths = Some(result.clone());
    });
    result
}

/// The validated list for display, empty when it is absent or invalid. An invalid list is
/// reported as the refusal instead, never shown as if it were in force.
pub fn verification_ignored_paths_for_display(root: &Path) -> Vec<String> {
    validated_verification_ignored_paths(root).unwrap_or_default()
}

/// Why the list refuses this project, or this change when one is given: an invalid entry, or an
/// entry overlapping the change's declared `affected_paths`. `None` when nothing refuses.
pub fn verification_ignored_paths_refusal(
    root: &Path,
    record: Option<&ChangeRecord>,
) -> Option<String> {
    match record {
        Some(record) => ensure_change_clear_of_verification_ignored(root, record).err(),
        None => validated_verification_ignored_paths(root).err(),
    }
}

/// The one next action an invalid or overlapping list leaves a change.
fn verification_ignored_paths_next_action(error: &str, id: &str) -> String {
    format!(
        "fix `.specsync/sdd.json` verification_ignored_paths ({error}), then run `specsync change status {id}`"
    )
}

/// Validate the list for the whole project, then refuse any entry that overlaps this change's
/// declared `affected_paths`.
fn ensure_change_clear_of_verification_ignored(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Vec<String>, String> {
    let entries = validated_verification_ignored_paths(root)?;
    ensure_affected_paths_clear_of_verification_ignored(
        &entries,
        &record.affected_paths,
        Some(&record.id),
    )?;
    Ok(entries)
}

/// A change claims its `affected_paths` and its acceptance evidence hashes them, so the digest
/// must not ignore any of them. `id` is `None` while the change is being created.
fn ensure_affected_paths_clear_of_verification_ignored(
    entries: &[String],
    affected_paths: &[String],
    id: Option<&str>,
) -> Result<(), String> {
    for entry in entries {
        for path in affected_paths {
            if verification_ignored_scopes_overlap(entry, path) {
                let owner = id
                    .map(|id| format!("change `{id}`"))
                    .unwrap_or_else(|| "this change".to_string());
                return Err(format!(
                    "verification_ignored_paths entry `{entry}` overlaps affected path `{path}` of {owner}; remove the entry from `.specsync/sdd.json` or narrow the change's paths"
                ));
            }
        }
    }
    Ok(())
}

fn validate_verification_ignored_paths(
    root: &Path,
    policy: &SddPolicy,
) -> Result<Vec<String>, String> {
    let entries = &policy.verification_ignored_paths;
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    if entries.len() > MAX_VERIFICATION_IGNORED_PATHS {
        return Err(format!(
            "{VERIFICATION_IGNORED_REFUSAL} lists {} entries; at most {MAX_VERIFICATION_IGNORED_PATHS} are allowed",
            entries.len()
        ));
    }
    let refuse = |entry: &str, rule: String| {
        format!(
            "{VERIFICATION_IGNORED_REFUSAL} entry `{}` {rule}",
            verification_ignored_display(entry)
        )
    };
    for entry in entries {
        verification_ignored_entry_shape(entry).map_err(|rule| refuse(entry, rule))?;
    }
    for (index, entry) in entries.iter().enumerate() {
        let scope = verification_ignored_overlap_scope(entry);
        for earlier in &entries[..index] {
            let other = verification_ignored_overlap_scope(earlier);
            if scope == other {
                return Err(refuse(
                    entry,
                    format!("duplicates entry `{earlier}` (ignoring ASCII case)"),
                ));
            }
            if scope.starts_with(&format!("{other}/")) {
                return Err(refuse(entry, format!("is nested in entry `{earlier}`")));
            }
            if other.starts_with(&format!("{scope}/")) {
                return Err(refuse(earlier, format!("is nested in entry `{entry}`")));
            }
        }
    }
    let protected = verification_ignored_protected_scopes(root, policy);
    for entry in entries {
        for (scope, label) in &protected {
            if verification_ignored_scopes_overlap(entry, scope) {
                return Err(refuse(entry, format!("overlaps {label}")));
            }
        }
        verification_ignored_symlink_free(root, entry).map_err(|rule| refuse(entry, rule))?;
    }
    let mut sorted = entries.clone();
    sorted.sort();
    Ok(sorted)
}

/// The shape rules, in a fixed order so the first broken one is named.
fn verification_ignored_entry_shape(entry: &str) -> Result<(), String> {
    if entry.is_empty() {
        return Err("is empty".into());
    }
    if entry.len() > MAX_VERIFICATION_IGNORED_PATH_BYTES {
        return Err(format!(
            "is longer than {MAX_VERIFICATION_IGNORED_PATH_BYTES} bytes"
        ));
    }
    if entry.chars().any(char::is_control) {
        return Err("contains a control character".into());
    }
    if entry.contains('\\') {
        return Err("contains a backslash; use `/`".into());
    }
    if let Some(character) = entry
        .chars()
        .find(|character| VERIFICATION_IGNORED_PATTERN_CHARACTERS.contains(character))
    {
        return Err(format!(
            "contains the pattern character `{character}`; entries are literal directory prefixes"
        ));
    }
    let bytes = entry.as_bytes();
    if entry.starts_with('/')
        || (bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic())
    {
        return Err("is absolute; entries are relative to the project root".into());
    }
    let Some(directory) = entry.strip_suffix('/') else {
        return Err("does not end in `/`; entries are directory prefixes".into());
    };
    if directory
        .split('/')
        .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err("has an empty, `.` or `..` component".into());
    }
    Ok(())
}

/// Every scope an entry may not equal, contain or lie inside, with the label its refusal names.
fn verification_ignored_protected_scopes(
    root: &Path,
    policy: &SddPolicy,
) -> Vec<(String, String)> {
    let config = crate::config::load_config(root);
    let mut scopes = vec![(".specsync".to_string(), "`.specsync`".to_string())];
    let specs = verification_ignored_overlap_scope(&config.specs_dir);
    scopes.push((specs.clone(), format!("the specs tree `{specs}`")));
    for source in &config.source_dirs {
        let normalized = verification_ignored_overlap_scope(source);
        scopes.push((normalized.clone(), format!("source_dirs `{normalized}`")));
    }
    if let Some(schema_dir) = &config.schema_dir {
        let normalized = verification_ignored_overlap_scope(schema_dir);
        scopes.push((normalized.clone(), format!("schema_dir `{normalized}`")));
    }
    for path in PROTECTED_SDD_PATHS {
        scopes.push((path.to_string(), format!("SDD policy file `{path}`")));
    }
    if let Some(principles) = &policy.principles_file {
        scopes.push((principles.clone(), format!("principles_file `{principles}`")));
    }
    for template in policy.custom_artifacts.values() {
        scopes.push((
            template.clone(),
            format!("custom_artifacts template `{template}`"),
        ));
    }
    for spec in crate::validator::find_spec_files(&root.join(&config.specs_dir)) {
        let Ok(content) = fs::read_to_string(&spec) else {
            continue;
        };
        let normalized = content.replace("\r\n", "\n");
        let Some(parsed) = crate::parser::parse_frontmatter(&normalized) else {
            continue;
        };
        let spec_path = portable_project_path(root, &spec);
        for file in &parsed.frontmatter.files {
            scopes.push((
                file.clone(),
                format!("the files: entry `{file}` of spec `{spec_path}`"),
            ));
        }
        // The validator checks a path-form `depends_on` entry on disk; a bare module name
        // resolves under the specs tree, already protected, and a cross-project reference is
        // resolved by `specsync resolve`, never read here.
        for dependency in &parsed.frontmatter.depends_on {
            if crate::validator::is_cross_project_ref(dependency)
                || (!dependency.contains('/') && !dependency.contains('.'))
            {
                continue;
            }
            scopes.push((
                dependency.clone(),
                format!("the depends_on entry `{dependency}` of spec `{spec_path}`"),
            ));
        }
        for reference in verification_ignored_consumed_by_references(&parsed.body) {
            let label = format!("the Consumed By reference `{reference}` of spec `{spec_path}`");
            scopes.push((reference, label));
        }
    }
    for scope in &policy.meaningful_paths {
        scopes.push((scope.clone(), format!("meaningful_paths `{scope}`")));
    }
    scopes
}

/// The files a spec's `### Consumed By` table names, which the validator checks on disk (a missing
/// one is a warning, so it can fail `change check --strict`). The two patterns are the validator's
/// own `CONSUMED_BY_RE` and `FILE_REF_RE`, which are private to it; the
/// `entry_covering_an_in_process_check_input_is_refused` test pins the shape they agree on.
fn verification_ignored_consumed_by_references(body: &str) -> Vec<String> {
    static CONSUMED_BY: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(?s)### Consumed By\s*\n(.*?)(?:\n## |\n### |$)").unwrap()
    });
    static FILE_REFERENCE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"\|\s*`([^`]+\.\w+)`\s*\|").unwrap());
    let Some(section) = CONSUMED_BY
        .captures(body)
        .and_then(|captures| captures.get(1))
    else {
        return Vec::new();
    };
    FILE_REFERENCE
        .captures_iter(section.as_str())
        .filter_map(|captures| captures.get(1))
        .map(|reference| reference.as_str().to_string())
        .collect()
}

/// An entry that is, or lies under, a symlink could point into sources, and Git lists a symlink
/// as one path. A path that does not exist yet is allowed: the tool may not have written yet.
fn verification_ignored_symlink_free(root: &Path, entry: &str) -> Result<(), String> {
    let mut current = root.to_path_buf();
    let mut relative = String::new();
    for component in entry.trim_end_matches('/').split('/') {
        current.push(component);
        if !relative.is_empty() {
            relative.push('/');
        }
        relative.push_str(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("is or lies under a symlink at `{relative}`"));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(format!("could not be inspected at `{relative}` ({error})"));
            }
        }
    }
    Ok(())
}

/// A project-relative path as a comparable scope: `/`-separated, without empty or `.`
/// components and without a trailing slash, lowercased so overlap is refused ignoring ASCII case
/// (`SRC/` and `src/` are one directory on a case-insensitive file system). `.` is the project.
fn verification_ignored_overlap_scope(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let components: Vec<&str> = normalized
        .split('/')
        .filter(|component| !component.is_empty() && *component != ".")
        .collect();
    if components.is_empty() {
        ".".to_string()
    } else {
        components.join("/").to_ascii_lowercase()
    }
}

/// Whether the entry equals, contains, or lies inside `scope`.
fn verification_ignored_scopes_overlap(entry: &str, scope: &str) -> bool {
    let entry = verification_ignored_overlap_scope(entry);
    let scope = verification_ignored_overlap_scope(scope);
    scope == "."
        || entry == scope
        || scope.starts_with(&format!("{entry}/"))
        || entry.starts_with(&format!("{scope}/"))
}

/// Exclusion is a byte-prefix test on the exact project-relative path. Every validated entry ends
/// in `/`, so a path equal to an entry without its slash never matches, and a case mismatch can
/// only include more paths.
fn path_is_verification_ignored(path: &str, entries: &[String]) -> bool {
    entries.iter().any(|entry| path.starts_with(entry.as_str()))
}

fn git_path_bytes_are_verification_ignored(path: &[u8], entries: &[String]) -> bool {
    entries
        .iter()
        .any(|entry| path.starts_with(entry.as_bytes()))
}

/// Bind a non-empty list into the project-input digest right after the domain frame. Both tags
/// are length-framed, and every path and file byte is a frame VALUE under `entry`, `path`,
/// `kind`, `mode`, `content` or `object-id` tags, so no project content can produce them. An
/// empty list adds nothing, which keeps the digest byte-identical.
fn frame_verification_ignored_paths(digest: &mut FramedDigest, entries: &[String]) {
    if entries.is_empty() {
        return;
    }
    digest.frame(b"verification-ignored", &(entries.len() as u64).to_be_bytes());
    for entry in entries {
        digest.frame(b"verification-ignored-prefix", entry.as_bytes());
    }
}

/// Entries are shown verbatim except control characters, which are escaped so a refusal cannot
/// rewrite the terminal line it is printed on.
fn verification_ignored_display(entry: &str) -> String {
    entry
        .chars()
        .map(|character| {
            if character.is_control() {
                character.escape_debug().to_string()
            } else {
                character.to_string()
            }
        })
        .collect()
}
