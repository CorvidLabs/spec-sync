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

/// #1028 review: `change answer affected_specs` also trims values and runs
/// `validate_module_name`, so a module name it would refuse gets prose, not an answer
/// command, and the `--spec` hint is quoted so a shell cannot read `|` as a pipe.
#[test]
fn ownership_recovery_never_prints_a_command_answer_would_refuse() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "billing|v2", &["src/billing/charge.ts"]);
    write_owned_source(root, "src/billing/charge.ts");
    let record = approve_ownership_record(root, &[], &["src/billing/charge.ts"]);
    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("the module is undeclared, so approve still refuses");
    assert!(
        !error.contains("On this draft"),
        "`change answer` refuses `billing|v2`, so no answer command may be printed: {error}"
    );
    assert!(
        error.contains("add `--spec 'billing|v2'`"),
        "the --spec hint is one quoted shell word: {error}"
    );
}

/// #1011: recognized reserved delivery metadata is signed with the exact delivery owner
/// at finalize, so `correct-owner --all-missing` never offers it as a missing owner,
/// while a real unowned production source still is.
#[test]
fn owner_discovery_skips_reserved_delivery_paths() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\".\"]\n",
    )
    .unwrap();
    ownership_spec(
        root,
        "tooling",
        &[".github/scripts/check.py", "src/tools/run.ts"],
    );
    write_owned_source(root, ".github/scripts/check.py");
    write_owned_source(root, "src/tools/run.ts");
    let mut record = approve_ownership_record(
        root,
        &[],
        &[".github/scripts/check.py", "src/tools/run.ts"],
    );
    record.state = ChangeState::Verifying;
    save_change(root, &record).unwrap();

    let missing = missing_acceptance_owner_paths(root, &record.id, "tooling").unwrap();
    assert_eq!(missing, vec!["src/tools/run.ts"]);
}

/// #1037 review: the changed set used to classify unowned inputs, in a real repository whose
/// project root is a subdirectory. Paths are root-relative (`--relative`), an untracked file and
/// both sides of a rename count, work committed before `change new` counts through the merge base
/// with the remote default branch, and a file outside the root never leaks in. Without a remote
/// default branch nothing is classified, so every input keeps the declare recovery.
#[test]
fn files_changed_since_base_is_root_relative_and_counts_pre_change_work() {
    let temp = TempDir::new().unwrap();
    let repo = temp.path().join("repo");
    let root = repo.join("proj");
    fs::create_dir_all(root.join("src")).unwrap();
    let git = |args: &[&str]| {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(&repo)
            .output()
            .unwrap();
        assert!(status.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&status.stderr));
        String::from_utf8_lossy(&status.stdout).trim().to_string()
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    for file in ["proj/src/early.ts", "proj/src/edited.ts", "proj/src/moved.ts", "proj/src/still.ts", "outside.ts"] {
        fs::write(repo.join(file), "const value = 1;\n").unwrap();
    }
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "seed"]);
    let remote = temp.path().join("remote.git");
    std::process::Command::new("git")
        .args(["init", "-q", "--bare", remote.to_str().unwrap()])
        .output()
        .unwrap();
    git(&["remote", "add", "origin", remote.to_str().unwrap()]);
    git(&["push", "-q", "origin", "main"]);
    git(&["remote", "set-head", "origin", "main"]);

    // Work committed before `change new`.
    fs::write(root.join("src/early.ts"), "const value = 2;\n").unwrap();
    git(&["commit", "-q", "-am", "early work"]);
    let mut record = completed_no_spec_record(&root);
    record.base_commit = Some(git(&["rev-parse", "HEAD"]));

    // Work after `change new`: an edit, an untracked file, a rename, and an edit outside the root.
    fs::write(root.join("src/edited.ts"), "const value = 3;\n").unwrap();
    fs::write(root.join("src/fresh ü.ts"), "const value = 4;\n").unwrap();
    git(&["mv", "proj/src/moved.ts", "proj/src/renamed.ts"]);
    fs::write(repo.join("outside.ts"), "const value = 5;\n").unwrap();

    let changed = files_changed_since_base(&root, &record).expect("a remote default branch exists");
    for path in ["src/early.ts", "src/edited.ts", "src/fresh ü.ts", "src/moved.ts", "src/renamed.ts"] {
        assert!(changed.contains(path), "{path} is modified: {changed:?}");
    }
    assert!(!changed.contains("src/still.ts"), "{changed:?}");
    assert!(
        !changed.iter().any(|path| path.contains("outside") || path.starts_with("proj/")),
        "paths are relative to the project root and stay inside it: {changed:?}"
    );

    // Without a remote default branch, nothing is classified.
    git(&["remote", "remove", "origin"]);
    assert!(files_changed_since_base(&root, &record).is_none());
}

/// The narrowing answer names every changed path inside the current scope (directory scopes
/// included, files outside the scope left out) so a deletion is not forgotten (#1037 review). It
/// falls back to the placeholder when nothing is classified, nothing in the scope changed, or a
/// path cannot round-trip through the comma-separated answer.
#[test]
fn the_narrowing_answer_names_every_changed_path_in_scope_or_falls_back_to_the_placeholder() {
    let temp = TempDir::new().unwrap();
    let mut record = completed_no_spec_record(temp.path());
    record.affected_paths = vec!["src/billing/".to_string(), "docs/notes.md".to_string()];
    let changed = |paths: &[&str]| paths.iter().map(|path| path.to_string()).collect::<BTreeSet<_>>();

    let answer = narrow_answer(
        &record,
        Some(&changed(&["docs/notes.md", "src/billing/charge.ts", "src/billing/old.ts", "src/other.ts"])),
    );
    assert!(
        answer.contains("affected_paths 'docs/notes.md,src/billing/charge.ts,src/billing/old.ts'"),
        "every in-scope path, a deleted one included, and nothing outside the scope: {answer}"
    );
    assert!(!answer.contains("<path>"), "{answer}");

    // `.` is the whole project, as finalize reads it.
    record.affected_paths = vec![".".to_string()];
    let answer = narrow_answer(&record, Some(&changed(&["docs/notes.md", "src/other.ts"])));
    assert!(answer.contains("affected_paths 'docs/notes.md,src/other.ts'"), "{answer}");
    record.affected_paths = vec!["src/billing/".to_string(), "docs/notes.md".to_string(), "-x/".to_string()];

    for (label, classified) in [
        ("nothing classified", None),
        ("nothing in scope changed", Some(changed(&["src/other.ts"]))),
        ("a comma-bearing path", Some(changed(&["src/billing/a,b.ts"]))),
        ("a backslash", Some(changed(&["src/billing/a\\b.ts"]))),
        ("a control character", Some(changed(&["src/billing/a\u{7}b.ts"]))),
        ("a list read as an option", Some(changed(&["-x/a.ts", "src/billing/b.ts"]))),
        ("a padded path", Some(changed(&["src/billing/padded.ts "]))),
    ] {
        let answer = narrow_answer(&record, classified.as_ref());
        assert!(answer.contains("affected_paths '<path>,<path>'"), "{label}: {answer}");
    }
}

/// #1026 review: a module that owns a path only through `[modules.X] owns` and has no spec on
/// disk is refused by `change answer affected_specs`, so the recovery scaffolds it first rather
/// than printing an answer that fails as printed.
#[test]
fn ownership_recovery_scaffolds_a_configured_owner_with_no_spec_before_declaring_it() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n\n[modules.core]\nowns = [\"src/core/\"]\n",
    )
    .unwrap();
    ownership_spec(root, "billing", &["src/billing/charge.ts"]);
    write_owned_source(root, "src/core/engine.ts");
    let record = approve_ownership_record(root, &[], &["src/core/engine.ts"]);
    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("the module is undeclared, so approve still refuses");
    assert!(
        !error.contains("On this draft"),
        "no answer is printed while the owner has no spec, since `change answer` would refuse it: {error}"
    );
    assert!(
        error.contains("which has no spec yet; create that module's spec, then declare it"),
        "the hint says to create the owner's spec first: {error}"
    );
}

/// #1026 third and sixth reviews: the printed answer re-lists every module, so an
/// already-declared module whose spec no longer exists blocks it too, and the refusal and the
/// preflight recovery name that module's missing spec instead of leaving it to the next step.
#[test]
fn ownership_recovery_names_every_listed_module_that_has_no_spec() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n\n[modules.core]\nowns = [\"src/core/\"]\n",
    )
    .unwrap();
    ownership_spec(root, "billing", &["src/billing/charge.ts"]);
    ownership_spec(root, "auth", &["src/auth/login.ts"]);
    write_owned_source(root, "src/core/engine.ts");
    let record = approve_ownership_record(root, &["auth"], &["src/core/engine.ts"]);
    // The declared module's spec disappears after it was declared.
    fs::remove_dir_all(root.join("specs/auth")).unwrap();
    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("the path's owner is undeclared, so approve refuses");
    assert!(
        !error.contains("affected_specs 'auth,core'") && !error.contains("On this draft"),
        "no answer is printed while a listed module (declared `auth` or new `core`) has no spec: {error}"
    );
    assert!(
        error.contains("The declared module `auth` has no spec at `specs/auth/auth.spec.md`"),
        "the refusal names the declared module whose spec is missing: {error}"
    );

    // The ship preflight's fallback recovery names it the same way.
    let mut verifying = load_change(root, &record.id).unwrap();
    verifying.state = ChangeState::Verifying;
    save_change(root, &verifying).unwrap();
    let (_, recovery) = ownership_preflight_blocker(root, &verifying)
        .unwrap()
        .expect("the configured owner `core` is still undeclared");
    assert!(
        recovery.contains("give each path an owning spec")
            && recovery.contains("The declared module `auth` has no spec at `specs/auth/auth.spec.md`"),
        "the fallback recovery names the missing declared spec: {recovery}"
    );
}

/// #1026 sixth review: in a project with no specs at all, `change answer` accepts any module
/// provisionally, but a declared module without a spec still fails verification. So the owner
/// hint and the printed answer agree there too: create the spec first, and no answer is printed.
#[test]
fn the_owner_hint_and_the_printed_answer_agree_in_a_project_with_no_specs() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n\n[modules.core]\nowns = [\"src/core/\"]\n",
    )
    .unwrap();
    write_owned_source(root, "src/core/engine.ts");
    let record = approve_ownership_record(root, &[], &["src/core/engine.ts"]);
    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("the configured owner is undeclared, so approve refuses");
    assert!(
        error.contains("which has no spec yet; create that module's spec, then declare it")
            && !error.contains("On this draft"),
        "the hint and the (absent) answer agree: {error}"
    );
}

/// #1026 sixth and seventh reviews: one predicate decides whether a module has its spec. A spec
/// reached through a symlink is refused naming the symlink, as verification refuses it, and a
/// case variant that a case-insensitive filesystem would resolve does not count.
#[cfg(unix)]
#[test]
fn a_spec_behind_a_symlink_or_a_case_variant_is_not_the_modules_spec() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    ownership_spec(root, "core", &["src/core/engine.ts"]);
    let shared = root.join("shared/pay");
    fs::create_dir_all(&shared).unwrap();
    fs::write(
        shared.join("pay.spec.md"),
        "---\nmodule: pay\nversion: 1\nstatus: draft\nfiles: []\n---\n\n# Pay\n",
    )
    .unwrap();
    std::os::unix::fs::symlink(&shared, root.join("specs/pay")).unwrap();

    assert!(module_has_canonical_spec(root, "specs", "core"));
    assert!(!module_has_canonical_spec(root, "specs", "pay"), "verification refuses a symlinked spec");
    assert!(!module_has_canonical_spec(root, "specs", "Core"), "a case variant never counts");
    assert!(!module_has_canonical_spec(root, "specs", "ghost"));
    let error = ensure_affected_specs_exist(root, &["pay".to_string()])
        .expect_err("the symlinked spec is refused");
    assert!(
        error.contains("is reached through a symlink at `specs/pay/pay.spec.md`"),
        "{error}"
    );
    let error = ensure_affected_specs_exist(root, &["Core".to_string()])
        .expect_err("a case variant is refused");
    assert!(error.contains("so name it as `core`"), "{error}");
    let error = ensure_affected_specs_exist(root, &["None".to_string()])
        .expect_err("`None` is a module name, not the clearing keyword");
    assert!(
        !error.contains("to clear the list"),
        "the clearing hint belongs to `change answer`, not to `change new --spec`: {error}"
    );
}

/// #1026 seventh review: only the module's own directory and file are held to an exact spelling.
/// A specs directory whose case differs from the config, and a module name in another Unicode
/// normalization form, are taken as the disk resolves them. Each case runs only where the disk
/// resolves the variant (a case-insensitive or normalization-insensitive filesystem, as on macOS).
#[test]
fn a_spelling_difference_above_the_module_or_in_normalization_is_not_a_case_variant() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::create_dir_all(root.join("Specs/auth")).unwrap();
    fs::write(
        root.join("Specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles: []\n---\n\n# Auth\n",
    )
    .unwrap();
    if root.join("specs/auth/auth.spec.md").is_file() {
        assert!(module_has_canonical_spec(root, "specs", "auth"));
        ensure_affected_specs_exist(root, &["auth".to_string()])
            .expect("the config's `specs` resolves to `Specs`, and the module is spelled exactly");
    }
    let decomposed = "cafe\u{301}";
    let composed = "caf\u{e9}";
    fs::create_dir_all(root.join(format!("Specs/{decomposed}"))).unwrap();
    fs::write(
        root.join(format!("Specs/{decomposed}/{decomposed}.spec.md")),
        format!("---\nmodule: {decomposed}\nversion: 1\nstatus: draft\nfiles: []\n---\n\n# Cafe\n"),
    )
    .unwrap();
    if root.join(format!("specs/{composed}/{composed}.spec.md")).is_file() {
        assert!(
            module_has_canonical_spec(root, "specs", composed),
            "a normalization difference is not an ASCII case variant"
        );
    }
}

/// #1026 sixth review: the actor rule refuses every format character (Cf) and default-ignorable
/// code point, the first and last of each range included, and keeps ZWNJ, ZWJ and ordinary
/// non-ASCII names.
#[test]
fn the_actor_rule_refuses_every_invisible_character_but_zwnj_and_zwj() {
    let refused = [
        '\u{0000}', '\u{001F}', '\u{007F}', '\u{009F}', '\u{00AD}', '\u{034F}', '\u{0600}',
        '\u{0605}', '\u{061C}', '\u{06DD}', '\u{070F}', '\u{0890}', '\u{0891}', '\u{08E2}',
        '\u{115F}', '\u{1160}', '\u{17B4}', '\u{17B5}', '\u{180B}', '\u{180F}', '\u{200B}',
        '\u{200E}', '\u{200F}', '\u{2028}', '\u{2029}', '\u{202A}', '\u{202E}', '\u{2060}',
        '\u{2065}', '\u{206F}', '\u{3164}', '\u{FE00}', '\u{FE0F}', '\u{FEFF}', '\u{FFA0}',
        '\u{FFF0}', '\u{FFFB}', '\u{110BD}', '\u{110CD}', '\u{13430}', '\u{1343F}',
        '\u{1BCA0}', '\u{1BCA3}', '\u{1D173}', '\u{1D17A}', '\u{E0000}', '\u{E0001}',
        '\u{E0041}', '\u{E007F}', '\u{E0100}', '\u{E01EF}', '\u{E0FFF}',
    ];
    for c in refused {
        let actor = format!("Ada{c}X");
        let error = normalize_actor(&actor, "approval").expect_err(&format!("U+{:04X}", c as u32));
        assert!(error.contains("single line of visible text"), "U+{:04X}: {error}", c as u32);
    }
    for actor in [
        "Ada\u{200C}X",
        "Ada\u{200D}X",
        "Zo\u{00EB} N\u{00FA}\u{00F1}ez",
        "\u{674E}\u{96F7}",
        "Ada \u{1F600}",
        "O'Brien-Smith",
        "\u{0645}\u{064A}\u{200C}\u{0631}",
    ] {
        assert_eq!(normalize_actor(&format!("  {actor} "), "approval").as_deref(), Ok(actor));
    }
}

/// #1026 with #1037's split: when no declare answer can be printed because a declared module has
/// no spec, the mixed declare-and-narrow recovery gives the modified inputs the same owning-spec
/// guidance as the declare-only recovery, and names the declared module whose spec is missing.
#[test]
fn the_mixed_recovery_names_a_declared_module_whose_spec_is_missing() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("work");
    fs::create_dir_all(&root).unwrap();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(&root);
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n\n[modules.core]\nowns = [\"src/core/\"]\n",
    )
    .unwrap();
    ownership_spec(&root, "auth", &["src/auth/login.ts"]);
    ownership_spec(&root, "billing", &["src/billing/charge.ts"]);
    for path in ["src/auth/login.ts", "src/core/engine.ts", "src/billing/charge.ts"] {
        write_owned_source(&root, path);
    }
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "seed"]);
    let remote = temp.path().join("remote.git");
    let output = std::process::Command::new("git")
        .args(["init", "-q", "--bare", "--initial-branch=main"])
        .arg(&remote)
        .output()
        .unwrap();
    assert!(output.status.success());
    git(&["remote", "add", "origin", remote.to_str().unwrap()]);
    git(&["push", "-q", "origin", "main"]);
    git(&["remote", "set-head", "origin", "main"]);

    let mut record = approve_ownership_record(
        &root,
        &["auth"],
        &["src/core/engine.ts", "src/billing/charge.ts"],
    );
    record.base_commit = Some(git(&["rev-parse", "HEAD"]));
    record.state = ChangeState::Verifying;
    save_change(&root, &record).unwrap();
    // The change edits engine.ts (owned by the undeclared `core`) and never touches charge.ts
    // (owned by the undeclared `billing`); the declared `auth` then loses its spec.
    fs::write(root.join("src/core/engine.ts"), "export const value = 2;\n").unwrap();
    fs::remove_dir_all(root.join("specs/auth")).unwrap();

    let (_, recovery) = ownership_preflight_blocker(&root, &record)
        .unwrap()
        .expect("core and billing are undeclared owners");
    assert!(
        recovery.contains("send it back once and rescope")
            && recovery.contains("give src/core/engine.ts an owning spec")
            && recovery
                .contains("The declared module `auth` has no spec at `specs/auth/auth.spec.md`"),
        "the modified input gets owning-spec guidance and the missing spec is named: {recovery}"
    );
    assert!(
        !recovery.contains("affected_specs '"),
        "no affected_specs answer is printed while a listed module has no spec: {recovery}"
    );
}
