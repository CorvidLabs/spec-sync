use clap::ValueEnum;
use serde::Deserialize;
use std::fmt;

/// Supported AI provider presets.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    Claude,
    Cursor,
    Copilot,
    Ollama,
    /// Direct Anthropic API (no CLI needed — uses ANTHROPIC_API_KEY).
    Anthropic,
    /// Direct OpenAI-compatible API (no CLI needed — uses OPENAI_API_KEY).
    #[serde(alias = "openai")]
    OpenAi,
    /// Google Gemini API (no CLI needed — uses GEMINI_API_KEY).
    Gemini,
    /// DeepSeek API (OpenAI-compatible — uses DEEPSEEK_API_KEY).
    #[serde(alias = "deepseek")]
    DeepSeek,
    /// Groq API (OpenAI-compatible — uses GROQ_API_KEY).
    Groq,
    /// Mistral API (OpenAI-compatible — uses MISTRAL_API_KEY).
    Mistral,
    /// xAI Grok API (OpenAI-compatible — uses XAI_API_KEY).
    #[serde(alias = "xai", alias = "grok")]
    XAi,
    /// Together AI API (OpenAI-compatible — uses TOGETHER_API_KEY).
    Together,
    Custom,
}

impl AiProvider {
    /// The CLI command for this provider (if it has one).
    #[allow(dead_code)]
    pub fn default_command(&self) -> Option<&'static str> {
        match self {
            AiProvider::Claude => Some("claude -p --output-format text"),
            AiProvider::Ollama => Some("ollama run llama3"),
            AiProvider::Copilot => Some("gh copilot suggest -t shell"),
            AiProvider::Cursor
            | AiProvider::Custom
            | AiProvider::Anthropic
            | AiProvider::OpenAi
            | AiProvider::Gemini
            | AiProvider::DeepSeek
            | AiProvider::Groq
            | AiProvider::Mistral
            | AiProvider::XAi
            | AiProvider::Together => None,
        }
    }

    /// The binary name to check for availability (empty for API-only providers).
    pub fn binary_name(&self) -> &'static str {
        match self {
            AiProvider::Claude => "claude",
            AiProvider::Cursor => "cursor",
            AiProvider::Copilot => "gh",
            AiProvider::Ollama => "ollama",
            AiProvider::Anthropic
            | AiProvider::OpenAi
            | AiProvider::Gemini
            | AiProvider::DeepSeek
            | AiProvider::Groq
            | AiProvider::Mistral
            | AiProvider::XAi
            | AiProvider::Together
            | AiProvider::Custom => "",
        }
    }

    /// Whether this provider uses a direct API call (no CLI binary needed).
    pub fn is_api_provider(&self) -> bool {
        matches!(
            self,
            AiProvider::Anthropic
                | AiProvider::OpenAi
                | AiProvider::Gemini
                | AiProvider::DeepSeek
                | AiProvider::Groq
                | AiProvider::Mistral
                | AiProvider::XAi
                | AiProvider::Together
        )
    }

    /// The environment variable name for the API key.
    pub fn api_key_env_var(&self) -> Option<&'static str> {
        match self {
            AiProvider::Anthropic => Some("ANTHROPIC_API_KEY"),
            AiProvider::OpenAi => Some("OPENAI_API_KEY"),
            AiProvider::Gemini => Some("GEMINI_API_KEY"),
            AiProvider::DeepSeek => Some("DEEPSEEK_API_KEY"),
            AiProvider::Groq => Some("GROQ_API_KEY"),
            AiProvider::Mistral => Some("MISTRAL_API_KEY"),
            AiProvider::XAi => Some("XAI_API_KEY"),
            AiProvider::Together => Some("TOGETHER_API_KEY"),
            _ => None,
        }
    }

    /// Default model for API providers.
    pub fn default_model(&self) -> Option<&'static str> {
        match self {
            AiProvider::Anthropic => Some("claude-sonnet-4-20250514"),
            AiProvider::OpenAi => Some("gpt-4o"),
            AiProvider::Gemini => Some("gemini-2.5-flash"),
            AiProvider::DeepSeek => Some("deepseek-chat"),
            AiProvider::Groq => Some("llama-3.3-70b-versatile"),
            AiProvider::Mistral => Some("mistral-large-latest"),
            AiProvider::XAi => Some("grok-3-mini"),
            AiProvider::Together => Some("meta-llama/Llama-3.3-70B-Instruct-Turbo"),
            _ => None,
        }
    }

    /// Default base URL for OpenAI-compatible providers (None = use OpenAI default).
    pub fn default_base_url(&self) -> Option<&'static str> {
        match self {
            AiProvider::DeepSeek => Some("https://api.deepseek.com"),
            AiProvider::Groq => Some("https://api.groq.com/openai"),
            AiProvider::Mistral => Some("https://api.mistral.ai"),
            AiProvider::XAi => Some("https://api.x.ai"),
            AiProvider::Together => Some("https://api.together.xyz"),
            _ => None,
        }
    }

    /// Parse a provider name from a string (for CLI flag).
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(AiProvider::Claude),
            "cursor" => Some(AiProvider::Cursor),
            "copilot" | "gh-copilot" => Some(AiProvider::Copilot),
            "ollama" => Some(AiProvider::Ollama),
            "anthropic" | "anthropic-api" => Some(AiProvider::Anthropic),
            "openai" | "openai-api" => Some(AiProvider::OpenAi),
            "gemini" | "google" => Some(AiProvider::Gemini),
            "deepseek" => Some(AiProvider::DeepSeek),
            "groq" => Some(AiProvider::Groq),
            "mistral" => Some(AiProvider::Mistral),
            "xai" | "grok" | "x-ai" => Some(AiProvider::XAi),
            "together" | "together-ai" => Some(AiProvider::Together),
            _ => None,
        }
    }

    /// All providers that can be auto-detected, in alphabetical order within
    /// each category. CLI providers are checked first (binary detection),
    /// then API providers (env var detection). No vendor preference.
    pub fn detection_order() -> &'static [AiProvider] {
        &[
            // CLI providers (binary detection) — alphabetical
            AiProvider::Claude,
            AiProvider::Copilot,
            AiProvider::Ollama,
            // API providers (env var detection) — alphabetical
            AiProvider::Anthropic,
            AiProvider::DeepSeek,
            AiProvider::Gemini,
            AiProvider::Groq,
            AiProvider::Mistral,
            AiProvider::OpenAi,
            AiProvider::Together,
            AiProvider::XAi,
        ]
    }
}

impl fmt::Display for AiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiProvider::Claude => write!(f, "claude"),
            AiProvider::Cursor => write!(f, "cursor"),
            AiProvider::Copilot => write!(f, "copilot"),
            AiProvider::Ollama => write!(f, "ollama"),
            AiProvider::Anthropic => write!(f, "anthropic"),
            AiProvider::OpenAi => write!(f, "openai"),
            AiProvider::Gemini => write!(f, "gemini"),
            AiProvider::DeepSeek => write!(f, "deepseek"),
            AiProvider::Groq => write!(f, "groq"),
            AiProvider::Mistral => write!(f, "mistral"),
            AiProvider::XAi => write!(f, "xai"),
            AiProvider::Together => write!(f, "together"),
            AiProvider::Custom => write!(f, "custom"),
        }
    }
}

/// Output format for CLI commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum OutputFormat {
    /// Colored terminal output (default)
    #[default]
    Text,
    /// Machine-readable JSON
    Json,
    /// Markdown suitable for PR comments and agent consumption
    Markdown,
    /// GitHub-flavored markdown with spec links, actionable suggestions, and checklists
    Github,
    /// ASCII table (useful for score --all --format table)
    Table,
    /// CSV output (useful for score --all --format csv, dashboards)
    Csv,
}

/// Valid spec lifecycle statuses.
///
/// Lifecycle order: draft → review → active → stable → deprecated → archived
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecStatus {
    Draft,
    Review,
    Active,
    Stable,
    Deprecated,
    Archived,
}

impl SpecStatus {
    /// Parse a status string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "draft" => Some(Self::Draft),
            "review" => Some(Self::Review),
            "active" => Some(Self::Active),
            "stable" => Some(Self::Stable),
            "deprecated" => Some(Self::Deprecated),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Review => "review",
            Self::Active => "active",
            Self::Stable => "stable",
            Self::Deprecated => "deprecated",
            Self::Archived => "archived",
        }
    }

    /// All valid statuses in lifecycle order.
    pub fn all() -> &'static [SpecStatus] {
        &[
            Self::Draft,
            Self::Review,
            Self::Active,
            Self::Stable,
            Self::Deprecated,
            Self::Archived,
        ]
    }

    /// Lifecycle ordinal (0-based) for transition logic.
    pub fn ordinal(&self) -> usize {
        match self {
            Self::Draft => 0,
            Self::Review => 1,
            Self::Active => 2,
            Self::Stable => 3,
            Self::Deprecated => 4,
            Self::Archived => 5,
        }
    }

    /// Next status in the lifecycle, or None if already at the end.
    pub fn next(&self) -> Option<Self> {
        let all = Self::all();
        let idx = self.ordinal();
        all.get(idx + 1).copied()
    }

    /// Previous status in the lifecycle, or None if already at the start.
    pub fn prev(&self) -> Option<Self> {
        let idx = self.ordinal();
        if idx == 0 {
            return None;
        }
        Some(Self::all()[idx - 1])
    }

    /// Valid transitions from this status.
    /// Forward: one step up. Backward: one step down.
    /// Special: any status can go to deprecated; deprecated can go to archived.
    pub fn valid_transitions(&self) -> Vec<Self> {
        let mut transitions = Vec::new();
        if let Some(next) = self.next() {
            transitions.push(next);
        }
        if let Some(prev) = self.prev() {
            transitions.push(prev);
        }
        // Any status can be deprecated directly
        if *self != Self::Deprecated
            && *self != Self::Archived
            && !transitions.contains(&Self::Deprecated)
        {
            transitions.push(Self::Deprecated);
        }
        transitions
    }

    /// Check if transitioning to `target` is valid.
    pub fn can_transition_to(&self, target: &Self) -> bool {
        self.valid_transitions().contains(target)
    }
}

/// YAML frontmatter parsed from a spec file.
#[derive(Debug, Default, Clone)]
pub struct Frontmatter {
    pub module: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub files: Vec<String>,
    pub db_tables: Vec<String>,
    pub depends_on: Vec<String>,
    pub agent_policy: Option<String>,
    /// GitHub issue numbers this spec implements (e.g., `[42, 57]`).
    pub implements: Vec<u64>,
    /// GitHub issue numbers for ongoing/epic-style tracking.
    pub tracks: Vec<u64>,
    /// Lifecycle transition history log entries (e.g. "2026-04-11: draft → review").
    pub lifecycle_log: Vec<String>,
}

impl Frontmatter {
    /// Parse the status field into a typed enum.
    pub fn parsed_status(&self) -> Option<SpecStatus> {
        self.status.as_deref().and_then(SpecStatus::from_str_loose)
    }
}

/// Result of validating a single spec.
#[derive(Debug)]
pub struct ValidationResult {
    pub spec_path: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub export_summary: Option<String>,
    /// Actionable fix suggestions mapped to errors.
    pub fixes: Vec<String>,
}

impl ValidationResult {
    pub fn new(spec_path: String) -> Self {
        Self {
            spec_path,
            errors: Vec::new(),
            warnings: Vec::new(),
            export_summary: None,
            fixes: Vec::new(),
        }
    }
}

/// Coverage report for the project.
#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub total_source_files: usize,
    pub specced_file_count: usize,
    pub unspecced_files: Vec<String>,
    pub unspecced_modules: Vec<String>,
    pub coverage_percent: usize,
    pub total_loc: usize,
    pub specced_loc: usize,
    pub loc_coverage_percent: usize,
    /// (file_path, line_count) sorted by LOC descending.
    pub unspecced_file_loc: Vec<(String, usize)>,
}

/// Controls export extraction granularity.
/// - `type`: Only top-level type declarations (class, struct, enum, protocol, trait, etc.)
/// - `member`: Every public symbol including members (functions, properties, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExportLevel {
    /// Only top-level type declarations (class, struct, enum, protocol, trait, etc.)
    Type,
    /// Every public symbol including members (default for backwards compatibility).
    #[default]
    Member,
}

/// Controls which parser backend to use for export extraction.
/// - `regex` (default): Fast regex-based parsing (current behavior).
/// - `ast`: Tree-sitter AST-based parsing for higher accuracy (TypeScript, Python, Rust only).
///   Falls back to regex for unsupported languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ParseMode {
    /// Regex-based parsing (default, all languages).
    #[default]
    Regex,
    /// AST-based parsing via tree-sitter (TypeScript, Python, Rust). Falls back to regex for others.
    Ast,
}

/// Controls how spec-sync responds to validation violations in CI.
///
/// - `warn` (default): report violations but always exit 0 (non-blocking).
/// - `enforce-new`: exit 1 only if files without specs exist (new files must be specced).
/// - `strict`: exit 1 on any validation error (blocking, opt-in).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum EnforcementMode {
    /// Report violations but always exit 0 (default, non-blocking).
    #[default]
    Warn,
    /// Exit 1 only if files without specs exist in the project.
    /// Existing specced files are not blocked even if they have errors.
    EnforceNew,
    /// Exit 1 on any validation error (strictest mode).
    Strict,
}

/// User-provided configuration (from specsync.json).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecSyncConfig {
    #[serde(default = "default_specs_dir")]
    pub specs_dir: String,

    #[serde(default = "default_source_dirs")]
    pub source_dirs: Vec<String>,

    pub schema_dir: Option<String>,
    pub schema_pattern: Option<String>,

    #[serde(default = "default_required_sections")]
    pub required_sections: Vec<String>,

    #[serde(default = "default_exclude_dirs")]
    pub exclude_dirs: Vec<String>,

    #[serde(default = "default_exclude_patterns")]
    pub exclude_patterns: Vec<String>,

    /// Source file extensions to scan (default: all supported languages).
    #[serde(default)]
    pub source_extensions: Vec<String>,

    /// Export granularity: "type" (top-level types only) or "member" (all public symbols).
    /// Default: "member" for backwards compatibility.
    #[serde(default)]
    pub export_level: ExportLevel,

    /// Parser backend: "regex" (default) or "ast" (tree-sitter, opt-in).
    /// AST mode supports TypeScript, Python, and Rust; other languages fall back to regex.
    #[serde(default)]
    pub parse_mode: ParseMode,

    /// Module definitions — override auto-detected modules with explicit groupings.
    /// Keys are module names, values are objects with `files` and optional `depends_on`.
    #[serde(default)]
    pub modules: std::collections::HashMap<String, ModuleDefinition>,

    /// AI provider preset: "claude", "cursor", "copilot", "ollama".
    /// Resolves to the correct CLI command automatically.
    #[serde(default)]
    pub ai_provider: Option<AiProvider>,

    /// Model name for the AI provider (e.g. "llama3" for ollama).
    #[serde(default)]
    pub ai_model: Option<String>,

    /// Command to run for AI-powered spec generation (overrides aiProvider).
    /// The prompt is piped to stdin; spec markdown is expected on stdout.
    /// Examples: "claude -p --output-format text", "ollama run llama3"
    #[serde(default)]
    pub ai_command: Option<String>,

    /// API key for direct API providers (anthropic, openai).
    /// Can also be set via ANTHROPIC_API_KEY or OPENAI_API_KEY env vars.
    #[serde(default)]
    pub ai_api_key: Option<String>,

    /// Base URL override for OpenAI-compatible APIs (e.g. local proxies).
    #[serde(default)]
    pub ai_base_url: Option<String>,

    /// Timeout in seconds for each AI command invocation (default: 120).
    #[serde(default)]
    pub ai_timeout: Option<u64>,

    /// Custom validation rules for project-specific lint checks.
    #[serde(default)]
    pub rules: ValidationRules,

    /// Declarative custom rules for flexible, user-defined validation.
    #[serde(default)]
    pub custom_rules: Vec<CustomRule>,

    /// Auto-archive completed tasks older than this many days.
    #[serde(default)]
    pub task_archive_days: Option<u32>,

    /// GitHub integration settings for linking specs to issues.
    #[serde(default)]
    pub github: Option<GitHubConfig>,

    /// Enforcement mode: controls how spec-sync responds to violations.
    /// - `warn` (default): report violations but always exit 0.
    /// - `enforce-new`: exit 1 if any files lack specs.
    /// - `strict`: exit 1 on any validation error.
    #[serde(default)]
    pub enforcement: EnforcementMode,

    /// Lifecycle transition guards — configurable rules that must pass before
    /// a spec can be promoted/transitioned.
    #[serde(default)]
    pub lifecycle: LifecycleConfig,
}

/// Lifecycle configuration for transition guards and history tracking.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleConfig {
    /// Transition guard rules keyed by "from→to" (e.g. "review→active").
    /// Use "*→<status>" to apply to all transitions into a status.
    #[serde(default)]
    pub guards: std::collections::HashMap<String, TransitionGuard>,

    /// Whether to record transitions in spec frontmatter (default: true).
    #[serde(default = "default_true")]
    pub track_history: bool,

    /// Maximum age (in days) a spec may stay in a given status before being flagged.
    /// Keys are status names (e.g. "draft": 30, "review": 14).
    #[serde(default)]
    pub max_age: std::collections::HashMap<String, u64>,

    /// Required statuses — specs must have one of these statuses, or `enforce` will flag them.
    /// Empty means no restriction.
    #[serde(default)]
    pub allowed_statuses: Vec<String>,
}

/// A transition guard — conditions that must be satisfied before a lifecycle
/// transition is allowed.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionGuard {
    /// Minimum spec quality score (0-100) required.
    #[serde(default)]
    pub min_score: Option<u32>,

    /// Sections that must exist and have non-empty content.
    #[serde(default)]
    pub require_sections: Vec<String>,

    /// Spec must not be stale (source files changed since spec was last updated).
    #[serde(default)]
    pub no_stale: Option<bool>,

    /// Maximum staleness threshold (commits behind) — only used when no_stale is true.
    #[serde(default)]
    pub stale_threshold: Option<usize>,

    /// Custom message shown when the guard blocks a transition.
    #[serde(default)]
    pub message: Option<String>,
}

/// GitHub integration configuration for linking specs to issues.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubConfig {
    /// Repository in `owner/repo` format (auto-detected from git remote if omitted).
    #[serde(default)]
    pub repo: Option<String>,
    /// Labels to apply when creating drift issues (default: `["spec-drift"]`).
    #[serde(default = "default_drift_labels")]
    pub drift_labels: Vec<String>,
    /// Whether to verify linked issues exist during `specsync check`.
    #[serde(default = "default_true")]
    pub verify_issues: bool,
}

/// Custom validation rules configurable per-project.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationRules {
    /// Warn if a spec's Change Log has more entries than this.
    #[serde(default)]
    pub max_changelog_entries: Option<usize>,
    /// Require at least one Behavioral Example scenario.
    #[serde(default)]
    pub require_behavioral_examples: Option<bool>,
    /// Minimum number of invariants required.
    #[serde(default)]
    pub min_invariants: Option<usize>,
    /// Warn if spec file exceeds this size in KB.
    #[serde(default)]
    pub max_spec_size_kb: Option<usize>,
    /// Require non-empty depends_on in frontmatter.
    #[serde(default)]
    pub require_depends_on: Option<bool>,
}

/// A declarative custom validation rule defined in specsync.json.
///
/// Supports four rule types:
/// - `require_section` — require a named `## Section` to exist
/// - `min_word_count` — require a section to have at least N words
/// - `require_pattern` — require a regex pattern to match somewhere in the spec body
/// - `forbid_pattern` — forbid a regex pattern from appearing in the spec body
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomRule {
    /// Human-readable rule name (e.g. "security-threat-model").
    pub name: String,
    /// Rule type: "require_section", "min_word_count", "require_pattern", "forbid_pattern".
    #[serde(rename = "type")]
    pub rule_type: CustomRuleType,
    /// Section name for `require_section` and `min_word_count` rules.
    #[serde(default)]
    pub section: Option<String>,
    /// Regex pattern for `require_pattern` and `forbid_pattern` rules.
    #[serde(default)]
    pub pattern: Option<String>,
    /// Minimum word count for `min_word_count` rules.
    #[serde(default)]
    pub min_words: Option<usize>,
    /// Severity level: "error", "warning", or "info" (default: "warning").
    #[serde(default)]
    pub severity: RuleSeverity,
    /// Custom message shown when the rule is violated.
    #[serde(default)]
    pub message: Option<String>,
    /// Optional filter — only apply to specs matching these criteria.
    #[serde(default)]
    pub applies_to: Option<RuleFilter>,
}

/// The type of a custom validation rule.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CustomRuleType {
    RequireSection,
    MinWordCount,
    RequirePattern,
    ForbidPattern,
}

/// Severity level for custom rules.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    Error,
    #[default]
    Warning,
    Info,
}

/// Filter to restrict which specs a custom rule applies to.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleFilter {
    /// Only apply to specs with this status (e.g. "active", "stable").
    #[serde(default)]
    pub status: Option<String>,
    /// Only apply to specs whose module name matches this regex.
    #[serde(default)]
    pub module: Option<String>,
}

/// A user-defined module grouping in specsync.json.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ModuleDefinition {
    /// Source files belonging to this module (relative to project root).
    #[serde(default)]
    pub files: Vec<String>,
    /// Other module names this module depends on.
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// Registry entry mapping module names to spec file paths.
/// Used in `specsync-registry.toml` for cross-project resolution.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RegistryEntry {
    pub name: String,
    pub specs: Vec<(String, String)>, // (module_name, spec_path)
}

/// Detected language for export extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    TypeScript,
    Rust,
    Go,
    Python,
    Swift,
    Kotlin,
    Java,
    CSharp,
    Dart,
    Php,
    Ruby,
    Yaml,
}

impl Language {
    /// Detect language from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" => Some(Language::TypeScript),
            "rs" => Some(Language::Rust),
            "go" => Some(Language::Go),
            "py" => Some(Language::Python),
            "swift" => Some(Language::Swift),
            "kt" | "kts" => Some(Language::Kotlin),
            "java" => Some(Language::Java),
            "cs" => Some(Language::CSharp),
            "dart" => Some(Language::Dart),
            "php" => Some(Language::Php),
            "rb" => Some(Language::Ruby),
            "yaml" | "yml" => Some(Language::Yaml),
            _ => None,
        }
    }

    /// Default source file extensions for this language.
    #[allow(dead_code)]
    pub fn extensions(&self) -> &[&str] {
        match self {
            Language::TypeScript => &["ts", "tsx", "js", "jsx", "mts", "cts"],
            Language::Rust => &["rs"],
            Language::Go => &["go"],
            Language::Python => &["py"],
            Language::Swift => &["swift"],
            Language::Kotlin => &["kt", "kts"],
            Language::Java => &["java"],
            Language::CSharp => &["cs"],
            Language::Dart => &["dart"],
            Language::Php => &["php"],
            Language::Ruby => &["rb"],
            Language::Yaml => &["yaml", "yml"],
        }
    }

    /// File patterns to exclude (test files, etc.).
    pub fn test_patterns(&self) -> &[&str] {
        match self {
            Language::TypeScript => &[".test.ts", ".spec.ts", ".test.tsx", ".spec.tsx", ".d.ts"],
            Language::Rust => &[], // Rust tests are inline, not separate files
            Language::Go => &["_test.go"],
            Language::Python => &["test_", "_test.py"],
            Language::Swift => &[
                "Tests.swift",
                "Test.swift",
                "Spec.swift",
                "Specs.swift",
                "Mock.swift",
                "Mocks.swift",
                "Stub.swift",
                "Fake.swift",
            ],
            Language::Kotlin => &[
                "Test.kt", "Tests.kt", "Spec.kt", "Specs.kt", "Mock.kt", "Fake.kt",
            ],
            Language::Java => &[
                "Test.java",
                "Tests.java",
                "Spec.java",
                "Mock.java",
                "IT.java",
            ],
            Language::CSharp => &["Tests.cs", "Test.cs", "Spec.cs", "Mock.cs"],
            Language::Dart => &["_test.dart"],
            Language::Php => &["Test.php", "test_"],
            Language::Ruby => &["_spec.rb", "_test.rb", "test_"],
            Language::Yaml => &[], // YAML files are typically not test files
        }
    }
}

// Default value functions for serde

fn default_specs_dir() -> String {
    "specs".to_string()
}

fn default_source_dirs() -> Vec<String> {
    vec!["src".to_string()]
}

fn default_required_sections() -> Vec<String> {
    vec![
        "Purpose".to_string(),
        "Public API".to_string(),
        "Invariants".to_string(),
        "Behavioral Examples".to_string(),
        "Error Cases".to_string(),
        "Dependencies".to_string(),
        "Change Log".to_string(),
    ]
}

fn default_exclude_dirs() -> Vec<String> {
    vec!["__tests__".to_string()]
}

fn default_exclude_patterns() -> Vec<String> {
    vec![
        "**/__tests__/**".to_string(),
        "**/*.test.ts".to_string(),
        "**/*.spec.ts".to_string(),
    ]
}

fn default_drift_labels() -> Vec<String> {
    vec!["spec-drift".to_string()]
}

fn default_true() -> bool {
    true
}

impl Default for SpecSyncConfig {
    fn default() -> Self {
        Self {
            specs_dir: default_specs_dir(),
            source_dirs: default_source_dirs(),
            schema_dir: None,
            schema_pattern: None,
            required_sections: default_required_sections(),
            exclude_dirs: default_exclude_dirs(),
            exclude_patterns: default_exclude_patterns(),
            source_extensions: Vec::new(),
            export_level: ExportLevel::default(),
            parse_mode: ParseMode::default(),
            modules: std::collections::HashMap::new(),
            ai_provider: None,
            ai_model: None,
            ai_command: None,
            ai_api_key: None,
            ai_base_url: None,
            ai_timeout: None,
            rules: ValidationRules::default(),
            custom_rules: Vec::new(),
            task_archive_days: None,
            github: None,
            enforcement: EnforcementMode::default(),
            lifecycle: LifecycleConfig::default(),
        }
    }
}
