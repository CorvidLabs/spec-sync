// REQ-change-151: the 4 KiB bound never splits a multi-byte character (#640 review).

#[test]
fn bounded_failure_output_respects_the_cap_at_a_char_boundary() {
    // 4099 bytes: "a" + 1366 × "↔" (3 bytes each) — the raw cap lands inside a `↔`.
    let findings = vec![format!("a{}", "↔".repeat(1366))];
    let output = bounded_failure_output(&findings).unwrap();
    assert!(output.len() <= 4096 + 3, "output stays near the cap: {}", output.len());
    assert!(output.starts_with("…\n"), "truncation is marked: {output:.40}");
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
