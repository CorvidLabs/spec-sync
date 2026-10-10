use super::*;
use crate::parser::parse_checked_issue_references;

#[test]
fn quoted_block_list_items_and_scalars_are_unquoted() {
    // #545: a quoted entry resolved to the literal path `"src/auth.ts"`,
    // was reported as a missing source file, and then cascaded into a bogus
    // "documents X but no matching export found" — while frontmatter was
    // still reported valid, so nothing pointed at the quotes.
    let content = "---\nmodule: \"auth\"\nversion: 1\nstatus: 'active'\nfiles:\n  - \"src/auth.ts\"\n  - 'src/auth.utils.ts'\n  - src/plain.ts\ndepends_on:\n  - \"specs/types/types.spec.md\"\n---\n\n# Auth\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
    assert_eq!(parsed.frontmatter.status.as_deref(), Some("active"));
    assert_eq!(
        parsed.frontmatter.files,
        vec![
            "src/auth.ts".to_string(),
            "src/auth.utils.ts".to_string(),
            "src/plain.ts".to_string(),
        ]
    );
    assert_eq!(
        parsed.frontmatter.depends_on,
        vec!["specs/types/types.spec.md".to_string()]
    );
}

#[test]
fn quoted_entry_keeps_a_trailing_comment_out_of_the_path() {
    let content = "---\nmodule: auth\nfiles:\n  - \"src/auth.ts\" # the main file\n---\n\n# Auth\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(parsed.frontmatter.files, vec!["src/auth.ts".to_string()]);
}

#[test]
fn a_hash_inside_a_quoted_path_is_not_a_comment() {
    let content = "---\nmodule: auth\nfiles:\n  - \"src/a#b.ts\"\n---\n\n# Auth\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(parsed.frontmatter.files, vec!["src/a#b.ts".to_string()]);
}

#[test]
fn an_unterminated_quote_is_an_error_not_a_literal_path() {
    // Silently keeping the literal is how #545 stayed invisible; fail loudly
    // instead, matching flow-style list behavior.
    let content = "---\nmodule: auth\nfiles:\n  - \"src/auth.ts\n---\n\n# Auth\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert!(
        parsed
            .errors
            .iter()
            .any(|error| error.contains("unterminated quoted string")),
        "{:?}",
        parsed.errors
    );
    assert!(parsed.frontmatter.files.is_empty());
}

#[test]
fn flow_style_lists_still_unquote_their_own_items() {
    let content = "---\nmodule: auth\nfiles: [\"src/a.ts\", 'src/b.ts', src/c.ts]\n---\n\n# Auth\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(
        parsed.frontmatter.files,
        vec![
            "src/a.ts".to_string(),
            "src/b.ts".to_string(),
            "src/c.ts".to_string()
        ]
    );
}

#[test]
fn test_parse_frontmatter_basic() {
    let content = "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.ts\ndb_tables: []\ndepends_on: []\n---\n\n# Auth\n\n## Purpose\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
    assert_eq!(parsed.frontmatter.version.as_deref(), Some("1"));
    assert_eq!(parsed.frontmatter.status.as_deref(), Some("active"));
    assert_eq!(parsed.frontmatter.files, vec!["src/auth.ts"]);
    assert!(parsed.frontmatter.db_tables.is_empty());
}

#[test]
fn test_scalar_depends_on_normalized_to_list() {
    // Regression (#419): a scalar `depends_on: alpha` was silently dropped,
    // disabling the dependency edge with no diagnostic.
    let content = "---\nmodule: beta\nversion: 1\nstatus: active\nfiles: []\ndepends_on: alpha\n---\n\n# Beta\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert_eq!(parsed.frontmatter.depends_on, vec!["alpha"]);
}

#[test]
fn test_inline_bracket_depends_on_parsed() {
    let content = "---\nmodule: beta\ndepends_on: [alpha, gamma]\n---\n\n# Beta\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert_eq!(parsed.frontmatter.depends_on, vec!["alpha", "gamma"]);
}

#[test]
fn test_duplicate_depends_on_deduped() {
    // Regression (#419): duplicate entries inflated edge counts and doubled
    // mermaid edges.
    let content = "---\nmodule: beta\ndepends_on:\n  - alpha\n  - alpha\n  - gamma\n  - alpha\n---\n\n# Beta\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert_eq!(parsed.frontmatter.depends_on, vec!["alpha", "gamma"]);
}

#[test]
fn test_strip_yaml_comment() {
    assert_eq!(strip_yaml_comment("active"), "active");
    assert_eq!(strip_yaml_comment("active # this is the status"), "active");
    assert_eq!(
        strip_yaml_comment("value #no-space-means-not-comment"),
        "value #no-space-means-not-comment"
    );
    assert_eq!(
        strip_yaml_comment("[42, 57] # issue list"),
        "[42, 57] # issue list"
    ); // brackets preserved
    assert_eq!(
        strip_yaml_comment("\"quoted # value\""),
        "\"quoted # value\""
    ); // quotes preserved
    assert_eq!(strip_yaml_comment("value #"), "value");
}

#[test]
fn test_parse_frontmatter_inline_comments() {
    let content = "---\nmodule: auth # the auth module\nversion: 1 # initial\nstatus: active # current status\nfiles:\n  - src/auth.ts # main file\n---\n\n# Auth\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
    assert_eq!(parsed.frontmatter.version.as_deref(), Some("1"));
    assert_eq!(parsed.frontmatter.status.as_deref(), Some("active"));
    assert_eq!(parsed.frontmatter.files, vec!["src/auth.ts"]);
}

#[test]
fn test_parse_frontmatter_leading_bom() {
    // A leading UTF-8 BOM must not break frontmatter parsing: the spec is valid,
    // the invisible byte just precedes the opening `---`.
    let content = "\u{feff}---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.ts\ndb_tables: []\ndepends_on: []\n---\n\n# Auth\n\n## Purpose\n";
    let parsed = parse_frontmatter(content).expect("BOM-prefixed frontmatter should parse");
    assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
    assert_eq!(parsed.frontmatter.files, vec!["src/auth.ts"]);
    // The body is BOM-free (the strip happens before the split).
    assert!(
        parsed.body.starts_with("\n# Auth"),
        "body: {:?}",
        parsed.body
    );
    assert!(!parsed.body.contains('\u{feff}'));

    // Repeated leading BOMs are also tolerated (trim_start_matches).
    let doubled = format!("\u{feff}{content}");
    let parsed2 = parse_frontmatter(&doubled).expect("repeated-BOM frontmatter should parse");
    assert_eq!(parsed2.frontmatter.module.as_deref(), Some("auth"));
}

#[test]
fn test_parse_frontmatter_bom_only_leading() {
    // Only a *leading* BOM is stripped; a U+FEFF elsewhere in the content is
    // real (zero-width no-break space) and must be preserved.
    let content =
        "---\nmodule: a\nfiles:\n  - src/a.ts\n---\n\n# A\n\nZero\u{feff}width in body.\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert_eq!(parsed.frontmatter.module.as_deref(), Some("a"));
    assert!(
        parsed.body.contains('\u{feff}'),
        "a non-leading BOM must be preserved"
    );
}

#[test]
fn test_parse_frontmatter_tabs_and_whitespace() {
    // Tabs used for indentation instead of spaces
    let content = "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n\t- src/auth.ts\n\t- src/auth.utils.ts\n---\n\n# Auth\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert_eq!(
        parsed.frontmatter.files,
        vec!["src/auth.ts", "src/auth.utils.ts"]
    );
}

#[test]
fn test_parse_frontmatter_trailing_spaces() {
    let content = "---\nmodule: auth   \nversion: 1  \nstatus: active  \nfiles:\n  - src/auth.ts   \n---\n\n# Auth\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
    assert_eq!(parsed.frontmatter.files, vec!["src/auth.ts"]);
}

#[test]
fn test_parse_frontmatter_missing() {
    let content = "# No frontmatter here\n\nJust markdown.";
    assert!(parse_frontmatter(content).is_none());
}

// The shipped Windows defect (#696): `FRONTMATTER_RE` is `^---\n(.*?)\n---\n(.*)$`, LF-only,
// and 18 of the 39 call sites outside this module hand it unnormalized bytes. On a clone with
// `core.autocrlf=true` this returned `None` for every spec in the project, which `view.rs`
// reports as "Cannot parse frontmatter".
//
// Discriminates: before normalization landed here, `parse_frontmatter` returned `None` and
// the `expect` failed.
#[test]
fn test_parse_frontmatter_crlf_document() {
    let content = "---\r\nmodule: auth\r\nversion: 1\r\nstatus: active\r\nfiles:\r\n  - src/auth.ts\r\n---\r\n\r\n# Auth\r\n\r\n## Purpose\r\n";

    let parsed = parse_frontmatter(content).expect("a CRLF spec must parse");

    assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
    assert_eq!(parsed.frontmatter.version.as_deref(), Some("1"));
    assert_eq!(parsed.frontmatter.status.as_deref(), Some("active"));
    // A trailing `\r` on a list item would resolve as the literal path "src/auth.ts\r",
    // reported as a missing source file — the cascade #696 describes for quoted paths.
    assert_eq!(parsed.frontmatter.files, vec!["src/auth.ts"]);
    // Callers index, split and compare the body as if it were LF, so it must be LF.
    assert!(
        !parsed.body.contains('\r'),
        "body must be LF-only, got: {:?}",
        parsed.body
    );
}

// The discriminating fixture #696 names: CRLF **and** a leading BOM **and** a horizontal rule
// in the body. No single implementation handled all three, and each one alone is survivable —
// it is the combination that separates a correct reader from one that merely looks correct.
#[test]
fn test_parse_frontmatter_crlf_with_bom_and_body_horizontal_rule() {
    let content = "\u{feff}---\r\nmodule: auth\r\nversion: 1\r\n---\r\n\r\n# Auth\r\n\r\nFirst.\r\n\r\n---\r\n\r\nSecond.\r\n";

    let parsed =
        parse_frontmatter(content).expect("CRLF + BOM frontmatter must parse (#696 fixture)");

    assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
    assert!(!parsed.body.contains('\u{feff}'));
    // The rule is body content. Ending frontmatter at the *next* `---` anywhere in the
    // document would swallow "First." and leave prose nobody can tell from prose never written.
    assert!(parsed.body.contains("First."), "body: {:?}", parsed.body);
    assert!(parsed.body.contains("Second."), "body: {:?}", parsed.body);
}

// Honest label: this is the CONTROL for the two tests above. An LF document must take the
// borrowed, zero-allocation path and be byte-identical to what it produced before, otherwise
// the guard is doing work on every spec in the repository.
#[test]
fn test_parse_frontmatter_lf_body_is_unchanged_by_the_crlf_guard() {
    let content = "---\nmodule: auth\nversion: 1\n---\n\n# Auth\n\nFirst.\n\n---\n\nSecond.\n";

    let parsed = parse_frontmatter(content).expect("LF spec parses");

    assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
    assert_eq!(parsed.body, "\n# Auth\n\nFirst.\n\n---\n\nSecond.\n");
}

// A lone `\r` is not a line ending we produce and is not what `replace("\r\n", "\n")` targets;
// it is real content. Pinned so the guard is never "simplified" into stripping every `\r`.
#[test]
fn test_parse_frontmatter_preserves_a_lone_carriage_return_in_the_body() {
    let content = "---\nmodule: auth\n---\n\nprogress\rbar\n";

    let parsed = parse_frontmatter(content).expect("parses");

    assert!(
        parsed.body.contains("progress\rbar"),
        "a lone CR is content, got: {:?}",
        parsed.body
    );
}

// Honest label: this is a CHARACTERIZATION test, not a discriminator for the canonical
// stripper. Its body is byte-identical to the `change::strip_frontmatter` this promoted, so
// it passes against that implementation too; it is RED only against `view::strip_frontmatter`.
// It pins the promoted behaviour so a future edit cannot quietly regress it.
//
// The canonical stripper (#696), on the six axes that separated the five implementations it
// replaces. Each case names the implementation it discriminates against.
#[test]
fn test_strip_frontmatter_all_six_axes() {
    // LF — the baseline every implementation got right.
    assert_eq!(
        strip_frontmatter("---\nmodule: a\n---\n\n## Purpose\n"),
        "\n## Purpose\n"
    );

    // CRLF — `view::strip_frontmatter` and `parser`'s regex were LF-only.
    assert_eq!(
        strip_frontmatter("---\r\nmodule: a\r\n---\r\n\r\n## Purpose\r\n"),
        "\r\n## Purpose\r\n"
    );

    // BOM — hides the opening delimiter from a bare `strip_prefix("---")`.
    assert_eq!(
        strip_frontmatter("\u{feff}---\nmodule: a\n---\n\n## Purpose\n"),
        "\n## Purpose\n"
    );

    // Unterminated frontmatter — keep the whole document rather than guess where it ended.
    let unterminated = "---\nmodule: a\nstill open\n";
    assert_eq!(strip_frontmatter(unterminated), unterminated);

    // Closing delimiter at EOF, with no trailing newline — `view::strip_frontmatter` searched
    // for `"\n---\n"` and so refused this, leaving the YAML block on screen.
    assert_eq!(strip_frontmatter("---\nmodule: a\n---"), "");

    // A horizontal rule in the body — `change::strip_yaml_frontmatter` scanned for `\n---\n`
    // across the WHOLE document before trying `\r\n---\r\n`, so this exact CRLF-body/LF-rule
    // shape silently deleted everything above the rule.
    let mixed = "---\r\nmodule: a\r\n---\r\n\r\nKeep me.\r\n\n---\n\nAnd me.\r\n";
    let body = strip_frontmatter(mixed);
    assert!(body.contains("Keep me."), "body: {body:?}");
    assert!(body.contains("And me."), "body: {body:?}");
    assert!(!body.contains("module: a"), "body: {body:?}");
}

// Honest label: this is the CONTROL for the stripper. A document with no frontmatter at all
// must come back untouched, rules and all — otherwise "strip frontmatter" becomes
// "delete everything before the first horizontal rule", which is what two of the five
// implementations this replaces actually did.
#[test]
fn test_strip_frontmatter_keeps_a_document_that_has_none() {
    let text = "# Notes\n\nFirst.\n\n---\n\nSecond.\n\n---\n\nThird.\n";

    assert_eq!(strip_frontmatter(text), text);
}

// Honest label: DISCRIMINATOR for #716, both directions of the same one-character mistake.
//
// An OPENER padded with a trailing space was not a delimiter, so the document came back whole
// and its YAML read as body prose — which is how `change`'s completeness gate approved an
// artifact with nothing written in it. A CLOSER padded the same way was walked past, so the
// scan stopped at the first horizontal rule in the BODY and deleted the prose above it.
//
// Discriminates: on the unfixed binary the opener case returns the whole document, and the
// closer case returns "\nMore prose.\n" — "Real prose." simply gone.
#[test]
fn test_strip_frontmatter_accepts_a_delimiter_padded_with_trailing_whitespace() {
    assert_eq!(
        strip_frontmatter("---  \nchange: CHG-1\nartifact: design\n---\n"),
        "",
        "a padded OPENER must open the block, or its YAML counts as prose"
    );

    let padded_closer = "---\nspec: a.spec.md\n---  \n\nReal prose.\n\n---\n\nMore prose.\n";
    let body = strip_frontmatter(padded_closer);
    assert!(
        body.contains("Real prose.") && body.contains("More prose."),
        "a padded CLOSER must end the block, not send the scan into the body: {body:?}"
    );
    assert!(!body.contains("spec: a.spec.md"), "body: {body:?}");

    // Tabs and a CRLF ending are the same shape and get the same answer.
    assert_eq!(
        strip_frontmatter("---\t\r\nspec: a.spec.md\r\n--- \t\r\n\r\nBody.\r\n"),
        "\r\nBody.\r\n"
    );
}

// Honest label: CONTROL, and it passes on the unfixed binary too — that is the point. It pins
// the half of the delimiter rule that must NOT loosen.
//
// `----` and `--- x` are legal Markdown: a thematic break, and a break followed by text. A
// document that opens with one is a document, not frontmatter. If a future edit "generalises"
// the trailing-whitespace tolerance into "starts with three dashes", every one of these
// becomes an opener, the scan runs forward to the next rule, and the prose in between is
// deleted — the exact failure this reader exists to prevent, and the one that reads like
// prose nobody ever wrote.
#[test]
fn test_strip_frontmatter_refuses_a_delimiter_that_is_not_three_dashes() {
    for opener in ["----", "--", "--- x", "---change: x", "  ---"] {
        let text = format!("{opener}\n\nReal prose.\n\n---\n\nMore prose.\n");
        assert_eq!(
            strip_frontmatter(&text),
            text,
            "`{opener}` must not open frontmatter"
        );
    }

    // The same rule at the closing end: an unterminated block keeps the whole document rather
    // than ending at a four-dash rule further down.
    let unterminated = "---\nspec: a.spec.md\n----\n\nReal prose.\n";
    assert_eq!(strip_frontmatter(unterminated), unterminated);
}

// Honest label: DISCRIMINATOR for #716 in the sibling reader. `strip_frontmatter` was the
// site the report named; `parse_frontmatter` had the same padded-closer hole and a worse
// consequence, because the over-long capture is handed to the YAML line parser as well as
// truncating the body.
//
// Discriminates: on the unfixed binary `parsed.body` is "\nSecond.\n" — "First." deleted —
// and `warnings` holds two "Ignoring malformed frontmatter line" entries naming `---` and
// `First.`, body prose that was never frontmatter.
#[test]
fn test_parse_frontmatter_closes_on_a_delimiter_padded_with_trailing_whitespace() {
    let content = "---\nmodule: auth\nversion: 1\n---  \n\n# Auth\n\nFirst.\n\n---\n\nSecond.\n";

    let parsed = parse_frontmatter(content).expect("a padded closer still closes frontmatter");

    assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
    assert!(parsed.body.contains("First."), "body: {:?}", parsed.body);
    assert!(parsed.body.contains("Second."), "body: {:?}", parsed.body);
    assert!(
        parsed.warnings.is_empty(),
        "body prose must never reach the frontmatter line parser: {:?}",
        parsed.warnings
    );
}

// Honest label: DISCRIMINATOR for #716. Discriminates: on the unfixed binary this returns
// `None`, which `view.rs` reports as "Cannot parse frontmatter" for a spec whose only defect
// is a space an author cannot see.
#[test]
fn test_parse_frontmatter_opens_on_a_delimiter_padded_with_trailing_whitespace() {
    let parsed = parse_frontmatter("---  \nmodule: auth\nversion: 1\n---\n\n# Auth\n")
        .expect("a padded opener still opens frontmatter");

    assert_eq!(parsed.frontmatter.module.as_deref(), Some("auth"));
    assert_eq!(parsed.body, "\n# Auth\n");
}

// Honest label: DISCRIMINATOR for #716 in the third reader. `parse_checked_issue_references`
// matched delimiters with its own pair of `strip_prefix`/`split_once` chains, which required
// BOTH delimiters to carry the SAME line ending and neither to be padded.
//
// Discriminates: on the unfixed binary the padded-opener case is
// `Err("missing or malformed YAML frontmatter")`, the padded-closer case is
// `Err("invalid YAML frontmatter")` — body prose reached the YAML parser — and the
// LF-opener/CRLF-closer case is `Err("missing or malformed YAML frontmatter")` for a document
// whose references are right there.
#[test]
fn all_frontmatter_readers_agree_on_what_a_delimiter_is() {
    // (opener, closer, is this frontmatter?)
    let cases = [
        ("---", "---", true),
        ("---  ", "---", true),
        ("---", "---  ", true),
        ("---\t", "--- \t", true),
        ("----", "---", false),
        ("---", "----", false),
        ("--- x", "---", false),
        ("  ---", "---", false),
    ];

    for (opener, closer, is_frontmatter) in cases {
        let text = format!("{opener}\nimplements: [7]\n{closer}\n\nBody prose.\n");
        let label = format!("opener {opener:?} / closer {closer:?}");

        assert_eq!(
            strip_frontmatter(&text) != text.as_str(),
            is_frontmatter,
            "strip_frontmatter disagrees for {label}"
        );
        assert_eq!(
            parse_frontmatter(&text).is_some(),
            is_frontmatter,
            "parse_frontmatter disagrees for {label}"
        );
        assert_eq!(
            parse_checked_issue_references(&text).is_ok(),
            is_frontmatter,
            "parse_checked_issue_references disagrees for {label}"
        );
    }
}

// Honest label: CHARACTERIZATION, and it passes on the unfixed binary — that is why it is
// here. #696 made `parse_frontmatter` normalize whenever the document contains a `\r`, so a
// document with LF frontmatter and a CRLF body now returns an LF body where it previously
// returned CRLF. That change was correct and was never stated. Every consumer is read-only
// analysis that never maps the body back to raw file bytes; the moment one does not, this
// test is the record of what it was promised.
#[test]
fn test_parse_frontmatter_body_is_lf_even_when_only_the_body_is_crlf() {
    let parsed = parse_frontmatter("---\nmodule: auth\n---\n\r\n# Auth\r\n\r\nPara.\r\n")
        .expect("LF frontmatter parses");

    assert_eq!(parsed.body, "\n# Auth\n\nPara.\n");
    assert!(!parsed.body.contains('\r'));
}

#[test]
fn test_parse_frontmatter_implements_list() {
    let content = "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.ts\nimplements:\n  - 42\n  - 57\ntracks:\n  - 10\n---\n\n# Auth\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert_eq!(parsed.frontmatter.implements, vec![42, 57]);
    assert_eq!(parsed.frontmatter.tracks, vec![10]);
}

#[test]
fn test_parse_frontmatter_implements_inline() {
    let content = "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.ts\nimplements: [42, 57]\ntracks: [10]\n---\n\n# Auth\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert_eq!(parsed.frontmatter.implements, vec![42, 57]);
    assert_eq!(parsed.frontmatter.tracks, vec![10]);
}

#[test]
fn test_parse_frontmatter_empty_implements() {
    let content = "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.ts\nimplements: []\n---\n\n# Auth\n";
    let parsed = parse_frontmatter(content).unwrap();
    assert!(parsed.frontmatter.implements.is_empty());
    assert!(parsed.frontmatter.tracks.is_empty());
}
