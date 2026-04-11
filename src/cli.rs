use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::types;

#[derive(Parser)]
#[command(
    name = "specsync",
    about = "Bidirectional spec-to-code validation — language-agnostic, blazing fast",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Treat warnings as errors
    #[arg(long, global = true)]
    pub strict: bool,

    /// Fail if file coverage percent is below this threshold
    #[arg(long, value_name = "N", global = true)]
    pub require_coverage: Option<usize>,

    /// Project root directory (default: cwd)
    #[arg(long, global = true)]
    pub root: Option<PathBuf>,

    /// Output format: text (default), json, or markdown
    #[arg(long, value_enum, global = true, default_value = "text")]
    pub format: types::OutputFormat,

    /// Output results as JSON (shorthand for --format json)
    #[arg(long, global = true)]
    pub json: bool,

    /// Enforcement mode: warn (default, exit 0), enforce-new (block unspecced files), strict (exit 1 on errors).
    /// Overrides the `enforcement` field in specsync.json.
    #[arg(long, value_name = "MODE", global = true)]
    pub enforcement: Option<types::EnforcementMode>,

    /// Exclude specs with these statuses (comma-separated, e.g. "deprecated,archived")
    #[arg(long, value_name = "STATUSES", global = true, value_delimiter = ',')]
    pub exclude_status: Vec<String>,

    /// Only include specs with these statuses (comma-separated, e.g. "active,stable")
    #[arg(long, value_name = "STATUSES", global = true, value_delimiter = ',')]
    pub only_status: Vec<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Validate all specs against source code (default)
    Check {
        /// Auto-add undocumented exports to spec Public API tables
        #[arg(long)]
        fix: bool,
        /// Skip hash cache and re-validate all specs
        #[arg(long, visible_alias = "no-cache")]
        force: bool,
        /// Create GitHub issues for specs with validation errors
        #[arg(long)]
        create_issues: bool,
        /// Show per-category score breakdown explaining why each spec lost points
        #[arg(long)]
        explain: bool,
        /// Include git-based staleness warnings (specs behind source by N+ commits)
        #[arg(long)]
        stale: Option<Option<usize>>,
        /// Spec filters — validates all if omitted. Matches by: module name (e.g. "cli"),
        /// filename stem ("cli.spec"), relative path ("specs/cli/cli.spec.md"), or absolute path.
        #[arg(value_name = "SPEC")]
        specs: Vec<String>,
    },
    /// Show file and module coverage report
    Coverage,
    /// Scaffold spec files for unspecced modules
    Generate {
        /// AI provider to use for spec generation. Without this flag, specs are
        /// generated from templates only.
        ///
        /// Use "auto" to auto-detect an installed provider, or specify one:
        /// claude, anthropic, openai, ollama, copilot.
        #[arg(long, value_name = "PROVIDER")]
        provider: Option<String>,
        /// Generate specs for all unspecced modules (default behavior, made explicit)
        #[arg(long)]
        uncovered: bool,
        /// Generate specs only for these specific modules (space or comma-separated list).
        /// Skips modules that already have specs. Ignores modules not found in coverage report.
        #[arg(long, value_name = "MODULE", num_args(1..))]
        batch: Vec<String>,
    },
    /// Create a specsync.json config file
    Init,
    /// Score spec quality (0-100) with letter grades and improvement suggestions
    Score {
        /// Show detailed per-category breakdown explaining exactly why each spec lost points
        #[arg(long)]
        explain: bool,
        /// Score all specs (default when no filters provided; enables batch summary stats)
        #[arg(long)]
        all: bool,
        /// Spec filters — scores all if omitted. Matches by: module name (e.g. "cli"),
        /// filename stem ("cli.spec"), relative path ("specs/cli/cli.spec.md"), or absolute path.
        #[arg(value_name = "SPEC")]
        specs: Vec<String>,
    },
    /// Watch spec and source files, re-running check on changes
    Watch,
    /// Run as an MCP (Model Context Protocol) server over stdio
    Mcp,
    /// Scaffold a new spec with companion files (requirements.md, tasks.md, context.md)
    AddSpec {
        /// Module name for the new spec
        name: String,
    },
    /// Scaffold a new module spec with companion files, auto-detect source files, and register in registry
    Scaffold {
        /// Module name for the new spec
        name: String,
        /// Target directory for spec output (default: specs dir from config)
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Custom template directory containing spec.md, tasks.md, context.md, requirements.md
        #[arg(long)]
        template: Option<PathBuf>,
    },
    /// Generate a specsync-registry.toml for cross-project references
    InitRegistry {
        /// Project name for the registry
        #[arg(long)]
        name: Option<String>,
    },
    /// Resolve cross-project spec references in depends_on
    Resolve {
        /// Fetch remote specsync-registry.toml files from GitHub to verify
        /// cross-project references actually exist. Off by default — no
        /// network calls without this flag.
        #[arg(long)]
        remote: bool,
        /// Deep-verify remote spec content: fetch actual spec files, check
        /// exports still exist, validate bidirectional dependencies, and
        /// detect drift. Implies --remote. Exit 1 if drift is detected.
        #[arg(long)]
        verify: bool,
        /// Cache TTL in seconds for remote spec content (default: 3600 = 1 hour)
        #[arg(long, default_value = "3600")]
        cache_ttl: u64,
    },
    /// Show export changes since last commit (useful for CI/PR comments)
    Diff {
        /// Git ref to compare against (default: HEAD).
        /// In GitHub Actions PR context, auto-detects the base branch
        /// from GITHUB_BASE_REF when set to HEAD.
        #[arg(long, default_value = "HEAD")]
        base: String,
    },
    /// Manage agent instruction files and git hooks for spec awareness
    Hooks {
        #[command(subcommand)]
        action: HooksAction,
    },
    /// Compact changelog entries in spec files to prevent unbounded growth
    Compact {
        /// Keep the last N changelog entries (default: 10)
        #[arg(long, default_value = "10")]
        keep: usize,
        /// Show what would be compacted without writing files
        #[arg(long)]
        dry_run: bool,
    },
    /// Archive completed tasks from companion tasks.md files
    ArchiveTasks {
        /// Show what would be archived without writing files
        #[arg(long)]
        dry_run: bool,
    },
    /// View a spec filtered by role (dev, qa, product, agent)
    View {
        /// Role to filter by: dev, qa, product, agent
        #[arg(long)]
        role: String,
        /// Specific spec module to view (shows all if omitted)
        #[arg(long)]
        spec: Option<String>,
    },
    /// Auto-resolve git merge conflicts in spec files
    Merge {
        /// Show what would be resolved without writing files
        #[arg(long)]
        dry_run: bool,
        /// Scan all spec files for conflict markers (not just git-reported)
        #[arg(long)]
        all: bool,
    },
    /// Verify GitHub issue references in spec frontmatter
    Issues {
        /// Create issues for specs with drift/validation errors
        #[arg(long)]
        create: bool,
    },
    /// Quick-create a minimal spec for a module (auto-detects source files)
    New {
        /// Module name for the new spec
        name: String,
        /// Also create companion files (tasks.md, context.md, requirements.md)
        #[arg(long)]
        full: bool,
    },
    /// Interactive wizard for creating new specs step by step
    Wizard,
    /// Validate cross-module dependency graph (cycles, missing deps, undeclared imports)
    Deps {
        /// Output dependency graph as Mermaid diagram
        #[arg(long)]
        mermaid: bool,
        /// Output dependency graph as Graphviz DOT format
        #[arg(long)]
        dot: bool,
    },
    /// Import specs from external systems (GitHub Issues, Jira, Confluence)
    Import {
        /// Import source: github, jira, or confluence (required unless --all-issues or --from-dir)
        #[arg(value_name = "SOURCE")]
        source: Option<String>,
        /// Issue number, key, or page ID to import (e.g., 42, PROJ-123, or 98765)
        /// Required unless --all-issues or --from-dir is set.
        #[arg(value_name = "ID")]
        id: Option<String>,
        /// GitHub repo (owner/repo) — only for GitHub source; auto-detected if omitted
        #[arg(long)]
        repo: Option<String>,
        /// Import all open GitHub issues as spec drafts (batch mode)
        #[arg(long)]
        all_issues: bool,
        /// Filter issues by label when using --all-issues
        #[arg(long, value_name = "LABEL")]
        label: Option<String>,
        /// Bulk import all markdown files from a directory as spec drafts
        #[arg(long, value_name = "PATH")]
        from_dir: Option<PathBuf>,
    },
    /// Detect specs that have drifted from their source files (git-based)
    Stale {
        /// Flag specs whose source files have N+ commits since the spec was last updated
        #[arg(long, default_value = "5")]
        threshold: usize,
    },
    /// Per-module coverage report with stale and incomplete detection
    Report {
        /// Flag modules whose specs are N+ commits behind their source files
        #[arg(long, default_value = "5")]
        stale_threshold: usize,
    },
    /// Post a spec-sync check summary as a PR comment (or print for piping)
    Comment {
        /// Pull request number to comment on (omit to just print the comment body)
        #[arg(long)]
        pr: Option<u64>,
        /// Git ref to compare against for diff-aware suggestions (default: main)
        #[arg(long, default_value = "main")]
        base: String,
    },
    /// List active validation rules (built-in and custom)
    Rules,
    /// Generate a changelog of spec changes between two git refs
    Changelog {
        /// Git ref range (e.g., v0.1..v0.2, HEAD~5..HEAD)
        #[arg(value_name = "RANGE")]
        range: String,
    },
    /// Manage spec lifecycle statuses (promote, demote, set, status)
    Lifecycle {
        #[command(subcommand)]
        action: LifecycleAction,
    },
}

#[derive(Subcommand)]
pub enum LifecycleAction {
    /// Advance a spec to the next lifecycle status
    Promote {
        /// Spec to promote (module name, filename, or path)
        spec: String,
        /// Skip transition validation
        #[arg(long)]
        force: bool,
    },
    /// Move a spec back to the previous lifecycle status
    Demote {
        /// Spec to demote (module name, filename, or path)
        spec: String,
        /// Skip transition validation
        #[arg(long)]
        force: bool,
    },
    /// Set a spec to a specific lifecycle status
    Set {
        /// Spec to update (module name, filename, or path)
        spec: String,
        /// Target status (draft, review, active, stable, deprecated, archived)
        status: String,
        /// Skip transition validation
        #[arg(long)]
        force: bool,
    },
    /// Show lifecycle status of specs (all or filtered)
    Status {
        /// Specific spec to show (shows all if omitted)
        spec: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum HooksAction {
    /// Install agent instructions and/or git hooks
    Install {
        /// Install CLAUDE.md instructions
        #[arg(long)]
        claude: bool,
        /// Install .cursorrules instructions
        #[arg(long)]
        cursor: bool,
        /// Install .github/copilot-instructions.md
        #[arg(long)]
        copilot: bool,
        /// Install AGENTS.md instructions
        #[arg(long)]
        agents: bool,
        /// Install git pre-commit hook
        #[arg(long)]
        precommit: bool,
        /// Install Claude Code settings.json hook
        #[arg(long)]
        claude_code_hook: bool,
    },
    /// Remove previously installed hooks
    Uninstall {
        /// Remove CLAUDE.md instructions
        #[arg(long)]
        claude: bool,
        /// Remove .cursorrules instructions
        #[arg(long)]
        cursor: bool,
        /// Remove .github/copilot-instructions.md
        #[arg(long)]
        copilot: bool,
        /// Remove AGENTS.md instructions
        #[arg(long)]
        agents: bool,
        /// Remove git pre-commit hook
        #[arg(long)]
        precommit: bool,
        /// Remove Claude Code settings.json hook
        #[arg(long)]
        claude_code_hook: bool,
    },
    /// Show installation status of all hooks
    Status,
}
