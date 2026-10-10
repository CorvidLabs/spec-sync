//! Shared analysis disclosures and terminal report rendering.

use super::DepsReport;

/// One sentence naming the languages whose imports could not be read, or `None`
/// when every declared source file that could have imports was analysable.
/// Single formatting site for every renderer, so a language cannot be disclosed
/// in one output format and hidden in another.
///
/// Only languages that HAVE imports appear here (see
/// `language_has_import_concept`): a note that fires for YAML and shell files
/// says nothing true and drowns the cases where something really was missed.
pub fn unanalyzed_languages_note(report: &DepsReport) -> Option<String> {
    if report.unanalyzed_languages.is_empty() {
        return None;
    }
    let listed = report
        .unanalyzed_languages
        .iter()
        .map(|(language, files)| format!("{language} ({files} file(s))"))
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!(
        "Import analysis is not implemented for {listed}; \
         undeclared imports in those files were not checked"
    ))
}

/// One sentence naming the imports that were read but could not be attributed
/// to a spec module, or `None` when every collected import was either owned or
/// recognisably foreign. Single formatting site, like
/// `unanalyzed_languages_note`.
///
/// The count is exact; the listing is capped so a large project cannot bury the
/// rest of the report. Machine consumers get the full list in the JSON
/// `unresolved_imports` array.
pub fn unresolved_imports_note(report: &DepsReport) -> Option<String> {
    const LISTED: usize = 5;
    if report.unresolved_imports.is_empty() {
        return None;
    }
    let total = report.unresolved_imports.len();
    let listed = report
        .unresolved_imports
        .iter()
        .take(LISTED)
        .map(|(module, import)| format!("{module} imports {import}"))
        .collect::<Vec<_>>()
        .join(", ");
    let remainder = total.saturating_sub(LISTED);
    let tail = if remainder > 0 {
        format!(", +{remainder} more")
    } else {
        String::new()
    };
    Some(format!(
        "{total} import(s) could not be mapped to a spec module, so they were \
         not checked against depends_on: {listed}{tail}"
    ))
}

/// The success sentence, qualified when part of the tree went unread or
/// unattributed. "Valid" over a graph the command could not fully build is a
/// claim it has not earned (#477).
pub fn valid_declarations_line(report: &DepsReport) -> &'static str {
    match (
        report.unanalyzed_languages.is_empty(),
        report.unresolved_imports.is_empty(),
    ) {
        (true, true) => "All dependency declarations are valid.",
        (false, true) => "All dependency declarations are valid for the languages analysed.",
        (true, false) => "All dependency declarations are valid for the imports that resolved.",
        (false, false) => {
            "All dependency declarations are valid for the languages analysed \
             and the imports that resolved."
        }
    }
}

/// Format the dependency report as a printable summary.
#[allow(dead_code)]
pub fn format_report(report: &DepsReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Modules: {}  Edges: {}\n",
        report.module_count, report.edge_count
    ));

    if report.errors.is_empty() && report.warnings.is_empty() {
        out.push_str(valid_declarations_line(report));
        out.push('\n');
        for note in [
            unanalyzed_languages_note(report),
            unresolved_imports_note(report),
        ]
        .into_iter()
        .flatten()
        {
            out.push_str(&format!("{note}\n"));
        }
        return out;
    }

    for note in [
        unanalyzed_languages_note(report),
        unresolved_imports_note(report),
    ]
    .into_iter()
    .flatten()
    {
        out.push_str(&format!("{note}\n"));
    }

    if !report.errors.is_empty() {
        out.push_str(&format!("\nErrors ({}):\n", report.errors.len()));
        for e in &report.errors {
            out.push_str(&format!("  - {e}\n"));
        }
    }

    if !report.warnings.is_empty() {
        out.push_str(&format!("\nWarnings ({}):\n", report.warnings.len()));
        for w in &report.warnings {
            out.push_str(&format!("  - {w}\n"));
        }
    }

    out
}

#[cfg(test)]
mod tests;
