use serde::Deserialize;

/// YAML frontmatter parsed from a spec file.
#[derive(Debug, Default, Clone)]
pub struct Frontmatter {
    pub module: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub files: Vec<String>,
    pub db_tables: Vec<String>,
    pub depends_on: Vec<String>,
}

/// Result of validating a single spec.
#[derive(Debug)]
pub struct ValidationResult {
    pub spec_path: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub export_summary: Option<String>,
}

impl ValidationResult {
    pub fn new(spec_path: String) -> Self {
        Self {
            spec_path,
            errors: Vec::new(),
            warnings: Vec::new(),
            export_summary: None,
        }
    }
}

/// Coverage report for the project.
#[derive(Debug)]
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

    /// Command to run for AI-powered spec generation.
    /// The prompt is piped to stdin; spec markdown is expected on stdout.
    /// Examples: "claude -p --output-format text", "ollama run llama3"
    #[serde(default)]
    pub ai_command: Option<String>,

    /// Timeout in seconds for each AI command invocation (default: 120).
    #[serde(default)]
    pub ai_timeout: Option<u64>,
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
        }
    }

    /// File patterns to exclude (test files, etc.).
    pub fn test_patterns(&self) -> &[&str] {
        match self {
            Language::TypeScript => &[".test.ts", ".spec.ts", ".test.tsx", ".spec.tsx", ".d.ts"],
            Language::Rust => &[], // Rust tests are inline, not separate files
            Language::Go => &["_test.go"],
            Language::Python => &["test_", "_test.py"],
            Language::Swift => &["Tests.swift", "Test.swift"],
            Language::Kotlin => &["Test.kt", "Tests.kt", "Spec.kt"],
            Language::Java => &["Test.java", "Tests.java"],
            Language::CSharp => &["Tests.cs", "Test.cs"],
            Language::Dart => &["_test.dart"],
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
            ai_command: None,
            ai_timeout: None,
        }
    }
}
