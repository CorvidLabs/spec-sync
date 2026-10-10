use crate::change;
use crate::change::ChangeRecord;
use crate::change::ChangeState;
use crate::change::InterviewQuestion;
use crate::types::OutputFormat;
use colored::Colorize;
use std::path::Path;
use std::process;

pub(super) const INVALID_CORRECTION_LEDGER_TEXT: &str = "correction ledger integrity is invalid; restore corrections.json from trusted history before inspecting lifecycle status";
pub(super) fn print_record(
    root: &Path,
    record: &ChangeRecord,
    format: OutputFormat,
    include_questions: bool,
    strict: bool,
) -> Result<(), String> {
    let questions = if include_questions {
        change::next_questions_for(root, record)
    } else {
        Vec::new()
    };
    match format {
        OutputFormat::Json => {
            // JSON keeps digests and correction ledgers for machine consumers.
            let summary = change::summarize_change_with_strict(root, record, strict);
            let effective_definition = change::effective_change_definition(root, record)?;
            let corrections = change::correction_history(root, record)?;
            // `change supersede --digest` needs a specsync.acceptance-entry.v1
            // digest and nothing emitted one, so callers had to read
            // verification.json by hand. Surface the entries here.
            let acceptance_entries = change::acceptance_entries(root, record);
            print_json(&serde_json::json!({
                "change": record,
                "effective_definition": effective_definition,
                "acceptance_entries": acceptance_entries,
                "corrections": corrections,
                "summary": summary,
                "questions": questions,
                "briefing": change::open_change_briefing(root, record),
            }));
        }
        _ => {
            // Text mode must not invoke digest-bearing loaders into cleartext sinks
            // (CodeQL rust/cleartext-logging / alert #58). Human output uses interview
            // identity/state only; digests and correction-ledger details stay JSON-only.
            ensure_text_correction_ledger_valid(root, record)?;
            // `text_mode_next_action` uses lightweight artifact file reads only.
            print_change_text_identity(record);
            print_open_change_briefing(root, record);
            let next = text_mode_next_action(root, record, &questions);
            println!("  Next: {next}");
            // The handoff line is literal prose plus the change ID; no digest reaches it.
            print_handoff_line(&change::handoff_summary(root, record));
            print_change_text_answers(record);
            print_change_text_correction_counts(record);
            if include_questions && !questions.is_empty() {
                println!("\nInterview:");
                for question in &questions {
                    println!("  {} — {}", question.id.cyan(), question.prompt);
                    if !question.choices.is_empty() {
                        println!("    Choices: {}", question.choices.join(", "));
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn print_mutation_record(
    root: &Path,
    id: &str,
    result: &change::DefinitionMutationResult,
    format: OutputFormat,
    include_questions: bool,
    strict: bool,
) -> Result<(), String> {
    let record = &result.change;
    let questions = if include_questions {
        change::next_questions_for(root, record)
    } else {
        Vec::new()
    };
    match format {
        OutputFormat::Json => {
            // The summary was built while the domain mutation still held the project lock.
            // Reusing it keeps the machine response consistent with the validated correction
            // snapshot even if corrections.json changes after persistence.
            let summary = if strict {
                &result.strict_summary
            } else {
                &result.summary
            };
            let acceptance_entries = change::acceptance_entries(root, record);
            print_json(&serde_json::json!({
                "change": record,
                "effective_definition": result.effective_definition,
                "acceptance_entries": acceptance_entries,
                "corrections": result.corrections,
                "summary": summary,
                "questions": questions,
                "briefing": change::open_change_briefing(root, record),
            }));
        }
        _ => {
            // The domain operation validated correction history while holding the mutation lock.
            // Do not turn successful persistence into a false command failure by rereading that
            // ledger after the transaction has completed.
            print_change_text_identity(record);
            print_open_change_briefing(root, record);
            let next = text_mode_next_action(root, record, &questions);
            println!("  Next: {next}");
            print_handoff_line(&change::handoff_summary(root, record));
            print_change_text_answers(record);
            // Keep correction-derived snapshot values out of the human sink. A state-only reload
            // preserves the established counts without rereading corrections.json; failure is
            // intentionally non-fatal because the mutation has already persisted successfully.
            if let Ok(current) = change::load_change(root, id) {
                print_change_text_correction_counts(&current);
            }
            if include_questions && !questions.is_empty() {
                println!("\nInterview:");
                for question in &questions {
                    println!("  {} — {}", question.id.cyan(), question.prompt);
                    if !question.choices.is_empty() {
                        println!("    Choices: {}", question.choices.join(", "));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Print only non-sensitive identity fields for human text sinks.
///
/// Digests / correction ledgers must not flow into `println!` (CodeQL
/// `rust/cleartext-logging`). Callers must not pass values produced by
/// `effective_change_definition` / `correction_history` into this path.
fn print_change_text_identity(record: &ChangeRecord) {
    let id = record.id.as_str();
    let title = record.title.as_str();
    let state = record.state.as_str();
    println!("{} {}", id.bold(), title);
    println!("  State: {state}");
    print_legacy_workflow_notice(record);
}

/// Name the lifecycle when — and only when — it is the weaker one.
///
/// A repository upgraded from 5.x keeps `version: 1` in its policy, `init` short-circuits on an
/// existing project without raising it, and every change created there is workflow-v1. Nothing said
/// so until `ship` refused several verbs later, by which point the change cannot be re-created on
/// the other lifecycle without redoing the work.
///
/// This announces STATE, not a verb. A reader who believes they are on v2 needs their assumption
/// contradicted; being told a command exists does not contradict anything, because they had no
/// reason to think it applied to them. v2 stays silent so the normal path gains no noise.
fn print_legacy_workflow_notice(record: &ChangeRecord) {
    if record.workflow_version >= 2 {
        return;
    }
    println!(
        "  {} workflow v1 (legacy) — this change uses `change accept` and `change archive`, not `change finalize`",
        "!".yellow().bold()
    );
    println!("    adopt the current lifecycle for NEW changes with `specsync change adopt`");
}

/// Print only non-sensitive correction counts from state.json for human text sinks.
///
/// Callers must source this record independently from correction-ledger validation results so
/// correction values, ledger bytes, and digest material cannot flow into cleartext output.
fn print_change_text_correction_counts(record: &ChangeRecord) {
    if record.correction_count > 0 {
        println!(
            "  Corrections: {} recorded (use --json for the audit ledger)",
            record.correction_count
        );
    }
    if !record.acceptance_owner_corrections.is_empty() {
        println!(
            "  Acceptance owner corrections: {} (use --json for details)",
            record.acceptance_owner_corrections.len()
        );
    }
}

fn print_change_text_answers(record: &ChangeRecord) {
    if record.answers.is_empty() {
        return;
    }
    let answer_summary = record
        .answers
        .iter()
        .map(|(field, value)| format!("{field}={value}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!("  Answers: {answer_summary}");
}

/// Human next-step string from interview/state and lightweight artifact completeness.
/// Avoids digest-bearing loaders (CodeQL cleartext-logging); only reads selected artifact files.
pub(super) fn text_mode_next_action(
    root: &Path,
    record: &ChangeRecord,
    questions: &[InterviewQuestion],
) -> String {
    let id = record.id.as_str();
    match record.state {
        ChangeState::Draft if !questions.is_empty() => {
            let question = questions
                .first()
                .map(|question| question.id.as_str())
                .unwrap_or("<question>");
            format!("run `specsync change answer {id} {question} <answer>`")
        }
        ChangeState::Draft if !change::artifacts_complete_for_guidance(root, record) => {
            format!("complete selected artifacts, then run `specsync change status {id}`")
        }
        ChangeState::Draft => {
            format!("run `specsync change approve {id} --actor <name>`")
        }
        ChangeState::Approved | ChangeState::Implementing => {
            format!("run `specsync change check {id}`")
        }
        ChangeState::Verifying => {
            format!(
                "run `specsync change check {id} --commit` if needed, then `specsync change ship-status {id}` (or `ship {id}`) — scoped review then finalize before merging; merging first orphans verification evidence and blocks earlier accepted changes sharing a delivery input from archiving"
            )
        }
        ChangeState::Accepted if record.workflow_version >= 2 => {
            format!("run `specsync change ship {id}` or `specsync change finalize {id}`")
        }
        ChangeState::Accepted => format!("run `specsync change archive {id}`"),
        ChangeState::Archived => "no further action".into(),
    }
}

pub(super) fn ensure_text_correction_ledger_valid(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(), String> {
    change::effective_change_definition(root, record)
        .is_ok()
        .then_some(())
        .ok_or_else(|| INVALID_CORRECTION_LEDGER_TEXT.to_string())
}
/// Show the same briefing text and JSON carry. A bounded excerpt, then the path for the rest.
fn print_open_change_briefing(root: &Path, record: &change::ChangeRecord) {
    let briefing = change::open_change_briefing(root, record);
    if briefing.lessons.is_empty() && briefing.principles.is_none() {
        return;
    }
    if !briefing.lessons.is_empty() {
        println!(
            "\n  {} what these modules already learned:",
            "Lessons:".bold()
        );
        for lesson in &briefing.lessons {
            let more = if lesson.truncated {
                ", more in the file"
            } else {
                ""
            };
            println!("    {} ({} line(s){more})", lesson.path, lesson.lines);
            for line in lesson.excerpt.lines() {
                println!("      {line}");
            }
        }
    }
    if let Some(principles) = &briefing.principles {
        println!("\n  {} {}", "Principles:".bold(), principles.path);
        if principles.missing {
            println!("    the file could not be read");
        } else {
            for line in principles.excerpt.lines() {
                println!("    {line}");
            }
            if principles.truncated {
                println!("    more in the file");
            }
        }
    }
}

pub(super) fn lesson_bundle_line() -> String {
    // The file name only. The archive directory is printed on its own line from
    // `finalize_change`'s path, and formatting that path into this string is what
    // CodeQL `rust/cleartext-logging` treats as logging correction history.
    format!("Lesson bundle: {}", change::LESSON_BUNDLE_FILE)
}
pub(super) fn unreadable_json(roster: &change::ChangeRoster) -> Vec<serde_json::Value> {
    roster
        .unreadable
        .iter()
        .map(|entry| serde_json::json!({ "id": entry.id, "error": entry.reason }))
        .collect()
}

pub(super) fn print_unreadable_rows(roster: &change::ChangeRoster) {
    for entry in &roster.unreadable {
        println!("{}  unreadable  {}", entry.id, entry.reason);
    }
}

/// The exit status for a partial roster.
///
/// A degraded roster must not exit 0. `change audit` already exits 1 on the very
/// same tree, and two commands disagreeing about whether one repository is
/// healthy is the defect this codebase keeps re-learning (#576). The rows are
/// printed first, so the operator sees every healthy change *and* the failure.
pub(super) fn unreadable_error(roster: &change::ChangeRoster) -> Result<(), String> {
    match roster.unreadable.len() {
        0 => Ok(()),
        1 => Err(roster.unreadable[0].reason.clone()),
        count => Err(format!(
            "{count} active change workspaces could not be read; the listing above is incomplete"
        )),
    }
}

/// Print the roster, then fail if any workspace could not be read.
///
/// Splitting this from [`print_records`] keeps one rule in one place: healthy
/// rows are always printed, the unreadable ones are always named, and the exit
/// status always reflects that the view is partial.
pub(super) fn print_roster(
    root: &Path,
    roster: &change::ChangeRoster,
    format: OutputFormat,
    strict: bool,
) -> Result<(), String> {
    if matches!(format, OutputFormat::Json) && roster.is_degraded() {
        // A bare array cannot say "and there were three I could not read", so a
        // degraded roster is reported as an object. Healthy rosters keep the
        // historical array shape exactly, which is every project that is not
        // already being lied to.
        let summaries: Vec<_> = roster
            .records
            .iter()
            .map(|record| summary_with_briefing(root, record, strict))
            .collect();
        let error = unreadable_error(roster).err();
        print_json(&serde_json::json!({
            "changes": summaries,
            "unreadable": unreadable_json(roster),
            "error": error,
        }));
        // Exit here rather than returning Err. `cmd_change`'s tail handler prints
        // its own `{"error": …}` document in JSON mode, and a second document
        // concatenated after this one makes stdout unparseable — the failure this
        // whole change exists to stop, reintroduced one layer up.
        process::exit(1);
    }
    if !roster.records.is_empty() || !roster.is_degraded() {
        print_records(root, &roster.records, format, strict)?;
    }
    print_unreadable_rows(roster);
    unreadable_error(roster)
}

fn summary_with_briefing(root: &Path, record: &ChangeRecord, strict: bool) -> serde_json::Value {
    let summary = change::summarize_change_with_strict(root, record, strict);
    let mut value = serde_json::to_value(summary).unwrap_or(serde_json::Value::Null);
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "briefing".to_string(),
            serde_json::to_value(change::open_change_briefing(root, record))
                .unwrap_or(serde_json::Value::Null),
        );
    }
    value
}

fn print_records(
    root: &Path,
    records: &[ChangeRecord],
    format: OutputFormat,
    strict: bool,
) -> Result<(), String> {
    match format {
        OutputFormat::Json => {
            let summaries: Vec<_> = records
                .iter()
                .map(|record| summary_with_briefing(root, record, strict))
                .collect();
            print_json(&summaries);
            Ok(())
        }
        _ if records.is_empty() => {
            println!("No active SDD changes.");
            Ok(())
        }
        _ => {
            // Text list view avoids digest-bearing summarize loaders (cleartext-logging).
            // Preflight every record before printing any successful lifecycle projection.
            // Otherwise a later invalid ledger would leave misleading earlier success rows.
            for record in records {
                ensure_text_correction_ledger_valid(root, record)?;
            }
            for record in records {
                let questions = change::next_questions_for(root, record);
                let id = record.id.clone();
                let title = record.title.clone();
                let state = record.state.as_str().to_owned();
                let next = text_mode_next_action(root, record, &questions);
                println!("{:<14}  {state:<13}  {title}  next: {next}", id.bold());
                print_open_change_briefing(root, record);
            }
            Ok(())
        }
    }
}

pub(super) fn print_transition(record: &ChangeRecord, format: OutputFormat, message: &str) {
    match format {
        OutputFormat::Json => print_json(&serde_json::json!({
            "id": record.id,
            "state": record.state,
            "message": message,
        })),
        _ => println!(
            "{} {}: {} ({})",
            "✓".green(),
            record.id,
            message,
            record.state.as_str()
        ),
    }
}

/// The approve transition, plus the handoff verdict the new state produces: text gets the
/// `Handoff:` line, JSON gets it as `handoff` beside the transition.
pub(super) fn print_transition_with_handoff(
    root: &Path,
    record: &ChangeRecord,
    format: OutputFormat,
    message: &str,
) {
    let handoff = change::handoff_summary(root, record);
    match format {
        OutputFormat::Json => print_json(&serde_json::json!({
            "id": record.id,
            "state": record.state,
            "message": message,
            "handoff": handoff,
        })),
        _ => {
            print_transition(record, format, message);
            print_handoff_line(&handoff);
        }
    }
}

/// The `Handoff:` line every lifecycle text result ends with: whether clearing context now
/// loses anything, in the domain's words. `HandoffSummary` is built from literals and the
/// change ID only, so no digest can reach this sink through it.
pub(super) fn print_handoff_line(handoff: &change::HandoffSummary) {
    let mut line = format!(
        "  Handoff: {} — {}",
        handoff.readiness.as_str(),
        handoff.reason
    );
    if !handoff.before_clearing.is_empty() {
        line.push_str(". Before clearing: ");
        line.push_str(&handoff.before_clearing.join("; "));
    }
    println!("{line}");
}

/// Reloads the record by ID — after review or finalize the caller holds only the evidence —
/// and prints its handoff line. A record that cannot be reloaded prints nothing rather than a
/// guess; the command's own result already stands.
pub(super) fn print_handoff_line_for(root: &Path, id: &str) {
    if let Ok(record) = change::load_change(root, id) {
        print_handoff_line(&change::handoff_summary(root, &record));
    }
}

pub(super) fn print_json<Value: serde::Serialize>(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(content) => println!("{content}"),
        Err(error) => eprintln!("{{\"error\":\"failed to serialize output: {error}\"}}"),
    }
}
