// Verifies REQ-change-120: an approve ownership refusal names the spec to edit.
// Leif (Grok Bot chat t811u): "`change approve` refuses affected paths that no
// module owns (for example a new module's own `.spec.ts` files), with a fix that
// requires editing the module's files list; say which spec to edit in the error."

fn ownership_spec(root: &Path, module: &str, files: &[&str]) {
    let listed: String = files.iter().map(|file| format!("  - {file}\n")).collect();
    fs::create_dir_all(root.join(format!("specs/{module}"))).unwrap();
    fs::write(
        root.join(format!("specs/{module}/{module}.spec.md")),
        format!(
            "---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n{listed}---\n\n# {module}\n\n## Purpose\n\nOwner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"
        ),
    )
    .unwrap();
}

fn approve_ownership_error(root: &Path, specs: &[&str], paths: &[&str]) -> String {
    for path in paths {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "export const value = 1;\n").unwrap();
    }
    let mut record = completed_no_spec_record(root);
    record.affected_specs = specs.iter().map(|spec| spec.to_string()).collect();
    record.affected_paths = paths.iter().map(|path| path.to_string()).collect();
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("approve must refuse an affected path no declared module owns")
}

#[test]
fn approve_ownership_refusal_names_the_declared_spec_to_edit() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing", &["src/billing/charge.ts"]);
    ownership_spec(root, "search", &["src/search/query.ts"]);

    let error = approve_ownership_error(
        root,
        &["billing", "search"],
        &["src/search/query.ts", "src/search/query.spec.ts"],
    );

    assert!(
        error.contains("src/search/query.spec.ts: add it to the `files:` list in `specs/search/search.spec.md`"),
        "the refusal must name the declared spec to edit: {error}"
    );
    assert!(
        !error.contains("specs/billing/billing.spec.md"),
        "an unrelated declared spec must not be suggested: {error}"
    );
}

#[test]
fn approve_ownership_refusal_names_an_undeclared_spec_that_lists_the_path() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing", &["src/billing/charge.ts"]);
    ownership_spec(root, "ledger", &["src/billing/ledger.ts"]);

    let error = approve_ownership_error(
        root,
        &["billing"],
        &["src/billing/charge.ts", "src/billing/ledger.ts"],
    );

    assert!(
        error.contains(
            "src/billing/ledger.ts: `specs/ledger/ledger.spec.md` lists it for module `ledger`, which this change does not declare; add `--spec ledger`"
        ),
        "the refusal must name the undeclared owning spec: {error}"
    );
}

// Review follow-ups on #841: the hint must agree with how ownership is decided (configured
// `owns`), never pick an arbitrary spec, and never depend on directory iteration order.

fn write_ownership_config(root: &Path, extra: &str) {
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        format!("specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n{extra}"),
    )
    .unwrap();
}

/// DISCRIMINATOR: an undeclared module that owns the path through `[modules."<name>"] owns` is
/// the module to declare. The hint must name it, not send the operator to a `files:` list.
#[test]
fn approve_ownership_refusal_names_an_undeclared_module_that_owns_the_path_in_config() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    write_ownership_config(root, "\n[modules.\"ledger\"]\nowns = [\"src/billing/ledger\"]\n");
    ownership_spec(root, "billing", &["src/billing/charge.ts"]);
    ownership_spec(root, "ledger", &["src/ledger/index.ts"]);

    let error = approve_ownership_error(
        root,
        &["billing"],
        &["src/billing/charge.ts", "src/billing/ledger/post.ts"],
    );

    assert!(
        error.contains(
            "src/billing/ledger/post.ts: the project configuration gives it to module `ledger` (`[modules.\"ledger\"] owns`), which this change does not declare; add `--spec ledger`"
        ),
        "the refusal must name the module whose configured `owns` covers the path: {error}"
    );
}

/// DISCRIMINATOR: when no declared spec lists a file in any directory the path is in, there is
/// no close match. The hint must say so and list every declared spec, sorted, rather than pick
/// whichever module happens to come first.
#[test]
fn approve_ownership_refusal_without_a_close_match_lists_declared_specs_in_order() {
    for order in [["search", "billing"], ["billing", "search"]] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_lifecycle_test_policy(root);
        write_ownership_config(root, "");
        ownership_spec(root, "billing", &["src/billing/charge.ts"]);
        ownership_spec(root, "search", &["src/search/query.ts"]);

        let error = approve_ownership_error(
            root,
            &order,
            &["src/billing/charge.ts", "src/search/query.ts", "src/web/page.ts"],
        );

        assert!(
            error.contains(
                "src/web/page.ts: no declared spec lists a file in its directory; add it to the `files:` list of the spec that should own it (declared: `specs/billing/billing.spec.md`, `specs/search/search.spec.md`)"
            ),
            "{order:?}: the refusal must say there is no close match and list candidates in order: {error}"
        );
    }
}

/// DISCRIMINATOR: two declared specs equally close to the path are both named, sorted.
#[test]
fn approve_ownership_refusal_names_every_equally_close_declared_spec() {
    for order in [["search", "billing"], ["billing", "search"]] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_lifecycle_test_policy(root);
        ownership_spec(root, "billing", &["src/app/billing.ts"]);
        ownership_spec(root, "search", &["src/app/search.ts"]);

        let error = approve_ownership_error(
            root,
            &order,
            &["src/app/billing.ts", "src/app/search.ts", "src/app/shared.ts"],
        );

        assert!(
            error.contains(
                "src/app/shared.ts: add it to the `files:` list of one of `specs/billing/billing.spec.md`, `specs/search/search.spec.md`"
            ),
            "{order:?}: equally close specs must all be named, in order: {error}"
        );
    }
}

/// DISCRIMINATOR: several undeclared specs listing the path are all named, sorted by spec path,
/// whatever order the file system returns them in.
#[test]
fn approve_ownership_refusal_names_every_undeclared_spec_that_lists_the_path_in_order() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing", &["src/billing/charge.ts"]);
    ownership_spec(root, "zledger", &["src/billing/ledger.ts"]);
    ownership_spec(root, "audit", &["src/billing/ledger.ts"]);

    let error = approve_ownership_error(
        root,
        &["billing"],
        &["src/billing/charge.ts", "src/billing/ledger.ts"],
    );

    assert!(
        error.contains(
            "src/billing/ledger.ts: `specs/audit/audit.spec.md` lists it for module `audit`, which this change does not declare; add `--spec audit`; `specs/zledger/zledger.spec.md` lists it for module `zledger`, which this change does not declare; add `--spec zledger`"
        ),
        "every listing spec must be named in path order: {error}"
    );
}

/// DISCRIMINATOR: a spec that lists the path but declares no `module:` owns nothing. The hint
/// must say exactly that, and still name the declared spec to edit, instead of a vague "check
/// that this change declares that spec's module".
#[test]
fn approve_ownership_refusal_explains_a_listing_spec_without_a_module() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing", &["src/billing/charge.ts"]);
    fs::create_dir_all(root.join("specs/notes")).unwrap();
    fs::write(
        root.join("specs/notes/notes.spec.md"),
        "---\nversion: 1\nstatus: stable\nfiles:\n  - src/billing/refund.ts\n---\n\n# Notes\n",
    )
    .unwrap();

    let error = approve_ownership_error(
        root,
        &["billing"],
        &["src/billing/charge.ts", "src/billing/refund.ts"],
    );

    assert!(
        error.contains(
            "src/billing/refund.ts: `specs/notes/notes.spec.md` lists it but has no `module:` in its frontmatter, so it owns nothing; add it to the `files:` list in `specs/billing/billing.spec.md`"
        ),
        "{error}"
    );
    assert!(!error.contains("check that this change declares"), "{error}");
}
