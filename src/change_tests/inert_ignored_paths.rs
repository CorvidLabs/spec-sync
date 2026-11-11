// Issue #676: an `ignored_paths` entry whose whole coverage is always meaningful — the specs
// tree, or an SDD policy file named directly — is inert, and the policy loader says so where
// the conflict exists instead of three verbs later at audit. An entry that also covers other
// files does real work and stays silent.

#[test]
fn a_specs_tree_entry_is_inert() {
    let policy = SddPolicy {
        ignored_paths: vec!["specs/".into()],
        ..Default::default()
    };
    assert_eq!(inert_ignored_paths(&policy, "specs"), vec!["specs/"]);
}

#[test]
fn a_nested_specs_entry_is_inert() {
    let policy = SddPolicy {
        ignored_paths: vec!["specs/auth/".into()],
        ..Default::default()
    };
    assert_eq!(inert_ignored_paths(&policy, "specs"), vec!["specs/auth/"]);
}

#[test]
fn a_directly_named_policy_file_is_inert() {
    let policy = SddPolicy {
        ignored_paths: vec![".specsync/sdd.json".into()],
        ..Default::default()
    };
    assert_eq!(
        inert_ignored_paths(&policy, "specs"),
        vec![".specsync/sdd.json"]
    );
}

#[test]
fn the_dot_specsync_prefix_is_not_inert() {
    // It covers the change workspaces, where ignoring does real work.
    let policy = SddPolicy {
        ignored_paths: vec![".specsync/".into()],
        ..Default::default()
    };
    assert!(inert_ignored_paths(&policy, "specs").is_empty());
}

#[test]
fn an_unrelated_entry_is_not_inert() {
    let policy = SddPolicy {
        ignored_paths: vec!["target/".into(), "specs2/".into()],
        ..Default::default()
    };
    assert!(inert_ignored_paths(&policy, "specs").is_empty());
}

#[test]
fn a_custom_specs_dir_is_respected() {
    let policy = SddPolicy {
        ignored_paths: vec!["contracts/".into()],
        ..Default::default()
    };
    assert_eq!(inert_ignored_paths(&policy, "contracts"), vec!["contracts/"]);
    assert!(inert_ignored_paths(&policy, "specs").is_empty());
}

#[test]
fn the_default_policy_writes_no_inert_entry() {
    assert!(inert_ignored_paths(&SddPolicy::default(), "specs").is_empty());
}
