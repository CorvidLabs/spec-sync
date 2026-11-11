// Members a newer SpecSync wrote are carried through every in-place rewrite of a lifecycle record
// or ledger by a SpecSync that does not know them (#1043, #434).
//
// A typed rewrite (`json_content`) serializes only the fields this binary knows, so a file written
// by a newer 6.x lost every field it added the moment an older 6.x updated it: one `change
// send-back` by 6.2.0 erased the relayed decisions and the `cancelled_review_attempts` a newer build
// had recorded, and the newer build then read the change's committed send-back history as edited.
//
// The rewrite now keeps what it cannot read. The file on disk, its typed round trip and the new
// typed value are compared as spans of their JSON text (`ledger_json_span`), never through
// `serde_json::Value`, whose objects are sorted in this build and whose numbers become floats:
//
// - an object member the typed round trip drops is one this binary does not know, and it is
//   appended, with its exact text, to the same object of the new value unless that object already
//   has the key;
// - the root is the same record or ledger before and after the rewrite;
// - an array entry is the same entry only when its known text is unchanged, so an entry the
//   rewrite changed, or a position that now holds another entry, never gains another record's
//   members.
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

enum JsonSpanKind {
    Object(Vec<(String, JsonSpanNode)>),
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
                    let key: String = serde_json::from_slice(ledger_json_span(bytes, offset)?).ok()?;
                    skip_json_whitespace(bytes, offset);
                    if bytes.get(*offset) != Some(&b':') {
                        return None;
                    }
                    *offset += 1;
                    members.push((key, parse_json_span_node(bytes, offset, level + 1)?));
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
fn json_span_member<'a>(members: &'a [(String, JsonSpanNode)], key: &str) -> Option<&'a JsonSpanNode> {
    members.iter().rev().find(|(name, _)| name == key).map(|(_, node)| node)
}

/// An unknown member to append to an object of the new document.
struct CarriedMember {
    /// The object's closing `}` offset in the new document.
    close: usize,
    /// The end offset of the object's last member in the new document, or `None` when it is empty.
    after_last: Option<usize>,
    level: usize,
    key: String,
    value: Vec<u8>,
    /// Where the member sat, for the refusal message (`approvals[0].relayed_by`).
    location: String,
}

/// Collects every member of `previous` that `known` (its typed round trip) dropped and `new` does
/// not have, at the same place in `new`. See the module comment for which places count as the same.
fn collect_unknown_members(
    previous: (&[u8], &JsonSpanNode),
    known: (&[u8], &JsonSpanNode),
    new: (&[u8], &JsonSpanNode),
    location: &str,
    carried: &mut Vec<CarriedMember>,
) {
    let ((previous_bytes, previous_node), (known_bytes, known_node), (new_bytes, new_node)) =
        (previous, known, new);
    match (&previous_node.kind, &known_node.kind, &new_node.kind) {
        (JsonSpanKind::Object(previous_members), JsonSpanKind::Object(known_members), JsonSpanKind::Object(new_members)) => {
            for (key, previous_value) in previous_members {
                let member_location = if location.is_empty() {
                    key.clone()
                } else {
                    format!("{location}.{key}")
                };
                match (json_span_member(known_members, key), json_span_member(new_members, key)) {
                    (Some(known_value), Some(new_value)) => collect_unknown_members(
                        (previous_bytes, previous_value),
                        (known_bytes, known_value),
                        (new_bytes, new_value),
                        &member_location,
                        carried,
                    ),
                    // Known, and the rewrite removed it: removed it stays.
                    (Some(_), None) => {}
                    // The new value sets it itself.
                    (None, Some(_)) => {}
                    (None, None) => carried.push(CarriedMember {
                        close: new_node.end - 1,
                        after_last: new_members.last().map(|(_, node)| node.end),
                        level: new_node.level,
                        key: key.clone(),
                        value: previous_bytes[previous_value.start..previous_value.end].to_vec(),
                        location: member_location,
                    }),
                }
            }
        }
        (JsonSpanKind::Array(previous_items), JsonSpanKind::Array(known_items), JsonSpanKind::Array(new_items))
            if previous_items.len() == known_items.len() =>
        {
            for (index, ((previous_item, known_item), new_item)) in
                previous_items.iter().zip(known_items).zip(new_items).enumerate()
            {
                if known_bytes[known_item.start..known_item.end]
                    == new_bytes[new_item.start..new_item.end]
                {
                    collect_unknown_members(
                        (previous_bytes, previous_item),
                        (known_bytes, known_item),
                        (new_bytes, new_item),
                        &format!("{location}[{index}]"),
                        carried,
                    );
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
            let key = serde_json::to_string(&member.key).map_err(|error| error.to_string())?;
            let value = std::str::from_utf8(&member.value).map_err(|error| error.to_string())?;
            if position > 0 {
                members.push(',');
            }
            members.push_str(&format!("\n{member_indent}{key}: {value}"));
        }
        match group[0].after_last {
            Some(after_last) => output.insert_str(after_last, &format!(",{}", &members[..])),
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
fn json_content_keeping_unknown_members<Record>(previous: &Path, value: &Record) -> Result<String, String>
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
    let mut carried = Vec::new();
    collect_unknown_members(
        (&previous_bytes, &previous_node),
        (known_text.as_bytes(), &known_node),
        (new_text.as_bytes(), &new_node),
        "",
        &mut carried,
    );
    if carried.is_empty() {
        return Ok(new_text);
    }
    let locations: Vec<String> = carried.iter().map(|member| member.location.clone()).collect();
    let refuse = |why: &str| {
        format!(
            "refusing to rewrite {}: it holds {} written by a newer SpecSync that this one does not know, and {why}; upgrade specsync",
            previous.display(),
            locations.join(", ")
        )
    };
    let output = splice_unknown_members(&new_text, carried).map_err(|error| refuse(&error))?;
    // The kept members must change nothing this binary reads.
    let reread: Record = serde_json::from_str(&output)
        .map_err(|error| refuse(&format!("keeping them would not parse ({error})")))?;
    if json_content(&reread)? != new_text {
        return Err(refuse("keeping them would change what this SpecSync reads"));
    }
    Ok(output)
}
