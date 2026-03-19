use crate::types::SpecSyncConfig;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const MAX_FILE_CHARS: usize = 30_000;
const MAX_PROMPT_CHARS: usize = 150_000;
const DEFAULT_AI_COMMAND: &str = "claude -p --output-format text";
const DEFAULT_AI_TIMEOUT_SECS: u64 = 120;

/// Resolve the AI command to use. Checks config, then env, then default.
pub fn resolve_ai_command(config: &SpecSyncConfig) -> Result<String, String> {
    // 1. Config file
    if let Some(cmd) = &config.ai_command {
        return Ok(cmd.clone());
    }

    // 2. Environment variable
    if let Ok(cmd) = std::env::var("SPECSYNC_AI_COMMAND") {
        return Ok(cmd);
    }

    // 3. Default: check if claude CLI is available
    let check = Command::new("sh")
        .args(["-c", "command -v claude"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match check {
        Ok(status) if status.success() => Ok(DEFAULT_AI_COMMAND.to_string()),
        _ => Err(
            "No AI command found. Install the Claude CLI, or set \"aiCommand\" in specsync.json, \
             or set SPECSYNC_AI_COMMAND env var.\n\n\
             Examples:\n  \
             \"aiCommand\": \"claude -p --output-format text\"   (Claude Code CLI)\n  \
             \"aiCommand\": \"ollama run llama3\"                 (local model)\n  \
             \"aiCommand\": \"cat > /dev/null && echo 'test'\"    (any command that reads stdin, writes stdout)"
                .to_string(),
        ),
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
                &content[..MAX_FILE_CHARS]
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

Other guidelines:
- For `## Invariants`, list rules that must always hold based on the code
- For `## Behavioral Examples`, use Given/When/Then format
- For `## Error Cases`, use a table of Condition | Behavior
- For `## Dependencies`, list what this module consumes from other modules
- For `## Change Log`, add a single entry with today's date and "Initial spec"
- Be accurate — only document what the code actually does"#
    )
}

/// Run the AI command with the given prompt, returning stdout.
/// Shows a spinner while waiting, then streams stdout lines to stderr in real time.
/// Times out after `timeout_secs` seconds (default 120).
fn run_ai_command(ai_command: &str, prompt: &str, timeout_secs: u64) -> Result<String, String> {
    let mut child = Command::new("sh")
        .args(["-c", ai_command])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start AI command: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|e| format!("Failed to write to AI command stdin: {e}"))?;
    }

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

/// Generate a spec file using AI for a given module.
pub fn generate_spec_with_ai(
    module_name: &str,
    source_files: &[String],
    root: &Path,
    config: &SpecSyncConfig,
    ai_command: &str,
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
    let mut spec = run_ai_command(ai_command, &prompt, timeout)?;

    // Strip code fences if the model wrapped the output
    if spec.trim_start().starts_with("```") {
        let trimmed = spec.trim();
        // Remove opening fence (```markdown or ```)
        if let Some(rest) = trimmed.strip_prefix("```markdown\n").or_else(|| trimmed.strip_prefix("```md\n")).or_else(|| trimmed.strip_prefix("```\n")) {
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
