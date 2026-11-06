// Relayed human decisions on approvals and reviews (#633, audit D7; CHANGE-13, SHIP-2.e).
//
// An approval's digest binds the content it approved and never its actor, so the name on an
// approval or a review is a claim (CHANGE-6.d). When an agent records a decision a person made
// somewhere else, `relayed_by` keeps who is said to have decided, their exact words, where they
// said it and which agent wrote it down. The shape is shared with Chough (chough#123) and Leif
// approved it there: exactly four non-blank strings of at most 4096 bytes, with `null` and
// unknown keys refused.
//
// It is a claim too. Nothing here authenticates anyone, no gate reads it, and no digest covers
// it: the approval digest never covered `actor`, and the finalization review digest is computed
// with `relayed_by` cleared ("Keep it out", Leif, 2026-11-05) so a 6.x binary that drops the
// unknown key computes the same digest from the same file.

/// The words `change status` prints beside an approval's actor and a review's reviewer.
pub const RECORDED_CLAIM_BOUNDARY: &str = "a recorded claim, not authenticated";

/// The boundary sentence printed with every relayed decision, word for word the same in Chough.
pub const RELAYED_CLAIM_BOUNDARY: &str = "a relayed claim, not authenticated: it records who is said to have decided, not proof that they did.";

const RELAYED_FIELD_MAX_BYTES: usize = 4096;
const RELAYED_FIELDS: [&str; 4] = ["decided_by", "words", "where", "recorded_by"];
const RELAYED_FLAG_HINT: &str = "--words \"<their exact words>\" --where \"<where and when, and who relayed it>\" --recorded-by \"<this agent>\"";

/// A human decision an agent relayed: who is said to have decided, their words verbatim, where
/// and when they said them (including any relay chain), and the agent that wrote the record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "serde_json::Value")]
pub struct RelayedDecision {
    pub decided_by: String,
    pub words: String,
    #[serde(rename = "where")]
    pub where_: String,
    pub recorded_by: String,
}

impl TryFrom<serde_json::Value> for RelayedDecision {
    type Error = String;

    fn try_from(value: serde_json::Value) -> Result<Self, String> {
        let serde_json::Value::Object(mut object) = value else {
            return Err(
                "relayed_by must be an object holding decided_by, words, where and recorded_by; \
                 a record with no relayed decision omits the key"
                    .into(),
            );
        };
        if let Some(unknown) = object
            .keys()
            .find(|key| !RELAYED_FIELDS.contains(&key.as_str()))
        {
            return Err(format!(
                "relayed_by carries the unknown key `{unknown}`; it holds exactly decided_by, \
                 words, where and recorded_by"
            ));
        }
        let mut field = |key: &str| match object.remove(key) {
            Some(serde_json::Value::String(text)) => {
                check_relayed_field(&text).map_err(|problem| format!("relayed_by `{key}` {problem}"))?;
                Ok(text)
            }
            Some(_) => Err(format!("relayed_by `{key}` must be a string")),
            None => Err(format!("relayed_by is missing `{key}`")),
        };
        Ok(Self {
            decided_by: field("decided_by")?,
            words: field("words")?,
            where_: field("where")?,
            recorded_by: field("recorded_by")?,
        })
    }
}

/// `deserialize_with` for the record fields: an absent key is `None` through `#[serde(default)]`,
/// and a present one, `null` included, must be a valid record.
fn deserialize_present_relayed_decision<'de, Deserializer>(
    deserializer: Deserializer,
) -> Result<Option<RelayedDecision>, Deserializer::Error>
where
    Deserializer: serde::Deserializer<'de>,
{
    RelayedDecision::deserialize(deserializer).map(Some)
}

fn check_relayed_field(text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("must not be blank".into());
    }
    if text.len() > RELAYED_FIELD_MAX_BYTES {
        return Err(format!(
            "is {} bytes; a relayed decision field holds at most {RELAYED_FIELD_MAX_BYTES}",
            text.len()
        ));
    }
    Ok(())
}

/// Build the optional relayed decision from command flags.
///
/// None of `--words`, `--where` and `--recorded-by` is `None`, a decision made at this terminal
/// (CHANGE-13.a). Some but not all of them is refused before anything is written, naming every
/// missing flag and the full `rerun` command (CHANGE-13.d). `decided_by` is the approval's
/// `--actor` or the review's reviewer claim; a relay without one is refused, because the actor
/// would otherwise come from Git or `$USER`, which names the recording agent.
pub fn relayed_decision(
    rerun: &str,
    decided_by: Option<&str>,
    words: Option<String>,
    where_: Option<String>,
    recorded_by: Option<String>,
) -> Result<Option<RelayedDecision>, String> {
    if words.is_none() && where_.is_none() && recorded_by.is_none() {
        return Ok(None);
    }
    let missing: Vec<&str> = [
        ("--words", words.is_none()),
        ("--where", where_.is_none()),
        ("--recorded-by", recorded_by.is_none()),
    ]
    .into_iter()
    .filter_map(|(flag, absent)| absent.then_some(flag))
    .collect();
    let (Some(words), Some(where_), Some(recorded_by)) = (words, where_, recorded_by) else {
        return Err(format!(
            "a relayed decision needs --words, --where and --recorded-by together; missing: {}. \
             Re-run `{rerun} {RELAYED_FLAG_HINT}`, or leave all three out for a decision made at \
             this terminal",
            missing.join(", ")
        ));
    };
    let Some(decided_by) = decided_by else {
        return Err(format!(
            "a relayed decision names who decided with --actor <who decided>; without it the \
             actor comes from Git or $USER, which names the recording agent. Re-run \
             `{rerun} {RELAYED_FLAG_HINT}`"
        ));
    };
    for (flag, text) in [
        ("who decided", decided_by),
        ("--words", words.as_str()),
        ("--where", where_.as_str()),
        ("--recorded-by", recorded_by.as_str()),
    ] {
        check_relayed_field(text).map_err(|problem| format!("{flag} {problem}"))?;
    }
    Ok(Some(RelayedDecision {
        decided_by: decided_by.to_string(),
        words,
        where_,
        recorded_by,
    }))
}

fn ensure_relay_names_the_decider(
    relayed_by: Option<&RelayedDecision>,
    decider: &str,
    flag: &str,
) -> Result<(), String> {
    // A library caller can build `RelayedDecision` directly, so the write APIs check every field
    // here, not only in `relayed_decision`: a record the next read refuses must never be written
    // (#900 review).
    if let Some(relay) = relayed_by {
        for (key, text) in [
            ("decided_by", relay.decided_by.as_str()),
            ("words", relay.words.as_str()),
            ("where", relay.where_.as_str()),
            ("recorded_by", relay.recorded_by.as_str()),
        ] {
            check_relayed_field(text).map_err(|problem| format!("relayed_by `{key}` {problem}"))?;
        }
    }
    match relayed_by {
        Some(relay) if relay.decided_by != decider => Err(format!(
            "the relayed decision says `{}` decided, but {flag} records `{decider}`; one record \
             names one decider, so pass the same name to both",
            relay.decided_by
        )),
        _ => Ok(()),
    }
}

/// The finalization review digest: the review's canonical bytes with `relayed_by` left out, so
/// records without a relay keep their digest and an older 6.x binary computes the same one.
fn scoped_review_digest(review: &ScopedReviewRecord) -> Result<String, String> {
    let mut projected = review.clone();
    projected.relayed_by = None;
    serde_json::to_vec(&projected)
        .map(|bytes| sha256_hex(&bytes))
        .map_err(|error| format!("failed to hash scoped review: {error}"))
}

/// A correction event as its digests hash it: the two approvals it embeds lose `relayed_by`, so
/// corrections recorded before this field existed keep their digests and an older 6.x binary
/// that drops the key computes the same ones.
fn correction_digest_projection(correction: &CorrectionRecord) -> CorrectionRecord {
    let mut projected = correction.clone();
    projected.superseded_definition_approval.relayed_by = None;
    projected.superseded_closing_approval.relayed_by = None;
    projected
}

/// A relay read back from a ledger must still name its record's own actor or reviewer: one
/// record names one decider, on read as on write.
fn ensure_read_relays_name_their_deciders<'a>(
    records: impl IntoIterator<Item = (&'a str, Option<&'a RelayedDecision>)>,
    flag: &str,
) -> Result<(), String> {
    for (decider, relay) in records {
        ensure_relay_names_the_decider(relay, decider, flag)
            .map_err(|error| format!("recorded relayed_by does not match its record: {error}"))?;
    }
    Ok(())
}

/// Review history as the append-only comparisons judge it: without relayed decisions. An older
/// 6.x binary that records the next attempt rewrites `review-attempts.json` without the key, and
/// that rewrite must not read as removed or rewritten evidence (#900 review). Every relay that
/// is present is still validated on read.
fn reviews_match_ignoring_relays(left: &[ScopedReviewRecord], right: &[ScopedReviewRecord]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            let mut left = left.clone();
            let mut right = right.clone();
            left.relayed_by = None;
            right.relayed_by = None;
            left == right
        })
}

/// A read refusal caused by a damaged relay names the one recovery that works for the state the
/// file is in (#904, CHANGE-13.e). An active change's review files stay uncommitted between
/// `change review` and `change ship`, so `git checkout HEAD` cannot restore them; the refusal
/// names `repair-relays`. Finalize commits an archived package, so `HEAD` restores it.
fn with_relayed_read_recovery(
    root: &Path,
    record: &ChangeRecord,
    path: &Path,
    message: String,
) -> String {
    if !message.contains("relayed_by") {
        return message;
    }
    let shown = portable_project_path(root, path);
    if record.state == ChangeState::Archived {
        return format!(
            "{message}. A relayed decision is written only by `specsync change approve` or \
             `specsync change review`; finalize committed this archived package, so restore it \
             with `git checkout HEAD -- {shown}`"
        );
    }
    format!(
        "{message}. The damaged relayed decision is in {shown}. A relayed decision is written \
         only by `specsync change approve` or `specsync change review`; repair it with \
         `specsync change repair-relays {}`, which puts the original back from the last commit or \
         from its twin copy and otherwise removes only the damaged relayed_by, never the approval \
         or review it belongs to",
        record.id
    )
}

/// The status projection of one recorded approval or review: who it names, the review verdict,
/// and any relayed decision. It carries no digest, so text output may print every field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordedClaim {
    pub gate: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verdict: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relayed_by: Option<RelayedDecision>,
}

/// The effective definition approval, never one a send-back cancelled, and the latest scoped
/// review, as named claims with any relayed decision. Empty when neither exists. A ledger or
/// review that cannot be read is an error naming its recovery.
pub fn recorded_claims(root: &Path, record: &ChangeRecord) -> Result<Vec<RecordedClaim>, String> {
    let mut claims = Vec::new();
    let Ok(workspace) = find_change_dir(root, &record.id) else {
        return Ok(claims);
    };
    if workspace.join("approvals.json").exists() {
        let ledger = load_approvals(root, record)?;
        if let Some(index) = ledger
            .approvals
            .iter()
            .rposition(|approval| approval.gate == "definition")
            && ensure_definition_approval_not_sent_back(record, &ledger, index).is_ok()
        {
            let approval = &ledger.approvals[index];
            claims.push(RecordedClaim {
                gate: "definition".into(),
                name: approval.actor.clone(),
                verdict: None,
                relayed_by: approval.relayed_by.clone(),
            });
        }
    }
    let review_path = workspace.join(SCOPED_REVIEW_FILE);
    if review_path.exists() {
        let content = fs::read_to_string(&review_path)
            .map_err(|error| format!("failed to read {}: {error}", review_path.display()))?;
        let review: ScopedReviewRecord = serde_json::from_str(&content).map_err(|error| {
            with_relayed_read_recovery(
                root,
                record,
                &review_path,
                format!("invalid independent scoped review evidence: {error}"),
            )
        })?;
        ensure_read_relays_name_their_deciders(
            [(review.reviewer.as_str(), review.relayed_by.as_ref())],
            "the review's reviewer",
        )
        .map_err(|message| with_relayed_read_recovery(root, record, &review_path, message))?;
        claims.push(RecordedClaim {
            gate: "review".into(),
            name: review.reviewer.clone(),
            verdict: Some(review.verdict.as_str().into()),
            relayed_by: review.relayed_by.clone(),
        });
    }
    Ok(claims)
}

/// One relayed decision `repair_relayed_decisions` put back or removed. `record` counts from 1
/// within `file`. A put-back names the copy it came `from`; a removal carries the damaged value
/// it `removed`, so nothing is dropped without being shown (CHANGE-13.g).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelayRepair {
    pub file: String,
    pub record: usize,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<serde_json::Value>,
}

/// What `repair_relayed_decisions` did to one change: empty `repairs` means nothing was damaged
/// and nothing was written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelayRepairReport {
    pub change: String,
    pub repairs: Vec<RelayRepair>,
}

/// One ledger that can hold relayed decisions, read as JSON so a damaged relay can be seen
/// without the typed reader refusing the whole file.
struct RelayLedgerFile {
    name: &'static str,
    path: PathBuf,
    /// The array holding the records, or `None` when the file is one record (`review.json`).
    records_key: Option<&'static str>,
    /// The field `decided_by` must equal: an approval's `actor`, a review's `reviewer`.
    decider_key: &'static str,
    current: serde_json::Value,
    committed: Option<serde_json::Value>,
}

/// Whether the reader refuses `record`'s relay: a shape it cannot read, or a `decided_by` that is
/// not the record's own actor or reviewer.
fn relay_is_damaged(record: &serde_json::Value, decider_key: &str) -> bool {
    let Some(relay) = record.get("relayed_by") else {
        return false;
    };
    match RelayedDecision::try_from(relay.clone()) {
        Ok(relay) => {
            record.get(decider_key).and_then(serde_json::Value::as_str)
                != Some(relay.decided_by.as_str())
        }
        Err(_) => true,
    }
}

/// `record` without its `relayed_by`, as SpecSync reads it: through the typed approval or review
/// record, so a key the reader ignores (for example one a hand edit left behind) is not part of
/// the record. A record the typed reader refuses stays as written, so it matches only itself.
fn without_relay(record: &serde_json::Value, decider_key: &str) -> serde_json::Value {
    let mut record = record.clone();
    if let Some(object) = record.as_object_mut() {
        object.remove("relayed_by");
    }
    let read = if decider_key == "actor" {
        serde_json::from_value::<ApprovalRecord>(record.clone())
            .ok()
            .and_then(|typed| serde_json::to_value(typed).ok())
    } else {
        serde_json::from_value::<ScopedReviewRecord>(record.clone())
            .ok()
            .and_then(|typed| serde_json::to_value(typed).ok())
    };
    read.unwrap_or(record)
}

/// The original relay `candidate` holds for `record`, when `candidate` is the same record (every
/// field other than `relayed_by` equal, as SpecSync reads it) and its relay reads cleanly.
/// `Some(None)` means the original had no relay; `None` means `candidate` is no copy of `record`.
fn original_relay_in(
    record: &serde_json::Value,
    candidate: Option<&serde_json::Value>,
    decider_key: &str,
) -> Option<Option<serde_json::Value>> {
    let candidate = candidate?;
    (without_relay(candidate, decider_key) == without_relay(record, decider_key)
        && !relay_is_damaged(candidate, decider_key))
    .then(|| candidate.get("relayed_by").cloned())
}

fn relay_ledger_len(value: &serde_json::Value, key: Option<&str>) -> usize {
    match key {
        Some(key) => value
            .get(key)
            .and_then(serde_json::Value::as_array)
            .map_or(0, Vec::len),
        None => 1,
    }
}

fn relay_ledger_record<'a>(
    value: &'a serde_json::Value,
    key: Option<&str>,
    index: usize,
) -> Option<&'a serde_json::Value> {
    match key {
        Some(key) => value.get(key)?.as_array()?.get(index),
        None => (index == 0).then_some(value),
    }
}

fn relay_ledger_record_mut<'a>(
    value: &'a mut serde_json::Value,
    key: Option<&str>,
    index: usize,
) -> Option<&'a mut serde_json::Map<String, serde_json::Value>> {
    match key {
        Some(key) => value
            .get_mut(key)?
            .as_array_mut()?
            .get_mut(index)?
            .as_object_mut(),
        None => value.as_object_mut(),
    }
}

/// Repair every damaged relayed decision of an active change (#904, CHANGE-13.e to CHANGE-13.g).
///
/// Each damaged `relayed_by` in `approvals.json`, `review-attempts.json` and `review.json` is
/// put back from the first copy of the same record that still reads cleanly: the record at the
/// same position in the file as committed at `HEAD`, then its twin (`review.json` and the last
/// attempt are copies of each other). With no copy, only that `relayed_by` is removed. No
/// approval or attempt is removed and no verdict changes, so no blocking review is erased
/// (REQ-change-046). No digest or gate reads a relay, so this stales nothing. Files are written
/// the way SpecSync writes them, all in one step under the project lock, and only when
/// something was repaired.
pub fn repair_relayed_decisions(root: &Path, id: &str) -> Result<RelayRepairReport, String> {
    let _lock = acquire_project_lock(root)?;
    let record = load_change(root, id)?;
    let workspace = find_change_dir(root, &record.id)?;
    if record.state == ChangeState::Archived {
        return Err(format!(
            "{} is archived, and finalize committed its package, so the last commit holds every \
             original; restore it with `git checkout HEAD -- {}`",
            record.id,
            portable_project_path(root, &workspace)
        ));
    }
    let mut files = Vec::new();
    for (name, records_key, decider_key) in [
        ("approvals.json", Some("approvals"), "actor"),
        (SCOPED_REVIEW_ATTEMPTS_FILE, Some("reviews"), "reviewer"),
        (SCOPED_REVIEW_FILE, None, "reviewer"),
    ] {
        let path = workspace.join(name);
        if !path.exists() {
            continue;
        }
        let shown = portable_project_path(root, &path);
        let committed_bytes = git_repo_relative_path(root, &shown)
            .and_then(|repository_path| git_file_at_commit(root, "HEAD", &repository_path))
            .ok()
            .flatten();
        let content =
            fs::read_to_string(&path).map_err(|error| format!("failed to read {shown}: {error}"))?;
        let current: serde_json::Value = serde_json::from_str(&content).map_err(|error| {
            let recovery = if committed_bytes.is_some() {
                format!("restore it with `git checkout HEAD -- {shown}`")
            } else {
                "it was never committed, so no copy of it exists to restore".to_string()
            };
            format!(
                "{shown} is not JSON ({error}); `specsync change repair-relays` repairs only \
                 relayed decisions, so {recovery}"
            )
        })?;
        let committed = committed_bytes.and_then(|bytes| serde_json::from_slice(&bytes).ok());
        files.push(RelayLedgerFile {
            name,
            path,
            records_key,
            decider_key,
            current,
            committed,
        });
    }

    // The twins, as they are before anything is repaired.
    let review_twin = files
        .iter()
        .find(|file| file.name == SCOPED_REVIEW_FILE)
        .map(|file| file.current.clone());
    let attempt_twin = files
        .iter()
        .find(|file| file.name == SCOPED_REVIEW_ATTEMPTS_FILE)
        .and_then(|file| {
            let attempts = file.current.get("reviews")?.as_array()?;
            Some((attempts.last()?.clone(), attempts.len()))
        });

    let mut repairs = Vec::new();
    let mut prepared = Vec::new();
    for file in &mut files {
        let count = relay_ledger_len(&file.current, file.records_key);
        let mut repaired = false;
        for index in 0..count {
            let Some(record_value) =
                relay_ledger_record(&file.current, file.records_key, index).cloned()
            else {
                continue;
            };
            if !relay_is_damaged(&record_value, file.decider_key) {
                continue;
            }
            let committed = file
                .committed
                .as_ref()
                .and_then(|committed| relay_ledger_record(committed, file.records_key, index));
            let twin = if file.name == SCOPED_REVIEW_FILE {
                attempt_twin.as_ref().map(|(attempt, position)| {
                    (
                        attempt,
                        format!("{SCOPED_REVIEW_ATTEMPTS_FILE} attempt {position}"),
                    )
                })
            } else if file.name == SCOPED_REVIEW_ATTEMPTS_FILE && index + 1 == count {
                review_twin
                    .as_ref()
                    .map(|review| (review, SCOPED_REVIEW_FILE.to_string()))
            } else {
                None
            };
            let original = original_relay_in(&record_value, committed, file.decider_key)
                .map(|relay| (relay, "HEAD".to_string()))
                .or_else(|| {
                    let (twin, name) = twin?;
                    original_relay_in(&record_value, Some(twin), file.decider_key)
                        .map(|relay| (relay, name))
                });
            let object = relay_ledger_record_mut(&mut file.current, file.records_key, index)
                .ok_or_else(|| format!("{} record {} is not an object", file.name, index + 1))?;
            let damaged = object.remove("relayed_by");
            let (action, from, removed) = match original {
                Some((relay, from)) => {
                    if let Some(relay) = relay {
                        object.insert("relayed_by".into(), relay);
                    }
                    ("put_back", Some(from), None)
                }
                None => ("removed", None, damaged),
            };
            repairs.push(RelayRepair {
                file: file.name.into(),
                record: index + 1,
                action: action.into(),
                from,
                removed,
            });
            repaired = true;
        }
        if !repaired {
            continue;
        }
        let name = file.name;
        let still = |error: serde_json::Error| {
            format!(
                "{name} still does not read after repairing its relayed decisions ({error}), so \
                 nothing was written; its damage is not only a relayed decision"
            )
        };
        let content = if name == "approvals.json" {
            json_content(
                &serde_json::from_value::<ApprovalLedger>(file.current.clone()).map_err(still)?,
            )?
        } else if name == SCOPED_REVIEW_ATTEMPTS_FILE {
            json_content(
                &serde_json::from_value::<ScopedReviewAttemptLedger>(file.current.clone())
                    .map_err(still)?,
            )?
        } else {
            json_content(
                &serde_json::from_value::<ScopedReviewRecord>(file.current.clone())
                    .map_err(still)?,
            )?
        };
        prepared.push((file.path.clone(), content));
    }
    if !prepared.is_empty() {
        write_prepared_files(root, &prepared)?;
    }
    Ok(RelayRepairReport {
        change: record.id,
        repairs,
    })
}
