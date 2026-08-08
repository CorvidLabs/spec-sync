//! GitHub integration for linking specs to issues.
//!
//! Read-only issue operations use in-process GitHub REST requests with an
//! explicit `GITHUB_TOKEN`. The `gh` CLI remains limited to explicit write
//! operations because portable subprocess containment cannot prevent every
//! Unix descendant from escaping with `setsid`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

const GITHUB_COMMAND_DEADLINE: Duration = Duration::from_secs(10);
const GITHUB_VERIFICATION_DEADLINE: Duration = Duration::from_secs(30);
const MAX_VERIFIED_ISSUES: usize = 100;
const MAX_ISSUE_LIST_PAGE_ENTRIES: usize = 100;
const MAX_ISSUE_LIST_PAGES: usize = 100;

#[derive(Debug, Clone)]
enum IssueFetchError {
    NotFound,
    Provider(String),
}

impl std::fmt::Display for IssueFetchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => formatter.write_str("issue not found"),
            Self::Provider(message) => formatter.write_str(message),
        }
    }
}

trait IssueBatchBackend {
    fn prepare(
        &mut self,
        repo: &str,
        remaining: &mut dyn FnMut() -> Result<Duration, String>,
    ) -> Result<(), String>;

    fn fetch(
        &mut self,
        repo: &str,
        number: u64,
        deadline: Duration,
        remaining: &mut dyn FnMut() -> Result<Duration, String>,
    ) -> Result<GitHubIssue, IssueFetchError>;
}

#[derive(Default)]
struct SystemIssueBatchBackend {
    token: Option<String>,
}

impl IssueBatchBackend for SystemIssueBatchBackend {
    fn prepare(
        &mut self,
        repo: &str,
        remaining: &mut dyn FnMut() -> Result<Duration, String>,
    ) -> Result<(), String> {
        remaining()?;
        let token = std::env::var("GITHUB_TOKEN").map_err(|_| {
            "GITHUB_TOKEN is required for hardened issue verification; gh CLI credentials are not executed because portable complete-tree subprocess containment is unavailable"
                .to_string()
        })?;
        let repository_deadline = remaining()?.min(GITHUB_COMMAND_DEADLINE);
        verify_api_repository(repo, &token, repository_deadline)?;
        self.token = Some(token);
        Ok(())
    }

    fn fetch(
        &mut self,
        repo: &str,
        number: u64,
        deadline: Duration,
        remaining: &mut dyn FnMut() -> Result<Duration, String>,
    ) -> Result<GitHubIssue, IssueFetchError> {
        let operation_started = Instant::now();
        let issue = match self.token.as_deref() {
            Some(token) => {
                fetch_issue_api_typed(repo, number, token, deadline).map(|details| details.issue)
            }
            None => Err(IssueFetchError::Provider(
                "GitHub issue provider was not prepared".to_string(),
            )),
        };
        revalidate_issue_not_found(repo, issue, || {
            let operation_remaining = deadline
                .checked_sub(operation_started.elapsed())
                .filter(|remaining| !remaining.is_zero())
                .ok_or_else(|| {
                    "GitHub repository recheck exceeded the issue operation deadline".to_string()
                })?;
            let invocation_remaining = remaining()?;
            let recheck_deadline = operation_remaining.min(invocation_remaining);
            match self.token.as_deref() {
                Some(token) => verify_api_repository(repo, token, recheck_deadline).map(|_| ()),
                None => Err("GitHub issue provider was not prepared".to_string()),
            }
        })
    }
}

fn revalidate_issue_not_found<Issue>(
    repo: &str,
    issue: Result<Issue, IssueFetchError>,
    repository_check: impl FnOnce() -> Result<(), String>,
) -> Result<Issue, IssueFetchError> {
    if !matches!(&issue, Err(IssueFetchError::NotFound)) {
        return issue;
    }
    match repository_check() {
        Ok(()) => Err(IssueFetchError::NotFound),
        Err(error) => Err(IssueFetchError::Provider(format!(
            "Issue lookup was inconclusive because repository access to {repo} could not be revalidated: {error}"
        ))),
    }
}

/// Redact a token from an error message before surfacing it.
///
/// The token is sent in the `Authorization` header, not the URL, so HTTP errors
/// should never contain it — this strips any verbatim occurrence as
/// defense-in-depth in case a proxy or future change echoes it back.
fn redact_token(message: String, token: &str) -> String {
    if !token.is_empty() && message.contains(token) {
        message.replace(token, "[REDACTED]")
    } else {
        message
    }
}

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

/// Full issue content used by importers after the common REST validation path.
#[derive(Debug, Clone)]
pub(crate) struct GitHubIssueDetails {
    pub issue: GitHubIssue,
    pub body: String,
}

#[derive(serde::Deserialize)]
struct GitHubIssuePayload {
    number: u64,
    title: String,
    state: String,
    labels: Vec<GitHubLabelPayload>,
    html_url: String,
    body: Option<String>,
}

#[derive(serde::Deserialize)]
struct GitHubLabelPayload {
    name: String,
}

#[derive(serde::Deserialize)]
struct GitHubRepositoryPayload {
    id: u64,
    full_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GitHubRepositoryIdentity {
    id: u64,
}

#[derive(Debug)]
struct ParsedGitHubIssueListPage {
    raw_numbers: Vec<u64>,
    issues: Vec<GitHubIssue>,
}

struct GitHubIssueListPage {
    raw_numbers: Vec<u64>,
    issues: Vec<GitHubIssue>,
    next_target: Option<String>,
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
    let repo = match config_repo {
        Some(repo) => repo.to_string(),
        None => detect_repo(root).ok_or_else(|| {
            "Cannot determine GitHub repo. Set `github.repo` in specsync.json or ensure a git remote is configured.".to_string()
        })?,
    };
    validate_repo(&repo)?;
    Ok(repo)
}

/// Check if the `gh` CLI is available and authenticated.
pub fn gh_is_available() -> bool {
    Command::new("gh")
        .args(["auth", "status"])
        .status()
        .is_ok_and(|status| status.success())
}

/// Reject legacy `gh` CLI issue reads because portable complete-tree
/// containment cannot prevent `setsid` escapes on every supported Unix host.
#[allow(dead_code)]
pub fn fetch_issue_gh(_repo: &str, _number: u64) -> Result<GitHubIssue, String> {
    Err(
        "gh CLI issue reads are disabled because portable complete-tree subprocess containment is unavailable; set GITHUB_TOKEN for in-process GitHub REST access"
            .to_string(),
    )
}

/// Fetch a single issue using the GitHub REST API with GITHUB_TOKEN.
#[allow(dead_code)]
pub fn fetch_issue_api(repo: &str, number: u64) -> Result<GitHubIssue, String> {
    fetch_issue_details(repo, number).map(|details| details.issue)
}

pub(crate) fn fetch_issue_details(repo: &str, number: u64) -> Result<GitHubIssueDetails, String> {
    let token = std::env::var("GITHUB_TOKEN")
        .map_err(|_| "GITHUB_TOKEN is required for in-process GitHub REST access".to_string())?;
    verify_api_repository(repo, &token, GITHUB_COMMAND_DEADLINE)?;
    let operation_started = Instant::now();
    let issue = fetch_issue_api_typed(repo, number, &token, GITHUB_COMMAND_DEADLINE);
    revalidate_issue_not_found(repo, issue, || {
        let recheck_deadline = GITHUB_COMMAND_DEADLINE
            .checked_sub(operation_started.elapsed())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(|| {
                "GitHub repository recheck exceeded the issue operation deadline".to_string()
            })?;
        verify_api_repository(repo, &token, recheck_deadline).map(|_| ())
    })
    .map_err(|error| issue_fetch_error_message(repo, number, error))
}

/// Fetch a single issue through in-process GitHub REST access.
#[allow(dead_code)]
pub fn fetch_issue(repo: &str, number: u64) -> Result<GitHubIssue, String> {
    fetch_issue_api(repo, number)
}

#[allow(dead_code)]
fn issue_fetch_error_message(repo: &str, number: u64, error: IssueFetchError) -> String {
    match error {
        IssueFetchError::NotFound => format!("Issue #{number} not found in {repo}"),
        IssueFetchError::Provider(message) => message,
    }
}

fn github_api_agent(deadline: Duration) -> ureq::Agent {
    ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(deadline))
            .http_status_as_error(false)
            .build(),
    )
}

fn github_api_base() -> String {
    #[cfg(test)]
    if let Ok(base) = std::env::var("SPECSYNC_TEST_GITHUB_API_BASE") {
        return base.trim_end_matches('/').to_string();
    }
    "https://api.github.com".to_string()
}

fn verify_api_repository(
    repo: &str,
    token: &str,
    deadline: Duration,
) -> Result<GitHubRepositoryIdentity, String> {
    validate_repo(repo)?;
    let url = format!("{}/repos/{repo}", github_api_base());
    let mut response = github_api_agent(deadline)
        .get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "specsync")
        .call()
        .map_err(|error| {
            redact_token(
                format!("GitHub repository access failed for {repo}: {error}"),
                token,
            )
        })?;
    if response.status() != 200 {
        return Err(format!(
            "GitHub repository access is inconclusive for {repo}: HTTP {}",
            response.status()
        ));
    }
    let body: GitHubRepositoryPayload = response
        .body_mut()
        .read_json()
        .map_err(|error| format!("Failed to parse GitHub repository response: {error}"))?;
    if !body.full_name.eq_ignore_ascii_case(repo) {
        return Err(format!(
            "GitHub repository response did not confirm the requested repository {repo}"
        ));
    }
    if body.id == 0 {
        return Err(format!(
            "GitHub repository response contained an invalid repository ID for {repo}"
        ));
    }
    Ok(GitHubRepositoryIdentity { id: body.id })
}

fn fetch_issue_api_typed(
    repo: &str,
    number: u64,
    token: &str,
    deadline: Duration,
) -> Result<GitHubIssueDetails, IssueFetchError> {
    let url = format!("{}/repos/{repo}/issues/{number}", github_api_base());
    let mut response = github_api_agent(deadline)
        .get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "specsync")
        .call()
        .map_err(|error| {
            IssueFetchError::Provider(redact_token(
                format!("GitHub API request failed: {error}"),
                token,
            ))
        })?;
    if response.status() == 404 {
        return Err(IssueFetchError::NotFound);
    }
    if response.status() != 200 {
        return Err(IssueFetchError::Provider(format!(
            "GitHub API returned HTTP {}",
            response.status()
        )));
    }
    let body: serde_json::Value = response.body_mut().read_json().map_err(|error| {
        IssueFetchError::Provider(format!("Failed to parse GitHub API response: {error}"))
    })?;
    parse_issue_details_json(repo, number, &body).map_err(IssueFetchError::Provider)
}

#[cfg(test)]
fn parse_issue_json(
    repo: &str,
    requested_number: u64,
    json: &serde_json::Value,
) -> Result<GitHubIssue, String> {
    let payload: GitHubIssuePayload = serde_json::from_value(json.clone())
        .map_err(|error| format!("Malformed GitHub issue response: {error}"))?;
    issue_from_payload(repo, requested_number, payload)
}

fn issue_from_payload(
    repo: &str,
    requested_number: u64,
    payload: GitHubIssuePayload,
) -> Result<GitHubIssue, String> {
    issue_from_payload_for_resource(
        repo,
        requested_number,
        payload,
        "issues",
        "GitHub issue response",
    )
}

fn issue_from_payload_for_resource(
    repo: &str,
    requested_number: u64,
    payload: GitHubIssuePayload,
    resource: &str,
    response_name: &str,
) -> Result<GitHubIssue, String> {
    if payload.number != requested_number {
        return Err(format!(
            "{response_name} returned #{} for requested issue #{requested_number}",
            payload.number
        ));
    }
    let state = payload.state.to_ascii_lowercase();
    if state != "open" && state != "closed" {
        return Err(format!("{response_name} has an invalid state"));
    }
    if payload.title.trim().is_empty() {
        return Err(format!("{response_name} has an empty `title`"));
    }
    if !is_valid_github_item_url(&payload.html_url, repo, resource, requested_number) {
        return Err(format!("{response_name} has an invalid `html_url`"));
    }
    if payload
        .labels
        .iter()
        .any(|label| label.name.trim().is_empty())
    {
        return Err(format!("{response_name} contains an empty label name"));
    }
    Ok(GitHubIssue {
        number: payload.number,
        title: payload.title,
        state,
        labels: payload.labels.into_iter().map(|label| label.name).collect(),
        url: payload.html_url,
    })
}

fn parse_issue_details_json(
    repo: &str,
    requested_number: u64,
    json: &serde_json::Value,
) -> Result<GitHubIssueDetails, String> {
    if json.get("pull_request").is_some() {
        return Err("GitHub issue response identifies a pull request".to_string());
    }
    if !json
        .get("body")
        .is_some_and(|body| body.is_null() || body.is_string())
    {
        return Err("GitHub issue response is missing a string or null `body`".to_string());
    }
    let payload: GitHubIssuePayload = serde_json::from_value(json.clone())
        .map_err(|error| format!("Malformed GitHub issue response: {error}"))?;
    let body = payload.body.clone().unwrap_or_default();
    let issue = issue_from_payload(repo, requested_number, payload)?;
    Ok(GitHubIssueDetails { issue, body })
}

fn is_valid_github_item_url(url: &str, repo: &str, resource: &str, number: u64) -> bool {
    let Some(path) = url.strip_prefix("https://github.com/") else {
        return false;
    };
    let Some((expected_owner, expected_repository)) = repo.split_once('/') else {
        return false;
    };
    let canonical_number = number.to_string();
    let mut segments = path.split('/');
    matches!(segments.next(), Some(owner) if owner.eq_ignore_ascii_case(expected_owner))
        && matches!(segments.next(), Some(repository) if repository.eq_ignore_ascii_case(expected_repository))
        && segments.next() == Some(resource)
        && segments.next() == Some(canonical_number.as_str())
        && segments.next().is_none()
}

fn validate_repo(repo: &str) -> Result<(), String> {
    let Some((owner, repository)) = repo.split_once('/') else {
        return Err("GitHub repository must use the `owner/repository` form".to_string());
    };
    let valid_component = |component: &str| {
        !component.is_empty()
            && component != "."
            && component != ".."
            && component
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character))
    };
    if !valid_component(owner) || !valid_component(repository) {
        return Err(
            "GitHub repository contains invalid owner or repository characters".to_string(),
        );
    }
    Ok(())
}

/// Verify all issue references from a spec's frontmatter.
#[allow(dead_code)]
pub fn verify_spec_issues(
    repo: &str,
    spec_path: &str,
    implements: &[u64],
    tracks: &[u64],
) -> IssueVerification {
    let references = vec![(spec_path.to_string(), implements.to_vec(), tracks.to_vec())];
    verify_issue_batch(repo, &references)
        .into_iter()
        .next()
        .unwrap_or_else(|| empty_issue_verification(spec_path))
}

pub(crate) fn verify_issue_batch(
    repo: &str,
    references: &[(String, Vec<u64>, Vec<u64>)],
) -> Vec<IssueVerification> {
    let started = Instant::now();
    let mut remaining = || remaining_verification_time(started);
    let mut backend = SystemIssueBatchBackend::default();
    verify_issue_batch_with_backend(repo, references, &mut backend, &mut remaining)
}

fn verify_issue_batch_with_backend(
    repo: &str,
    references: &[(String, Vec<u64>, Vec<u64>)],
    backend: &mut dyn IssueBatchBackend,
    remaining: &mut dyn FnMut() -> Result<Duration, String>,
) -> Vec<IssueVerification> {
    let mut results: Vec<IssueVerification> = references
        .iter()
        .map(|(spec_path, _, _)| empty_issue_verification(spec_path))
        .collect();
    let all_issues = match bounded_batch_issue_numbers(references) {
        Ok(issues) => issues,
        Err(error) => {
            for result in &mut results {
                result.errors.push(error.clone());
            }
            return results;
        }
    };
    if all_issues.is_empty() {
        return results;
    }

    if let Err(error) = backend.prepare(repo, remaining) {
        for result in &mut results {
            result.errors.push(error.clone());
        }
        return results;
    }

    let mut fetched = BTreeMap::new();
    for number in all_issues {
        let invocation_remaining = match remaining() {
            Ok(duration) => duration,
            Err(error) => {
                fetched.insert(number, Err(IssueFetchError::Provider(error)));
                break;
            }
        };
        let request_deadline = invocation_remaining.min(GITHUB_COMMAND_DEADLINE);
        let issue = backend.fetch(repo, number, request_deadline, remaining);
        fetched.insert(number, issue);
    }

    for (result, (_, implements, tracks)) in results.iter_mut().zip(references) {
        let issues: BTreeSet<u64> = implements.iter().chain(tracks.iter()).copied().collect();
        for number in issues {
            match fetched.get(&number) {
                Some(Ok(issue)) if issue.state == "closed" => result.closed.push(issue.clone()),
                Some(Ok(issue)) => result.valid.push(issue.clone()),
                Some(Err(IssueFetchError::NotFound)) => result.not_found.push(number),
                Some(Err(IssueFetchError::Provider(error))) => {
                    result.errors.push(format!("#{number}: {error}"));
                }
                None => result.errors.push(format!(
                    "#{number}: issue verification ended before this reference was checked"
                )),
            }
        }
    }

    results
}

fn remaining_verification_time(started: Instant) -> Result<Duration, String> {
    GITHUB_VERIFICATION_DEADLINE
        .checked_sub(started.elapsed())
        .filter(|remaining| !remaining.is_zero())
        .ok_or_else(|| {
            format!(
                "Issue verification exceeded its {:?} invocation deadline",
                GITHUB_VERIFICATION_DEADLINE
            )
        })
}

fn empty_issue_verification(spec_path: &str) -> IssueVerification {
    IssueVerification {
        spec_path: spec_path.to_string(),
        valid: Vec::new(),
        closed: Vec::new(),
        not_found: Vec::new(),
        errors: Vec::new(),
    }
}

fn bounded_batch_issue_numbers(
    references: &[(String, Vec<u64>, Vec<u64>)],
) -> Result<BTreeSet<u64>, String> {
    let issues: BTreeSet<u64> = references
        .iter()
        .flat_map(|(_, implements, tracks)| implements.iter().chain(tracks.iter()).copied())
        .collect();
    if issues.len() > MAX_VERIFIED_ISSUES {
        return Err(format!(
            "Issue verification exceeds the {MAX_VERIFIED_ISSUES}-issue invocation limit"
        ));
    }
    Ok(issues)
}

/// List open GitHub issues for a repository.
/// Optionally filter by label. Uses in-process GitHub REST access.
pub fn list_issues(repo: &str, label: Option<&str>) -> Result<Vec<GitHubIssue>, String> {
    list_issues_api(repo, label)
}

fn list_issues_api(repo: &str, label: Option<&str>) -> Result<Vec<GitHubIssue>, String> {
    validate_repo(repo)?;
    let token = std::env::var("GITHUB_TOKEN")
        .map_err(|_| "GITHUB_TOKEN is required for in-process GitHub REST access".to_string())?;
    let repository = verify_api_repository(repo, &token, GITHUB_COMMAND_DEADLINE)?;
    let mut next_target = None;
    collect_issue_pages(|page| {
        let result = fetch_issue_list_page(
            repo,
            repository,
            label,
            &token,
            page,
            next_target.as_deref(),
        )?;
        next_target.clone_from(&result.next_target);
        Ok(result)
    })
}

fn parse_issue_list_json(
    repo: &str,
    body: &serde_json::Value,
) -> Result<ParsedGitHubIssueListPage, String> {
    let entries = body
        .as_array()
        .ok_or_else(|| "GitHub issue-list response must be a JSON array".to_string())?;
    if entries.len() > MAX_ISSUE_LIST_PAGE_ENTRIES {
        return Err(format!(
            "GitHub issue-list response exceeds the {MAX_ISSUE_LIST_PAGE_ENTRIES}-entry page limit (received {} entries)",
            entries.len()
        ));
    }
    let mut seen = BTreeSet::new();
    let mut raw_numbers = Vec::with_capacity(entries.len());
    let mut issues = Vec::with_capacity(entries.len());
    for entry in entries {
        let is_pull_request = match entry.get("pull_request") {
            None => false,
            Some(serde_json::Value::Object(_)) => true,
            Some(_) => {
                return Err(
                    "GitHub issue-list response has a malformed pull_request marker".to_string(),
                );
            }
        };
        let payload: GitHubIssuePayload = serde_json::from_value(entry.clone())
            .map_err(|error| format!("Malformed GitHub issue-list response: {error}"))?;
        if payload.number == 0 {
            return Err("GitHub issue-list response contains an invalid issue number".to_string());
        }
        if payload.state != "open" {
            return Err(
                "GitHub issue-list response contains an item whose raw state is not exactly `open`"
                    .to_string(),
            );
        }
        let number = payload.number;
        let resource = if is_pull_request { "pull" } else { "issues" };
        let issue = issue_from_payload_for_resource(
            repo,
            number,
            payload,
            resource,
            "GitHub issue-list response",
        )?;
        if !seen.insert(number) {
            return Err(format!(
                "GitHub issue-list response contains duplicate or ambiguous item #{number}"
            ));
        }
        raw_numbers.push(number);
        if !is_pull_request {
            issues.push(issue);
        }
    }
    Ok(ParsedGitHubIssueListPage {
        raw_numbers,
        issues,
    })
}

fn fetch_issue_list_page(
    repo: &str,
    repository: GitHubRepositoryIdentity,
    label: Option<&str>,
    token: &str,
    page: usize,
    next_target: Option<&str>,
) -> Result<GitHubIssueListPage, String> {
    let agent = github_api_agent(GITHUB_COMMAND_DEADLINE);
    let request = if let Some(target) = next_target {
        agent.get(target)
    } else {
        let url = format!("{}/repos/{repo}/issues", github_api_base());
        let page_string = page.to_string();
        let mut initial_request = agent
            .get(&url)
            .query("state", "open")
            .query("per_page", "100")
            .query("page", &page_string);
        if let Some(label) = label {
            initial_request = initial_request.query("labels", label);
        }
        initial_request
    };

    let mut response = request
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "specsync")
        .call()
        .map_err(|error| {
            redact_token(
                format!("GitHub issue-list request for page {page} failed: {error}"),
                token,
            )
        })?;

    classify_issue_list_status(repo, response.status().as_u16(), || {
        verify_api_repository(repo, token, GITHUB_COMMAND_DEADLINE).map(|_| ())
    })?;
    let next_target = response
        .headers()
        .get("link")
        .map(|value| {
            value
                .to_str()
                .map_err(|_| "GitHub issue-list Link header is not valid text".to_string())
                .and_then(|header| {
                    parse_github_link_next_target(header, repo, repository, label, page)
                })
        })
        .transpose()?
        .flatten();
    let body: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|error| format!("Failed to parse GitHub issue-list page {page}: {error}"))?;
    let parsed = parse_issue_list_json(repo, &body)?;
    Ok(GitHubIssueListPage {
        raw_numbers: parsed.raw_numbers,
        issues: parsed.issues,
        next_target,
    })
}

fn collect_issue_pages(
    mut fetch_page: impl FnMut(usize) -> Result<GitHubIssueListPage, String>,
) -> Result<Vec<GitHubIssue>, String> {
    let mut issues = Vec::new();
    let mut seen = BTreeSet::new();
    for page in 1..=MAX_ISSUE_LIST_PAGES {
        let result = fetch_page(page)?;
        for number in result.raw_numbers {
            if !seen.insert(number) {
                return Err(format!(
                    "GitHub issue-list pagination returned duplicate or ambiguous item #{number}"
                ));
            }
        }
        issues.extend(result.issues);
        if result.next_target.is_none() {
            return Ok(issues);
        }
    }
    Err(format!(
        "GitHub issue-list pagination exceeded the {MAX_ISSUE_LIST_PAGES}-page safety limit"
    ))
}

fn classify_issue_list_status(
    repo: &str,
    status: u16,
    repository_check: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if status == 200 {
        return Ok(());
    }
    if status != 404 {
        return Err(format!("GitHub issue-list API returned HTTP {status}"));
    }
    match repository_check() {
        Ok(()) => Err(format!(
            "GitHub issue-list API returned HTTP 404 for accessible repository {repo}"
        )),
        Err(error) => Err(format!(
            "GitHub issue-list lookup was inconclusive because repository access to {repo} could not be revalidated: {error}"
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GitHubLinkPagination {
    Page(usize),
    After,
    Before,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedGitHubLinkTarget {
    target: String,
    pagination: GitHubLinkPagination,
}

fn parse_github_link_next_target(
    header: &str,
    repo: &str,
    repository: GitHubRepositoryIdentity,
    label: Option<&str>,
    current_page: usize,
) -> Result<Option<String>, String> {
    if header.trim().is_empty() {
        return Err("GitHub issue-list Link header is empty".to_string());
    }
    let mut next_target = None;
    for entry in header.split(',') {
        let mut parts = entry.trim().split(';');
        let target = parts.next().unwrap_or_default().trim();
        let parsed_target = parse_github_link_target(target, repo, repository, label)?;
        let mut relation_seen = false;
        let mut seen_parameters = BTreeSet::new();
        for parameter in parts {
            let Some((name, value)) = parameter.trim().split_once('=') else {
                return Err("GitHub issue-list Link header has a malformed parameter".to_string());
            };
            let normalized_name = name.trim().to_ascii_lowercase();
            if !seen_parameters.insert(normalized_name.clone()) {
                return Err(format!(
                    "GitHub issue-list Link header repeats the {normalized_name} parameter"
                ));
            }
            if normalized_name != "rel" {
                continue;
            }
            relation_seen = true;
            let raw_relations = value.trim();
            let relations = if let Some(quoted) = raw_relations.strip_prefix('"') {
                quoted.strip_suffix('"').ok_or_else(|| {
                    "GitHub issue-list Link header has an unterminated relation".to_string()
                })?
            } else if raw_relations.contains('"') {
                return Err("GitHub issue-list Link header has a malformed relation".to_string());
            } else {
                raw_relations
            };
            if relations.is_empty() || relations.contains('"') {
                return Err("GitHub issue-list Link header has a malformed relation".to_string());
            }
            if relations
                .split_ascii_whitespace()
                .any(|relation| relation == "next")
            {
                if next_target.is_some() {
                    return Err("GitHub issue-list Link header repeats the next page".to_string());
                }
                match parsed_target.pagination {
                    GitHubLinkPagination::Page(target_page) => {
                        let expected_page = current_page.checked_add(1).ok_or_else(|| {
                            "GitHub issue-list pagination page overflowed".to_string()
                        })?;
                        if target_page != expected_page {
                            return Err(format!(
                                "GitHub issue-list Link header points to page {target_page} instead of expected page {expected_page}"
                            ));
                        }
                    }
                    GitHubLinkPagination::After => {}
                    GitHubLinkPagination::Before => {
                        return Err(
                            "GitHub issue-list next link uses a backwards cursor".to_string()
                        );
                    }
                }
                next_target = Some(parsed_target.target.clone());
            }
        }
        if !relation_seen {
            return Err("GitHub issue-list Link header is missing a relation".to_string());
        }
    }
    Ok(next_target)
}

fn parse_github_link_target(
    target: &str,
    repo: &str,
    repository: GitHubRepositoryIdentity,
    label: Option<&str>,
) -> Result<ParsedGitHubLinkTarget, String> {
    let target_url = target
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .ok_or_else(|| "GitHub issue-list Link header has a malformed target".to_string())?;
    let target = target_url
        .strip_prefix("https://api.github.com/")
        .ok_or_else(|| {
            "GitHub issue-list Link header target is outside the GitHub API".to_string()
        })?;
    if target.contains('#') {
        return Err("GitHub issue-list Link header target contains a fragment".to_string());
    }
    let (path, query) = target
        .split_once('?')
        .ok_or_else(|| "GitHub issue-list Link header target is missing a query".to_string())?;
    let named_path = format!("repos/{repo}/issues");
    let numeric_path = format!("repositories/{}/issues", repository.id);
    if path != named_path && path != numeric_path {
        return Err(
            "GitHub issue-list Link header target does not match the requested repository endpoint"
                .to_string(),
        );
    }
    let mut pagination = None;
    let mut state_seen = false;
    let mut per_page_seen = false;
    let mut label_seen = false;
    let mut seen_parameters = BTreeSet::new();
    for parameter in query.split('&') {
        let Some((name, value)) = parameter.split_once('=') else {
            return Err("GitHub issue-list Link header target has a malformed query".to_string());
        };
        let name = decode_github_query_component(name)?;
        let value = decode_github_query_component(value)?;
        if !seen_parameters.insert(name.clone()) {
            return Err(format!(
                "GitHub issue-list Link header target repeats the {name} query"
            ));
        }
        match name.as_str() {
            "state" if value == "open" => state_seen = true,
            "per_page" if value == "100" => per_page_seen = true,
            "page" => {
                if pagination.is_some() {
                    return Err(
                        "GitHub issue-list Link header target has multiple pagination queries"
                            .to_string(),
                    );
                }
                let parsed_page = value.parse::<usize>().map_err(|_| {
                    "GitHub issue-list Link header target has an invalid page query".to_string()
                })?;
                if parsed_page == 0 || parsed_page.to_string() != value {
                    return Err(
                        "GitHub issue-list Link header target has an invalid page query"
                            .to_string(),
                    );
                }
                pagination = Some(GitHubLinkPagination::Page(parsed_page));
            }
            "after" | "before" => {
                if pagination.is_some() {
                    return Err(
                        "GitHub issue-list Link header target has multiple pagination queries"
                            .to_string(),
                    );
                }
                if value.is_empty() || value.chars().any(is_unsafe_github_untrusted_text_character)
                {
                    return Err(format!(
                        "GitHub issue-list Link header target has an invalid {name} cursor"
                    ));
                }
                pagination = Some(if name == "after" {
                    GitHubLinkPagination::After
                } else {
                    GitHubLinkPagination::Before
                });
            }
            "labels" if label.is_some_and(|expected| value == expected) => label_seen = true,
            "state" | "per_page" | "labels" => {
                return Err(format!(
                    "GitHub issue-list Link header target has an unexpected {name} query"
                ));
            }
            _ => {
                return Err(format!(
                    "GitHub issue-list Link header target has an unexpected {name} query"
                ));
            }
        }
    }
    if !state_seen || !per_page_seen || label.is_some() != label_seen {
        return Err(
            "GitHub issue-list Link header target does not preserve the requested query"
                .to_string(),
        );
    }
    let pagination = pagination.ok_or_else(|| {
        "GitHub issue-list Link header target is missing a pagination query".to_string()
    })?;
    Ok(ParsedGitHubLinkTarget {
        target: target_url.to_string(),
        pagination,
    })
}

fn decode_github_query_component(component: &str) -> Result<String, String> {
    let bytes = component.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => decoded.push(b' '),
            b'%' => {
                let high = bytes
                    .get(index + 1)
                    .and_then(|byte| decode_hex_digit(*byte));
                let low = bytes
                    .get(index + 2)
                    .and_then(|byte| decode_hex_digit(*byte));
                let (Some(high), Some(low)) = (high, low) else {
                    return Err(
                        "GitHub issue-list Link header target has invalid query encoding"
                            .to_string(),
                    );
                };
                decoded.push((high << 4) | low);
                index += 2;
            }
            byte => decoded.push(byte),
        }
        index += 1;
    }
    String::from_utf8(decoded)
        .map_err(|_| "GitHub issue-list Link header target query is not valid UTF-8".to_string())
}

fn decode_hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn is_unsafe_github_untrusted_text_character(character: char) -> bool {
    matches!(
        character as u32,
        0x0000..=0x001f
            | 0x007f..=0x009f
            | 0x00ad
            | 0x061c
            | 0x200b..=0x200f
            | 0x2028..=0x202e
            | 0x2060..=0x206f
            | 0xfeff
    )
}

fn push_visible_github_character(output: &mut String, character: char) {
    if is_unsafe_github_untrusted_text_character(character) {
        output.push_str(&format!("U+{:04X}", character as u32));
    } else {
        output.push(character);
    }
}

fn sanitize_github_title_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        push_visible_github_character(&mut output, character);
    }
    output
}

fn sanitize_github_markdown_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        if is_unsafe_github_untrusted_text_character(character) {
            push_visible_github_character(&mut output, character);
        } else {
            if character.is_ascii_punctuation() {
                output.push('\\');
            }
            output.push(character);
        }
    }
    output
}

/// One GitHub Actions check-run (or legacy status context) for a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitCheckRun {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
}

/// Aggregated check-run view for a commit SHA (ship-status live trust).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitCheckSummary {
    pub repo: String,
    pub sha: String,
    /// `green`, `pending`, `failed`, or `empty` (no check-runs returned).
    pub overall: String,
    pub check_runs: Vec<CommitCheckRun>,
}

/// Fetch check-runs for a commit via in-process GitHub REST (`GITHUB_TOKEN`).
///
/// Used by `change ship-status` to replace pure local guidance when online.
/// Never spawns `gh`. Soft-fail at the call site if this returns Err.
pub fn fetch_commit_check_summary(repo: &str, sha: &str) -> Result<CommitCheckSummary, String> {
    validate_repo(repo)?;
    let sha = sha.trim();
    if sha.is_empty() {
        return Err("commit SHA is required for check-run lookup".to_string());
    }
    if !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("invalid commit SHA `{sha}`"));
    }
    let token = std::env::var("GITHUB_TOKEN")
        .map_err(|_| "GITHUB_TOKEN is required for in-process GitHub REST access".to_string())?;
    verify_api_repository(repo, &token, GITHUB_COMMAND_DEADLINE)?;
    let url = format!(
        "{}/repos/{repo}/commits/{sha}/check-runs?per_page=100",
        github_api_base()
    );
    let mut response = github_api_agent(GITHUB_COMMAND_DEADLINE)
        .get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "specsync")
        .call()
        .map_err(|error| {
            redact_token(
                format!("GitHub check-runs request failed for {repo}@{sha}: {error}"),
                &token,
            )
        })?;
    if response.status() == 404 {
        return Err(format!("commit `{sha}` not found in {repo} (check-runs)"));
    }
    if response.status() != 200 {
        return Err(format!(
            "GitHub check-runs API returned HTTP {} for {repo}@{sha}",
            response.status()
        ));
    }
    let body: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|error| format!("Failed to parse GitHub check-runs response: {error}"))?;
    parse_commit_check_summary(repo, sha, &body)
}

/// Pure parse + aggregate of a check-runs list payload (unit-testable).
pub fn parse_commit_check_summary(
    repo: &str,
    sha: &str,
    body: &serde_json::Value,
) -> Result<CommitCheckSummary, String> {
    let runs = body
        .get("check_runs")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "GitHub check-runs response missing check_runs array".to_string())?;
    let mut check_runs = Vec::with_capacity(runs.len());
    for run in runs {
        let name = run
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            return Err("GitHub check-runs response contains a check without a name".to_string());
        }
        let status = run
            .get("status")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown")
            .to_string();
        let conclusion = run
            .get("conclusion")
            .and_then(|value| value.as_str())
            .map(str::to_string);
        check_runs.push(CommitCheckRun {
            name,
            status,
            conclusion,
        });
    }
    let overall = aggregate_check_runs(&check_runs).to_string();
    Ok(CommitCheckSummary {
        repo: repo.to_string(),
        sha: sha.to_string(),
        overall,
        check_runs,
    })
}

fn aggregate_check_runs(runs: &[CommitCheckRun]) -> &'static str {
    if runs.is_empty() {
        return "empty";
    }
    let mut pending = false;
    for run in runs {
        let status = run.status.to_ascii_lowercase();
        if status != "completed" {
            pending = true;
            continue;
        }
        let conclusion = run.conclusion.as_deref().unwrap_or("").to_ascii_lowercase();
        match conclusion.as_str() {
            "success" | "neutral" | "skipped" => {}
            "failure" | "cancelled" | "timed_out" | "action_required" | "startup_failure"
            | "stale" => {
                return "failed";
            }
            "" => pending = true,
            _ => {
                // Unknown conclusion: treat as failed-closed for ship readiness.
                return "failed";
            }
        }
    }
    if pending { "pending" } else { "green" }
}

/// Whether a check name is trust-lane relevant for ship guidance (heuristic).
pub fn is_trust_relevant_check_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("trust")
        || lower.contains("implementation ready")
        || lower.contains("specsync trusted")
        || lower.contains("required ci")
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

    let title_path = sanitize_github_title_text(spec_path);
    let markdown_path = sanitize_github_markdown_text(spec_path);
    let title = format!("Spec drift detected: {title_path}");
    let body = format!(
        "## Spec Drift Detected\n\n\
         **Spec:** {markdown_path}\n\n\
         ### Validation Errors\n\n{}\n\n\
         ---\n\
         *Auto-created by `specsync check --create-issues`*",
        errors
            .iter()
            .map(|error| format!("- {}", sanitize_github_markdown_text(error)))
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

    #[derive(Default)]
    struct FakeBatchBackend {
        prepare_calls: usize,
        prepare_failure_after: Option<(usize, String)>,
        fetch_calls: Vec<u64>,
        responses: BTreeMap<u64, Result<GitHubIssue, IssueFetchError>>,
    }

    impl IssueBatchBackend for FakeBatchBackend {
        fn prepare(
            &mut self,
            _repo: &str,
            remaining: &mut dyn FnMut() -> Result<Duration, String>,
        ) -> Result<(), String> {
            self.prepare_calls += 1;
            remaining()?;
            if self
                .prepare_failure_after
                .as_ref()
                .is_some_and(|(step, _)| *step == 1)
            {
                return Err(self
                    .prepare_failure_after
                    .as_ref()
                    .map(|(_, error)| error.clone())
                    .unwrap_or_default());
            }
            remaining()?;
            if self
                .prepare_failure_after
                .as_ref()
                .is_some_and(|(step, _)| *step == 2)
            {
                return Err(self
                    .prepare_failure_after
                    .as_ref()
                    .map(|(_, error)| error.clone())
                    .unwrap_or_default());
            }
            Ok(())
        }

        fn fetch(
            &mut self,
            _repo: &str,
            number: u64,
            _deadline: Duration,
            _remaining: &mut dyn FnMut() -> Result<Duration, String>,
        ) -> Result<GitHubIssue, IssueFetchError> {
            self.fetch_calls.push(number);
            self.responses.get(&number).cloned().unwrap_or_else(|| {
                Err(IssueFetchError::Provider(format!(
                    "no fake response for issue #{number}"
                )))
            })
        }
    }

    fn fake_issue(number: u64, state: &str) -> GitHubIssue {
        GitHubIssue {
            number,
            title: format!("Issue {number}"),
            state: state.to_string(),
            labels: Vec::new(),
            url: format!("https://example.test/issues/{number}"),
        }
    }

    fn fake_issue_list_entry(number: u64, pull_request: bool) -> serde_json::Value {
        let resource = if pull_request { "pull" } else { "issues" };
        let mut entry = serde_json::json!({
            "number": number,
            "title": format!("Entry {number}"),
            "state": "open",
            "labels": [],
            "html_url": format!("https://github.com/owner/repo/{resource}/{number}")
        });
        if pull_request {
            entry["pull_request"] = serde_json::json!({});
        }
        entry
    }

    fn fake_repository_identity() -> GitHubRepositoryIdentity {
        GitHubRepositoryIdentity { id: 1_300_192 }
    }

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

    #[test]
    fn configured_repository_validation_is_stable_without_provider_access_or_references() {
        let root = Path::new(".");
        assert_eq!(
            resolve_repo(Some("owner/repo"), root),
            Ok("owner/repo".to_string())
        );

        for unsafe_repo in [
            "owner/repo\ninjected",
            "owner/repo?state=closed",
            "owner/repo/extra",
            "owner/repo\u{1b}[31m",
        ] {
            let error = resolve_repo(Some(unsafe_repo), root)
                .expect_err("unsafe configured repositories must fail before provider access");
            assert_eq!(
                error,
                "GitHub repository contains invalid owner or repository characters"
            );
            assert!(!error.contains(unsafe_repo));
        }

        let unsafe_repo = "owner\ninjected";
        let error = resolve_repo(Some(unsafe_repo), root)
            .expect_err("configured repositories must use owner/repository syntax");
        assert_eq!(
            error,
            "GitHub repository must use the `owner/repository` form"
        );
        assert!(!error.contains(unsafe_repo));
    }

    #[test]
    fn malformed_issue_provider_output_fails_closed() {
        let json: serde_json::Value =
            serde_json::from_slice(br#"{"number":42,"state":"OPEN"}"#).unwrap();
        let error = parse_issue_json("owner/repo", 42, &json)
            .expect_err("missing provider fields must be inconclusive");

        assert!(error.contains("title"));
    }

    #[test]
    fn issue_list_strictly_parses_issues_and_rejects_malformed_entries() {
        let valid = serde_json::json!([
            {
                "number": 42,
                "title": "Issue",
                "state": "open",
                "labels": [],
                "html_url": "https://github.com/owner/repo/issues/42"
            },
            {
                "number": 43,
                "title": "Pull request",
                "state": "open",
                "labels": [],
                "html_url": "https://github.com/owner/repo/pull/43",
                "pull_request": {}
            }
        ]);
        let parsed =
            parse_issue_list_json("owner/repo", &valid).expect("valid issue lists must parse");
        assert_eq!(parsed.raw_numbers, [42, 43]);
        assert_eq!(parsed.issues.len(), 1);
        assert_eq!(parsed.issues[0].number, 42);

        let malformed_marker = serde_json::json!([
            {
                "number": 42,
                "title": "Issue",
                "state": "open",
                "labels": [],
                "html_url": "https://github.com/owner/repo/issues/42",
                "pull_request": "not-an-object"
            }
        ]);
        assert!(
            parse_issue_list_json("owner/repo", &malformed_marker)
                .expect_err("malformed pull-request markers must fail closed")
                .contains("pull_request")
        );

        let malformed_issue = serde_json::json!([{ "number": 42, "state": "open" }]);
        assert!(parse_issue_list_json("owner/repo", &malformed_issue).is_err());
    }

    #[test]
    fn issue_list_rejects_semantically_malformed_pull_requests_before_filtering() {
        let mut zero_number = fake_issue_list_entry(43, true);
        zero_number["number"] = serde_json::json!(0);
        zero_number["html_url"] = serde_json::json!("https://github.com/owner/repo/pull/0");

        let mut empty_title = fake_issue_list_entry(43, true);
        empty_title["title"] = serde_json::json!(" \t");

        let mut empty_label = fake_issue_list_entry(43, true);
        empty_label["labels"] = serde_json::json!([{"name": ""}]);

        for (entry, expected_error) in [
            (zero_number, "invalid issue number"),
            (empty_title, "empty `title`"),
            (empty_label, "empty label name"),
        ] {
            let error = parse_issue_list_json("owner/repo", &serde_json::Value::Array(vec![entry]))
                .expect_err("malformed pull requests must reject the complete page");
            assert!(error.contains(expected_error), "{error}");
        }
    }

    #[test]
    fn issue_list_requires_exact_raw_open_state_for_issues_and_pull_requests() {
        for pull_request in [false, true] {
            for state in ["closed", "OPEN", "Open", "oPeN"] {
                let mut entry = fake_issue_list_entry(42, pull_request);
                entry["state"] = serde_json::json!(state);

                let error =
                    parse_issue_list_json("owner/repo", &serde_json::Value::Array(vec![entry]))
                        .expect_err("the open-issue endpoint must require exact raw open state");
                assert!(error.contains("raw state is not exactly `open`"), "{error}");
            }
        }
    }

    #[test]
    fn issue_list_rejects_wrong_issue_and_pull_request_url_identity() {
        let mut issue_with_pull_url = fake_issue_list_entry(42, false);
        issue_with_pull_url["html_url"] =
            serde_json::json!("https://github.com/owner/repo/pull/42");

        let mut pull_request_with_issue_url = fake_issue_list_entry(43, true);
        pull_request_with_issue_url["html_url"] =
            serde_json::json!("https://github.com/owner/repo/issues/43");

        let mut pull_request_with_wrong_number = fake_issue_list_entry(43, true);
        pull_request_with_wrong_number["html_url"] =
            serde_json::json!("https://github.com/owner/repo/pull/44");

        for entry in [
            issue_with_pull_url,
            pull_request_with_issue_url,
            pull_request_with_wrong_number,
        ] {
            let error = parse_issue_list_json("owner/repo", &serde_json::Value::Array(vec![entry]))
                .expect_err("item kind and number must match the trusted GitHub URL");
            assert!(error.contains("html_url"), "{error}");
        }
    }

    #[test]
    fn provider_item_urls_require_canonical_decimal_numbers_in_list_and_detail() {
        for (number, pull_request, url) in [
            (42, false, "https://github.com/owner/repo/issues/00042"),
            (43, true, "https://github.com/owner/repo/pull/00043"),
        ] {
            let mut entry = fake_issue_list_entry(number, pull_request);
            entry["html_url"] = serde_json::json!(url);
            let error = parse_issue_list_json("owner/repo", &serde_json::Value::Array(vec![entry]))
                .expect_err("noncanonical list item numbers must fail closed");
            assert!(error.contains("html_url"), "{error}");
        }

        let details = serde_json::json!({
            "number": 42,
            "title": "Issue",
            "state": "open",
            "labels": [],
            "html_url": "https://github.com/owner/repo/issues/00042",
            "body": null
        });
        let error = parse_issue_details_json("owner/repo", 42, &details)
            .expect_err("noncanonical detail item numbers must fail closed");
        assert!(error.contains("html_url"), "{error}");
    }

    #[test]
    fn issue_list_rejects_duplicate_identities_involving_pull_requests_before_filtering() {
        for entries in [
            vec![
                fake_issue_list_entry(42, false),
                fake_issue_list_entry(42, true),
            ],
            vec![
                fake_issue_list_entry(42, true),
                fake_issue_list_entry(42, true),
            ],
        ] {
            let error = parse_issue_list_json("owner/repo", &serde_json::Value::Array(entries))
                .expect_err("duplicate pull-request identities must reject the complete page");
            assert!(error.contains("duplicate or ambiguous item #42"), "{error}");
        }
    }

    #[test]
    fn issue_list_filters_fully_valid_pull_requests_after_validation() {
        let body = serde_json::json!([
            fake_issue_list_entry(42, false),
            fake_issue_list_entry(43, true)
        ]);
        let parsed = parse_issue_list_json("owner/repo", &body)
            .expect("fully valid pull requests may be filtered");

        assert_eq!(parsed.raw_numbers, [42, 43]);
        assert_eq!(
            parsed
                .issues
                .iter()
                .map(|issue| issue.number)
                .collect::<Vec<_>>(),
            [42]
        );
    }

    #[test]
    fn issue_list_requires_present_pull_request_marker_to_be_an_object() {
        let valid_object = serde_json::json!([{
            "number": 43,
            "title": "Pull request",
            "state": "open",
            "labels": [],
            "html_url": "https://github.com/owner/repo/pull/43",
            "pull_request": {}
        }]);
        let parsed = parse_issue_list_json("owner/repo", &valid_object)
            .expect("object pull-request markers must be accepted");
        assert_eq!(parsed.raw_numbers, [43]);
        assert!(parsed.issues.is_empty());

        let null_marker = serde_json::json!([
            {
                "number": 42,
                "title": "Issue",
                "state": "open",
                "labels": [],
                "html_url": "https://github.com/owner/repo/issues/42"
            },
            {
                "number": 43,
                "title": "Invalid pull request",
                "state": "open",
                "labels": [],
                "html_url": "https://github.com/owner/repo/pull/43",
                "pull_request": null
            }
        ]);
        let error = parse_issue_list_json("owner/repo", &null_marker)
            .expect_err("null pull-request markers must fail the whole page");
        assert!(error.contains("pull_request"));
    }

    #[test]
    fn issue_list_accepts_one_hundred_provider_entries_including_pull_requests() {
        let mut entries = (1..MAX_ISSUE_LIST_PAGE_ENTRIES as u64)
            .map(|number| fake_issue_list_entry(number, false))
            .collect::<Vec<_>>();
        entries.push(fake_issue_list_entry(100, true));

        let parsed = parse_issue_list_json("owner/repo", &serde_json::Value::Array(entries))
            .expect("a provider page with exactly 100 entries must parse");

        assert_eq!(parsed.raw_numbers.len(), 100);
        assert_eq!(parsed.issues.len(), 99);
        assert_eq!(parsed.issues.first().map(|issue| issue.number), Some(1));
        assert_eq!(parsed.issues.last().map(|issue| issue.number), Some(99));
    }

    #[test]
    fn issue_list_rejects_one_hundred_one_entries_before_parsing_malformed_pull_request() {
        let mut entries = (1..=MAX_ISSUE_LIST_PAGE_ENTRIES as u64)
            .map(|number| fake_issue_list_entry(number, false))
            .collect::<Vec<_>>();
        entries.push(serde_json::json!({ "pull_request": {} }));

        let error = parse_issue_list_json("owner/repo", &serde_json::Value::Array(entries))
            .expect_err("a provider page with 101 entries must fail before item parsing");

        assert!(error.contains("100-entry page limit"));
        assert!(error.contains("received 101 entries"));
    }

    #[test]
    fn issue_details_reject_pull_request_markers_of_any_shape() {
        let valid = serde_json::json!({
            "number": 42,
            "title": "Issue",
            "state": "open",
            "labels": [{"name": "security"}],
            "html_url": "https://github.com/owner/repo/issues/42",
            "body": null
        });

        for (shape, marker) in [
            ("object", serde_json::json!({})),
            ("null", serde_json::Value::Null),
            ("scalar", serde_json::json!("not-an-object")),
        ] {
            let mut pull_request = valid.clone();
            pull_request["pull_request"] = marker;

            let error = parse_issue_details_json("owner/repo", 42, &pull_request)
                .expect_err("the issue endpoint must reject every pull request marker shape");
            assert!(error.contains("pull request"), "{shape}: {error}");
        }
    }

    #[test]
    fn issue_details_require_typed_identity_body_and_url() {
        let valid = serde_json::json!({
            "number": 42,
            "title": "Issue",
            "state": "open",
            "labels": [{"name": "security"}],
            "html_url": "https://github.com/owner/repo/issues/42",
            "body": null
        });
        let details =
            parse_issue_details_json("owner/repo", 42, &valid).expect("valid details must parse");
        assert_eq!(details.issue.number, 42);
        assert_eq!(details.issue.labels, ["security"]);
        assert!(details.body.is_empty());

        let mut pull_request = valid.clone();
        pull_request["pull_request"] = serde_json::json!({
            "url": "https://api.github.com/repos/owner/repo/pulls/42"
        });
        assert!(
            parse_issue_details_json("owner/repo", 42, &pull_request)
                .expect_err("the issue endpoint must not accept pull requests")
                .contains("pull request")
        );

        let mut malformed = valid.clone();
        malformed["body"] = serde_json::json!(["not", "text"]);
        assert!(
            parse_issue_details_json("owner/repo", 42, &malformed)
                .expect_err("non-string bodies must fail closed")
                .contains("body")
        );

        let mut wrong_identity = valid.clone();
        wrong_identity["number"] = serde_json::json!(43);
        assert!(
            parse_issue_details_json("owner/repo", 42, &wrong_identity)
                .expect_err("mismatched identities must fail closed")
                .contains("requested issue #42")
        );

        let mut wrong_url = valid.clone();
        wrong_url["html_url"] = serde_json::json!("https://example.test/issues/42");
        assert!(
            parse_issue_details_json("owner/repo", 42, &wrong_url)
                .expect_err("untrusted issue URLs must fail closed")
                .contains("html_url")
        );

        let mut wrong_repo_url = valid.clone();
        wrong_repo_url["html_url"] = serde_json::json!("https://github.com/other/repo/issues/42");
        assert!(
            parse_issue_details_json("owner/repo", 42, &wrong_repo_url)
                .expect_err("cross-repository issue URLs must fail closed")
                .contains("html_url")
        );

        let mut empty_label = valid;
        empty_label["labels"] = serde_json::json!([{"name": ""}]);
        assert!(
            parse_issue_details_json("owner/repo", 42, &empty_label)
                .expect_err("empty label names must fail closed")
                .contains("label")
        );
    }

    #[test]
    fn repository_identifiers_are_validated_before_rest_url_construction() {
        assert!(validate_repo("owner/repo").is_ok());
        assert!(validate_repo("owner").is_err());
        assert!(validate_repo("owner/repo/extra").is_err());
        assert!(validate_repo("owner/repo?state=closed").is_err());
    }

    #[test]
    fn link_header_parsing_detects_next_and_rejects_malformed_values() {
        let paginated = concat!(
            "<https://api.github.com/repos/owner/repo/issues?state=open&per_page=100&page=2>; rel=\"next\", ",
            "<https://api.github.com/repos/owner/repo/issues?page=4&per_page=100&state=open>; rel=\"last\""
        );
        let next = parse_github_link_next_target(
            paginated,
            "owner/repo",
            fake_repository_identity(),
            None,
            1,
        )
        .expect("GitHub Link header must parse");
        assert_eq!(
            next.as_deref(),
            Some("https://api.github.com/repos/owner/repo/issues?state=open&per_page=100&page=2")
        );
        assert!(
            parse_github_link_next_target(
                "<https://api.github.com/repos/owner/repo/issues?state=open&per_page=100&page=1>; rel=\"prev\"",
                "owner/repo",
                fake_repository_identity(),
                None,
                2,
            )
            .expect("previous-only Link header must parse")
            .is_none()
        );
        assert!(
            parse_github_link_next_target(
                "https://example.test/page/2; rel=\"next\"",
                "owner/repo",
                fake_repository_identity(),
                None,
                1,
            )
            .is_err()
        );
        assert!(
            parse_github_link_next_target(
                "<https://example.test/page/2>",
                "owner/repo",
                fake_repository_identity(),
                None,
                1,
            )
            .is_err()
        );
        assert!(
            parse_github_link_next_target(
                "<https://example.test/page/2>; rel=\"next",
                "owner/repo",
                fake_repository_identity(),
                None,
                1,
            )
            .is_err()
        );
        assert!(
            parse_github_link_next_target(
                "<https://api.github.com/repos/owner/repo/issues?state=open&per_page=100&page=3>; rel=\"next\"",
                "owner/repo",
                fake_repository_identity(),
                None,
                1,
            )
            .is_err()
        );
    }

    #[test]
    fn link_header_accepts_matching_numeric_repository_and_cursor() {
        let target = concat!(
            "https://api.github.com/repositories/1300192/issues?",
            "state=open&per_page=100&after=Y3Vyc29yOnYyOpHO"
        );
        let header = format!("<{target}>; rel=\"next\"");
        let parsed = parse_github_link_next_target(
            &header,
            "owner/repo",
            fake_repository_identity(),
            None,
            1,
        )
        .expect("GitHub's canonical numeric repository path and cursor must be accepted");

        assert_eq!(parsed.as_deref(), Some(target));
    }

    #[test]
    fn link_header_rejects_wrong_or_malformed_repository_identity_and_resource() {
        for target in [
            "https://api.github.com/repos/other/repo/issues?state=open&per_page=100&page=2",
            "https://api.github.com/repos/owner/repo/pulls?state=open&per_page=100&page=2",
            "https://api.github.com/repositories/1/issues?state=open&per_page=100&page=2",
            "https://api.github.com/repositories/0/issues?state=open&per_page=100&page=2",
            "https://api.github.com/repositories/01300192/issues?state=open&per_page=100&page=2",
            "https://api.github.com/repositories/not-a-number/issues?state=open&per_page=100&page=2",
            "https://api.github.com/repositories/1300192/pulls?state=open&per_page=100&page=2",
        ] {
            let header = format!("<{target}>; rel=\"next\"");
            let error = parse_github_link_next_target(
                &header,
                "owner/repo",
                fake_repository_identity(),
                None,
                1,
            )
            .expect_err("Link targets must remain on the authenticated repository endpoint");
            assert!(error.contains("requested repository endpoint"));
        }
    }

    #[test]
    fn link_header_rejects_untrusted_origins() {
        for target in [
            "http://api.github.com/repositories/1300192/issues?state=open&per_page=100&page=2",
            "https://api.github.com.evil.test/repositories/1300192/issues?state=open&per_page=100&page=2",
            "https://user@api.github.com/repositories/1300192/issues?state=open&per_page=100&page=2",
            "https://api.github.com:443/repositories/1300192/issues?state=open&per_page=100&page=2",
        ] {
            let header = format!("<{target}>; rel=\"next\"");
            assert!(
                parse_github_link_next_target(
                    &header,
                    "owner/repo",
                    fake_repository_identity(),
                    None,
                    1,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn link_header_rejects_query_mismatch() {
        for query in [
            "state=closed&per_page=100&page=2",
            "state=open&per_page=50&page=2",
            "state=open&per_page=100&page=2&sort=created",
            "state=open&per_page=100&page=2&labels=bug",
            "state=open&per_page=100&page=2&since=2026-01-01",
            "state=open&per_page=100&page=2&after=cursor",
            "state=open&per_page=100&after=",
            "state=open&per_page=100&after=cursor&after=other",
            "state=open&per_page=100&page=2&%73tate=open",
            "state=open&per_page=100&after=cursor%0Ainjected",
        ] {
            let header =
                format!("<https://api.github.com/repos/owner/repo/issues?{query}>; rel=\"next\"");
            assert!(
                parse_github_link_next_target(
                    &header,
                    "owner/repo",
                    fake_repository_identity(),
                    None,
                    1,
                )
                .is_err()
            );
        }

        let matching_label = concat!(
            "<https://api.github.com/repos/owner/repo/issues?",
            "labels=needs%20triage&page=2&state=open&per_page=100>; rel=\"next\""
        );
        assert!(
            parse_github_link_next_target(
                matching_label,
                "owner/repo",
                fake_repository_identity(),
                Some("needs triage"),
                1,
            )
            .expect("the requested encoded label must be preserved")
            .is_some()
        );

        let mismatched_label = concat!(
            "<https://api.github.com/repos/owner/repo/issues?",
            "labels=bug&page=2&state=open&per_page=100>; rel=\"next\""
        );
        assert!(
            parse_github_link_next_target(
                mismatched_label,
                "owner/repo",
                fake_repository_identity(),
                Some("needs triage"),
                1,
            )
            .is_err()
        );
    }

    #[test]
    fn issue_list_pagination_collects_every_page_in_order() {
        let mut requested_pages = Vec::new();
        let issues = collect_issue_pages(|page| {
            requested_pages.push(page);
            let number = page as u64;
            Ok(GitHubIssueListPage {
                raw_numbers: vec![number],
                issues: vec![fake_issue(number, "open")],
                next_target: (page < 3).then(|| "next".to_string()),
            })
        })
        .expect("all pages must be collected");

        assert_eq!(requested_pages, [1, 2, 3]);
        assert_eq!(
            issues.iter().map(|issue| issue.number).collect::<Vec<_>>(),
            [1, 2, 3]
        );
    }

    #[test]
    fn issue_list_pagination_fails_instead_of_truncating_or_deduplicating() {
        let limit_error = collect_issue_pages(|page| {
            let number = page as u64;
            Ok(GitHubIssueListPage {
                raw_numbers: vec![number],
                issues: vec![fake_issue(number, "open")],
                next_target: Some("next".to_string()),
            })
        })
        .expect_err("a next page beyond the cap must be explicit");
        assert!(limit_error.contains("safety limit"));

        let duplicate_error = collect_issue_pages(|page| {
            Ok(GitHubIssueListPage {
                raw_numbers: vec![42],
                issues: vec![fake_issue(42, "open")],
                next_target: (page == 1).then(|| "next".to_string()),
            })
        })
        .expect_err("duplicate pages must fail closed");
        assert!(duplicate_error.contains("duplicate or ambiguous item #42"));
    }

    #[test]
    fn issue_list_pagination_rejects_duplicates_hidden_by_pull_request_filtering() {
        let error = collect_issue_pages(|page| {
            if page == 1 {
                return Ok(GitHubIssueListPage {
                    raw_numbers: vec![42],
                    issues: vec![fake_issue(42, "open")],
                    next_target: Some("next".to_string()),
                });
            }
            Ok(GitHubIssueListPage {
                raw_numbers: vec![42],
                issues: Vec::new(),
                next_target: None,
            })
        })
        .expect_err("a pull request cannot hide a duplicate identity across pages");

        assert!(error.contains("duplicate or ambiguous item #42"), "{error}");
    }

    #[test]
    fn issue_list_404_revalidates_repository_access() {
        let accessible = classify_issue_list_status("owner/repo", 404, || Ok(()))
            .expect_err("an endpoint 404 remains a provider error");
        assert!(accessible.contains("accessible repository"));

        let inaccessible = classify_issue_list_status("owner/repo", 404, || {
            Err("repository returned HTTP 404".to_string())
        })
        .expect_err("repository access changes must be inconclusive");
        assert!(inaccessible.contains("could not be revalidated"));
        assert!(inaccessible.contains("repository returned HTTP 404"));
    }

    #[test]
    fn batch_prepares_once_deduplicates_fetches_and_attributes_per_spec() {
        let references = vec![
            ("one".to_string(), vec![3, 1, 1], vec![2, 3]),
            ("two".to_string(), vec![2], vec![1]),
        ];
        let mut backend = FakeBatchBackend::default();
        backend.responses.insert(1, Ok(fake_issue(1, "open")));
        backend.responses.insert(2, Ok(fake_issue(2, "closed")));
        backend.responses.insert(3, Err(IssueFetchError::NotFound));
        let mut remaining = || Ok(Duration::from_secs(30));

        let results = verify_issue_batch_with_backend(
            "owner/repo",
            &references,
            &mut backend,
            &mut remaining,
        );

        assert_eq!(backend.prepare_calls, 1);
        assert_eq!(backend.fetch_calls, vec![1, 2, 3]);
        assert_eq!(
            results[0]
                .valid
                .iter()
                .map(|issue| issue.number)
                .collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(
            results[0]
                .closed
                .iter()
                .map(|issue| issue.number)
                .collect::<Vec<_>>(),
            vec![2]
        );
        assert_eq!(results[0].not_found, vec![3]);
        assert_eq!(
            results[1]
                .valid
                .iter()
                .map(|issue| issue.number)
                .collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(
            results[1]
                .closed
                .iter()
                .map(|issue| issue.number)
                .collect::<Vec<_>>(),
            vec![2]
        );
    }

    #[test]
    fn batch_prepare_auth_and_repository_errors_are_inconclusive_for_every_spec() {
        for (step, message) in [(1, "auth failed"), (2, "repository preflight failed")] {
            let references = vec![
                ("one".to_string(), vec![1], Vec::new()),
                ("two".to_string(), vec![2], Vec::new()),
            ];
            let mut backend = FakeBatchBackend {
                prepare_failure_after: Some((step, message.to_string())),
                ..FakeBatchBackend::default()
            };
            let mut remaining = || Ok(Duration::from_secs(30));

            let results = verify_issue_batch_with_backend(
                "owner/repo",
                &references,
                &mut backend,
                &mut remaining,
            );

            assert_eq!(backend.prepare_calls, 1);
            assert!(backend.fetch_calls.is_empty());
            assert!(results.iter().all(|result| result.errors == [message]));
        }
    }

    #[test]
    fn batch_attributes_malformed_and_transport_provider_errors() {
        let references = vec![
            ("one".to_string(), vec![4], Vec::new()),
            ("two".to_string(), vec![5], Vec::new()),
        ];
        let mut backend = FakeBatchBackend::default();
        backend.responses.insert(
            4,
            Err(IssueFetchError::Provider(
                "malformed provider response".to_string(),
            )),
        );
        backend.responses.insert(
            5,
            Err(IssueFetchError::Provider(
                "provider transport failed".to_string(),
            )),
        );
        let mut remaining = || Ok(Duration::from_secs(30));

        let results = verify_issue_batch_with_backend(
            "owner/repo",
            &references,
            &mut backend,
            &mut remaining,
        );

        assert_eq!(backend.fetch_calls, vec![4, 5]);
        assert_eq!(results[0].errors, ["#4: malformed provider response"]);
        assert_eq!(results[1].errors, ["#5: provider transport failed"]);
    }

    #[test]
    fn batch_full_deadline_includes_prepare_and_stops_before_fetch() {
        let references = vec![("one".to_string(), vec![1, 2], Vec::new())];
        let mut backend = FakeBatchBackend::default();
        let mut clock = vec![
            Ok(Duration::from_secs(30)),
            Ok(Duration::from_secs(20)),
            Err("full invocation deadline exhausted".to_string()),
        ]
        .into_iter();
        let mut remaining = || {
            clock
                .next()
                .unwrap_or_else(|| Err("clock read beyond expected deadline".to_string()))
        };

        let results = verify_issue_batch_with_backend(
            "owner/repo",
            &references,
            &mut backend,
            &mut remaining,
        );

        assert_eq!(backend.prepare_calls, 1);
        assert!(backend.fetch_calls.is_empty());
        assert!(results[0].errors[0].contains("full invocation deadline exhausted"));
        assert!(results[0].errors[1].contains("ended before"));
    }

    #[test]
    fn batch_cap_is_enforced_before_provider_prepare() {
        let excessive: Vec<u64> = (1..=(MAX_VERIFIED_ISSUES as u64 + 1)).collect();
        let references = [("excessive".to_string(), excessive, Vec::new())];
        let mut backend = FakeBatchBackend::default();
        let mut remaining = || -> Result<Duration, String> {
            panic!("the provider clock must not be consulted before the issue cap")
        };

        let results = verify_issue_batch_with_backend(
            "owner/repo",
            &references,
            &mut backend,
            &mut remaining,
        );

        assert_eq!(backend.prepare_calls, 0);
        assert!(backend.fetch_calls.is_empty());
        assert!(results[0].errors[0].contains("invocation limit"));
    }

    #[test]
    fn rest_not_found_is_confirmed_when_repository_recheck_succeeds() {
        let result = revalidate_issue_not_found(
            "owner/repo",
            Err::<GitHubIssue, _>(IssueFetchError::NotFound),
            || Ok(()),
        );

        assert!(matches!(result, Err(IssueFetchError::NotFound)));
    }

    #[test]
    fn rest_not_found_is_inconclusive_when_repository_recheck_fails() {
        let inaccessible = revalidate_issue_not_found(
            "owner/repo",
            Err::<GitHubIssue, _>(IssueFetchError::NotFound),
            || Err("HTTP 404".to_string()),
        );
        assert!(matches!(inaccessible, Err(IssueFetchError::Provider(_))));
        let message = inaccessible.unwrap_err().to_string();
        assert!(message.contains("could not be revalidated"));
        assert!(message.contains("HTTP 404"));
    }

    #[cfg(unix)]
    #[test]
    fn drift_issue_capture_sanitizes_untrusted_title_and_markdown_arguments() {
        const CHILD_ENV: &str = "SPECSYNC_DRIFT_CAPTURE_CHILD";
        const CAPTURE_ENV: &str = "SPECSYNC_DRIFT_CAPTURE_PATH";
        const SPEC_PATH: &str = "specs/auth\n# [spoof](target)\u{202e}.spec.md";
        const ERRORS: &[&str] = &[
            "primary failure\n- injected [click](javascript:alert(1))\u{0085}<details>",
            "*bold* | table\u{2028}next\u{2066}",
        ];

        if std::env::var_os(CHILD_ENV).is_some() {
            let errors = ERRORS
                .iter()
                .map(|error| (*error).to_string())
                .collect::<Vec<_>>();
            let issue = create_drift_issue(
                "owner/repo",
                SPEC_PATH,
                &errors,
                &["spec-drift".to_string()],
            )
            .expect("the fake gh provider must capture the sanitized issue");
            assert_eq!(issue.number, 42);
            return;
        }

        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::TempDir::new().unwrap();
        let fake_gh = temp.path().join("gh");
        let capture = temp.path().join("arguments");
        std::fs::write(
            &fake_gh,
            concat!(
                "#!/bin/sh\n",
                "if [ \"$1\" = \"auth\" ] && [ \"$2\" = \"status\" ]; then exit 0; fi\n",
                ": > \"$SPECSYNC_DRIFT_CAPTURE_PATH\"\n",
                "for argument in \"$@\"; do\n",
                "  printf '%s\\0' \"$argument\" >> \"$SPECSYNC_DRIFT_CAPTURE_PATH\"\n",
                "done\n",
                "printf '%s\\n' 'https://github.com/owner/repo/issues/42'\n",
            ),
        )
        .unwrap();
        let mut permissions = std::fs::metadata(&fake_gh).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&fake_gh, permissions).unwrap();

        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "github::tests::drift_issue_capture_sanitizes_untrusted_title_and_markdown_arguments",
                "--nocapture",
            ])
            .env(CHILD_ENV, "1")
            .env(CAPTURE_ENV, &capture)
            .env("PATH", temp.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "isolated drift capture failed:\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let captured = std::fs::read(&capture).unwrap();
        let arguments = captured
            .split(|byte| *byte == 0)
            .filter(|argument| !argument.is_empty())
            .map(|argument| String::from_utf8(argument.to_vec()).unwrap())
            .collect::<Vec<_>>();
        let value_after = |flag: &str| {
            let index = arguments
                .iter()
                .position(|argument| argument == flag)
                .unwrap();
            arguments[index + 1].as_str()
        };
        let title = value_after("--title");
        let body = value_after("--body");

        assert_eq!(
            title,
            "Spec drift detected: specs/authU+000A# [spoof](target)U+202E.spec.md"
        );
        assert!(!title.chars().any(is_unsafe_github_untrusted_text_character));
        assert!(body.contains("specs\\/authU+000A\\# \\[spoof\\]\\(target\\)U+202E\\.spec\\.md"));
        assert!(body.contains("primary failureU+000A\\- injected"));
        assert!(body.contains("\\[click\\]\\(javascript\\:alert\\(1\\)\\)"));
        assert!(body.contains("U+0085\\<details\\>"));
        assert!(body.contains("\\*bold\\* \\| tableU+2028nextU+2066"));
        assert!(!body.contains("\n- injected"));
        assert!(!body.contains("[click](javascript:alert(1))"));
        assert!(!body.contains("<details>"));
        assert!(!body.contains('\u{0085}'));
        assert!(!body.contains('\u{2028}'));
        assert!(!body.contains('\u{202e}'));
        assert!(!body.contains('\u{2066}'));
    }

    #[test]
    fn provider_process_construction_is_absent_from_every_read_path() {
        let github_source = include_str!("github.rs");
        let read_start = github_source.find("pub fn fetch_issue_gh").unwrap();
        let write_start = github_source.find("pub fn create_drift_issue").unwrap();
        assert!(
            !github_source[read_start..write_start].contains("Command::new(\"gh\")"),
            "GitHub read/list/verify code must not construct a gh process"
        );

        for (path, source) in [
            ("src/importer.rs", include_str!("importer.rs")),
            ("src/commands/import.rs", include_str!("commands/import.rs")),
            ("src/commands/issues.rs", include_str!("commands/issues.rs")),
            ("src/mcp.rs", include_str!("mcp.rs")),
        ] {
            assert!(
                !source.contains("Command::new(\"gh\")")
                    && !source.contains("process::Command::new(\"gh\")"),
                "{path} must not construct a gh process"
            );
        }
    }

    #[test]
    fn token_present_read_list_verify_and_import_paths_never_spawn_gh() {
        const CHILD_ENV: &str = "SPECSYNC_NO_SUBPROCESS_CHILD";
        const API_ENV: &str = "SPECSYNC_TEST_GITHUB_API_BASE";
        const SENTINEL_ENV: &str = "SPECSYNC_GH_SENTINEL";

        if std::env::var_os(CHILD_ENV).is_some() {
            fetch_issue("owner/repo", 42)
                .expect_err("typed read must fail through the unreachable REST endpoint");
            list_issues("owner/repo", None)
                .expect_err("listing must fail through the unreachable REST endpoint");

            let references = [("specs/auth/auth.spec.md".to_string(), vec![42], Vec::new())];
            let verified = verify_issue_batch("owner/repo", &references);
            assert_eq!(verified.len(), 1);
            assert!(verified[0].valid.is_empty());
            assert_eq!(verified[0].errors.len(), 1);

            crate::importer::import_github_issue("owner/repo", 42)
                .expect_err("import must fail through the shared typed REST reader");
            return;
        }

        let temp = tempfile::TempDir::new().unwrap();
        let sentinel = temp.path().join("gh-spawned");
        #[cfg(not(windows))]
        let fake_gh = temp.path().join("gh");
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::write(
                &fake_gh,
                "#!/bin/sh\n: > \"$SPECSYNC_GH_SENTINEL\"\nexit 97\n",
            )
            .unwrap();
            let mut permissions = std::fs::metadata(&fake_gh).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&fake_gh, permissions).unwrap();
        }

        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "github::tests::token_present_read_list_verify_and_import_paths_never_spawn_gh",
                "--nocapture",
            ])
            .env(CHILD_ENV, "1")
            .env(API_ENV, "http://127.0.0.1:0")
            .env(SENTINEL_ENV, &sentinel)
            .env("GITHUB_TOKEN", "test-token")
            .env("PATH", temp.path())
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "isolated provider-path test failed:\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        #[cfg(not(windows))]
        assert!(
            !sentinel.exists(),
            "read/list/verify/import must never execute a gh provider"
        );
    }

    #[test]
    fn gh_issue_reads_fail_closed_without_spawning_a_provider() {
        let error =
            fetch_issue_gh("owner/repo", 42).expect_err("legacy gh issue reads must be disabled");

        assert!(error.contains("disabled"));
        assert!(error.contains("GITHUB_TOKEN"));
        assert!(error.contains("complete-tree subprocess containment"));
    }

    #[test]
    fn parse_commit_check_summary_aggregates_green_pending_failed() {
        let green = serde_json::json!({
            "total_count": 2,
            "check_runs": [
                {"name": "trust", "status": "completed", "conclusion": "success"},
                {"name": "SpecSync implementation ready", "status": "completed", "conclusion": "success"}
            ]
        });
        let summary = parse_commit_check_summary("owner/repo", "abc", &green).unwrap();
        assert_eq!(summary.overall, "green");
        assert_eq!(summary.check_runs.len(), 2);
        assert!(is_trust_relevant_check_name("trust"));
        assert!(is_trust_relevant_check_name(
            "SpecSync implementation ready"
        ));

        let pending = serde_json::json!({
            "check_runs": [
                {"name": "trust", "status": "in_progress", "conclusion": null}
            ]
        });
        assert_eq!(
            parse_commit_check_summary("owner/repo", "abc", &pending)
                .unwrap()
                .overall,
            "pending"
        );

        let failed = serde_json::json!({
            "check_runs": [
                {"name": "trust", "status": "completed", "conclusion": "cancelled"}
            ]
        });
        assert_eq!(
            parse_commit_check_summary("owner/repo", "abc", &failed)
                .unwrap()
                .overall,
            "failed"
        );

        let empty = serde_json::json!({"check_runs": []});
        assert_eq!(
            parse_commit_check_summary("owner/repo", "abc", &empty)
                .unwrap()
                .overall,
            "empty"
        );
    }
}
