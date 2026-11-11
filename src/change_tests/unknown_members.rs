// Verifies the rewrite acceptance line of REQ-change-079 (#1043, #434): a lifecycle record or
// ledger rewritten in place keeps the members a newer SpecSync wrote that this one does not know.
//
// `OlderLedger` is what an older SpecSync knows; `NewerLedger` is what a newer one wrote. The older
// one appends an entry, exactly as `change send-back` and `change approve` append to
// `approvals.json`.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct OlderEntry {
    actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct OlderLedger {
    schema_version: u32,
    entries: Vec<OlderEntry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct NewerRelay {
    decided_by: String,
    words: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct NewerEntry {
    actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    relayed_by: Option<NewerRelay>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancelled_review_attempts: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct NewerLedger {
    schema_version: u32,
    entries: Vec<NewerEntry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    labels: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    archive_hint: Option<String>,
}

fn newer_ledger() -> NewerLedger {
    NewerLedger {
        schema_version: 1,
        entries: vec![NewerEntry {
            actor: "Ada".into(),
            note: None,
            relayed_by: Some(NewerRelay {
                decided_by: "Ada".into(),
                words: "Approve it".into(),
            }),
            cancelled_review_attempts: Some(0),
        }],
        labels: BTreeMap::new(),
        archive_hint: Some("keep".into()),
    }
}

fn older_entry(actor: &str) -> OlderEntry {
    OlderEntry {
        actor: actor.into(),
        note: None,
    }
}

#[test]
fn an_older_specsync_appending_to_a_ledger_keeps_what_a_newer_one_wrote() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("approvals.json");
    fs::write(&path, json_content(&newer_ledger()).unwrap()).unwrap();

    // The older SpecSync reads what it knows and appends an entry.
    let mut older: OlderLedger = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    older.entries.push(older_entry("Grace"));
    let written = json_content_keeping_unknown_members(&path, &older).unwrap();

    let reread: NewerLedger = serde_json::from_str(&written).unwrap();
    let mut expected = newer_ledger();
    expected.entries.push(NewerEntry {
        actor: "Grace".into(),
        note: None,
        relayed_by: None,
        cancelled_review_attempts: None,
    });
    assert_eq!(reread, expected, "{written}");

    // Without the fix, the typed rewrite loses all of it: the #1043 failure.
    let dropped: NewerLedger = serde_json::from_str(&json_content(&older).unwrap()).unwrap();
    assert_eq!(dropped.entries[0].relayed_by, None);
    assert_eq!(dropped.archive_hint, None);
}

#[test]
fn an_entry_the_rewrite_changed_or_replaced_never_gains_another_records_members() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("approvals.json");
    let mut newer = newer_ledger();
    newer.entries.push(NewerEntry {
        actor: "Grace".into(),
        note: None,
        relayed_by: None,
        cancelled_review_attempts: Some(3),
    });
    fs::write(&path, json_content(&newer).unwrap()).unwrap();

    // The first entry is edited: its own unknown members are not carried onto the edited entry.
    let mut older: OlderLedger = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    older.entries[0].note = Some("edited".into());
    let reread: NewerLedger =
        serde_json::from_str(&json_content_keeping_unknown_members(&path, &older).unwrap()).unwrap();
    assert_eq!(reread.entries[0].relayed_by, None);
    assert_eq!(reread.entries[0].cancelled_review_attempts, None);
    assert_eq!(reread.entries[1].cancelled_review_attempts, Some(3), "the unchanged entry keeps its own");
    assert_eq!(reread.archive_hint.as_deref(), Some("keep"), "the root is the same ledger");

    // The first entry is removed: the entry now at position 0 is Grace, and it never takes Ada's
    // relay.
    let mut older: OlderLedger = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    older.entries.remove(0);
    let reread: NewerLedger =
        serde_json::from_str(&json_content_keeping_unknown_members(&path, &older).unwrap()).unwrap();
    assert_eq!(reread.entries.len(), 1);
    assert_eq!(reread.entries[0].actor, "Grace");
    assert_eq!(reread.entries[0].relayed_by, None);
}

#[test]
fn kept_members_keep_their_exact_text_and_known_members_their_order() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("state.json");
    // Written by hand as a newer SpecSync would: an integer past u64, an exact decimal, an escape,
    // a nested object, and a member inside an object that this SpecSync knows as a map.
    let previous = "{\n  \"schema_version\": 1,\n  \"future_count\": 123456789012345678901234567890,\n  \"entries\": [\n    {\n      \"actor\": \"Ada\",\n      \"future_ratio\": 1.50\n    }\n  ],\n  \"future_text\": \"caf\\u00e9\",\n  \"future_nested\": {\n    \"depth\": [\n      1,\n      {\n        \"x\": null\n      }\n    ]\n  }\n}\n";
    fs::write(&path, previous).unwrap();
    let mut older: OlderLedger = serde_json::from_str(previous).unwrap();
    older.entries.push(older_entry("Grace"));
    let written = json_content_keeping_unknown_members(&path, &older).unwrap();
    for exact in [
        "\"future_count\": 123456789012345678901234567890",
        "\"future_ratio\": 1.50",
        "\"future_text\": \"caf\\u00e9\"",
        "\"future_nested\": {\n    \"depth\": [\n      1,\n      {\n        \"x\": null\n      }\n    ]\n  }",
    ] {
        assert!(written.contains(exact), "{exact} kept verbatim:\n{written}");
    }
    // Known members come first, in the order this SpecSync writes them.
    let known = json_content(&older).unwrap();
    let schema = written.find("\"schema_version\"").unwrap();
    let entries = written.find("\"entries\"").unwrap();
    let carried = written.find("\"future_count\"").unwrap();
    assert!(schema < entries && entries < carried, "{written}");
    let reread: OlderLedger = serde_json::from_str(&written).unwrap();
    assert_eq!(json_content(&reread).unwrap(), known, "nothing this SpecSync reads changed");
    let value: serde_json::Value = serde_json::from_str(&written).unwrap();
    assert_eq!(value["entries"][0]["future_ratio"], serde_json::json!(1.5));
}

#[test]
fn a_rewrite_with_nothing_unknown_writes_exactly_what_it_wrote_before() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("approvals.json");
    let mut ledger = OlderLedger {
        schema_version: 1,
        entries: vec![older_entry("Ada")],
        labels: BTreeMap::new(),
    };
    // No previous file.
    assert_eq!(
        json_content_keeping_unknown_members(&path, &ledger).unwrap(),
        json_content(&ledger).unwrap()
    );
    fs::write(&path, json_content(&ledger).unwrap()).unwrap();
    ledger.entries.push(older_entry("Grace"));
    assert_eq!(
        json_content_keeping_unknown_members(&path, &ledger).unwrap(),
        json_content(&ledger).unwrap()
    );
    // A previous file that is not this record's shape is the reader's problem, not the writer's.
    fs::write(&path, "{\"schema_version\": \"one\"}\n").unwrap();
    assert_eq!(
        json_content_keeping_unknown_members(&path, &ledger).unwrap(),
        json_content(&ledger).unwrap()
    );
}

#[test]
fn a_known_field_the_rewrite_sets_or_removes_is_never_overridden_by_the_old_text() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("approvals.json");
    // An explicit default the type skips on write (`"note": null`) and a map entry.
    fs::write(
        &path,
        "{\n  \"schema_version\": 1,\n  \"entries\": [\n    {\n      \"actor\": \"Ada\",\n      \"note\": null\n    }\n  ],\n  \"labels\": {\n    \"gone\": \"x\"\n  },\n  \"future\": 1\n}\n",
    )
    .unwrap();
    let mut older: OlderLedger = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    older.entries[0].note = Some("set".into());
    older.labels.clear();
    let written = json_content_keeping_unknown_members(&path, &older).unwrap();
    let reread: OlderLedger = serde_json::from_str(&written).unwrap();
    assert_eq!(reread.entries[0].note.as_deref(), Some("set"), "{written}");
    assert!(reread.labels.is_empty(), "a removed map entry stays removed: {written}");
    assert!(written.contains("\"future\": 1"), "{written}");
    assert_eq!(written.matches("\"note\"").count(), 1, "{written}");
}

#[test]
fn an_empty_object_in_the_new_text_still_takes_its_unknown_members() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Holder {
        inner: Inner,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Inner {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        known: Option<String>,
    }
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("state.json");
    fs::write(&path, "{\n  \"inner\": {\n    \"future\": true\n  }\n}\n").unwrap();
    let holder: Holder = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let written = json_content_keeping_unknown_members(&path, &holder).unwrap();
    assert_eq!(written, "{\n  \"inner\": {\n    \"future\": true\n  }\n}\n");
}
