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

    /// Output format: text, json, markdown, github, table, or csv
    #[arg(long, value_enum, global = true, default_value = "text")]
    pub format: types::OutputFormat,

    /// Output results as JSON (shorthand for --format json)
    #[arg(long, global = true)]
    pub json: bool,

    /// Enforcement mode: warn (default, exit 0), enforce-new (block unspecced files), strict (exit 1 on errors).
    /// Overrides the `enforcement` field in .specsync/config.toml.
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
        /// Preview --fix changes without writing files
        #[arg(long)]
        dry_run: bool,
        /// Create .bak backup files before --fix modifies specs
        #[arg(long)]
        backup: bool,
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
        /// Generate specs for all unspecced modules (default behavior, made explicit)
        #[arg(long)]
        uncovered: bool,
        /// Generate specs only for these specific modules (space or comma-separated list).
        /// Skips modules that already have specs. Ignores modules not found in coverage report.
        #[arg(long, value_name = "MODULE", num_args(1..))]
        batch: Vec<String>,
    },
    /// Create .specsync/config.toml and initialize the verified SDD layout
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
    /// Scaffold a new spec with required companion files and optional design.md
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
    /// Manage native skill/slash-command files for AI coding tools (Claude Code, Cursor, Codex, Gemini CLI)
    Agents {
        #[command(subcommand)]
        action: AgentsAction,
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
        /// Also create required companions (requirements.md, tasks.md, context.md, testing.md)
        /// and optional design.md when design artifacts are enabled
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
    /// Regenerate the hash cache for all specs (useful after git pull or manual edits)
    Rehash,
    /// Migrate a spec-sync project from v3.x to v4.0.0
    Migrate {
        /// Preview migration without writing any files
        #[arg(long)]
        dry_run: bool,
        /// Skip backup creation (not recommended)
        #[arg(long)]
        no_backup: bool,
    },
    /// Manage spec lifecycle statuses (promote, demote, set, status)
    Lifecycle {
        #[command(subcommand)]
        action: LifecycleAction,
    },
    /// Manage verified spec-driven development change workspaces
    Change {
        #[command(subcommand)]
        action: ChangeAction,
    },
}

#[derive(Subcommand)]
pub enum ChangeAction {
    /// Create a draft change and return the deterministic interview
    New {
        /// Plain-language description of the intended change
        description: String,
        /// Change type: feature, bug-fix, refactor, migration, documentation, operations
        #[arg(long, default_value = "feature")]
        kind: String,
        /// Affected canonical spec module (repeatable)
        #[arg(long = "spec")]
        specs: Vec<String>,
        /// Affected repository path or prefix (repeatable)
        #[arg(long = "path")]
        paths: Vec<String>,
        /// Optional artifact to add to the adaptive selection (repeatable)
        #[arg(long = "artifact")]
        artifacts: Vec<String>,
        /// Declare that canonical specs do not change
        #[arg(long)]
        no_spec_change: bool,
        /// Required explanation when --no-spec-change is used
        #[arg(long)]
        rationale: Option<String>,
    },
    /// Answer one deterministic interview question
    Answer {
        /// Change ID
        id: String,
        /// Stable question ID returned by `change new` or `change show`
        question: String,
        /// Answer text; comma-separated values are accepted for list questions
        answer: String,
    },
    /// Declare deterministic ordering between active changes
    Depend {
        /// Change that owns the dependency
        id: String,
        /// Change ID that must be ordered first
        on: String,
    },
    /// List active changes
    List,
    /// Show one change, its gate health, and next questions
    Show {
        /// Change ID
        id: String,
    },
    /// Show lifecycle status for one change or all active changes
    Status {
        /// Optional change ID
        id: Option<String>,
    },
    /// Record the mandatory definition approval
    Approve {
        /// Change ID
        id: String,
        /// Human actor recorded in portable approval evidence
        #[arg(long)]
        actor: Option<String>,
        /// Optional approval note
        #[arg(long)]
        note: Option<String>,
    },
    /// Transition an approved change into implementation
    Start {
        /// Change ID
        id: String,
    },
    /// Run the configured verification gate and record evidence
    Verify {
        /// Change ID
        id: String,
    },
    /// Record closing approval and atomically apply semantic deltas
    Accept {
        /// Change ID
        id: String,
        /// Human actor recorded in portable approval evidence
        #[arg(long)]
        actor: Option<String>,
        /// Optional acceptance note
        #[arg(long)]
        note: Option<String>,
    },
    /// Move an accepted change into the immutable dated archive
    Archive {
        /// Change ID
        id: String,
    },
    /// Validate all active change workspaces and CI coverage
    Check,
    /// Adopt the 5.0 SDD lifecycle in an existing project
    Adopt {
        /// Preview adoption without writing files
        #[arg(long)]
        dry_run: bool,
        /// Import source: openspec or speckit (auto-detected when omitted)
        #[arg(long)]
        source: Option<String>,
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
    /// Show transition history for a spec
    History {
        /// Spec to show history for (module name, filename, or path)
        spec: String,
    },
    /// Dry-run guard evaluation — check if a transition would pass guards
    Guard {
        /// Spec to check (module name, filename, or path)
        spec: String,
        /// Target status to check (checks all valid transitions if omitted)
        target: Option<String>,
    },
    /// Auto-promote all specs that pass their guards
    AutoPromote {
        /// Show what would be promoted without writing files
        #[arg(long)]
        dry_run: bool,
    },
    /// CI enforcement — validate lifecycle rules, exit non-zero on violations
    Enforce {
        /// Require all specs to have a status field
        #[arg(long)]
        require_status: bool,
        /// Flag specs that exceed their max-age for the current status (configured in lifecycle.maxAge)
        #[arg(long)]
        max_age: bool,
        /// Require all specs to be in one of the allowed statuses (configured in lifecycle.allowedStatuses)
        #[arg(long)]
        allowed: bool,
        /// Run all enforcement checks (equivalent to --require-status --max-age --allowed)
        #[arg(long)]
        all: bool,
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

#[derive(Subcommand)]
pub enum AgentsAction {
    /// Install native skill/command files for AI coding tools
    Install {
        /// Install Claude Code skill + /specsync:create-spec command
        #[arg(long)]
        claude: bool,
        /// Install Cursor skill + /specsync-create-spec command
        #[arg(long)]
        cursor: bool,
        /// Install Codex CLI skill (project-scoped, .codex/skills/)
        #[arg(long)]
        codex: bool,
        /// Install Gemini CLI /specsync:create-spec command
        #[arg(long)]
        gemini: bool,
    },
    /// Remove previously installed skill/command files
    Uninstall {
        /// Remove Claude Code skill + command
        #[arg(long)]
        claude: bool,
        /// Remove Cursor skill + command
        #[arg(long)]
        cursor: bool,
        /// Remove Codex CLI skill
        #[arg(long)]
        codex: bool,
        /// Remove Gemini CLI command
        #[arg(long)]
        gemini: bool,
    },
    /// Show installation status of all agent tools
    Status,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_subcommand_yields_none_and_text_default() {
        let cli = Cli::try_parse_from(["specsync"]).unwrap();
        assert!(cli.command.is_none());
        assert!(!cli.strict);
        assert_eq!(cli.format, types::OutputFormat::Text);
        assert!(cli.require_coverage.is_none());
    }

    #[test]
    fn global_flags_parse_before_subcommand() {
        let cli = Cli::try_parse_from([
            "specsync",
            "--strict",
            "--require-coverage",
            "80",
            "coverage",
        ])
        .unwrap();
        assert!(cli.strict);
        assert_eq!(cli.require_coverage, Some(80));
        assert!(matches!(cli.command, Some(Command::Coverage)));
    }

    #[test]
    fn json_format_value_enum_parses() {
        let cli = Cli::try_parse_from(["specsync", "--format", "json", "check"]).unwrap();
        assert_eq!(cli.format, types::OutputFormat::Json);
    }

    #[test]
    fn check_collects_flags_and_positional_specs() {
        let cli = Cli::try_parse_from(["specsync", "check", "--fix", "--dry-run", "cli", "parser"])
            .unwrap();
        match cli.command {
            Some(Command::Check {
                fix,
                dry_run,
                specs,
                ..
            }) => {
                assert!(fix);
                assert!(dry_run);
                assert_eq!(specs, vec!["cli".to_string(), "parser".to_string()]);
            }
            _ => panic!("expected a Check command"),
        }
    }

    #[test]
    fn stale_threshold_defaults_and_overrides() {
        let default = Cli::try_parse_from(["specsync", "stale"]).unwrap();
        assert!(matches!(
            default.command,
            Some(Command::Stale { threshold: 5 })
        ));

        let overridden = Cli::try_parse_from(["specsync", "stale", "--threshold", "10"]).unwrap();
        assert!(matches!(
            overridden.command,
            Some(Command::Stale { threshold: 10 })
        ));
    }

    #[test]
    fn exclude_status_splits_on_commas() {
        let cli = Cli::try_parse_from([
            "specsync",
            "--exclude-status",
            "deprecated,archived",
            "check",
        ])
        .unwrap();
        assert_eq!(cli.exclude_status, vec!["deprecated", "archived"]);
    }

    #[test]
    fn unknown_subcommand_is_rejected() {
        assert!(Cli::try_parse_from(["specsync", "definitely-not-a-command"]).is_err());
    }

    #[test]
    fn non_numeric_threshold_is_rejected() {
        assert!(Cli::try_parse_from(["specsync", "stale", "--threshold", "abc"]).is_err());
    }

    #[test]
    fn change_new_collects_sdd_scope() {
        let cli = Cli::try_parse_from([
            "specsync",
            "change",
            "new",
            "Add passkeys",
            "--kind",
            "feature",
            "--spec",
            "auth",
            "--path",
            "src/auth.rs",
        ])
        .unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Change {
                action: ChangeAction::New { .. }
            })
        ));
    }
}
