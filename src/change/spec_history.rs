// Spec history fragments (#1084, REQ-change-159, REQ-change-160).
//
// A change's record in a module's history used to be two edits to the module's spec: a row
// appended to its `## Change Log` table and a `version:` bump. Two changes to one module then
// edited the same lines, so every pair conflicted, and GitHub's merge button, which runs no merge
// driver, could not merge them. Each change now writes its own file,
// `history/<change-id>.md` beside the spec, so two changes never touch the same line. A table
// written before this stays exactly as it is, and `specsync history` reads it as the frozen head
// of the module's history.

/// The directory beside a module's spec that holds its history fragments.
pub const SPEC_HISTORY_DIR: &str = "history";

/// The largest fragment `check` and `history` read. A fragment is a date and one line.
const MAX_HISTORY_FRAGMENT_BYTES: u64 = 64 * 1024;

/// One change's record in a module's history (REQ-change-160).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryFragment {
    /// The change id, equal to the fragment's file name without `.md`.
    pub change: String,
    /// The UTC date the change was recorded, `YYYY-MM-DD`.
    pub date: String,
    /// One line: the change's title.
    pub summary: String,
}

/// A module's whole history: its frozen Change Log table and its fragments (REQ-cmd-change-037).
#[derive(Debug, Clone, Serialize)]
pub struct SpecHistory {
    /// The module name.
    pub module: String,
    /// The spec, project-relative.
    pub spec: String,
    /// The spec's frozen `version:` scalar, unquoted, when it has one.
    pub version: Option<String>,
    /// The Change Log table's header and separator lines as written; empty without a table.
    pub table_header: Vec<String>,
    /// The Change Log table's rows as written.
    pub rows: Vec<String>,
    /// The fragments, in date then change-id order.
    pub fragments: Vec<HistoryFragment>,
}

/// The `history/` directory beside `spec_path`.
pub fn spec_history_dir(spec_path: &Path) -> PathBuf {
    spec_path
        .parent()
        .map_or_else(|| PathBuf::from(SPEC_HISTORY_DIR), |parent| parent.join(SPEC_HISTORY_DIR))
}

/// Where `change_id`'s fragment for the module whose spec is `spec_path` lives.
pub fn history_fragment_path(spec_path: &Path, change_id: &str) -> PathBuf {
    spec_history_dir(spec_path).join(format!("{change_id}.md"))
}

/// Whether the project-relative `path` is one of the history fragments of the module whose spec is
/// `spec_path`: a `.md` file directly in its `history/` directory. A module owns every one of them,
/// as it owns its companions, so a later change declaring the module covers an edit to any of them
/// and a successor's acceptance entry for a predecessor's fragment has an owner.
fn is_module_history_path(root: &Path, spec_path: &Path, path: &str) -> bool {
    let directory = portable_project_path(root, &spec_history_dir(spec_path));
    path.strip_prefix(directory.as_str())
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(|name| name.len() > ".md".len() && !name.contains('/') && name.ends_with(".md"))
}

/// The bytes `change check` writes for one change's fragment.
pub fn history_fragment_content(change_id: &str, date: &str, summary: &str) -> String {
    format!(
        "---\nchange: {}\ndate: {date}\n---\n\n{}\n",
        yaml_double_quoted(change_id),
        one_line_summary(summary)
    )
}

/// `value` as a YAML double-quoted scalar, so every id `validate_change_id` accepts (it refuses
/// control characters) reads back as the same string: `true`, `'foo'` and `"x"` stay text (#1086
/// review).
fn yaml_double_quoted(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
}

/// A title as one line: line breaks become spaces, so the fragment always parses back.
fn one_line_summary(text: &str) -> String {
    text.split(['\r', '\n'])
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether the module whose spec text is `spec` and whose spec file is `spec_path` records
/// `change_id`: by its fragment, or, for a change an earlier SpecSync materialized, by a Change
/// Log row naming it (REQ-change-159). This is the evidence that the module's materialization ran
/// (#741), so it errs towards "recorded": a reformatted row still counts.
fn module_records_change(spec_path: &Path, spec: &str, change_id: &str) -> bool {
    fs::symlink_metadata(history_fragment_path(spec_path, change_id)).is_ok()
        || changelog_records_change(spec, change_id)
}

/// Parse one fragment. `file_name` is the entry's name in `history/`. The error says what is wrong
/// with the file, without naming it; the caller names it.
pub fn parse_history_fragment(file_name: &str, content: &str) -> Result<HistoryFragment, String> {
    let stem = file_name
        .strip_suffix(".md")
        .ok_or_else(|| "a fragment is named `<change-id>.md`".to_string())?;
    validate_change_id(stem).map_err(|_| {
        format!(
            "`{}` is not a change id, and a fragment is named `<change-id>.md`",
            stem.escape_default()
        )
    })?;
    let normalized = content.replace("\r\n", "\n");
    let rest = normalized
        .strip_prefix("---\n")
        .ok_or_else(|| "it does not open with `---` frontmatter".to_string())?;
    let (frontmatter, body) = if let Some(body) = rest.strip_prefix("---\n") {
        ("", body)
    } else if let Some(end) = rest.find("\n---\n") {
        (&rest[..end], &rest[end + "\n---\n".len()..])
    } else if let Some(frontmatter) = rest.strip_suffix("\n---") {
        (frontmatter, "")
    } else {
        return Err("its frontmatter has no closing `---`".into());
    };
    let mut change = None;
    let mut date = None;
    for line in frontmatter.lines() {
        // Only top-level `key: value` lines are read; anything else belongs to a key this
        // version does not know, and other keys are allowed (REQ-change-160).
        if line.starts_with(char::is_whitespace) || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let slot = match key.trim() {
            "change" => &mut change,
            "date" => &mut date,
            _ => continue,
        };
        if slot.is_some() {
            return Err(format!("its frontmatter sets `{}:` twice", key.trim()));
        }
        *slot = Some(
            yaml_scalar(value.trim())
                .map_err(|problem| format!("its `{}:` {problem}", key.trim()))?,
        );
    }
    let change = change.ok_or_else(|| "its frontmatter has no `change:`".to_string())?;
    if change != stem {
        return Err(format!(
            "its `change:` is `{}`, but the file is named for `{stem}`",
            change.escape_default()
        ));
    }
    let date = date.ok_or_else(|| "its frontmatter has no `date:`".to_string())?;
    if !is_calendar_date(&date) {
        return Err(format!(
            "its `date:` `{}` is not a YYYY-MM-DD date",
            date.escape_default()
        ));
    }
    let summary = body.trim();
    if summary.is_empty() {
        return Err("it has no summary after its frontmatter".into());
    }
    if summary.contains('\n') {
        return Err("its summary is more than one line".into());
    }
    Ok(HistoryFragment {
        change,
        date,
        summary: summary.to_string(),
    })
}

/// A one-line YAML scalar as text: double-quoted with `\"` and `\\` escapes, single-quoted with
/// `''`, or plain. A fragment written before #1086's review holds a plain id, and still reads.
fn yaml_scalar(value: &str) -> Result<String, String> {
    if value.len() >= 2
        && let Some(inner) = value
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
    {
        let mut text = String::with_capacity(inner.len());
        let mut characters = inner.chars();
        while let Some(character) = characters.next() {
            match character {
                '\\' => match characters.next() {
                    Some(escaped @ ('"' | '\\')) => text.push(escaped),
                    _ => return Err("has an escape other than `\\\"` or `\\\\`".into()),
                },
                '"' => return Err("has an unescaped `\"` inside its quotes".into()),
                other => text.push(other),
            }
        }
        return Ok(text);
    }
    if value.len() >= 2
        && let Some(inner) = value
            .strip_prefix('\'')
            .and_then(|rest| rest.strip_suffix('\''))
    {
        return Ok(inner.replace("''", "'"));
    }
    Ok(value.to_string())
}

/// `YYYY-MM-DD` naming a real calendar day.
fn is_calendar_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    let number = |range: Range<usize>| -> Option<u32> {
        let digits = &value[range];
        digits
            .bytes()
            .all(|byte| byte.is_ascii_digit())
            .then(|| digits.parse().ok())
            .flatten()
    };
    let (Some(year), Some(month), Some(day)) = (number(0..4), number(5..7), number(8..10)) else {
        return false;
    };
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

/// Every fragment beside `spec_path`, and one error per entry of `history/` that is not a valid
/// fragment or whose change the spec's Change Log also records (REQ-validator-048). Each error names
/// the entry by its project-relative path.
pub fn scan_history_fragments(
    root: &Path,
    spec_path: &Path,
    spec: &str,
) -> (Vec<HistoryFragment>, Vec<String>) {
    let relative = |path: &Path| {
        path.strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    };
    let directory = spec_history_dir(spec_path);
    let mut fragments = Vec::new();
    let mut errors = Vec::new();
    match fs::symlink_metadata(&directory) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (fragments, errors);
        }
        Err(error) => {
            errors.push(format!("cannot read `{}`: {error}", relative(&directory)));
            return (fragments, errors);
        }
        Ok(metadata) if !metadata.is_dir() => {
            errors.push(format!(
                "`{}` is not a directory; a spec's history fragments live in a `{SPEC_HISTORY_DIR}/` directory beside it",
                relative(&directory)
            ));
            return (fragments, errors);
        }
        Ok(_) => {}
    }
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(format!("cannot read `{}`: {error}", relative(&directory)));
            return (fragments, errors);
        }
    };
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => paths.push(entry.path()),
            Err(error) => {
                errors.push(format!("cannot read `{}`: {error}", relative(&directory)));
            }
        }
    }
    paths.sort();
    let spec_relative = relative(spec_path);
    for path in paths {
        let shown = relative(&path);
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            errors.push(format!(
                "history entry `{shown}` is not a fragment: its name is not UTF-8"
            ));
            continue;
        };
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                errors.push(format!("cannot read history entry `{shown}`: {error}"));
                continue;
            }
        };
        if !metadata.is_file() {
            errors.push(format!(
                "history entry `{shown}` is not a fragment: a fragment is a regular `<change-id>.md` file"
            ));
            continue;
        }
        if metadata.len() > MAX_HISTORY_FRAGMENT_BYTES {
            errors.push(format!(
                "history fragment `{shown}` is {} bytes, more than the {MAX_HISTORY_FRAGMENT_BYTES} a fragment may hold",
                metadata.len()
            ));
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) => {
                errors.push(format!("cannot read history fragment `{shown}`: {error}"));
                continue;
            }
        };
        match parse_history_fragment(name, &content) {
            Ok(fragment) => {
                if changelog_records_change(spec, &fragment.change) {
                    errors.push(format!(
                        "history fragment `{shown}` records `{}`, and the Change Log in `{spec_relative}` records it too; keep one of the two records",
                        fragment.change
                    ));
                }
                fragments.push(fragment);
            }
            Err(problem) => errors.push(format!("history fragment `{shown}`: {problem}")),
        }
    }
    fragments.sort_by(|left, right| {
        (left.date.as_str(), left.change.as_str()).cmp(&(right.date.as_str(), right.change.as_str()))
    });
    (fragments, errors)
}

/// A spec's `## Change Log` table: the header and separator lines and the rows, as written.
struct ChangeLogTable {
    header: Vec<String>,
    rows: Vec<String>,
}

/// The table in the spec's last `## Change Log` section: its first table line is the header, a
/// following `|---|` line the separator, and every table line after it up to the first line that
/// is not one is a row.
fn change_log_table(spec: &str) -> Option<ChangeLogTable> {
    let lines: Vec<&str> = spec
        .lines()
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    let start = lines
        .iter()
        .rposition(|line| line.trim_end() == "## Change Log")?;
    let section = lines[start + 1..]
        .iter()
        .take_while(|line| !line.starts_with("## "));
    let is_table_line = |line: &str| {
        let line = line.trim();
        line.len() >= 2 && line.starts_with('|') && line.ends_with('|')
    };
    let mut section = section.skip_while(|line| !is_table_line(line));
    let header_line = section.next()?.trim_end();
    let mut header = vec![header_line.to_string()];
    let mut rows = Vec::new();
    for (index, line) in section.enumerate() {
        if !is_table_line(line) {
            break;
        }
        let line = line.trim_end();
        let is_separator = line
            .trim()
            .trim_matches('|')
            .split('|')
            .all(|cell| {
                let cell = cell.trim();
                !cell.is_empty() && cell.chars().all(|ch| matches!(ch, '-' | ':'))
            });
        if index == 0 && is_separator {
            header.push(line.to_string());
        } else {
            rows.push(line.to_string());
        }
    }
    Some(ChangeLogTable { header, rows })
}

/// The `version:` scalar in the spec's frontmatter, unquoted and without a trailing comment.
fn spec_frontmatter_version(spec: &str) -> Option<String> {
    let mut lines = spec.lines().map(|line| line.strip_suffix('\r').unwrap_or(line));
    if lines.next()? != "---" {
        return None;
    }
    lines.take_while(|line| *line != "---").find_map(|line| {
        let raw = line.strip_prefix("version:")?.trim();
        let without_comment = raw.split_once(" #").map_or(raw, |(value, _)| value);
        Some(without_comment.trim().trim_matches(['\'', '"']).to_string())
    })
}

/// The version `steps` changes after `frozen`: an integer counts on; `major.minor.patch` counts
/// its patch on. Anything else has no successor, and the cell stays empty.
fn version_after(frozen: &str, steps: u64) -> Option<String> {
    if let Ok(integer) = frozen.parse::<u64>() {
        return integer.checked_add(steps).map(|value| value.to_string());
    }
    let parts: Vec<&str> = frozen.split('.').collect();
    let [major, minor, patch] = parts.as_slice() else {
        return None;
    };
    let (major, minor, patch) = (
        major.parse::<u64>().ok()?,
        minor.parse::<u64>().ok()?,
        patch.parse::<u64>().ok()?,
    );
    Some(format!("{major}.{minor}.{}", patch.checked_add(steps)?))
}

/// One Change Log row in the table's own columns, as `change check` used to write one: Version,
/// Date, the description column (Change, Changes, Description or Notes) and Author are filled, any
/// other column is left empty. `None` when the table has no description column.
fn change_log_row(
    columns: &[String],
    version: Option<&str>,
    date: &str,
    description: &str,
) -> Option<String> {
    if columns.len() < 2 {
        return None;
    }
    let mut recognized_description = false;
    let cells: Vec<String> = columns
        .iter()
        .map(|column| match column.to_ascii_lowercase().as_str() {
            "version" => version.unwrap_or_default().to_string(),
            "date" => date.to_string(),
            "change" | "changes" | "description" | "notes" => {
                recognized_description = true;
                description.to_string()
            }
            "author" => "SpecSync".to_string(),
            _ => String::new(),
        })
        .collect();
    recognized_description.then(|| format!("| {} |", cells.join(" | ")))
}

/// Read `module`'s history: the spec the registry-aware resolver names, its frozen Change Log table
/// and its fragments (REQ-cmd-change-037). An unknown module, or any entry of `history/` that
/// `check` would refuse, is an error naming it.
pub fn read_spec_history(root: &Path, module: &str) -> Result<SpecHistory, String> {
    let single_component = {
        let mut components = Path::new(module).components();
        matches!(components.next(), Some(std::path::Component::Normal(_)))
            && components.next().is_none()
    };
    if module.is_empty() || !single_component || module.contains(['/', '\\']) {
        return Err(format!(
            "`{}` is not a spec module name",
            module.escape_default()
        ));
    }
    let specs_dir = crate::config::load_config(root).specs_dir;
    let (spec_path, _) = canonical_module_paths(root, &specs_dir, module)?;
    let spec_relative = spec_path
        .strip_prefix(root)
        .unwrap_or(&spec_path)
        .to_string_lossy()
        .replace('\\', "/");
    let spec = match fs::read_to_string(&spec_path) {
        Ok(spec) => spec,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!(
                "no spec module named `{module}`: `{spec_relative}` does not exist"
            ));
        }
        Err(error) => return Err(format!("cannot read `{spec_relative}`: {error}")),
    };
    let (fragments, errors) = scan_history_fragments(root, &spec_path, &spec);
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let table = change_log_table(&spec);
    Ok(SpecHistory {
        module: module.to_string(),
        spec: spec_relative,
        version: spec_frontmatter_version(&spec),
        table_header: table
            .as_ref()
            .map(|table| table.header.clone())
            .unwrap_or_default(),
        rows: table.map(|table| table.rows).unwrap_or_default(),
        fragments,
    })
}

/// The history as one Markdown table: the frozen table's header and rows unchanged, then one row
/// per fragment in its columns, the Version column numbered on from the frozen `version:`. A spec
/// with no table gets `| Date | Change |`.
pub fn render_spec_history_table(history: &SpecHistory) -> String {
    let default_header = ["| Date | Change |".to_string(), "|------|--------|".to_string()];
    let header: &[String] = if history.table_header.is_empty() {
        &default_header
    } else {
        &history.table_header
    };
    let columns: Vec<String> = header[0]
        .trim()
        .trim_matches('|')
        .split('|')
        .map(|column| column.trim().to_string())
        .collect();
    let mut out = String::new();
    for line in header.iter().chain(&history.rows) {
        out.push_str(line);
        out.push('\n');
    }
    for (index, fragment) in history.fragments.iter().enumerate() {
        let version = history
            .version
            .as_deref()
            .and_then(|frozen| version_after(frozen, index as u64 + 1));
        let description = format!("{}: {}", fragment.change, fragment.summary.replace('|', "\\|"));
        let row = change_log_row(&columns, version.as_deref(), &fragment.date, &description)
            .unwrap_or_else(|| format!("| {} | {description} |", fragment.date));
        out.push_str(&row);
        out.push('\n');
    }
    out
}
