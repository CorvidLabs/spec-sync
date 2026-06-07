use crate::types::{AiProvider, SpecSyncConfig};
use std::fs;
use std::io::{BufRead, BufReader, IsTerminal, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const MAX_FILE_CHARS: usize = 30_000;
const MAX_PROMPT_CHARS: usize = 150_000;
const DEFAULT_AI_TIMEOUT_SECS: u64 = 120;

/// Default local Ollama host (native API; corvid-ai appends `/v1` for the
/// OpenAI-compatible transport it uses).
const OLLAMA_DEFAULT_HOST: &str = "http://localhost:11434";
/// Ollama Cloud host, targeted for `-cloud` model tags when `OLLAMA_API_KEY`
/// is set.
const OLLAMA_CLOUD_HOST: &str = "https://ollama.com";

/// Resolve the Ollama host (without a trailing `/v1`).
///
/// Precedence (matches fledge): `OLLAMA_HOST` env > `aiBaseUrl` config >
/// `-cloud` auto-routing (model tag contains `-cloud` and `OLLAMA_API_KEY` set
/// ⇒ Ollama Cloud) > default localhost.
fn ollama_host(config: &SpecSyncConfig) -> String {
    let strip = |h: &str| h.trim_end_matches('/').trim_end_matches("/v1").to_string();

    if let Ok(host) = std::env::var("OLLAMA_HOST")
        && !host.is_empty()
    {
        return strip(&host);
    }
    if let Some(base) = config.ai_base_url.as_deref()
        && !base.is_empty()
    {
        return strip(base);
    }
    let is_cloud_model = config
        .ai_model
        .as_deref()
        .is_some_and(|m| m.contains("-cloud"));
    if is_cloud_model && std::env::var("OLLAMA_API_KEY").is_ok_and(|k| !k.is_empty()) {
        return OLLAMA_CLOUD_HOST.to_string();
    }
    OLLAMA_DEFAULT_HOST.to_string()
}

/// Truncate a string to at most `max_bytes` bytes on a valid UTF-8 char boundary.
fn safe_truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// A resolved provider ready to execute — either a CLI command or a direct API
/// call routed through [`corvid_ai`].
#[derive(Clone)]
pub enum ResolvedProvider {
    /// Shell out to a CLI tool (e.g. `claude -p --output-format text`).
    Cli(String),
    /// Call an LLM HTTP API via the shared `corvid-ai` client. The `Settings`
    /// carry the provider name (registry key), plus any model/key/base-url/
    /// timeout overrides; `corvid-ai` owns the registry of endpoints and
    /// default models.
    Api(corvid_ai::Settings),
}

impl std::fmt::Debug for ResolvedProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedProvider::Cli(cmd) => f.debug_tuple("Cli").field(cmd).finish(),
            // Custom impl rather than deriving: `corvid_ai::Settings`'s derived
            // Debug would print the API key verbatim. Redact it here.
            ResolvedProvider::Api(settings) => f
                .debug_struct("Api")
                .field("provider", &settings.provider)
                .field("model", &settings.model)
                .field("api_key", &settings.api_key.as_ref().map(|_| "[REDACTED]"))
                .field("base_url", &settings.base_url)
                .finish(),
        }
    }
}

impl std::fmt::Display for ResolvedProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedProvider::Cli(cmd) => write!(f, "CLI: {cmd}"),
            ResolvedProvider::Api(settings) => {
                let provider = settings
                    .provider
                    .as_deref()
                    .unwrap_or(corvid_ai::DEFAULT_PROVIDER);
                match (&settings.model, &settings.base_url) {
                    (Some(model), Some(url)) => write!(f, "{provider} API ({model} @ {url})"),
                    (Some(model), None) => write!(f, "{provider} API ({model})"),
                    (None, Some(url)) => write!(f, "{provider} API (@ {url})"),
                    (None, None) => write!(f, "{provider} API"),
                }
            }
        }
    }
}

/// Check whether a binary is available on PATH.
fn is_binary_available(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    // Attempt to launch the binary with --version. If the OS cannot find it,
    // `status()` returns Err (ENOENT); if it launches at all, is_ok() is true
    // regardless of exit code. This uses the OS-level execvp PATH search, which
    // handles symlinks and execute-permission checks correctly.
    Command::new(name)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Build the CLI command string for a provider, optionally using a custom model.
fn command_for_provider(provider: &AiProvider, model: Option<&str>) -> Result<String, String> {
    let _ = model;
    match provider {
        AiProvider::Claude => Ok("claude -p --output-format text".to_string()),
        AiProvider::Copilot => Ok("gh copilot suggest -t shell".to_string()),
        AiProvider::Cursor => Err(
            "Cursor does not have a CLI pipe mode (stdin→stdout) for spec generation.\n\
             Workarounds:\n  \
             1. Use \"aiProvider\": \"anthropic\" with an ANTHROPIC_API_KEY\n  \
             2. Use \"aiProvider\": \"openai\" with an OPENAI_API_KEY\n  \
             3. Use \"aiProvider\": \"ollama\" for a local model\n  \
             4. Set \"aiCommand\" to any CLI tool that reads stdin and writes stdout"
                .to_string(),
        ),
        AiProvider::Anthropic
        | AiProvider::OpenAi
        | AiProvider::OpenRouter
        | AiProvider::Gemini
        | AiProvider::DeepSeek
        | AiProvider::Groq
        | AiProvider::Mistral
        | AiProvider::XAi
        | AiProvider::Together
        | AiProvider::Ollama => Err(
            "API providers should use resolve_api_provider(), not command_for_provider()"
                .to_string(),
        ),
        AiProvider::Custom => {
            Err("Custom provider requires \"aiCommand\" to be set in specsync.json".to_string())
        }
    }
}

/// Map a spec-sync API provider to its `corvid-ai` registry name.
///
/// Returns `None` for providers that don't use API-key auth (CLI/custom).
fn corvid_provider_name(provider: &AiProvider) -> Option<&'static str> {
    match provider {
        AiProvider::Anthropic => Some("anthropic"),
        AiProvider::OpenAi => Some("openai"),
        AiProvider::OpenRouter => Some("openrouter"),
        AiProvider::Gemini => Some("gemini"),
        AiProvider::DeepSeek => Some("deepseek"),
        AiProvider::Groq => Some("groq"),
        AiProvider::Mistral => Some("mistral"),
        AiProvider::XAi => Some("xai"),
        AiProvider::Together => Some("together"),
        AiProvider::Ollama => Some("ollama"),
        _ => None,
    }
}

/// Resolve an API provider into [`corvid_ai::Settings`].
///
/// Only the user-supplied overrides (model/key/base-url/timeout) are carried
/// here; `corvid-ai` owns the endpoint registry, default models, and the
/// `<PROVIDER>_API_KEY` env-var fallback, which it resolves lazily when the
/// request runs.
fn resolve_api_provider(
    provider: &AiProvider,
    config: &SpecSyncConfig,
) -> Result<ResolvedProvider, String> {
    let name = corvid_provider_name(provider).ok_or_else(|| {
        format!("Provider \"{provider}\" does not support API key authentication")
    })?;

    // Fail fast with a clear, env-var-named message when a key is required but
    // missing, rather than resolving lazily and only erroring after scanning the
    // project. corvid-ai validates again at request time; this is purely for UX.
    //
    // Ollama runs keyless against a local server, and a custom `aiBaseUrl`
    // signals a self-hosted/proxy gateway that may not need a key — skip the
    // eager check in those cases and let the request surface any auth error.
    let key_optional = matches!(provider, AiProvider::Ollama) || config.ai_base_url.is_some();
    let has_key = config.ai_api_key.as_deref().is_some_and(|k| !k.is_empty())
        || provider
            .api_key_env_var()
            .is_some_and(|env_var| std::env::var(env_var).is_ok());
    if !key_optional && !has_key {
        let env_var = provider.api_key_env_var().unwrap_or_default();
        return Err(format!(
            "Provider \"{provider}\" requires an API key. Set {env_var} or \
             add \"aiApiKey\" to specsync.json"
        ));
    }

    let mut settings = corvid_ai::Settings::provider(name);
    if let Some(model) = &config.ai_model {
        settings = settings.model(model);
    }
    if let Some(key) = &config.ai_api_key {
        settings = settings.api_key(key);
    }
    if matches!(provider, AiProvider::Ollama) {
        // Ollama's host honors OLLAMA_HOST / config / -cloud routing; corvid-ai
        // speaks to it over the OpenAI-compatible `/v1` endpoint.
        settings = settings.base_url(format!("{}/v1", ollama_host(config)));
    } else if let Some(base_url) = &config.ai_base_url {
        settings = settings.base_url(base_url);
    }
    settings.timeout_secs = Some(config.ai_timeout.unwrap_or(DEFAULT_AI_TIMEOUT_SECS));

    Ok(ResolvedProvider::Api(settings))
}

/// Emit a deprecation notice when a (still-functional) CLI provider is selected.
///
/// API providers are the supported path; these warnings steer users there
/// before the CLI providers are removed in a future major release.
fn warn_deprecated_cli(provider: &AiProvider) {
    if let AiProvider::Copilot = provider {
        eprintln!(
            "  ⚠ The `copilot` CLI provider is deprecated. Prefer an API provider \
             (anthropic / openai / gemini / ollama / …) with its API key."
        );
    }
}

/// Resolve an explicitly-named provider (from `--provider` or config).
///
/// The deprecated `claude` alias routes to the `anthropic` API with a warning —
/// it no longer shells out to the agentic `claude -p`. API providers go to the
/// shared client; the remaining deprecated CLI providers (`copilot`) and the
/// no-pipe `cursor` use the legacy path. `selected_as` is "selected" or
/// "configured" for the not-installed error message.
fn resolve_named_provider(
    provider: &AiProvider,
    config: &SpecSyncConfig,
    selected_as: &str,
) -> Result<ResolvedProvider, String> {
    if matches!(provider, AiProvider::Claude) {
        // Shared deprecation message template (matches fledge).
        eprintln!(
            "  ⚠ provider 'claude' is deprecated and now uses anthropic. Set \
             \"aiProvider\": \"anthropic\" (and ANTHROPIC_API_KEY). This alias is removed \
             in spec-sync 5.0."
        );
        return resolve_api_provider(&AiProvider::Anthropic, config);
    }

    if provider.is_api_provider() {
        return resolve_api_provider(provider, config);
    }

    // Deprecated CLI (copilot), no-pipe (cursor), or custom — legacy CLI path.
    let cmd = command_for_provider(provider, config.ai_model.as_deref())?;
    if !is_binary_available(provider.binary_name()) {
        return Err(format!(
            "Provider \"{provider}\" {selected_as} but `{}` is not installed or not on PATH",
            provider.binary_name()
        ));
    }
    warn_deprecated_cli(provider);
    Ok(ResolvedProvider::Cli(cmd))
}

/// Resolve the AI provider to use.
///
/// Resolution order (shared mental model with fledge):
/// 1. `--provider` CLI flag (explicit selection)
/// 2. `aiCommand` in config — spec-sync-only explicit, trusted CLI escape hatch
/// 3. `aiProvider` in config (explicit selection)
/// 4. `SPECSYNC_AI_PROVIDER` env (provider name)
/// 5. `SPECSYNC_AI_COMMAND` env — spec-sync-only explicit CLI hatch
/// 6. Auto-detect: **Ollama if usable** (local daemon reachable or
///    `OLLAMA_API_KEY`) → otherwise **API-first** by key → otherwise default to
///    local Ollama. No CLI is ever auto-selected.
pub fn resolve_ai_provider(
    config: &SpecSyncConfig,
    cli_provider: Option<&str>,
) -> Result<ResolvedProvider, String> {
    // 1. CLI --provider flag
    if let Some(name) = cli_provider {
        let provider = AiProvider::from_str_loose(name).ok_or_else(|| {
            format!(
                "Unknown provider \"{name}\". Available: anthropic, openai, openrouter, gemini, \
                 deepseek, groq, mistral, xai, together, ollama (+ deprecated claude, copilot)"
            )
        })?;

        return resolve_named_provider(&provider, config, "selected");
    }

    // 2. aiCommand in config (explicit override — the trusted CLI escape hatch)
    if let Some(cmd) = &config.ai_command {
        return Ok(ResolvedProvider::Cli(cmd.clone()));
    }

    // 3. aiProvider in config
    if let Some(provider) = &config.ai_provider {
        return resolve_named_provider(provider, config, "configured");
    }

    // 4. SPECSYNC_AI_PROVIDER env (provider name) — below config to match fledge.
    if let Ok(name) = std::env::var("SPECSYNC_AI_PROVIDER")
        && !name.is_empty()
    {
        let provider = AiProvider::from_str_loose(&name)
            .ok_or_else(|| format!("Unknown provider \"{name}\" from SPECSYNC_AI_PROVIDER"))?;
        return resolve_named_provider(&provider, config, "selected via SPECSYNC_AI_PROVIDER");
    }

    // 5. SPECSYNC_AI_COMMAND env (explicit CLI hatch)
    if let Ok(cmd) = std::env::var("SPECSYNC_AI_COMMAND") {
        return Ok(ResolvedProvider::Cli(cmd));
    }

    // 6. Auto-detect by API-key presence (no CLI shell-out, no network probe).
    //    Collect every provider with a key set, in deterministic order.
    let configured: Vec<&AiProvider> = AiProvider::detection_order()
        .iter()
        .filter(|p| {
            // Separate variable for the env-var name so CodeQL doesn't confuse
            // the *name* with the *value* returned by std::env::var.
            p.api_key_env_var()
                .is_some_and(|v| std::env::var(v).is_ok_and(|k| !k.is_empty()))
        })
        .collect();

    match configured.as_slice() {
        // None configured → keyless local Ollama (the zero-config default).
        [] => {
            eprintln!(
                "  No AI API key detected — defaulting to local Ollama ({}).\n  \
                 Start a local Ollama server, set OLLAMA_API_KEY for the cloud, or set \
                 another provider's key (ANTHROPIC_API_KEY, OPENAI_API_KEY, …).",
                ollama_host(config)
            );
            resolve_api_provider(&AiProvider::Ollama, config)
        }
        // Exactly one configured → use it.
        [provider] => {
            eprintln!("  Auto-detected AI provider: {provider} (API key found)");
            resolve_api_provider(provider, config)
        }
        // Several configured → prompt interactively, else deterministic order.
        providers => {
            if std::io::stdin().is_terminal() && std::io::stderr().is_terminal() {
                prompt_provider_and_model(providers, config)
            } else {
                let provider = providers[0];
                eprintln!(
                    "  Multiple AI providers configured ({}); using {provider} (first in \
                     order). Set \"aiProvider\" or --provider to choose explicitly.",
                    providers
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                resolve_api_provider(provider, config)
            }
        }
    }
}

/// Interactively choose a provider (and optional model) when several have keys.
fn prompt_provider_and_model(
    providers: &[&AiProvider],
    config: &SpecSyncConfig,
) -> Result<ResolvedProvider, String> {
    use dialoguer::{Input, Select};

    let labels: Vec<String> = providers.iter().map(|p| p.to_string()).collect();
    let idx = Select::new()
        .with_prompt("Multiple AI providers configured — pick one")
        .items(&labels)
        .default(0)
        .interact()
        .map_err(|e| format!("provider selection cancelled: {e}"))?;
    let provider = providers[idx];

    let model: String = Input::new()
        .with_prompt(format!("Model for {provider} (blank = provider default)"))
        .allow_empty(true)
        .interact_text()
        .map_err(|e| format!("model entry cancelled: {e}"))?;

    let mut chosen = config.clone();
    if !model.trim().is_empty() {
        chosen.ai_model = Some(model.trim().to_string());
    }
    resolve_api_provider(provider, &chosen)
}

// Keep the old name as an alias for compatibility with tests
#[allow(dead_code)]
pub fn resolve_ai_command(
    config: &SpecSyncConfig,
    cli_provider: Option<&str>,
) -> Result<String, String> {
    match resolve_ai_provider(config, cli_provider)? {
        ResolvedProvider::Cli(cmd) => Ok(cmd),
        other => Ok(format!("[api:{other}]")),
    }
}

/// Build the prompt for spec generation.
fn build_prompt(
    module_name: &str,
    source_contents: &[(String, String)],
    required_sections: &[String],
) -> String {
    let sections_list = required_sections
        .iter()
        .map(|s| format!("## {s}"))
        .collect::<Vec<_>>()
        .join("\n");

    let files_yaml = source_contents
        .iter()
        .map(|(path, _)| format!("  - {path}"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut source_block = String::new();
    let mut total_chars = 0;
    for (path, content) in source_contents {
        if total_chars > MAX_PROMPT_CHARS {
            source_block.push_str(&format!("\n--- {path} ---\n[skipped: prompt size limit]\n"));
            continue;
        }
        let truncated = if content.len() > MAX_FILE_CHARS {
            format!(
                "{}\n\n[... truncated at {MAX_FILE_CHARS} chars ...]",
                safe_truncate(content, MAX_FILE_CHARS)
            )
        } else {
            content.clone()
        };
        total_chars += truncated.len();
        source_block.push_str(&format!("\n--- {path} ---\n{truncated}\n"));
    }

    format!(
        r#"You are a technical writer generating specification documents for software modules.
Output ONLY the raw markdown spec file content. Do NOT wrap it in code fences.
The spec must start with `---` YAML frontmatter and include all required sections.
Be concise but thorough. Infer purpose, invariants, and error cases from the code.
For the Public API section, list every public/exported symbol in markdown tables with
backtick-quoted names in the first column.

Generate a spec file for the module "{module_name}".

The frontmatter must be exactly:
---
module: {module_name}
version: 1
status: draft
files:
{files_yaml}
db_tables: []
depends_on: []
---

Required markdown sections (in this order):
{sections_list}

Source files:
{source_block}

CRITICAL rules for the `## Public API` section:
- Use markdown tables with backtick-quoted symbol names in the FIRST COLUMN
- ONLY document symbols that are PUBLIC/EXPORTED from this module's external interface
- Do NOT document: private functions, internal constants, private helpers, submodule names, struct fields, or implementation details
- In Rust: only `pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub type` that are re-exported or accessible from outside the module
- In TypeScript/JS: only symbols with `export` keyword
- In Python: only symbols in `__all__` or top-level non-underscore names
- In Go: only capitalized names
- If a symbol is private/internal (e.g. `const`, `fn` without `pub`, `mod` declarations), do NOT put it in the Public API table
- Use subsection headers like `### Exported Functions`, `### Exported Types` — NOT `### Constants`, `### Per-language extractors`, `### Methods`, etc.

Context boundaries:
- This spec covers ONLY the files listed in the frontmatter — do not document symbols from imported/dependent modules
- If a file imports symbols from other modules, those belong to the dependency's spec, not this one
- Only document the public contract that THIS module exposes to its consumers
- Group related types and functions logically, not by file

Other guidelines:
- For `## Invariants`, list rules that must always hold based on the code
- For `## Behavioral Examples`, use Given/When/Then format
- For `## Error Cases`, use a table of Condition | Behavior
- For `## Dependencies`, list what this module consumes from other modules (imports from outside this module's files)
- For `## Change Log`, add a single entry with today's date and "Initial spec"
- Be accurate — only document what the code actually does"#
    )
}

/// Run a CLI command with the given prompt, returning stdout.
/// Shows a spinner while waiting, then streams stdout lines to stderr in real time.
fn run_cli_command(ai_command: &str, prompt: &str, timeout_secs: u64) -> Result<String, String> {
    let mut child = Command::new("sh")
        .args(["-c", ai_command])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start AI command: {e}"))?;

    // Write stdin in a background thread to avoid a pipe deadlock: if the child
    // process writes to stdout before consuming all stdin (e.g. streaming tokens),
    // the stdout pipe buffer fills, the child blocks, and a synchronous write_all
    // would block too — neither side making progress until the 120s timeout fires.
    let stdin_thread = if let Some(mut stdin) = child.stdin.take() {
        let prompt_bytes = prompt.as_bytes().to_vec();
        Some(std::thread::spawn(move || -> std::io::Result<()> {
            stdin.write_all(&prompt_bytes)
        }))
    } else {
        None
    };

    // Read stdout in a background thread, streaming lines to stderr for live output
    let stdout_pipe = child.stdout.take().ok_or("Failed to capture stdout")?;
    let (tx, rx) = mpsc::channel::<String>();
    let reader_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout_pipe);
        let mut captured = String::new();
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // Stream to stderr so the user sees live progress
                    let _ = tx.send(line.clone());
                    captured.push_str(&line);
                    captured.push('\n');
                }
                Err(_) => break,
            }
        }
        captured
    });

    // Read stderr in a background thread
    let stderr_pipe = child.stderr.take().ok_or("Failed to capture stderr")?;
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr_pipe);
        let mut captured = String::new();
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    captured.push_str(&line);
                    captured.push('\n');
                }
                Err(_) => break,
            }
        }
        captured
    });

    let timeout = Duration::from_secs(timeout_secs);
    let start = Instant::now();
    let mut line_count = 0;
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut spinner_idx = 0;
    let mut got_first_line = false;

    // Poll for lines and check timeout
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => {
                if !got_first_line {
                    // Clear the spinner line before printing first output line
                    eprint!("\r\x1b[2K");
                    got_first_line = true;
                }
                line_count += 1;
                // Print live to stderr with a prefix so it's visually distinct
                eprintln!("    │ {line}");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if !got_first_line {
                    let elapsed = start.elapsed().as_secs();
                    let frame = spinner_frames[spinner_idx % spinner_frames.len()];
                    eprint!("\r\x1b[2K    {frame} Waiting for AI response... ({elapsed}s)");
                    let _ = std::io::stderr().flush();
                    spinner_idx += 1;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if !got_first_line {
                    eprint!("\r\x1b[2K");
                    let _ = std::io::stderr().flush();
                }
                break;
            }
        }

        if start.elapsed() > timeout {
            eprint!("\r\x1b[2K");
            let _ = std::io::stderr().flush();
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "AI command timed out after {timeout_secs}s ({line_count} lines received). \
                 Set \"aiTimeout\" in specsync.json to increase the limit."
            ));
        }
    }

    // Drain any remaining lines
    for line in rx.try_iter() {
        eprintln!("    │ {line}");
    }

    if let Some(handle) = stdin_thread {
        handle
            .join()
            .map_err(|_| "stdin writer thread panicked".to_string())?
            .map_err(|e| format!("Failed to write to AI command stdin: {e}"))?;
    }

    let stdout = reader_thread
        .join()
        .map_err(|_| "stdout reader thread panicked".to_string())?;

    let stderr_output = stderr_thread
        .join()
        .map_err(|_| "stderr reader thread panicked".to_string())?;

    let status = child
        .wait()
        .map_err(|e| format!("AI command failed: {e}"))?;

    if !status.success() {
        return Err(format!(
            "AI command exited with {}: {}",
            status,
            stderr_output.trim()
        ));
    }

    if stdout.trim().is_empty() {
        return Err("AI command returned empty output".to_string());
    }

    Ok(stdout)
}

/// Run the resolved provider with the given prompt.
///
/// CLI providers shell out (legacy path); API providers go through the shared
/// `corvid-ai` client, which owns the three HTTP wire shapes (Anthropic /
/// OpenAI-compatible / Gemini), endpoint registry, key resolution, and secret
/// redaction in errors.
fn run_provider(
    provider: &ResolvedProvider,
    prompt: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    match provider {
        ResolvedProvider::Cli(cmd) => run_cli_command(cmd, prompt, timeout_secs),
        ResolvedProvider::Api(settings) => {
            let mut settings = settings.clone();
            // Fall back to the caller's timeout if Settings didn't carry one.
            if settings.timeout_secs.is_none() {
                settings.timeout_secs = Some(timeout_secs);
            }
            eprintln!(
                "    Calling {} API...",
                settings.provider.as_deref().unwrap_or("LLM")
            );
            let _ = std::io::stderr().flush();

            let completion = corvid_ai::Completion::new(prompt).max_tokens(8192);
            corvid_ai::complete(&settings, &completion).map_err(|e| e.to_string())
        }
    }
}

/// Build a prompt for regenerating a spec when requirements have changed.
fn build_regen_prompt(
    module_name: &str,
    current_spec: &str,
    requirements: &str,
    source_contents: &[(String, String)],
) -> String {
    let mut prompt = format!(
        "You are updating a module specification for `{module_name}` because its requirements have changed.\n\n\
         ## Current Spec\n\n```markdown\n{current_spec}\n```\n\n\
         ## Updated Requirements\n\n```markdown\n{requirements}\n```\n\n"
    );

    if !source_contents.is_empty() {
        prompt.push_str("## Source Files\n\n");
        let mut total_len = 0usize;
        for (path, content) in source_contents {
            if total_len > 150_000 {
                prompt.push_str(&format!("(Skipping {path} — size budget exceeded)\n\n"));
                continue;
            }
            let truncated = safe_truncate(content, 30_000);
            prompt.push_str(&format!("### `{path}`\n\n```\n{truncated}\n```\n\n"));
            total_len += truncated.len();
        }
    }

    prompt.push_str(
        "## Instructions\n\n\
         Re-validate and update the spec to reflect the new requirements. Preserve the existing \
         YAML frontmatter fields (module, version, status, files, db_tables, depends_on) and \
         bump the version by 1. Keep the same markdown structure and section headings. \
         Focus on updating:\n\
         - Purpose section (if the module's role has changed)\n\
         - Public API table (if the interface should change)\n\
         - Invariants (if constraints have changed)\n\
         - Behavioral Examples (if behavior expectations have changed)\n\
         - Error Cases (if error handling should change)\n\n\
         Output ONLY the complete updated spec as valid markdown with YAML frontmatter. \
         Do not wrap in code fences.\n",
    );

    prompt
}

/// Regenerate a spec file using AI when requirements have drifted.
pub fn regenerate_spec_with_ai(
    module_name: &str,
    spec_path: &Path,
    requirements_path: &Path,
    root: &Path,
    config: &SpecSyncConfig,
    provider: &ResolvedProvider,
) -> Result<String, String> {
    let current_spec =
        fs::read_to_string(spec_path).map_err(|e| format!("Cannot read spec: {e}"))?;
    let requirements = fs::read_to_string(requirements_path)
        .map_err(|e| format!("Cannot read requirements: {e}"))?;

    // Read source files from frontmatter
    let files = crate::hash_cache::extract_frontmatter_files(&current_spec);
    let mut source_contents = Vec::new();
    for file in &files {
        let full_path = root.join(file);
        if let Ok(content) = fs::read_to_string(&full_path) {
            source_contents.push((file.clone(), content));
        }
    }

    let prompt = build_regen_prompt(module_name, &current_spec, &requirements, &source_contents);
    let timeout = config.ai_timeout.unwrap_or(DEFAULT_AI_TIMEOUT_SECS);
    let raw = run_provider(provider, &prompt, timeout)?;

    postprocess_spec(&raw)
}

/// Strip code fences and validate frontmatter.
fn postprocess_spec(raw: &str) -> Result<String, String> {
    let mut spec = raw.to_string();

    // Strip code fences if the model wrapped the output
    if spec.trim_start().starts_with("```") {
        let trimmed = spec.trim();
        // Remove opening fence (```markdown or ```)
        if let Some(rest) = trimmed
            .strip_prefix("```markdown\n")
            .or_else(|| trimmed.strip_prefix("```md\n"))
            .or_else(|| trimmed.strip_prefix("```\n"))
        {
            spec = rest.to_string();
        }
        // Remove closing fence
        if let Some(rest) = spec.trim_end().strip_suffix("```") {
            spec = rest.to_string();
        }
    }

    // Validate the response has frontmatter
    if !spec.trim_start().starts_with("---") {
        return Err("AI response missing YAML frontmatter delimiters".to_string());
    }

    Ok(spec)
}

/// Generate a spec file using AI for a given module.
pub fn generate_spec_with_ai(
    module_name: &str,
    source_files: &[String],
    root: &Path,
    config: &SpecSyncConfig,
    provider: &ResolvedProvider,
) -> Result<String, String> {
    let mut source_contents = Vec::new();
    for file in source_files {
        let full_path = root.join(file);
        let rel_path = full_path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file.clone());
        let content =
            fs::read_to_string(&full_path).map_err(|e| format!("Cannot read {file}: {e}"))?;
        source_contents.push((rel_path, content));
    }

    let prompt = build_prompt(module_name, &source_contents, &config.required_sections);
    let timeout = config.ai_timeout.unwrap_or(DEFAULT_AI_TIMEOUT_SECS);
    let raw = run_provider(provider, &prompt, timeout)?;

    postprocess_spec(&raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AiProvider;

    // ── safe_truncate ──────────────────────────────────────────────

    #[test]
    fn safe_truncate_within_limit() {
        assert_eq!(safe_truncate("hello", 10), "hello");
    }

    #[test]
    fn safe_truncate_exact_limit() {
        assert_eq!(safe_truncate("hello", 5), "hello");
    }

    #[test]
    fn safe_truncate_truncates_ascii() {
        assert_eq!(safe_truncate("hello world", 5), "hello");
    }

    #[test]
    fn safe_truncate_respects_utf8_boundary() {
        // '€' is 3 bytes (E2 82 AC). Cutting at byte 2 should back up to 0.
        let s = "€abc";
        assert_eq!(safe_truncate(s, 2), "");
        // Cutting at byte 3 should give the full '€'.
        assert_eq!(safe_truncate(s, 3), "€");
        // Cutting at byte 4 gives '€a'.
        assert_eq!(safe_truncate(s, 4), "€a");
    }

    #[test]
    fn safe_truncate_multibyte_sequence() {
        // '🦀' is 4 bytes. Cutting at 1, 2, 3 should all yield "".
        let s = "🦀rust";
        assert_eq!(safe_truncate(s, 1), "");
        assert_eq!(safe_truncate(s, 3), "");
        assert_eq!(safe_truncate(s, 4), "🦀");
    }

    #[test]
    fn safe_truncate_empty_string() {
        assert_eq!(safe_truncate("", 10), "");
    }

    // ── command_for_provider ───────────────────────────────────────

    #[test]
    fn command_for_claude() {
        let cmd = command_for_provider(&AiProvider::Claude, None).unwrap();
        assert_eq!(cmd, "claude -p --output-format text");
    }

    #[test]
    fn ollama_is_an_api_provider() {
        // Ollama now routes through corvid-ai's OpenAI-compatible HTTP endpoint
        // rather than shelling out to `ollama run`.
        assert!(AiProvider::Ollama.is_api_provider());
        assert_eq!(corvid_provider_name(&AiProvider::Ollama), Some("ollama"));
        assert_eq!(AiProvider::Ollama.api_key_env_var(), Some("OLLAMA_API_KEY"));
    }

    #[test]
    fn ollama_resolves_keyless() {
        // No OLLAMA_API_KEY needed — a local Ollama server runs keyless, so the
        // eager key check must not reject it.
        let config = SpecSyncConfig::default();
        let resolved = resolve_api_provider(&AiProvider::Ollama, &config).unwrap();
        match resolved {
            ResolvedProvider::Api(settings) => {
                assert_eq!(settings.provider.as_deref(), Some("ollama"));
            }
            _ => panic!("expected an Api provider"),
        }
    }

    #[test]
    fn openrouter_maps_to_corvid_ai() {
        assert!(AiProvider::OpenRouter.is_api_provider());
        assert_eq!(
            corvid_provider_name(&AiProvider::OpenRouter),
            Some("openrouter")
        );
        assert_eq!(
            AiProvider::from_str_loose("openrouter"),
            Some(AiProvider::OpenRouter)
        );
    }

    #[test]
    fn claude_routes_to_anthropic_api_not_cli() {
        // The deprecated `claude` alias must resolve to the anthropic API rather
        // than shelling out to the agentic `claude -p`. Key supplied via config
        // so the test is independent of the environment.
        let config = SpecSyncConfig {
            ai_api_key: Some("sk-test".to_string()),
            ..SpecSyncConfig::default()
        };
        let resolved = resolve_named_provider(&AiProvider::Claude, &config, "selected").unwrap();
        match resolved {
            ResolvedProvider::Api(settings) => {
                assert_eq!(settings.provider.as_deref(), Some("anthropic"));
            }
            ResolvedProvider::Cli(_) => panic!("claude must route to the anthropic API, not a CLI"),
        }
    }

    #[test]
    fn detection_order_is_api_only_and_ollama_first() {
        // Auto-detection must never include a CLI shell-out provider.
        for provider in AiProvider::detection_order() {
            assert!(
                provider.is_api_provider(),
                "{provider} is in detection_order but is not an API provider"
            );
        }
        // Ollama (cloud key) is the first preference in the shared order.
        assert_eq!(
            AiProvider::detection_order().first(),
            Some(&AiProvider::Ollama)
        );
    }

    #[test]
    fn ollama_host_strips_v1_and_trailing_slash_from_config() {
        // Guarded so a developer's OLLAMA_HOST doesn't fail the test.
        if std::env::var("OLLAMA_HOST").is_err() {
            let config = SpecSyncConfig {
                ai_base_url: Some("http://example.com:1234/v1/".to_string()),
                ..SpecSyncConfig::default()
            };
            assert_eq!(ollama_host(&config), "http://example.com:1234");
        }
    }

    #[test]
    fn ollama_host_defaults_to_localhost() {
        if std::env::var("OLLAMA_HOST").is_err() {
            let config = SpecSyncConfig::default();
            assert_eq!(ollama_host(&config), "http://localhost:11434");
        }
    }

    #[test]
    fn command_for_copilot() {
        let cmd = command_for_provider(&AiProvider::Copilot, None).unwrap();
        assert_eq!(cmd, "gh copilot suggest -t shell");
    }

    #[test]
    fn command_for_cursor_errors() {
        let err = command_for_provider(&AiProvider::Cursor, None).unwrap_err();
        assert!(err.contains("Cursor does not have a CLI pipe mode"));
    }

    #[test]
    fn command_for_anthropic_errors() {
        let err = command_for_provider(&AiProvider::Anthropic, None).unwrap_err();
        assert!(err.contains("resolve_api_provider"));
    }

    #[test]
    fn command_for_custom_errors() {
        let err = command_for_provider(&AiProvider::Custom, None).unwrap_err();
        assert!(err.contains("aiCommand"));
    }

    // ── ResolvedProvider Display ────────────────────────────────────

    #[test]
    fn display_cli_provider() {
        let p = ResolvedProvider::Cli("claude -p".to_string());
        assert_eq!(format!("{p}"), "CLI: claude -p");
    }

    #[test]
    fn display_anthropic_provider() {
        let p = ResolvedProvider::Api(
            corvid_ai::Settings::provider("anthropic").model("claude-sonnet-4-6"),
        );
        assert_eq!(format!("{p}"), "anthropic API (claude-sonnet-4-6)");
    }

    #[test]
    fn display_openai_provider_no_base_url() {
        let p = ResolvedProvider::Api(corvid_ai::Settings::provider("openai").model("gpt-4o"));
        assert_eq!(format!("{p}"), "openai API (gpt-4o)");
    }

    #[test]
    fn display_openai_provider_with_base_url() {
        let p = ResolvedProvider::Api(
            corvid_ai::Settings::provider("openai")
                .model("gpt-4o")
                .base_url("https://custom.api.com"),
        );
        assert_eq!(
            format!("{p}"),
            "openai API (gpt-4o @ https://custom.api.com)"
        );
    }

    #[test]
    fn debug_api_provider_redacts_key() {
        let p = ResolvedProvider::Api(
            corvid_ai::Settings::provider("anthropic").api_key("sk-secret-value"),
        );
        let debug = format!("{p:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("sk-secret-value"));
    }

    // ── postprocess_spec ───────────────────────────────────────────

    #[test]
    fn postprocess_strips_markdown_fence() {
        let raw = "```markdown\n---\nmodule: test\n---\n# Test\n```";
        let result = postprocess_spec(raw).unwrap();
        assert!(result.starts_with("---"));
        assert!(!result.contains("```"));
    }

    #[test]
    fn postprocess_strips_plain_fence() {
        let raw = "```\n---\nmodule: test\n---\n# Test\n```";
        let result = postprocess_spec(raw).unwrap();
        assert!(result.starts_with("---"));
        assert!(!result.contains("```"));
    }

    #[test]
    fn postprocess_strips_md_fence() {
        let raw = "```md\n---\nmodule: test\n---\n# Test\n```";
        let result = postprocess_spec(raw).unwrap();
        assert!(result.starts_with("---"));
    }

    #[test]
    fn postprocess_no_fence_passthrough() {
        let raw = "---\nmodule: test\n---\n# Test\n";
        let result = postprocess_spec(raw).unwrap();
        assert_eq!(result, raw);
    }

    #[test]
    fn postprocess_missing_frontmatter_errors() {
        let raw = "# No frontmatter here\nJust some text.";
        let err = postprocess_spec(raw).unwrap_err();
        assert!(err.contains("missing YAML frontmatter"));
    }

    #[test]
    fn postprocess_leading_whitespace_before_frontmatter() {
        let raw = "  \n---\nmodule: test\n---\n# Test\n";
        let result = postprocess_spec(raw).unwrap();
        assert!(result.contains("module: test"));
    }

    // ── build_prompt ───────────────────────────────────────────────

    #[test]
    fn build_prompt_contains_module_name() {
        let prompt = build_prompt(
            "auth",
            &[("src/auth.rs".to_string(), "pub fn login() {}".to_string())],
            &["Purpose".to_string(), "Public API".to_string()],
        );
        assert!(prompt.contains("\"auth\""));
        assert!(prompt.contains("## Purpose"));
        assert!(prompt.contains("## Public API"));
        assert!(prompt.contains("src/auth.rs"));
        assert!(prompt.contains("pub fn login() {}"));
    }

    #[test]
    fn build_prompt_truncates_large_files() {
        let large_content = "x".repeat(MAX_FILE_CHARS + 1000);
        let prompt = build_prompt(
            "big",
            &[("src/big.rs".to_string(), large_content)],
            &["Purpose".to_string()],
        );
        assert!(prompt.contains("truncated at"));
        // The full content should not appear
        assert!(prompt.len() < MAX_FILE_CHARS + 10_000);
    }

    #[test]
    fn build_prompt_skips_files_over_prompt_limit() {
        // Create enough files to exceed MAX_PROMPT_CHARS
        let file_content = "a".repeat(MAX_FILE_CHARS);
        let mut files = Vec::new();
        for i in 0..10 {
            files.push((format!("src/file{i}.rs"), file_content.clone()));
        }
        let prompt = build_prompt("multi", &files, &["Purpose".to_string()]);
        assert!(prompt.contains("skipped: prompt size limit"));
    }

    #[test]
    fn build_prompt_empty_files() {
        let prompt = build_prompt("empty", &[], &["Purpose".to_string()]);
        assert!(prompt.contains("\"empty\""));
        assert!(prompt.contains("Source files:"));
    }

    // ── build_regen_prompt ─────────────────────────────────────────

    #[test]
    fn build_regen_prompt_contains_spec_and_requirements() {
        let current = "---\nmodule: auth\n---\n# Auth\n";
        let requirements = "## User Stories\n- login flow\n";
        let prompt = build_regen_prompt(
            "auth",
            current,
            requirements,
            &[("src/auth.rs".to_string(), "pub fn login() {}".to_string())],
        );
        assert!(prompt.contains("## Current Spec"));
        assert!(prompt.contains("## Updated Requirements"));
        assert!(prompt.contains("login flow"));
        assert!(prompt.contains("src/auth.rs"));
        assert!(prompt.contains("bump the version by 1"));
    }

    #[test]
    fn build_regen_prompt_no_source_files() {
        let prompt = build_regen_prompt("auth", "spec content", "requirements", &[]);
        assert!(!prompt.contains("## Source Files"));
        assert!(prompt.contains("## Instructions"));
    }

    #[test]
    fn build_regen_prompt_truncates_large_sources() {
        let large = "y".repeat(40_000);
        let prompt =
            build_regen_prompt("big", "spec", "reqs", &[("src/big.rs".to_string(), large)]);
        // safe_truncate should have capped it at 30_000
        assert!(prompt.len() < 200_000);
    }

    // ── resolve_ai_provider ────────────────────────────────────────

    #[test]
    fn resolve_with_ai_command_in_config() {
        let mut config = SpecSyncConfig::default();
        config.ai_command = Some("my-custom-ai".to_string());
        let result = resolve_ai_provider(&config, None).unwrap();
        match result {
            ResolvedProvider::Cli(cmd) => assert_eq!(cmd, "my-custom-ai"),
            _ => panic!("Expected CLI provider"),
        }
    }

    #[test]
    fn resolve_with_env_var() {
        let config = SpecSyncConfig::default();
        // SAFETY: single-threaded test — no concurrent env access
        unsafe {
            std::env::set_var("SPECSYNC_AI_COMMAND", "env-ai-tool");
        }
        let result = resolve_ai_provider(&config, None);
        unsafe {
            std::env::remove_var("SPECSYNC_AI_COMMAND");
        }
        match result.unwrap() {
            ResolvedProvider::Cli(cmd) => assert_eq!(cmd, "env-ai-tool"),
            _ => panic!("Expected CLI provider"),
        }
    }

    #[test]
    fn resolve_unknown_provider_errors() {
        let config = SpecSyncConfig::default();
        let err = resolve_ai_provider(&config, Some("nonexistent")).unwrap_err();
        assert!(err.contains("Unknown provider"));
    }

    #[test]
    fn resolve_cursor_provider_errors() {
        let config = SpecSyncConfig::default();
        let err = resolve_ai_provider(&config, Some("cursor")).unwrap_err();
        // Error depends on whether `cursor` binary is on PATH:
        // - If not on PATH: "not installed or not on PATH"
        // - If on PATH: "Cursor does not have a CLI pipe mode"
        assert!(
            err.contains("not installed or not on PATH")
                || err.contains("Cursor does not have a CLI pipe mode"),
            "unexpected error: {err}"
        );
    }

    // ── resolve_ai_command (compat alias) ──────────────────────────

    #[test]
    fn resolve_ai_command_returns_cli_string() {
        let mut config = SpecSyncConfig::default();
        config.ai_command = Some("test-cmd".to_string());
        let result = resolve_ai_command(&config, None).unwrap();
        assert_eq!(result, "test-cmd");
    }

    // ── constants ──────────────────────────────────────────────────

    #[test]
    fn constants_are_reasonable() {
        assert_eq!(MAX_FILE_CHARS, 30_000);
        assert_eq!(MAX_PROMPT_CHARS, 150_000);
        assert_eq!(DEFAULT_AI_TIMEOUT_SECS, 120);
    }
}
