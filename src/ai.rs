use crate::types::{AiProvider, SpecSyncConfig};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const MAX_FILE_CHARS: usize = 30_000;
const MAX_PROMPT_CHARS: usize = 150_000;
const DEFAULT_AI_TIMEOUT_SECS: u64 = 120;

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

/// A resolved provider ready to execute — either a CLI command or a direct API call.
#[derive(Debug, Clone)]
pub enum ResolvedProvider {
    /// Shell out to a CLI tool (e.g. `claude -p --output-format text`).
    Cli(String),
    /// Call the Anthropic Messages API directly.
    AnthropicApi {
        api_key: String,
        model: String,
        base_url: Option<String>,
    },
    /// Call an OpenAI-compatible Chat Completions API directly.
    OpenAiApi {
        api_key: String,
        model: String,
        base_url: Option<String>,
    },
}

impl std::fmt::Display for ResolvedProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedProvider::Cli(cmd) => write!(f, "CLI: {cmd}"),
            ResolvedProvider::AnthropicApi { model, .. } => {
                write!(f, "Anthropic API ({model})")
            }
            ResolvedProvider::OpenAiApi {
                model, base_url, ..
            } => {
                if let Some(url) = base_url {
                    write!(f, "OpenAI API ({model} @ {url})")
                } else {
                    write!(f, "OpenAI API ({model})")
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
    match provider {
        AiProvider::Claude => Ok("claude -p --output-format text".to_string()),
        AiProvider::Ollama => {
            let model = model.unwrap_or("llama3");
            Ok(format!("ollama run {model}"))
        }
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
        AiProvider::Anthropic | AiProvider::OpenAi => Err(
            "API providers should use resolve_api_provider(), not command_for_provider()"
                .to_string(),
        ),
        AiProvider::Custom => {
            Err("Custom provider requires \"aiCommand\" to be set in specsync.json".to_string())
        }
    }
}

/// Resolve an API provider to a ResolvedProvider.
fn resolve_api_provider(
    provider: &AiProvider,
    config: &SpecSyncConfig,
) -> Result<ResolvedProvider, String> {
    let env_var = provider.api_key_env_var().unwrap();
    let api_key = config
        .ai_api_key
        .clone()
        .or_else(|| std::env::var(env_var).ok())
        .ok_or_else(|| {
            format!(
                "Provider \"{provider}\" requires an API key. Set {env_var} or \
                 add \"aiApiKey\" to specsync.json"
            )
        })?;

    let model = config
        .ai_model
        .clone()
        .unwrap_or_else(|| provider.default_model().unwrap().to_string());

    match provider {
        AiProvider::Anthropic => Ok(ResolvedProvider::AnthropicApi {
            api_key,
            model,
            base_url: config.ai_base_url.clone(),
        }),
        AiProvider::OpenAi => Ok(ResolvedProvider::OpenAiApi {
            api_key,
            model,
            base_url: config.ai_base_url.clone(),
        }),
        _ => unreachable!(),
    }
}

/// Resolve the AI provider to use.
///
/// Resolution order:
/// 1. `--provider` CLI flag (passed as `cli_provider`)
/// 2. `aiCommand` in config (explicit override always wins)
/// 3. `aiProvider` in config (resolved to CLI command or API)
/// 4. `SPECSYNC_AI_COMMAND` env var
/// 5. Auto-detect installed CLIs, then check for API keys
pub fn resolve_ai_provider(
    config: &SpecSyncConfig,
    cli_provider: Option<&str>,
) -> Result<ResolvedProvider, String> {
    // 1. CLI --provider flag
    if let Some(name) = cli_provider {
        let provider = AiProvider::from_str_loose(name).ok_or_else(|| {
            format!(
                "Unknown provider \"{name}\". Available: claude, anthropic, openai, ollama, copilot"
            )
        })?;

        if provider.is_api_provider() {
            return resolve_api_provider(&provider, config);
        }

        if !is_binary_available(provider.binary_name()) {
            return Err(format!(
                "Provider \"{name}\" selected but `{}` is not installed or not on PATH",
                provider.binary_name()
            ));
        }
        return command_for_provider(&provider, config.ai_model.as_deref())
            .map(ResolvedProvider::Cli);
    }

    // 2. aiCommand in config (explicit override)
    if let Some(cmd) = &config.ai_command {
        return Ok(ResolvedProvider::Cli(cmd.clone()));
    }

    // 3. aiProvider in config
    if let Some(provider) = &config.ai_provider {
        if provider.is_api_provider() {
            return resolve_api_provider(provider, config);
        }

        if !is_binary_available(provider.binary_name()) {
            return Err(format!(
                "Provider \"{}\" configured but `{}` is not installed or not on PATH",
                provider,
                provider.binary_name()
            ));
        }
        return command_for_provider(provider, config.ai_model.as_deref())
            .map(ResolvedProvider::Cli);
    }

    // 4. Environment variable
    if let Ok(cmd) = std::env::var("SPECSYNC_AI_COMMAND") {
        return Ok(ResolvedProvider::Cli(cmd));
    }

    // 5. Auto-detect: check installed CLIs first, then API keys
    for provider in AiProvider::detection_order() {
        if provider.is_api_provider() {
            // Check for API key in env — use a separate variable for the
            // env-var name so CodeQL doesn't confuse the *name* with the
            // *value* returned by std::env::var.
            let has_key = provider
                .api_key_env_var()
                .is_some_and(|v| std::env::var(v).is_ok());
            if has_key {
                eprintln!("  Auto-detected AI provider: {provider} (API key found)");
                return resolve_api_provider(provider, config);
            }
        } else if is_binary_available(provider.binary_name())
            && let Ok(cmd) = command_for_provider(provider, config.ai_model.as_deref())
        {
            eprintln!(
                "  Auto-detected AI provider: {} ({})",
                provider,
                provider.binary_name()
            );
            return Ok(ResolvedProvider::Cli(cmd));
        }
    }

    Err("No AI provider found. Options:\n\n\
         CLI providers (install a tool):\n  \
         claude     — Claude Code CLI (npm i -g @anthropic-ai/claude-code)\n  \
         ollama     — Local models (ollama.com)\n  \
         copilot    — GitHub Copilot (gh extension install github/gh-copilot)\n\n\
         API providers (just set a key — no CLI needed):\n  \
         anthropic  — set ANTHROPIC_API_KEY env var\n  \
         openai     — set OPENAI_API_KEY env var\n\n\
         Or configure in specsync.json:\n  \
         \"aiProvider\": \"anthropic\"    + ANTHROPIC_API_KEY\n  \
         \"aiProvider\": \"openai\"       + OPENAI_API_KEY\n  \
         \"aiCommand\":  \"any-cli\"      (custom command)\n\n\
         Use --provider <name> to select one, or --provider auto to auto-detect."
        .to_string())
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

/// Call the Anthropic Messages API directly.
fn call_anthropic_api(
    api_key: &str,
    model: &str,
    base_url: Option<&str>,
    prompt: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    let url = format!(
        "{}/v1/messages",
        base_url.unwrap_or("https://api.anthropic.com")
    );

    eprintln!("    Calling Anthropic API ({model})...");
    let _ = std::io::stderr().flush();

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 8192,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(timeout_secs)))
            .build(),
    );

    let mut response = agent
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .send_json(&body)
        .map_err(|e| {
            // Sanitize error to avoid leaking API key from request headers
            let msg = e.to_string();
            if msg.contains(api_key) {
                "Anthropic API request failed: connection error".to_string()
            } else {
                format!("Anthropic API request failed: {msg}")
            }
        })?;

    let status = response.status();
    let response_body: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse Anthropic API response: {e}"))?;

    if status != 200 {
        let error_msg = response_body["error"]["message"]
            .as_str()
            .unwrap_or("unknown error");
        return Err(format!("Anthropic API error (HTTP {status}): {error_msg}"));
    }

    // Extract text from the response content blocks
    let content = response_body["content"]
        .as_array()
        .ok_or("Anthropic API response missing 'content' array")?;

    let mut text = String::new();
    for block in content {
        if block["type"].as_str() == Some("text")
            && let Some(t) = block["text"].as_str()
        {
            text.push_str(t);
        }
    }

    if text.trim().is_empty() {
        return Err("Anthropic API returned empty response".to_string());
    }

    let usage = &response_body["usage"];
    let input_tokens = usage["input_tokens"].as_u64().unwrap_or(0);
    let output_tokens = usage["output_tokens"].as_u64().unwrap_or(0);
    eprintln!("    ✓ Anthropic API: {input_tokens} input + {output_tokens} output tokens");

    Ok(text)
}

/// Call an OpenAI-compatible Chat Completions API directly.
fn call_openai_api(
    api_key: &str,
    model: &str,
    base_url: Option<&str>,
    prompt: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    let url = format!(
        "{}/v1/chat/completions",
        base_url.unwrap_or("https://api.openai.com")
    );

    eprintln!("    Calling OpenAI API ({model})...");
    let _ = std::io::stderr().flush();

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 8192,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(timeout_secs)))
            .build(),
    );

    let mut response = agent
        .post(&url)
        .header("Authorization", &format!("Bearer {api_key}"))
        .header("content-type", "application/json")
        .send_json(&body)
        .map_err(|e| {
            // Sanitize error to avoid leaking API key from request headers
            let msg = e.to_string();
            if msg.contains(api_key) {
                "OpenAI API request failed: connection error".to_string()
            } else {
                format!("OpenAI API request failed: {msg}")
            }
        })?;

    let status = response.status();
    let response_body: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse OpenAI API response: {e}"))?;

    if status != 200 {
        let error_msg = response_body["error"]["message"]
            .as_str()
            .unwrap_or("unknown error");
        return Err(format!("OpenAI API error (HTTP {status}): {error_msg}"));
    }

    let text = response_body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("OpenAI API response missing choices[0].message.content")?
        .to_string();

    if text.trim().is_empty() {
        return Err("OpenAI API returned empty response".to_string());
    }

    let usage = &response_body["usage"];
    let prompt_tokens = usage["prompt_tokens"].as_u64().unwrap_or(0);
    let completion_tokens = usage["completion_tokens"].as_u64().unwrap_or(0);
    eprintln!("    ✓ OpenAI API: {prompt_tokens} input + {completion_tokens} output tokens");

    Ok(text)
}

/// Run the resolved provider with the given prompt.
fn run_provider(
    provider: &ResolvedProvider,
    prompt: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    match provider {
        ResolvedProvider::Cli(cmd) => run_cli_command(cmd, prompt, timeout_secs),
        ResolvedProvider::AnthropicApi {
            api_key,
            model,
            base_url,
        } => call_anthropic_api(api_key, model, base_url.as_deref(), prompt, timeout_secs),
        ResolvedProvider::OpenAiApi {
            api_key,
            model,
            base_url,
        } => call_openai_api(api_key, model, base_url.as_deref(), prompt, timeout_secs),
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
    fn command_for_ollama_default_model() {
        let cmd = command_for_provider(&AiProvider::Ollama, None).unwrap();
        assert_eq!(cmd, "ollama run llama3");
    }

    #[test]
    fn command_for_ollama_custom_model() {
        let cmd = command_for_provider(&AiProvider::Ollama, Some("mistral")).unwrap();
        assert_eq!(cmd, "ollama run mistral");
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
        let p = ResolvedProvider::AnthropicApi {
            api_key: "sk-test".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            base_url: None,
        };
        assert_eq!(format!("{p}"), "Anthropic API (claude-sonnet-4-20250514)");
    }

    #[test]
    fn display_openai_provider_no_base_url() {
        let p = ResolvedProvider::OpenAiApi {
            api_key: "sk-test".to_string(),
            model: "gpt-4o".to_string(),
            base_url: None,
        };
        assert_eq!(format!("{p}"), "OpenAI API (gpt-4o)");
    }

    #[test]
    fn display_openai_provider_with_base_url() {
        let p = ResolvedProvider::OpenAiApi {
            api_key: "sk-test".to_string(),
            model: "gpt-4o".to_string(),
            base_url: Some("https://custom.api.com".to_string()),
        };
        assert_eq!(format!("{p}"), "OpenAI API (gpt-4o @ https://custom.api.com)");
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
        let prompt = build_regen_prompt(
            "big",
            "spec",
            "reqs",
            &[("src/big.rs".to_string(), large)],
        );
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
        assert!(err.contains("Cursor does not have a CLI pipe mode"));
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
