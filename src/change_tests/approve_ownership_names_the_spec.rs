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

/// DISCRIMINATOR for CHANGE-10.a (Leif's #841 nit): "`ownership_hint` picks the first declared
/// spec even at depth 0, so `lib/new.ts` can be pointed at `specs/billing/billing.spec.md`. Guard
/// on `depth > 0` and fall back to the generic message." Checked with both `--spec` orders.
#[test]
fn approve_ownership_refusal_at_depth_zero_falls_back_to_the_generic_message() {
    for order in [["search", "billing"], ["billing", "search"]] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_lifecycle_test_policy(root);
        write_ownership_config(root, "");
        fs::write(
            root.join(".specsync/config.toml"),
            "specs_dir = \"specs\"\nsource_dirs = [\"src\", \"lib\"]\n",
        )
        .unwrap();
        ownership_spec(root, "billing", &["src/billing/charge.ts"]);
        ownership_spec(root, "search", &["src/search/query.ts"]);

        let error = approve_ownership_error(
            root,
            &order,
            &["src/billing/charge.ts", "src/search/query.ts", "lib/new.ts"],
        );

        let hint = error
            .lines()
            .find(|line| line.contains("- lib/new.ts:"))
            .unwrap_or_else(|| panic!("{order:?}: the refusal must list lib/new.ts: {error}"));
        assert_eq!(
            hint.trim(),
            "- lib/new.ts: add it to the `files:` list of the spec that owns it",
            "{order:?}: a path sharing no directory with any declared spec gets the generic message: {error}"
        );
    }
}

/// CHANGE-10.c, `src/foo` vs `src/foobar` (fails on 02818ce, which scored a shared source root as
/// 0; passes on main f65dbed, whose matching Leif approved): directories match by whole path
/// component, so `src/foobar` shares only `src` with `src/foo` (depth 1, which is above 0 and still
/// a match), and a spec listing a file in `src/foo` itself wins over it.
#[test]
fn approve_ownership_refusal_matches_whole_directory_components() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing", &["src/foobar/charge.ts"]);

    let error = approve_ownership_error(root, &["billing"], &["src/foobar/charge.ts", "src/foo/new.ts"]);

    assert!(
        error.contains("src/foo/new.ts: add it to the `files:` list in `specs/billing/billing.spec.md`"),
        "`src/foobar` shares `src` with `src/foo`, depth 1: {error}"
    );

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing", &["src/foobar/charge.ts"]);
    ownership_spec(root, "search", &["src/foo/query.ts"]);

    let error = approve_ownership_error(
        root,
        &["billing", "search"],
        &["src/foobar/charge.ts", "src/foo/query.ts", "src/foo/new.ts"],
    );

    assert!(
        error.contains("src/foo/new.ts: add it to the `files:` list in `specs/search/search.spec.md`"),
        "`src/foo` must not be read as a prefix of `src/foobar`: {error}"
    );
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

/// COVERAGE for CHANGE-10.c, a glob entry (Leif: "globs score correctly"): a `files:` glob scores
/// by its directory components like any other entry. Passes on main f65dbed too.
#[test]
fn approve_ownership_refusal_scores_a_glob_entry_by_its_directory() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing", &["src/billing/*.ts"]);
    ownership_spec(root, "search", &["src/search/query.ts"]);

    let error = approve_ownership_error(
        root,
        &["search", "billing"],
        &["src/search/query.ts", "src/billing/ledger/post.ts"],
    );

    assert!(
        error.contains("src/billing/ledger/post.ts: add it to the `files:` list in `specs/billing/billing.spec.md`"),
        "the glob's directory `src/billing` is the closest match: {error}"
    );
}

/// CHANGE-10.c, the `_ =>` arm: a spec that lists the path for a module this change does declare,
/// but is not that module's canonical spec, does not own it. The hint says so instead of "check
/// that this change declares that spec's module", and still names the canonical spec to edit.
/// Fails on main f65dbed; the fix is 02818ce, which precedes this test (see PR notes).
#[test]
fn approve_ownership_refusal_explains_a_declared_module_listing_outside_its_canonical_spec() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing", &["src/billing/charge.ts"]);
    fs::create_dir_all(root.join("specs/extra")).unwrap();
    fs::write(
        root.join("specs/extra/billing-notes.spec.md"),
        "---\nmodule: billing\nversion: 1\nstatus: stable\nfiles:\n  - src/billing/refund.ts\n---\n\n# Billing notes\n",
    )
    .unwrap();

    let error = approve_ownership_error(
        root,
        &["billing"],
        &["src/billing/charge.ts", "src/billing/refund.ts"],
    );

    assert!(
        error.contains(
            "src/billing/refund.ts: `specs/extra/billing-notes.spec.md` lists it for declared module `billing`, but it is not that module's canonical spec `specs/billing/billing.spec.md`, so the listing does not count; add it to the `files:` list in `specs/billing/billing.spec.md`"
        ),
        "{error}"
    );
    assert!(!error.contains("check that this change declares"), "{error}");
}

/// #986/#956: a no-spec-change draft that touches production source a module owns used to pass
/// approve, check and review, and was refused only by finalize, after a human had approved and
/// reviewed it. Approve now refuses it, naming the owning module and the `change answer` command
/// that fixes this draft, and that command clears the refusal (SHIP-4.a, CHANGE-10;
/// verifies REQ-change-053 and REQ-change-120).
#[test]
fn approve_refuses_a_no_spec_change_draft_over_owned_production_source_and_names_the_answer() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing", &["src/billing/charge.ts"]);
    fs::create_dir_all(root.join("src/billing")).unwrap();
    fs::write(root.join("src/billing/charge.ts"), "export const value = 1;\n").unwrap();
    // The #986 shape: no spec change, so no declared module at all.
    let mut record = completed_no_spec_current_record(root);
    record.affected_specs = Vec::new();
    assert!(record.no_spec_change);
    record.affected_paths = vec!["src/billing/charge.ts".into()];
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();

    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("approve must refuse production source no declared module owns");
    assert!(
        error.contains("src/billing/charge.ts")
            && error.contains("`specs/billing/billing.spec.md` lists it for module `billing`"),
        "the refusal names the path and the module that owns it: {error}"
    );
    let answer = format!("specsync change answer '{}' affected_specs 'billing'", record.id);
    assert!(
        error.contains(&answer),
        "the refusal names the command that fixes this draft: {error}"
    );

    // Running the named command clears the refusal.
    answer_question(root, &record.id, "affected_specs", "billing").unwrap();
    approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect("after the named answer the change approves");
}

/// Codex P2 on #999: a spec that lists the path but is not its module's canonical spec owns
/// nothing, so declaring that module would be refused again. The refusal says why, and the draft
/// answer does not name the module.
#[test]
fn approve_does_not_recommend_a_module_whose_listing_spec_is_not_canonical() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::create_dir_all(root.join("specs/legacy")).unwrap();
    fs::write(
        root.join("specs/legacy/charges.spec.md"),
        "---\nmodule: billing\nversion: 1\nstatus: stable\nfiles:\n  - src/billing/charge.ts\n---\n\n# Charges\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src/billing")).unwrap();
    fs::write(root.join("src/billing/charge.ts"), "export const value = 1;\n").unwrap();
    let mut record = completed_no_spec_current_record(root);
    record.affected_specs = Vec::new();
    record.affected_paths = vec!["src/billing/charge.ts".into()];
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();

    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("approve must refuse production source no declared module owns");
    assert!(
        !error.contains("affected_specs 'billing'") && !error.contains("add `--spec billing`"),
        "declaring `billing` would not own the path, so it must not be recommended: {error}"
    );
    assert!(
        error.contains("`specs/legacy/charges.spec.md` lists it for module `billing`")
            && error.contains("canonical spec"),
        "the refusal says why the listing does not count: {error}"
    );
}

/// Codex P1 on #999: module names are validated only as portable path components, so a name may
/// carry spaces or shell syntax. The draft answer quotes every word it interpolates, so copying
/// it runs `change answer` with the module as one argument and runs nothing else.
#[test]
fn the_draft_answer_quotes_every_interpolated_word() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let module = "billing;touch PWN";
    ownership_spec(root, module, &["src/billing/charge.ts"]);
    fs::create_dir_all(root.join("src/billing")).unwrap();
    fs::write(root.join("src/billing/charge.ts"), "export const value = 1;\n").unwrap();
    let mut record = completed_no_spec_current_record(root);
    record.affected_specs = Vec::new();
    record.affected_paths = vec!["src/billing/charge.ts".into()];
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();

    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("approve must refuse production source no declared module owns");
    let line = error
        .lines()
        .find(|line| line.starts_with("On this draft"))
        .unwrap_or_else(|| panic!("no draft answer in: {error}"));
    assert_eq!(
        shell_argv_of_step(line),
        vec![vec![
            "change".to_string(),
            "answer".into(),
            record.id.clone(),
            "affected_specs".into(),
            module.into(),
        ]],
        "{line}"
    );
}

/// Codex P2 on #999: the ship preflight resolves listing specs once, not once per unowned path
/// (each walk rereads up to 10,000 spec-tree entries). Four delivered inputs, one walk.
#[test]
fn the_ownership_preflight_walks_the_spec_tree_once() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing", &["src/billing/charge.ts"]);
    fs::create_dir_all(root.join("src/billing")).unwrap();
    for file in ["charge", "a", "b", "c"] {
        fs::write(root.join(format!("src/billing/{file}.ts")), "const value = 1;\n").unwrap();
    }
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "billing"]);
    let mut record = completed_no_spec_current_record(root);
    record.affected_specs = Vec::new();
    record.affected_paths = vec!["src/billing/".into()];
    // A delivering change covers its scope; a draft covers nothing to sign.
    record.state = ChangeState::Verifying;
    save_change(root, &record).unwrap();

    reset_test_spec_listing_walks();
    let (blocker, _) = ownership_preflight_blocker(root, &record)
        .unwrap()
        .expect("four unowned inputs");
    assert!(blocker.contains("src/billing/c.ts"), "{blocker}");
    assert_eq!(test_spec_listing_walks(), 1);
}

// #1011 follow-ups from the #999 review rounds: recovery that cannot run as printed is
// never printed, the bounded listing says when it is incomplete, reserved delivery paths
// are not ownership blockers, registry-backed canonical specs join owner discovery, and a
// directory scope is not ownership-checked as a file.

/// Write an existing production source file: approve checks ownership only for
/// declared paths that already exist (REQ-change-053).
fn write_owned_source(root: &Path, path: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "export const value = 1;\n").unwrap();
}

fn approve_ownership_record(root: &Path, specs: &[&str], paths: &[&str]) -> ChangeRecord {
    let mut record = completed_no_spec_record(root);
    record.affected_specs = specs.iter().map(|spec| spec.to_string()).collect();
    record.affected_paths = paths.iter().map(|path| path.to_string()).collect();
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    record
}

#[test]
fn approve_no_spec_change_allows_reserved_delivery_paths() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\".\"]\n",
    )
    .unwrap();
    let script = root.join(".github/scripts/check.py");
    fs::create_dir_all(script.parent().unwrap()).unwrap();
    fs::write(&script, "print('ok')\n").unwrap();
    let record = approve_ownership_record(root, &[], &[".github/scripts/check.py"]);
    approve_definition(root, &record.id, Some("Reviewer".into()), None).expect(
        "finalize signs recognized delivery metadata with the exact delivery owner, so approve must not refuse it",
    );
}

#[test]
fn ownership_recovery_never_prints_a_command_for_a_comma_bearing_module() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing,legacy", &["src/billing/charge.ts"]);
    write_owned_source(root, "src/billing/charge.ts");
    let record = approve_ownership_record(root, &[], &["src/billing/charge.ts"]);
    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("the module is undeclared, so approve still refuses");
    assert!(
        error.contains("billing,legacy"),
        "the refusal still names the owning module in prose: {error}"
    );
    assert!(
        !error.contains("On this draft"),
        "`change answer` splits on commas, so no answer command can declare a comma-bearing module: {error}"
    );
}

#[test]
fn ownership_recovery_says_when_the_spec_listing_is_truncated() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "ledger", &["src/billing/ledger.ts"]);
    let filler = root.join("specs/zz-filler");
    fs::create_dir_all(&filler).unwrap();
    for index in 0..=10_000 {
        fs::write(filler.join(format!("filler-{index}.md")), "filler\n").unwrap();
    }
    write_owned_source(root, "src/billing/ledger.ts");
    let record = approve_ownership_record(root, &[], &["src/billing/ledger.ts"]);
    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("the module is undeclared, so approve still refuses");
    assert!(
        error.contains("10,000-entry walk bound"),
        "an incomplete listing must say so instead of omitting the owner: {error}"
    );
    assert!(
        !error.contains("On this draft"),
        "a truncated listing cannot promise the answer declares every owner: {error}"
    );
}

#[test]
fn ownership_recovery_renews_the_constraints_answer_it_clears() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "ledger", &["src/billing/ledger.ts"]);
    write_owned_source(root, "src/billing/ledger.ts");
    let mut record = approve_ownership_record(root, &[], &["src/billing/ledger.ts"]);
    record.answers.insert("constraints".into(), "none".into());
    save_change(root, &record).unwrap();
    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("the module is undeclared, so approve still refuses");
    assert!(
        error.contains("renew the constraints answer"),
        "answering affected_specs clears the constraints answer, so the recovery renews it: {error}"
    );
}

#[test]
fn owner_discovery_includes_registry_specs_outside_the_specs_dir() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let outside = root.join("vendor/registry-specs/ledger.spec.md");
    fs::create_dir_all(outside.parent().unwrap()).unwrap();
    fs::write(
        &outside,
        "---\nmodule: ledger\nversion: 1\nstatus: stable\nfiles:\n  - src/billing/ledger.ts\n---\n\n# Ledger\n\n## Purpose\n\nOwner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
    )
    .unwrap();
    fs::write(
        root.join(".specsync/registry.toml"),
        "name = \"proj\"\n\n[specs]\nledger = \"vendor/registry-specs/ledger.spec.md\"\n",
    )
    .unwrap();
    write_owned_source(root, "src/billing/ledger.ts");
    let record = approve_ownership_record(root, &[], &["src/billing/ledger.ts"]);
    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("the module is undeclared, so approve still refuses");
    assert!(
        error.contains("add `--spec ledger`"),
        "the registry-backed canonical spec owns the path, so the hint offers the declaration: {error}"
    );
    assert!(
        error.contains("affected_specs 'ledger'"),
        "the printed answer declares the registry-backed module: {error}"
    );
}

#[test]
fn approve_does_not_file_check_a_directory_scope_with_a_source_like_extension() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let directory = root.join("src/generated.rs");
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("README.md"), "generated docs\n").unwrap();
    let record = approve_ownership_record(root, &[], &["src/generated.rs"]);
    approve_definition(root, &record.id, Some("Reviewer".into()), None).expect(
        "a directory scope is walked for descendant files at finalize, not extension-checked at approve",
    );
}
