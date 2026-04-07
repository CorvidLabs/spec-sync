//! GitHub integration for linking specs to issues.
//!
//! Uses the `gh` CLI for authenticated API calls. Falls back to `GITHUB_TOKEN`
//! with `ureq` if `gh` is not available.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// A GitHub issue's relevant fields.
#[derive(Debug, Clone)]
pub struct GitHubIssue {
    pub number: u64,
    pub title: String,
    pub state: String, // "open" or "closed"
    #[allow(dead_code)]
    pub labels: Vec<String>,
    pub url: String,
}

/// Result of verifying issue references from spec frontmatter.
#[derive(Debug)]
pub struct IssueVerification {
    #[allow(dead_code)]
    pub spec_path: String,
    pub valid: Vec<GitHubIssue>,
    pub closed: Vec<GitHubIssue>,
    pub not_found: Vec<u64>,
    pub errors: Vec<String>,
}

/// Auto-detect the GitHub repository from git remote origin.
pub fn detect_repo(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_repo_from_url(&url)
}

/// Parse `owner/repo` from a git remote URL.
fn parse_repo_from_url(url: &str) -> Option<String> {
    // SSH: git@github.com:owner/repo.git
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        let repo = rest.strip_suffix(".git").unwrap_or(rest);
        return Some(repo.to_string());
    }
    // HTTPS: https://github.com/owner/repo.git
    if let Some(rest) = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
    {
        let repo = rest.strip_suffix(".git").unwrap_or(rest);
        return Some(repo.to_string());
    }
    None
}

/// Resolve the effective repo: explicit config > auto-detect from git.
pub fn resolve_repo(config_repo: Option<&str>, root: &Path) -> Result<String, String> {
    if let Some(repo) = config_repo {
        return Ok(repo.to_string());
    }
    detect_repo(root).ok_or_else(|| {
        "Cannot determine GitHub repo. Set `github.repo` in specsync.json or ensure a git remote is configured.".to_string()
    })
}

/// Check if the `gh` CLI is available and authenticated.
pub fn gh_is_available() -> bool {
    Command::new("gh")
        .args(["auth", "status"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Fetch a single issue using `gh` CLI.
pub fn fetch_issue_gh(repo: &str, number: u64) -> Result<GitHubIssue, String> {
    let output = Command::new("gh")
        .args([
            "issue",
            "view",
            &number.to_string(),
            "--repo",
            repo,
            "--json",
            "number,title,state,labels,url",
        ])
        .output()
        .map_err(|e| format!("Failed to run gh: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("Could not resolve") {
            return Err(format!("Issue #{number} not found in {repo}"));
        }
        return Err(format!("gh error: {}", stderr.trim()));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse gh output: {e}"))?;

    Ok(GitHubIssue {
        number,
        title: json["title"].as_str().unwrap_or("").to_string(),
        state: json["state"].as_str().unwrap_or("OPEN").to_lowercase(),
        labels: json["labels"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| l["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        url: json["url"].as_str().unwrap_or("").to_string(),
    })
}

/// Fetch a single issue using the GitHub REST API with GITHUB_TOKEN.
pub fn fetch_issue_api(repo: &str, number: u64) -> Result<GitHubIssue, String> {
    let token = std::env::var("GITHUB_TOKEN")
        .map_err(|_| "GITHUB_TOKEN not set and gh CLI not available".to_string())?;

    let url = format!("https://api.github.com/repos/{repo}/issues/{number}");

    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .build(),
    );

    let mut response = agent
        .get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "specsync")
        .call()
        .map_err(|e| format!("GitHub API request failed: {e}"))?;

    if response.status() == 404 {
        return Err(format!("Issue #{number} not found in {repo}"));
    }
    if response.status() != 200 {
        return Err(format!("GitHub API returned HTTP {}", response.status()));
    }

    let body: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse GitHub API response: {e}"))?;

    Ok(GitHubIssue {
        number,
        title: body["title"].as_str().unwrap_or("").to_string(),
        state: body["state"].as_str().unwrap_or("open").to_string(),
        labels: body["labels"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| l["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        url: body["html_url"].as_str().unwrap_or("").to_string(),
    })
}

/// Fetch a single issue, trying `gh` CLI first, falling back to API.
pub fn fetch_issue(repo: &str, number: u64) -> Result<GitHubIssue, String> {
    if gh_is_available() {
        fetch_issue_gh(repo, number)
    } else {
        fetch_issue_api(repo, number)
    }
}

/// Verify all issue references from a spec's frontmatter.
pub fn verify_spec_issues(
    repo: &str,
    spec_path: &str,
    implements: &[u64],
    tracks: &[u64],
) -> IssueVerification {
    let mut result = IssueVerification {
        spec_path: spec_path.to_string(),
        valid: Vec::new(),
        closed: Vec::new(),
        not_found: Vec::new(),
        errors: Vec::new(),
    };

    let all_issues: Vec<u64> = implements.iter().chain(tracks.iter()).copied().collect();

    for number in all_issues {
        match fetch_issue(repo, number) {
            Ok(issue) => {
                if issue.state == "closed" {
                    result.closed.push(issue);
                } else {
                    result.valid.push(issue);
                }
            }
            Err(e) => {
                if e.contains("not found") {
                    result.not_found.push(number);
                } else {
                    result.errors.push(format!("#{number}: {e}"));
                }
            }
        }
    }

    result
}

/// Create a GitHub issue for spec drift using `gh` CLI.
pub fn create_drift_issue(
    repo: &str,
    spec_path: &str,
    errors: &[String],
    labels: &[String],
) -> Result<GitHubIssue, String> {
    if !gh_is_available() {
        return Err("gh CLI is required to create issues".to_string());
    }

    let title = format!("Spec drift detected: {spec_path}");
    let body = format!(
        "## Spec Drift Detected\n\n\
         **Spec:** `{spec_path}`\n\n\
         ### Validation Errors\n\n{}\n\n\
         ---\n\
         *Auto-created by `specsync check --create-issues`*",
        errors
            .iter()
            .map(|e| format!("- {e}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let mut args = vec![
        "issue", "create", "--repo", repo, "--title", &title, "--body", &body,
    ];

    let label_str = labels.join(",");
    if !labels.is_empty() {
        args.push("--label");
        args.push(&label_str);
    }

    let output = Command::new("gh")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run gh: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create issue: {}", stderr.trim()));
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // Extract issue number from URL (last path segment)
    let number = url
        .rsplit('/')
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    Ok(GitHubIssue {
        number,
        title,
        state: "open".to_string(),
        labels: labels.to_vec(),
        url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo_from_url_https() {
        assert_eq!(
            parse_repo_from_url("https://github.com/CorvidLabs/spec-sync.git"),
            Some("CorvidLabs/spec-sync".to_string())
        );
        assert_eq!(
            parse_repo_from_url("https://github.com/CorvidLabs/spec-sync"),
            Some("CorvidLabs/spec-sync".to_string())
        );
    }

    #[test]
    fn test_parse_repo_from_url_ssh() {
        assert_eq!(
            parse_repo_from_url("git@github.com:CorvidLabs/spec-sync.git"),
            Some("CorvidLabs/spec-sync".to_string())
        );
    }

    #[test]
    fn test_parse_repo_from_url_unknown() {
        assert_eq!(parse_repo_from_url("https://gitlab.com/foo/bar.git"), None);
    }
}
