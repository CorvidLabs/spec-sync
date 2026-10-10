use super::*;
use crate::mcp::tests::{setup_project, setup_project_with_spec};

#[test]
fn test_handle_resources_list_returns_resources() {
    let resp = handle_resources_list(Some(json!(1)));
    assert_eq!(resp["jsonrpc"], "2.0");
    let resources = resp["result"]["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 4);
    let uris: Vec<&str> = resources
        .iter()
        .map(|r| r["uri"].as_str().unwrap())
        .collect();
    assert!(uris.contains(&"specsync:///specs"));
    assert!(uris.contains(&"specsync:///graph"));
    assert!(uris.contains(&"specsync:///config"));
    assert!(uris.contains(&"specsync:///coverage"));
}

#[test]
fn test_handle_resources_list_has_templates() {
    let resp = handle_resources_list(Some(json!(1)));
    let templates = resp["result"]["resourceTemplates"].as_array().unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(
        templates[0]["uriTemplate"].as_str().unwrap(),
        "specsync:///specs/{module}"
    );
}

#[test]
fn test_resource_specs_list_empty() {
    let tmp = setup_project();
    let params = json!({ "uri": "specsync:///specs" });
    let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
    assert_eq!(resp["jsonrpc"], "2.0");
    let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    assert_eq!(parsed["count"], 0);
}

#[test]
fn test_resource_specs_list_with_spec() {
    let spec_content = "---\nmodule: auth\nversion: 2.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
    let tmp = setup_project_with_spec("auth", spec_content);

    let params = json!({ "uri": "specsync:///specs" });
    let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
    let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    assert_eq!(parsed["count"], 1);
    assert_eq!(parsed["specs"][0]["module"], "auth");
    assert!(parsed["specs"][0]["score"].is_number());
    assert_eq!(parsed["git_freshness_available"], false);
    assert_eq!(parsed["specs"][0]["git_freshness_available"], false);
}

#[test]
fn test_resource_spec_by_module() {
    let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n";
    let tmp = setup_project_with_spec("auth", spec_content);

    let params = json!({ "uri": "specsync:///specs/auth" });
    let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
    let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
    assert!(text.contains("module: auth"));
    assert_eq!(
        resp["result"]["contents"][0]["mimeType"].as_str().unwrap(),
        "text/markdown"
    );
}

#[test]
fn test_resource_spec_by_module_not_found() {
    let tmp = setup_project();
    let params = json!({ "uri": "specsync:///specs/nonexistent" });
    let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
    assert!(
        resp["error"]["message"]
            .as_str()
            .unwrap()
            .contains("No spec found")
    );
}

#[test]
fn test_resource_graph_empty() {
    let tmp = setup_project();
    let params = json!({ "uri": "specsync:///graph" });
    let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
    let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    assert_eq!(parsed["module_count"], 0);
    assert_eq!(parsed["edge_count"], 0);
}

#[test]
fn test_resource_config() {
    let tmp = setup_project();
    let params = json!({ "uri": "specsync:///config" });
    let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
    let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    assert_eq!(parsed["specs_dir"], "specs");
}

#[test]
fn test_resource_coverage_empty() {
    let tmp = setup_project();
    let params = json!({ "uri": "specsync:///coverage" });
    let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
    let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    // This test used to assert the bug: an empty project has no source
    // file, so there is no percentage to report. `100.0` told an agent the
    // project was fully specced when nothing had been looked at (#582).
    assert!(parsed["file_coverage_percent"].is_null(), "{parsed}");
    assert!(parsed["loc_coverage_percent"].is_null(), "{parsed}");
    assert_eq!(parsed["files_total"], 0, "{parsed}");
}

#[test]
fn test_resource_unknown_uri() {
    let tmp = setup_project();
    let params = json!({ "uri": "specsync:///bogus" });
    let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
    assert!(
        resp["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Unknown resource URI")
    );
}

#[test]
fn test_resources_read_rejects_non_exact_arguments() {
    let tmp = setup_project();
    for params in [
        json!([]),
        json!({}),
        json!({"uri": 7}),
        json!({"uri": "specsync:///config", "extra": true}),
    ] {
        let response = handle_resources_read(Some(json!(1)), &params, tmp.path());
        assert_eq!(response["error"]["code"], -32602);
    }
}
