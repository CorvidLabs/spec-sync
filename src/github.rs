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
                Some(token) => verify_api_repository(repo, token, recheck_deadline),
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
struct GitHubIssueListPayload {
    #[serde(flatten)]
    issue: GitHubIssuePayload,
    pull_request: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(serde::Deserialize)]
struct GitHubRepositoryPayload {
    full_name: String,
}

struct GitHubIssueListPage {
    issues: Vec<GitHubIssue>,
    has_next: bool,
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
        verify_api_repository(repo, &token, recheck_deadline)
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

fn verify_api_repository(repo: &str, token: &str, deadline: Duration) -> Result<(), String> {
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
    Ok(())
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
    if payload.number != requested_number {
        return Err(format!(
            "GitHub issue response returned #{} for requested issue #{requested_number}",
            payload.number
        ));
    }
    let state = payload.state.to_ascii_lowercase();
    if state != "open" && state != "closed" {
        return Err(format!("GitHub issue response has invalid state `{state}`"));
    }
    if payload.title.trim().is_empty() {
        return Err("GitHub issue response has an empty `title`".to_string());
    }
    if !is_valid_github_issue_url(&payload.html_url, repo, requested_number) {
        return Err("GitHub issue response has an invalid `html_url`".to_string());
    }
    if payload
        .labels
        .iter()
        .any(|label| label.name.trim().is_empty())
    {
        return Err("GitHub issue response contains an empty label name".to_string());
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

fn is_valid_github_issue_url(url: &str, repo: &str, number: u64) -> bool {
    let Some(path) = url.strip_prefix("https://github.com/") else {
        return false;
    };
    let Some((expected_owner, expected_repository)) = repo.split_once('/') else {
        return false;
    };
    let mut segments = path.split('/');
    matches!(segments.next(), Some(owner) if owner.eq_ignore_ascii_case(expected_owner))
        && matches!(segments.next(), Some(repository) if repository.eq_ignore_ascii_case(expected_repository))
        && segments.next() == Some("issues")
        && segments.next().and_then(|value| value.parse::<u64>().ok()) == Some(number)
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
    collect_issue_pages(|page| fetch_issue_list_page(repo, label, &token, page))
}

fn parse_issue_list_json(repo: &str, body: &serde_json::Value) -> Result<Vec<GitHubIssue>, String> {
    let entries = body
        .as_array()
        .ok_or_else(|| "GitHub issue-list response must be a JSON array".to_string())?;
    for entry in entries {
        match entry.get("pull_request") {
            None | Some(serde_json::Value::Null | serde_json::Value::Object(_)) => {}
            Some(_) => {
                return Err(
                    "GitHub issue-list response has a malformed pull_request marker".to_string(),
                );
            }
        }
    }
    let payloads: Vec<GitHubIssueListPayload> = serde_json::from_value(body.clone())
        .map_err(|error| format!("Malformed GitHub issue-list response: {error}"))?;
    let mut issues = Vec::with_capacity(payloads.len());
    for payload in payloads {
        if payload.pull_request.is_some() {
            continue;
        }
        let number = payload.issue.number;
        issues.push(issue_from_payload(repo, number, payload.issue)?);
    }
    Ok(issues)
}

fn fetch_issue_list_page(
    repo: &str,
    label: Option<&str>,
    token: &str,
    page: usize,
) -> Result<GitHubIssueListPage, String> {
    let agent = github_api_agent(GITHUB_COMMAND_DEADLINE);
    let url = format!("{}/repos/{repo}/issues", github_api_base());
    let page_string = page.to_string();
    let mut request = agent
        .get(&url)
        .query("state", "open")
        .query("per_page", "100")
        .query("page", &page_string);
    if let Some(label) = label {
        request = request.query("labels", label);
    }

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
        verify_api_repository(repo, token, GITHUB_COMMAND_DEADLINE)
    })?;
    let has_next = response
        .headers()
        .get("link")
        .map(|value| {
            value
                .to_str()
                .map_err(|_| "GitHub issue-list Link header is not valid text".to_string())
                .and_then(|header| parse_link_has_next(header, repo, label, page))
        })
        .transpose()?
        .unwrap_or(false);
    let body: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|error| format!("Failed to parse GitHub issue-list page {page}: {error}"))?;
    let issues = parse_issue_list_json(repo, &body)?;
    Ok(GitHubIssueListPage { issues, has_next })
}

fn collect_issue_pages(
    mut fetch_page: impl FnMut(usize) -> Result<GitHubIssueListPage, String>,
) -> Result<Vec<GitHubIssue>, String> {
    let mut issues = Vec::new();
    let mut seen = BTreeSet::new();
    for page in 1..=MAX_ISSUE_LIST_PAGES {
        let result = fetch_page(page)?;
        for issue in result.issues {
            if !seen.insert(issue.number) {
                return Err(format!(
                    "GitHub issue-list pagination returned duplicate issue #{}",
                    issue.number
                ));
            }
            issues.push(issue);
        }
        if !result.has_next {
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

fn parse_link_has_next(
    header: &str,
    repo: &str,
    label: Option<&str>,
    current_page: usize,
) -> Result<bool, String> {
    if header.trim().is_empty() {
        return Err("GitHub issue-list Link header is empty".to_string());
    }
    let mut has_next = false;
    for entry in header.split(',') {
        let mut parts = entry.trim().split(';');
        let target = parts.next().unwrap_or_default().trim();
        let target_page = parse_github_link_target_page(target, repo, label)?;
        let mut relation_seen = false;
        for parameter in parts {
            let Some((name, value)) = parameter.trim().split_once('=') else {
                return Err("GitHub issue-list Link header has a malformed parameter".to_string());
            };
            if !name.trim().eq_ignore_ascii_case("rel") {
                continue;
            }
            if relation_seen {
                return Err("GitHub issue-list Link header repeats a relation".to_string());
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
                if has_next {
                    return Err("GitHub issue-list Link header repeats the next page".to_string());
                }
                let expected_page = current_page
                    .checked_add(1)
                    .ok_or_else(|| "GitHub issue-list pagination page overflowed".to_string())?;
                if target_page != expected_page {
                    return Err(format!(
                        "GitHub issue-list Link header points to page {target_page} instead of expected page {expected_page}"
                    ));
                }
                has_next = true;
            }
        }
        if !relation_seen {
            return Err("GitHub issue-list Link header is missing a relation".to_string());
        }
    }
    Ok(has_next)
}

fn parse_github_link_target_page(
    target: &str,
    repo: &str,
    label: Option<&str>,
) -> Result<usize, String> {
    let target = target
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .ok_or_else(|| "GitHub issue-list Link header has a malformed target".to_string())?;
    let target = target
        .strip_prefix("https://api.github.com/")
        .ok_or_else(|| {
            "GitHub issue-list Link header target is outside the GitHub API".to_string()
        })?;
    let (path, query) = target
        .split_once('?')
        .ok_or_else(|| "GitHub issue-list Link header target is missing a query".to_string())?;
    let expected_path = format!("repos/{repo}/issues");
    if path != expected_path {
        return Err(format!(
            "GitHub issue-list Link header target does not match the requested repository endpoint /{expected_path}"
        ));
    }
    let mut page = None;
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
                let parsed_page = value.parse::<usize>().map_err(|_| {
                    "GitHub issue-list Link header target has an invalid page query".to_string()
                })?;
                if parsed_page == 0 {
                    return Err(
                        "GitHub issue-list Link header target has an invalid page query"
                            .to_string(),
                    );
                }
                page = Some(parsed_page);
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
    page.ok_or_else(|| "GitHub issue-list Link header target is missing the page query".to_string())
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
        let issues =
            parse_issue_list_json("owner/repo", &valid).expect("valid issue lists must parse");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].number, 42);

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
        assert!(
            parse_link_has_next(paginated, "owner/repo", None, 1)
                .expect("GitHub Link header must parse")
        );
        assert!(
            !parse_link_has_next(
                "<https://api.github.com/repos/owner/repo/issues?state=open&per_page=100&page=1>; rel=\"prev\"",
                "owner/repo",
                None,
                2,
            )
            .expect("previous-only Link header must parse")
        );
        assert!(
            parse_link_has_next(
                "https://example.test/page/2; rel=\"next\"",
                "owner/repo",
                None,
                1,
            )
            .is_err()
        );
        assert!(
            parse_link_has_next("<https://example.test/page/2>", "owner/repo", None, 1,).is_err()
        );
        assert!(
            parse_link_has_next(
                "<https://example.test/page/2>; rel=\"next",
                "owner/repo",
                None,
                1,
            )
            .is_err()
        );
        assert!(
            parse_link_has_next(
                "<https://api.github.com/repos/owner/repo/issues?state=open&per_page=100&page=3>; rel=\"next\"",
                "owner/repo",
                None,
                1,
            )
            .is_err()
        );
    }

    #[test]
    fn link_header_rejects_wrong_repository_or_resource_path() {
        for target in [
            "https://api.github.com/repos/other/repo/issues?state=open&per_page=100&page=2",
            "https://api.github.com/repos/owner/repo/pulls?state=open&per_page=100&page=2",
            "https://api.github.com/repositories/1/issues?state=open&per_page=100&page=2",
        ] {
            let header = format!("<{target}>; rel=\"next\"");
            let error = parse_link_has_next(&header, "owner/repo", None, 1)
                .expect_err("Link targets must remain on the requested issue-list endpoint");
            assert!(error.contains("requested repository endpoint"));
        }
    }

    #[test]
    fn link_header_rejects_query_mismatch() {
        for query in [
            "state=closed&per_page=100&page=2",
            "state=open&per_page=50&page=2",
            "state=open&per_page=100&page=2&sort=created",
            "state=open&per_page=100&page=2&labels=bug",
        ] {
            let header =
                format!("<https://api.github.com/repos/owner/repo/issues?{query}>; rel=\"next\"");
            assert!(parse_link_has_next(&header, "owner/repo", None, 1).is_err());
        }

        let matching_label = concat!(
            "<https://api.github.com/repos/owner/repo/issues?",
            "labels=needs%20triage&page=2&state=open&per_page=100>; rel=\"next\""
        );
        assert!(
            parse_link_has_next(matching_label, "owner/repo", Some("needs triage"), 1)
                .expect("the requested encoded label must be preserved")
        );

        let mismatched_label = concat!(
            "<https://api.github.com/repos/owner/repo/issues?",
            "labels=bug&page=2&state=open&per_page=100>; rel=\"next\""
        );
        assert!(
            parse_link_has_next(mismatched_label, "owner/repo", Some("needs triage"), 1).is_err()
        );
    }

    #[test]
    fn issue_list_pagination_collects_every_page_in_order() {
        let mut requested_pages = Vec::new();
        let issues = collect_issue_pages(|page| {
            requested_pages.push(page);
            Ok(GitHubIssueListPage {
                issues: vec![fake_issue(page as u64, "open")],
                has_next: page < 3,
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
            Ok(GitHubIssueListPage {
                issues: vec![fake_issue(page as u64, "open")],
                has_next: true,
            })
        })
        .expect_err("a next page beyond the cap must be explicit");
        assert!(limit_error.contains("safety limit"));

        let duplicate_error = collect_issue_pages(|page| {
            Ok(GitHubIssueListPage {
                issues: vec![fake_issue(42, "open")],
                has_next: page == 1,
            })
        })
        .expect_err("duplicate pages must fail closed");
        assert!(duplicate_error.contains("duplicate issue #42"));
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
}
