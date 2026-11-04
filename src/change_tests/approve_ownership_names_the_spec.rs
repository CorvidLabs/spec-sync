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
