// Members a newer SpecSync wrote are carried through every in-place rewrite of a lifecycle record
// or ledger by a SpecSync that does not know them (#1043, #434).
//
// A typed rewrite (`json_content`) serializes only the fields this binary knows, so a file written
// by a newer 6.x lost every field it added the moment an older 6.x updated it: one `change
// send-back` by 6.2.0 erased the relayed decisions and the `cancelled_review_attempts` a newer build
// had recorded, and the newer build then read the change's committed send-back history as edited.
//
// The rewrite now keeps what it cannot read, or refuses. The file on disk, its typed round trip and
// the new typed value are compared as spans of their JSON text (`ledger_json_span`), never through
// `serde_json::Value`, whose objects are sorted in this build and whose numbers become floats:
//
// - an object member the typed round trip drops is one this binary does not know, and it is
//   appended, with its exact text (key and value), to the same object of the new value unless that
//   object already has the key;
// - the root is the same record or ledger before and after the rewrite;
// - an array entry is matched to the new entry with the same known text, wherever it moved (an
//   insertion, a removal or a sort shifts entries); an entry left unmatched in an array whose length
//   did not change is the same record edited in place, as a relay repair edits one approval;
// - an entry that holds unknown members and matches nothing (it was removed, or replaced in a way
//   that cannot be told apart) makes the rewrite refuse rather than drop them.
//
// The result must parse back to exactly the new value; otherwise the write is refused rather than
// losing anything silently. A file this binary writes afresh (a new `verification.json`, a new
// review) is not a rewrite and keeps using `json_content`.

/// One JSON value of a document, as byte offsets into that document.
struct JsonSpanNode {
    start: usize,
    end: usize,
    /// Nesting level: the root is 0, its members 1, and so on.
    level: usize,
    kind: JsonSpanKind,
}

/// One object member: its decoded key, the byte range of its key token, and its value.
struct JsonSpanMember {
    key: String,
    key_span: std::ops::Range<usize>,
    value: JsonSpanNode,
}

enum JsonSpanKind {
    Object(Vec<JsonSpanMember>),
    Array(Vec<JsonSpanNode>),
    Leaf,
}

/// The nesting serde_json accepts by default; deeper documents are not lifecycle records.
const MAX_JSON_SPAN_DEPTH: usize = 128;

fn skip_json_whitespace(bytes: &[u8], offset: &mut usize) {
    while bytes.get(*offset).is_some_and(u8::is_ascii_whitespace) {
        *offset += 1;
    }
}

/// Parses one value of `bytes` at `offset` into spans, or `None` when it is not valid JSON.
fn parse_json_span_node(bytes: &[u8], offset: &mut usize, level: usize) -> Option<JsonSpanNode> {
    skip_json_whitespace(bytes, offset);
    let start = *offset;
    match bytes.get(start)? {
        b'{' | b'[' if level >= MAX_JSON_SPAN_DEPTH => None,
        b'{' => {
            *offset += 1;
            let mut members = Vec::new();
            skip_json_whitespace(bytes, offset);
            if bytes.get(*offset) == Some(&b'}') {
                *offset += 1;
            } else {
                loop {
                    skip_json_whitespace(bytes, offset);
                    let key_start = *offset;
                    let key: String = serde_json::from_slice(ledger_json_span(bytes, offset)?).ok()?;
                    let key_span = key_start..*offset;
                    skip_json_whitespace(bytes, offset);
                    if bytes.get(*offset) != Some(&b':') {
                        return None;
                    }
                    *offset += 1;
                    let value = parse_json_span_node(bytes, offset, level + 1)?;
                    members.push(JsonSpanMember {
                        key,
                        key_span,
                        value,
                    });
                    skip_json_whitespace(bytes, offset);
                    match bytes.get(*offset)? {
                        b',' => *offset += 1,
                        b'}' => {
                            *offset += 1;
                            break;
                        }
                        _ => return None,
                    }
                }
            }
            Some(JsonSpanNode {
                start,
                end: *offset,
                level,
                kind: JsonSpanKind::Object(members),
            })
        }
        b'[' => {
            *offset += 1;
            let mut items = Vec::new();
            skip_json_whitespace(bytes, offset);
            if bytes.get(*offset) == Some(&b']') {
                *offset += 1;
            } else {
                loop {
                    items.push(parse_json_span_node(bytes, offset, level + 1)?);
                    skip_json_whitespace(bytes, offset);
                    match bytes.get(*offset)? {
                        b',' => *offset += 1,
                        b']' => {
                            *offset += 1;
                            break;
                        }
                        _ => return None,
                    }
                }
            }
            Some(JsonSpanNode {
                start,
                end: *offset,
                level,
                kind: JsonSpanKind::Array(items),
            })
        }
        _ => {
            ledger_json_span(bytes, offset)?;
            Some(JsonSpanNode {
                start,
                end: *offset,
                level,
                kind: JsonSpanKind::Leaf,
            })
        }
    }
}

/// Parses a whole document into spans: one value, then only whitespace.
fn parse_json_span_document(bytes: &[u8]) -> Option<JsonSpanNode> {
    let mut offset = 0;
    let node = parse_json_span_node(bytes, &mut offset, 0)?;
    skip_json_whitespace(bytes, &mut offset);
    (offset == bytes.len()).then_some(node)
}

/// The last member called `key`, as serde reads a duplicate key.
fn json_span_member<'a>(members: &'a [JsonSpanMember], key: &str) -> Option<&'a JsonSpanNode> {
    members
        .iter()
        .rev()
        .find(|member| member.key == key)
        .map(|member| &member.value)
}

/// An unknown member to append to an object of the new document.
struct CarriedMember {
    /// The object's closing `}` offset in the new document.
    close: usize,
    /// The end offset of the object's last member in the new document, or `None` when it is empty.
    after_last: Option<usize>,
    level: usize,
    /// The member's key token and value, exactly as the previous document wrote them.
    key: Vec<u8>,
    value: Vec<u8>,
    /// Where the member sat, for the refusal message (`approvals[0].relayed_by`).
    location: String,
}

/// What a rewrite would carry over, and the unknown members it could not place.
#[derive(Default)]
struct UnknownMemberPlan {
    carried: Vec<CarriedMember>,
    lost: Vec<String>,
}

/// Whether `previous` holds any member its typed round trip `known` dropped, at any depth.
fn holds_unknown_members(previous: (&[u8], &JsonSpanNode), known: (&[u8], &JsonSpanNode)) -> bool {
    let mut plan = UnknownMemberPlan::default();
    collect_unknown_members(previous, known, known, "", &mut plan);
    !plan.carried.is_empty() || !plan.lost.is_empty()
}

/// Collects every member of `previous` that `known` (its typed round trip) dropped and `new` does
/// not have, at the same place in `new`, and every one it cannot place. See the module comment for
/// which places count as the same.
fn collect_unknown_members(
    previous: (&[u8], &JsonSpanNode),
    known: (&[u8], &JsonSpanNode),
    new: (&[u8], &JsonSpanNode),
    location: &str,
    plan: &mut UnknownMemberPlan,
) {
    let ((previous_bytes, previous_node), (known_bytes, known_node), (new_bytes, new_node)) =
        (previous, known, new);
    match (&previous_node.kind, &known_node.kind, &new_node.kind) {
        (
            JsonSpanKind::Object(previous_members),
            JsonSpanKind::Object(known_members),
            JsonSpanKind::Object(new_members),
        ) => {
            for member in previous_members {
                let member_location = if location.is_empty() {
                    member.key.clone()
                } else {
                    format!("{location}.{}", member.key)
                };
                match (
                    json_span_member(known_members, &member.key),
                    json_span_member(new_members, &member.key),
                ) {
                    (Some(known_value), Some(new_value)) => collect_unknown_members(
                        (previous_bytes, &member.value),
                        (known_bytes, known_value),
                        (new_bytes, new_value),
                        &member_location,
                        plan,
                    ),
                    // Known, and the rewrite removed it: removed it stays.
                    (Some(_), None) => {}
                    // The new value sets it itself.
                    (None, Some(_)) => {}
                    (None, None) => plan.carried.push(CarriedMember {
                        close: new_node.end - 1,
                        after_last: new_members.last().map(|last| last.value.end),
                        level: new_node.level,
                        key: previous_bytes[member.key_span.clone()].to_vec(),
                        value: previous_bytes[member.value.start..member.value.end].to_vec(),
                        location: member_location,
                    }),
                }
            }
        }
        (
            JsonSpanKind::Array(previous_items),
            JsonSpanKind::Array(known_items),
            JsonSpanKind::Array(new_items),
        ) if previous_items.len() == known_items.len() => {
            let text = |bytes: &[u8], node: &JsonSpanNode| bytes[node.start..node.end].to_vec();
            let new_texts: Vec<Vec<u8>> = new_items.iter().map(|item| text(new_bytes, item)).collect();
            let mut taken = vec![false; new_items.len()];
            let mut matches: Vec<Option<usize>> = vec![None; known_items.len()];
            // The same known text, preferring the same position, then the first free one.
            for (index, known_item) in known_items.iter().enumerate() {
                let known_text = text(known_bytes, known_item);
                let same = (index < new_items.len() && !taken[index] && new_texts[index] == known_text)
                    .then_some(index)
                    .or_else(|| {
                        (0..new_items.len()).find(|&other| !taken[other] && new_texts[other] == known_text)
                    });
                if let Some(other) = same {
                    taken[other] = true;
                    matches[index] = Some(other);
                }
            }
            // An entry edited in place: its position still holds an unmatched entry, and the array
            // neither grew nor shrank.
            if previous_items.len() == new_items.len() {
                for (index, matched) in matches.iter_mut().enumerate() {
                    if matched.is_none() && !taken[index] {
                        taken[index] = true;
                        *matched = Some(index);
                    }
                }
            }
            for (index, ((previous_item, known_item), matched)) in
                previous_items.iter().zip(known_items).zip(&matches).enumerate()
            {
                let entry_location = format!("{location}[{index}]");
                match matched {
                    Some(other) => collect_unknown_members(
                        (previous_bytes, previous_item),
                        (known_bytes, known_item),
                        (new_bytes, &new_items[*other]),
                        &entry_location,
                        plan,
                    ),
                    None => {
                        if holds_unknown_members(
                            (previous_bytes, previous_item),
                            (known_bytes, known_item),
                        ) {
                            plan.lost.push(entry_location);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// Appends `carried` to the new document's text: each after its object's last member, in the
/// layout `serde_json::to_string_pretty` uses, or into an empty `{}`.
fn splice_unknown_members(new_text: &str, mut carried: Vec<CarriedMember>) -> Result<String, String> {
    // Splice from the end of the text so earlier offsets stay valid: an object's insertion point is
    // after every object it contains and before every object that follows it.
    carried.sort_by_key(|member| std::cmp::Reverse(member.close));
    let mut output = new_text.to_string();
    let mut index = 0;
    while index < carried.len() {
        let close = carried[index].close;
        let group_end = carried[index..]
            .iter()
            .position(|member| member.close != close)
            .map_or(carried.len(), |offset| index + offset);
        // `sort_by_key` is stable, so one object's members keep the order they had on disk.
        let group: Vec<&CarriedMember> = carried[index..group_end].iter().collect();
        let level = group[0].level;
        let member_indent = "  ".repeat(level + 1);
        let mut members = String::new();
        for (position, member) in group.iter().enumerate() {
            let key = std::str::from_utf8(&member.key).map_err(|error| error.to_string())?;
            let value = std::str::from_utf8(&member.value).map_err(|error| error.to_string())?;
            if position > 0 {
                members.push(',');
            }
            members.push_str(&format!("\n{member_indent}{key}: {value}"));
        }
        match group[0].after_last {
            Some(after_last) => output.insert_str(after_last, &format!(",{members}")),
            None => output.insert_str(close, &format!("{members}\n{}", "  ".repeat(level))),
        }
        index = group_end;
    }
    Ok(output)
}

/// `json_content` for a lifecycle record or ledger that replaces the file at `previous` in place:
/// members of that file this SpecSync does not know are kept (#1043, #434). A missing or
/// unreadable previous file has nothing to keep, so the value is written as `json_content` writes
/// it; anything this binary cannot carry over refuses the write instead of losing it.
fn json_content_keeping_unknown_members<Record>(
    previous: &Path,
    value: &Record,
) -> Result<String, String>
where
    Record: Serialize + serde::de::DeserializeOwned,
{
    let new_text = json_content(value)?;
    let Ok(previous_bytes) = fs::read(previous) else {
        return Ok(new_text);
    };
    let Ok(previous_record) = serde_json::from_slice::<Record>(&previous_bytes) else {
        // Not this record's shape: the reader that loaded it reports the damage.
        return Ok(new_text);
    };
    let known_text = json_content(&previous_record)?;
    let (Some(previous_node), Some(known_node), Some(new_node)) = (
        parse_json_span_document(&previous_bytes),
        parse_json_span_document(known_text.as_bytes()),
        parse_json_span_document(new_text.as_bytes()),
    ) else {
        return Ok(new_text);
    };
    let mut plan = UnknownMemberPlan::default();
    collect_unknown_members(
        (&previous_bytes, &previous_node),
        (known_text.as_bytes(), &known_node),
        (new_text.as_bytes(), &new_node),
        "",
        &mut plan,
    );
    let refuse = |what: &str, why: &str| {
        format!(
            "refusing to rewrite {}: {what} written by a newer SpecSync that this one does not know, and {why}; upgrade specsync",
            previous.display(),
        )
    };
    if !plan.lost.is_empty() {
        return Err(refuse(
            &format!("{} hold members", plan.lost.join(", ")),
            "this rewrite removes or replaces those entries, which would drop them",
        ));
    }
    if plan.carried.is_empty() {
        return Ok(new_text);
    }
    let locations: Vec<String> = plan
        .carried
        .iter()
        .map(|member| member.location.clone())
        .collect();
    let what = format!("it holds {}", locations.join(", "));
    let output =
        splice_unknown_members(&new_text, plan.carried).map_err(|error| refuse(&what, &error))?;
    // The kept members must change nothing this binary reads.
    let reread: Record = serde_json::from_str(&output)
        .map_err(|error| refuse(&what, &format!("keeping them would not parse ({error})")))?;
    if json_content(&reread)? != new_text {
        return Err(refuse(&what, "keeping them would change what this SpecSync reads"));
    }
    Ok(output)
}
