use std::path::Path;

#[derive(Debug, Clone)]
pub(super) struct HeadTip {
    pub(super) tip_class: String,
    pub(super) tip_sha: Option<String>,
    pub(super) parent_sha: Option<String>,
}

/// Classify HEAD the way CI tip lanes roughly do: archive_only, review_only, product, or other.
pub(super) fn classify_head_tip(root: &Path) -> Result<HeadTip, String> {
    let tip_sha = git_rev_parse(root, "HEAD").ok();
    let parent_sha = tip_sha
        .as_ref()
        .and_then(|_| git_rev_parse(root, "HEAD^").ok());

    let paths = if let Some(tip) = tip_sha.as_ref() {
        if let Some(parent) = parent_sha.as_ref() {
            git_diff_name_only(root, parent, tip).unwrap_or_default()
        } else {
            git_show_name_only(root, tip).unwrap_or_default()
        }
    } else {
        Vec::new()
    };

    // Dirty working tree paths also count as product when present.
    let mut dirty = git_status_name_only(root).unwrap_or_default();
    let mut all_paths = paths;
    all_paths.append(&mut dirty);
    all_paths.sort();
    all_paths.dedup();

    let tip_class = classify_paths(&all_paths).to_string();
    Ok(HeadTip {
        tip_class,
        tip_sha,
        parent_sha,
    })
}

pub(super) fn classify_paths(paths: &[String]) -> &'static str {
    if paths.is_empty() {
        return "other";
    }
    let mut has_archive = false;
    let mut has_review = false;
    let mut has_product = false;
    for path in paths {
        if path.starts_with(".specsync/archive/changes/") {
            has_archive = true;
            continue;
        }
        if path.starts_with(".specsync/changes/")
            && (path.ends_with("/review.json")
                || path.ends_with("/review-attempts.json")
                || path.ends_with("review.json")
                || path.ends_with("review-attempts.json"))
        {
            has_review = true;
            continue;
        }
        // Lifecycle metadata alone is not a full product tip.
        if path == ".specsync/change-sequence.json"
            || path.starts_with(".specsync/changes/")
                && (path.ends_with("/verification.json")
                    || path.ends_with("/verification-attempts.json")
                    || path.ends_with("/state.json")
                    || path.ends_with("/approvals.json"))
        {
            continue;
        }
        has_product = true;
    }
    if has_product {
        "product"
    } else if has_archive && !has_review {
        "archive_only"
    } else if has_review && !has_archive {
        "review_only"
    } else if has_archive && has_review {
        "product"
    } else {
        "other"
    }
}
pub(super) fn git_commit_present(root: &Path, commit: &str) -> Result<bool, String> {
    let output = std::process::Command::new("git")
        .args(["cat-file", "-e", &format!("{commit}^{{commit}}")])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git cat-file failed: {error}"))?;
    Ok(output.status.success())
}

pub(super) fn git_is_ancestor(
    root: &Path,
    maybe_ancestor: &str,
    tip: &str,
) -> Result<bool, String> {
    let output = std::process::Command::new("git")
        .args(["merge-base", "--is-ancestor", maybe_ancestor, tip])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git merge-base failed: {error}"))?;
    Ok(output.status.success())
}

pub(super) fn git_rev_parse(root: &Path, rev: &str) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--verify", rev])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git rev-parse failed: {error}"))?;
    if !output.status.success() {
        return Err(format!("git rev-parse {rev} failed"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_diff_name_only(root: &Path, from: &str, to: &str) -> Result<Vec<String>, String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", from, to])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git diff failed: {error}"))?;
    if !output.status.success() {
        return Err("git diff --name-only failed".into());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

fn git_show_name_only(root: &Path, commit: &str) -> Result<Vec<String>, String> {
    let output = std::process::Command::new("git")
        .args(["show", "--pretty=format:", "--name-only", commit])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git show failed: {error}"))?;
    if !output.status.success() {
        return Err("git show --name-only failed".into());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

fn git_status_name_only(root: &Path) -> Result<Vec<String>, String> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain", "-z"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git status failed: {error}"))?;
    if !output.status.success() {
        return Err("git status failed".into());
    }
    // Porcelain -z: XY path\0, renames have extra path.
    let mut paths = Vec::new();
    for entry in output.stdout.split(|byte| *byte == 0) {
        if entry.len() < 4 {
            continue;
        }
        let text = String::from_utf8_lossy(entry);
        // "XY path" or "R  old\0new" handled loosely: take path after first three bytes.
        let path = text.get(3..).unwrap_or("").trim();
        if !path.is_empty() {
            paths.push(path.to_string());
        }
    }
    Ok(paths)
}
