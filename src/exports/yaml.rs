use regex::Regex;
use std::sync::LazyLock;

/// Top-level key: a line starting at column 0 with `key:` followed by space, newline, or EOF.
///
/// The key name may optionally be wrapped in matching double or single quotes
/// (`"name": "CI"` / `'name': "CI"`), which is valid YAML and common in
/// GitHub Actions workflows specifically for the `on:` key — unquoted `on` is
/// parsed as the boolean `true` under YAML 1.1, so `"on":` is the
/// recommended, and frequently seen, spelling. Without this, such keys were
/// silently missing from the top-level symbol list. Only identifier-shaped
/// quoted contents are matched (not arbitrary quoted strings containing
/// spaces/punctuation) to keep this from over-matching prose-like quoted
/// keys such as `'Keys can be quoted too.':`.
static TOP_LEVEL_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?m)^(?:"([a-zA-Z_][a-zA-Z0-9_-]*)"|'([a-zA-Z_][a-zA-Z0-9_-]*)'|([a-zA-Z_][a-zA-Z0-9_-]*)):(?:\s|$)"#,
    )
    .unwrap()
});

/// YAML anchor: `&anchor-name`
///
/// Requires the `&` to be preceded by start-of-line or a YAML structural
/// delimiter (whitespace, `,`, `[`, `{`, `(`). This avoids false positives on
/// `&` that appears mid-word inside unquoted scalars (e.g. a URL query string
/// like `...?key=abc&format=json`), since a real YAML anchor is always its
/// own token, never glued to the preceding character.
static ANCHOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|[\s,\[{(])&([a-zA-Z_][a-zA-Z0-9_.-]*)").unwrap());

/// Well-known YAML keys whose children represent named entries worth extracting.
/// e.g., `jobs:` in GitHub Actions, `services:` in Docker Compose.
const NESTED_SYMBOL_PARENTS: &[&str] = &[
    "jobs",
    "services",
    "volumes",
    "networks",
    "secrets",
    "stages",
    "steps",
    "targets",
    "outputs",
    "inputs",
    "permissions",
    "deployments",
];

/// Extract symbols from YAML source content.
///
/// Symbols extracted:
/// - All top-level keys
/// - Named entries under well-known parent keys (e.g., `jobs.test`, `services.web`)
/// - YAML anchors (`&anchor-name`)
pub fn extract_exports(content: &str) -> Vec<String> {
    let mut symbols = Vec::new();

    // Collect top-level keys (deduplicated, since multi-document YAML files
    // separated by `---` repeat the same top-level key names once per document).
    let top_level_keys: Vec<String> = TOP_LEVEL_KEY
        .captures_iter(content)
        .filter_map(|caps| {
            caps.get(1)
                .or_else(|| caps.get(2))
                .or_else(|| caps.get(3))
                .map(|m| m.as_str().to_string())
        })
        .collect();

    for key in &top_level_keys {
        if !symbols.contains(key) {
            symbols.push(key.clone());
        }
    }

    // For well-known parent keys, extract second-level children as `parent.child`.
    // Well-known parents are matched at *any* indentation depth (not just column 0),
    // so e.g. a `outputs:` block nested under `jobs.build:` in a GitHub Actions
    // workflow is picked up the same as a top-level one. For each match we scan
    // subsequent lines, detecting the indent of the first child to only match
    // direct children (not deeper nested keys), and stop once we reach a line
    // at or above the parent's own indent (i.e. a sibling or ancestor key).
    let key_line = Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_-]*):").unwrap();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed_line = line.trim_start();
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            i += 1;
            continue;
        }
        let parent_indent = line.len() - trimmed_line.len();
        // Check if this line is a well-known parent key (at any indentation).
        if let Some(caps) = key_line.captures(trimmed_line)
            && let Some(key_match) = caps.get(1)
        {
            let parent = key_match.as_str();
            if NESTED_SYMBOL_PARENTS.contains(&parent) {
                // Scan subsequent lines for second-level keys under this parent
                let mut j = i + 1;
                let mut child_indent: Option<usize> = None;
                while j < lines.len() {
                    let child_line = lines[j];
                    let trimmed = child_line.trim_start();
                    if trimmed.is_empty() {
                        j += 1;
                        continue;
                    }
                    let indent = child_line.len() - trimmed.len();
                    // Stop once we hit a line at or above the parent's own indent
                    // (a sibling or ancestor key), unless it's a comment.
                    if indent <= parent_indent && !trimmed.starts_with('#') {
                        break;
                    }
                    if trimmed.starts_with('#') {
                        j += 1;
                        continue;
                    }
                    // Detect indent of first child
                    if child_indent.is_none() {
                        child_indent = Some(indent);
                    }
                    // Only match lines at the exact child indent level
                    if Some(indent) == child_indent
                        && let Some(child_caps) = key_line.captures(trimmed)
                        && let Some(child_match) = child_caps.get(1)
                    {
                        let symbol = format!("{}.{}", parent, child_match.as_str());
                        if !symbols.contains(&symbol) {
                            symbols.push(symbol);
                        }
                    }
                    j += 1;
                }
            }
        }
        i += 1;
    }

    // Extract anchors
    for caps in ANCHOR.captures_iter(content) {
        if let Some(anchor) = caps.get(1) {
            let name = anchor.as_str().to_string();
            if !symbols.contains(&name) {
                symbols.push(name);
            }
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_learnxinyminutes_scalar_types() {
        // Excerpt straight from learnxinyminutes.com's yaml.md "SCALAR TYPES"
        // section: real idiomatic YAML demonstrating plain scalars, the
        // yes/no boolean gotcha, quoted keys/values, and literal/folded
        // block scalars with strip/keep chomping indicators.
        let content = r#"key: value
another_key: Another value goes here.
a_number_value: 100
scientific_notation: 1e+12
hex_notation: 0x123  # evaluates to 291
octal_notation: 0123 # evaluates to 83

boolean: true
null_value: null
another_null_value: ~
key with spaces: value

no: no            # evaluates to "no": false
yes: No           # evaluates to "yes": false
not_enclosed: yes # evaluates to "not_enclosed": true
enclosed: "yes"   # evaluates to "enclosed": yes

however: 'A string, enclosed in quotes.'
'Keys can be quoted too.': "Useful if you want to put a ':' in your key."
single quotes: 'have ''one'' escape pattern'
double quotes: "have many: \", \0, \t, ☺, \x0d\x0a == \r\n, and more."
Superscript two: ²

special_characters: "[ John ] & { Jane } - <Doe>"

literal_block: |
  This entire block of text will be the value of the 'literal_block' key,
  with line breaks being preserved.

  The literal continues until de-dented, and the leading indentation is
  stripped.

      Any lines that are 'more-indented' keep the rest of their indentation -
      these lines will be indented by 4 spaces.
folded_style: >
  This entire block of text will be the value of 'folded_style', but this
  time, all newlines will be replaced with a single space.
literal_strip: |-
  This entire block of text will be the value of the 'literal_strip' key,
  with trailing blank line being stripped.
block_keep: >+
  This entire block of text will be the value of 'block_keep', but this
  time, all newlines will be replaced with a single space and
  trailing blank line being kept.
"#;
        // Must not panic/hang on real-world scalar syntax breadth.
        let symbols = extract_exports(content);

        // Plain identifier-style keys are found.
        for key in [
            "key",
            "another_key",
            "a_number_value",
            "scientific_notation",
            "hex_notation",
            "octal_notation",
            "boolean",
            "null_value",
            "another_null_value",
            "no",
            "yes",
            "not_enclosed",
            "enclosed",
            "however",
            "special_characters",
            "literal_block",
            "folded_style",
            "literal_strip",
            "block_keep",
        ] {
            assert!(symbols.contains(&key.to_string()), "missing key: {key}");
        }

        // The `&` inside the quoted `special_characters` value is not a YAML
        // anchor (it's plain text data), so it must not leak "Jane" or any
        // other fragment out as a spurious anchor/symbol.
        assert!(!symbols.iter().any(|s| s.contains("Jane")));

        // Indented lines inside literal/folded block scalars (prose that
        // happens to contain `key:`-shaped substrings, e.g. "the leading
        // indentation is") must never be mistaken for top-level keys.
        assert!(!symbols.contains(&"stripped".to_string()));
    }

    #[test]
    fn test_real_learnxinyminutes_anchors_merge_keys_and_tags() {
        // Excerpt from learnxinyminutes.com's yaml.md "EXTRA YAML FEATURES"
        // section: real anchors/aliases, the `<<` merge key, and explicit
        // `!!type` tags.
        let content = r#"anchored_content: &anchor_name This string will appear as the value of two keys.
other_anchor: *anchor_name

base: &base
  name: Everyone has same name

foo:
  <<: *base # doesn't merge the anchor
  age: 10
  name: John
bar:
  <<: *base # base anchor will be merged
  age: 20

explicit_boolean: !!bool true
explicit_integer: !!int 42
explicit_float: !!float -42.24
explicit_string: !!str 0.5
explicit_datetime: !!timestamp 2022-11-17 12:34:56.78 +9
explicit_null: !!null null

python_complex_number: !!python/complex 1+2j

? !!python/tuple [ 5, 7 ]
: Fifty Seven
"#;
        let symbols = extract_exports(content);

        // Top-level keys are found.
        for key in [
            "anchored_content",
            "other_anchor",
            "base",
            "foo",
            "bar",
            "explicit_boolean",
            "explicit_integer",
            "explicit_float",
            "explicit_string",
            "explicit_datetime",
            "explicit_null",
            "python_complex_number",
        ] {
            assert!(symbols.contains(&key.to_string()), "missing key: {key}");
        }

        // The anchor `&anchor_name` is extracted as its own symbol.
        assert!(symbols.contains(&"anchor_name".to_string()));

        // The `<<` merge key and `*base` alias reference are not anchors
        // (no `&`) and must not produce a stray "base" duplicate or any
        // garbage symbol from the merge-key syntax itself.
        assert!(!symbols.iter().any(|s| s == "<<"));

        // The complex mapping key introduced by `? !!python/tuple [ 5, 7 ]`
        // must not be mistaken for a top-level `key:` declaration.
        assert!(!symbols.iter().any(|s| s.contains("Fifty")));
    }

    #[test]
    fn test_github_actions_workflow() {
        let content = r#"name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps: []
  lint:
    runs-on: ubuntu-latest
    steps: []
  build:
    runs-on: ubuntu-latest
    steps: []
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"name".to_string()));
        assert!(symbols.contains(&"on".to_string()));
        assert!(symbols.contains(&"jobs".to_string()));
        assert!(symbols.contains(&"jobs.test".to_string()));
        assert!(symbols.contains(&"jobs.lint".to_string()));
        assert!(symbols.contains(&"jobs.build".to_string()));
    }

    #[test]
    fn test_docker_compose() {
        let content = r#"version: "3.8"
services:
  web:
    image: nginx
    ports: ["80:80"]
  db:
    image: postgres
    environment: {}
volumes:
  pgdata:
    driver: local
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"version".to_string()));
        assert!(symbols.contains(&"services".to_string()));
        assert!(symbols.contains(&"services.web".to_string()));
        assert!(symbols.contains(&"services.db".to_string()));
        assert!(symbols.contains(&"volumes".to_string()));
        assert!(symbols.contains(&"volumes.pgdata".to_string()));
    }

    #[test]
    fn test_anchors() {
        let content = r#"defaults: &defaults
  timeout: 30
  retries: 3
jobs:
  test:
    <<: *defaults
    command: test
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"defaults".to_string()));
        assert!(symbols.contains(&"jobs".to_string()));
        assert!(symbols.contains(&"jobs.test".to_string()));
        assert!(symbols.contains(&"defaults".to_string())); // anchor name matches key
    }

    #[test]
    fn test_top_level_only() {
        let content = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
spec:
  replicas: 3
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"apiVersion".to_string()));
        assert!(symbols.contains(&"kind".to_string()));
        assert!(symbols.contains(&"metadata".to_string()));
        assert!(symbols.contains(&"spec".to_string()));
        // metadata.name should NOT be extracted since metadata is not a well-known parent
        assert!(!symbols.contains(&"metadata.name".to_string()));
    }

    #[test]
    fn test_four_space_indentation() {
        let content = r#"name: CI
on: push
jobs:
    test:
        runs-on: ubuntu-latest
    build:
        runs-on: ubuntu-latest
services:
    web:
        image: nginx
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"jobs.test".to_string()));
        assert!(symbols.contains(&"jobs.build".to_string()));
        assert!(symbols.contains(&"services.web".to_string()));
    }

    #[test]
    fn test_four_space_nested_not_extracted() {
        let content = r#"jobs:
    test:
        runs-on: ubuntu-latest
        steps:
            - name: checkout
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"jobs.test".to_string()));
        // `runs-on` is deeper nesting, not a direct child of `jobs`
        assert!(!symbols.contains(&"jobs.runs-on".to_string()));
    }

    #[test]
    fn test_tab_indentation() {
        let content = "services:\n\tweb:\n\t\timage: nginx\n\tdb:\n\t\timage: postgres\n";
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"services.web".to_string()));
        assert!(symbols.contains(&"services.db".to_string()));
    }

    #[test]
    fn test_comments_and_empty_lines() {
        let content = r#"# This is a comment
name: test

# Another comment
on:
  push:
    branches: [main]
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"name".to_string()));
        assert!(symbols.contains(&"on".to_string()));
        assert_eq!(symbols.len(), 2);
    }

    #[test]
    fn test_ampersand_in_url_query_string_not_treated_as_anchor() {
        // Docker Compose environment values often embed URLs with query strings.
        // The `&` there is a query-parameter separator, not a YAML anchor sigil,
        // since it's glued directly to the preceding word character (no
        // delimiter before it as a real anchor would have).
        let content = r#"services:
  web:
    image: nginx
    environment:
      - API_URL=https://api.example.com/v1?key=abc&format=json
  db:
    image: postgres
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"services.web".to_string()));
        assert!(symbols.contains(&"services.db".to_string()));
        assert!(!symbols.contains(&"format".to_string()));

        // A GitHub Actions `run:` block with a curl command hits the same case.
        let gha_content = r#"jobs:
  notify:
    runs-on: ubuntu-latest
    steps:
      - run: |
          curl -X POST "https://hooks.example.com/service?ref=main&token=abc123"
"#;
        let gha_symbols = extract_exports(gha_content);
        assert!(gha_symbols.contains(&"jobs.notify".to_string()));
        assert!(!gha_symbols.contains(&"token".to_string()));
    }

    #[test]
    fn test_real_anchor_still_extracted_after_delimiter_tightening() {
        // Legitimate anchors (preceded by whitespace, a dash, or a flow-collection
        // delimiter) must still be recognized after tightening the ANCHOR regex.
        let content = r#"defaults: &defaults
  timeout: 30
list: [&first-item value, &second-item other]
jobs:
  test:
    <<: *defaults
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"defaults".to_string()));
        assert!(symbols.contains(&"first-item".to_string()));
    }

    #[test]
    fn test_nested_outputs_under_job_extracted() {
        // `outputs:` is a well-known parent, but in a real GitHub Actions workflow
        // it commonly appears nested under `jobs.<job>.outputs`, not at column 0.
        // The named entries there are part of the workflow's public surface
        // (consumed elsewhere via `needs.<job>.outputs.<name>`) and must not be
        // silently dropped just because they aren't top-level.
        let content = r#"jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      artifact-name: ${{ steps.build.outputs.name }}
      version: ${{ steps.build.outputs.version }}
    steps:
      - id: build
        run: echo "name=app" >> "$GITHUB_OUTPUT"
  deploy:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ needs.build.outputs.artifact-name }}"
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"jobs.build".to_string()));
        assert!(symbols.contains(&"jobs.deploy".to_string()));
        assert!(symbols.contains(&"outputs.artifact-name".to_string()));
        assert!(symbols.contains(&"outputs.version".to_string()));
    }

    #[test]
    fn test_quoted_top_level_keys_extracted() {
        // Real bug found this session: `TOP_LEVEL_KEY` required the key to
        // start directly with an identifier character, so a quoted top-level
        // key like `"name":` or `"on":` was never captured at all. This
        // matters specifically for GitHub Actions workflows: unquoted `on`
        // is parsed as the boolean `true` under YAML 1.1, so linters/editors
        // commonly recommend (and real-world workflow files use) `"on":`
        // instead — meaning the single most identifying top-level key of a
        // CI workflow could be silently missing from its exported symbols.
        // Fixed by allowing an optional matching pair of double or single
        // quotes around an otherwise-identifier-shaped key.
        let content = r#""name": "CI"
'on': push
jobs:
  test:
    runs-on: ubuntu-latest
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"name".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"on".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"jobs".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"jobs.test".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_quoted_non_identifier_top_level_key_not_matched() {
        // Verified-correct (not a bug): a quoted top-level key whose content
        // isn't identifier-shaped (contains spaces/punctuation) must
        // continue to be skipped rather than partially matched or corrupted
        // — the fix for quoted identifier keys must not start capturing
        // arbitrary quoted prose as a symbol.
        let content = r#"'Keys can be quoted too.': "Useful if you want a colon."
plain_key: value
"#;
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"plain_key".to_string()));
        assert!(
            !symbols.iter().any(|s| s.contains("Keys can be quoted")),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_flow_style_mapping_under_well_known_parent_not_expanded() {
        // Documents a known, accepted limitation rather than a bug: this
        // backend's nested-child scan under `NESTED_SYMBOL_PARENTS` is
        // line-based (it looks at indentation of subsequent lines), so an
        // entirely flow-style value (`services: {web: {...}, db: {...}}`)
        // on a single line has no "subsequent lines" to scan. The top-level
        // key itself is still correctly found; the flow-style children are
        // simply not expanded into `services.web` / `services.db`. This is
        // an acceptable gap for a regex/line-oriented extractor targeting
        // real-world CI/compose files, which are overwhelmingly block-style
        // for these particular keys — but must not panic or produce a
        // false-positive substitute symbol either.
        let content = "services: {web: {image: nginx}, db: {image: postgres}}\n";
        let symbols = extract_exports(content);
        assert!(symbols.contains(&"services".to_string()));
        assert!(!symbols.contains(&"services.web".to_string()));
        assert!(!symbols.contains(&"services.db".to_string()));
    }

    #[test]
    fn test_hash_and_colon_inside_quoted_child_value_do_not_break_extraction() {
        // Verified-correct (not a bug): a nested child's quoted scalar value
        // containing both a `:` and a `#` (`"Hello: World # not a comment"`)
        // must not be mistaken for a comment (stopping the child scan early)
        // or for additional nested structure — only the child *key* before
        // the first top-level `:` on its line matters.
        let content = r#"jobs:
  build:
    outputs:
      greeting: "Hello: World # not a comment"
      other: value
"#;
        let symbols = extract_exports(content);
        assert!(
            symbols.contains(&"outputs.greeting".to_string()),
            "{symbols:?}"
        );
        assert!(
            symbols.contains(&"outputs.other".to_string()),
            "{symbols:?}"
        );
    }

    #[test]
    fn test_anchor_dedupes_across_multi_document_yaml() {
        // Genuinely new case beyond the existing top-level-key dedup test:
        // the same anchor NAME reused in two `---`-separated documents (a
        // common pattern — e.g. redefining `&defaults` per environment in a
        // multi-doc Kubernetes/CI file) must be deduped the same way
        // top-level keys already are, not emitted once per document.
        let content = r#"defaults: &defaults
  timeout: 30
---
defaults: &defaults
  timeout: 60
"#;
        let symbols = extract_exports(content);
        assert_eq!(
            symbols.iter().filter(|s| *s == "defaults").count(),
            1,
            "{symbols:?}"
        );
    }

    #[test]
    fn test_multi_document_yaml_dedupes_top_level_keys() {
        // A multi-document Kubernetes manifest (Deployment + Service, `---`-separated)
        // repeats the same top-level key names in every document. The symbol list
        // should behave like a set, consistent with how anchors are already deduped.
        let content = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
spec:
  replicas: 3
---
apiVersion: v1
kind: Service
metadata:
  name: my-app-svc
spec:
  ports: []
"#;
        let symbols = extract_exports(content);
        assert_eq!(symbols.iter().filter(|s| *s == "apiVersion").count(), 1);
        assert_eq!(symbols.iter().filter(|s| *s == "kind").count(), 1);
        assert_eq!(symbols.iter().filter(|s| *s == "metadata").count(), 1);
        assert_eq!(symbols.iter().filter(|s| *s == "spec").count(), 1);
    }
}
