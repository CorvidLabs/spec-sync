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

/// A read refusal caused by a malformed relay record names restoring the file from Git.
fn with_relayed_read_recovery(root: &Path, path: &Path, message: String) -> String {
    if !message.contains("relayed_by") {
        return message;
    }
    let shown = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    format!(
        "{message}. A relayed decision is written only by `specsync change approve` or \
         `specsync change review`; restore the recorded file with \
         `git checkout HEAD -- {shown}` (or `git checkout <commit> -- {shown}` from the commit that \
         recorded it)"
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
                &review_path,
                format!("invalid independent scoped review evidence: {error}"),
            )
        })?;
        claims.push(RecordedClaim {
            gate: "review".into(),
            name: review.reviewer.clone(),
            verdict: Some(review.verdict.as_str().into()),
            relayed_by: review.relayed_by.clone(),
        });
    }
    Ok(claims)
}
