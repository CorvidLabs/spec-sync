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
}

#[derive(Subcommand)]
pub enum Command {
    /// Validate all specs against source code (default)
    Check {
        /// Auto-add undocumented exports to spec Public API tables
        #[arg(long)]
        fix: bool,
        /// Skip hash cache and re-validate all specs
        #[arg(long)]
        force: bool,
        /// Create GitHub issues for specs with validation errors
        #[arg(long)]
        create_issues: bool,
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
    },
    /// Create a specsync.json config file
    Init,
    /// Score spec quality (0-100) with letter grades and improvement suggestions
    Score {
        /// Show detailed per-category breakdown explaining exactly why each spec lost points
        #[arg(long)]
        explain: bool,
    },
    /// Watch spec and source files, re-running check on changes
    Watch,
    /// Run as an MCP (Model Context Protocol) server over stdio
    Mcp,
    /// Scaffold a new spec with companion files (tasks.md, context.md)
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
    },
    /// Show export changes since last commit (useful for CI/PR comments)
    Diff {
        /// Git ref to compare against (default: HEAD)
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
    /// Interactive wizard for creating new specs step by step
    Wizard,
    /// Validate cross-module dependency graph (cycles, missing deps, undeclared imports)
    Deps,
    /// Import specs from external systems (GitHub Issues, Jira, Confluence)
    Import {
        /// Import source: github, jira, or confluence
        #[arg(value_name = "SOURCE")]
        source: String,
        /// Issue number, key, or page ID to import (e.g., 42, PROJ-123, or 98765)
        #[arg(value_name = "ID")]
        id: String,
        /// GitHub repo (owner/repo) — only for GitHub source; auto-detected if omitted
        #[arg(long)]
        repo: Option<String>,
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
    /// Generate a changelog of spec changes between two git refs
    Changelog {
        /// Git ref range (e.g., v0.1..v0.2, HEAD~5..HEAD)
        #[arg(value_name = "RANGE")]
        range: String,
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
