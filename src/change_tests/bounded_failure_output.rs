// REQ-change-153: the 4 KiB bound never splits a multi-byte character (#640 review).

#[test]
fn bounded_failure_output_respects_the_cap_at_a_char_boundary() {
    // 4099 bytes: "a" + 1366 × "↔" (3 bytes each) — the raw cap lands inside a `↔`.
    let findings = vec![format!("a{}", "↔".repeat(1366))];
    let output = bounded_failure_output(&findings).unwrap();
    assert!(output.len() <= 4096 + "…\n".len(), "output stays within the cap plus its marker: {}", output.len());
    assert!(output.starts_with("…\n"), "truncation is marked: {output:.40}");
    // The cut walks forward to the next character, so the tail is whole `↔`s: 4099 - 4 bytes.
    let body = output.strip_prefix("…\n").unwrap();
    assert!(body.starts_with('↔'), "the tail starts on a whole character: {body:.12}");
    assert_eq!(body.len(), 4095, "the walk moves forward to the next boundary, not back past the cap");
}

#[test]
fn bounded_failure_output_starts_on_a_line_boundary() {
    let mut lines: Vec<String> = (0..300)
        .map(|i| format!("finding number {i} with enough text to matter"))
        .collect();
    lines.push("short".into());
    let output = bounded_failure_output(&lines).unwrap();
    let body = output.strip_prefix("…\n").unwrap();
    assert!(
        body.starts_with("finding number") || body.starts_with("short"),
        "the tail starts at a line boundary, not mid-line: {body:.60}"
    );
}

#[test]
fn bounded_failure_output_under_the_cap_is_verbatim() {
    let findings = vec!["one".to_string(), "two".to_string()];
    assert_eq!(bounded_failure_output(&findings).as_deref(), Some("one\ntwo"));
    assert_eq!(bounded_failure_output(&[]), None);
}

#[test]
fn bounded_failure_output_keeps_the_tail_when_the_only_newline_is_far_or_trailing() {
    // The only newline sits far into the tail: snapping to it would keep 5 bytes.
    let far = vec!["y".repeat(5000), "short".to_string()];
    let output = bounded_failure_output(&far).unwrap();
    assert!(output.len() > 4000, "the tail is kept, not cut to its last line: {}", output.len());
    // A trailing newline leaves nothing after it, so the tail starts at the char boundary.
    let trailing = vec!["x".repeat(5000), String::new()];
    let output = bounded_failure_output(&trailing).unwrap();
    assert!(output.len() > 4000, "a trailing newline does not empty the tail: {}", output.len());
    // Nothing to record is None, not an empty record.
    assert_eq!(bounded_failure_output(&[String::new()]), None);
}

// REQ-change-153: a record written before `output` existed parses, and reads as having no
// output; passing records still serialize without the key (#640 review).
#[test]
fn command_evidence_written_before_the_output_field_parses_without_output() {
    let old: CommandEvidence =
        serde_json::from_str(r#"{"command":"specsync check --spec lib","success":false,"exit_code":1}"#)
            .expect("a record from an older binary parses");
    assert_eq!(old.output, None);
    assert!(!old.success);
    let passing = CommandEvidence {
        command: "specsync check --spec lib".into(),
        success: true,
        exit_code: Some(0),
        output: None,
    };
    assert_eq!(
        serde_json::to_string(&passing).unwrap(),
        r#"{"command":"specsync check --spec lib","success":true,"exit_code":0}"#,
        "a passing record serializes exactly as before the field existed"
    );
}
