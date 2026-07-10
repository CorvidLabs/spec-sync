use crate::helpers::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ─── Provider / Multi-Agent Tests ────────────────────────────────────────

#[test]
fn provider_flag_unknown_provider_errors() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("nonexistent")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown provider"));
}

#[test]
fn provider_flag_enables_ai() {
    // --provider enables AI mode (and fails if binary not found)
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("cursor")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cursor").or(predicate::str::contains("Cursor")));
}

#[test]
fn ai_provider_config_field_is_respected() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Overwrite config with aiProvider set to cursor (not installed)
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "cursor",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("auto")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cursor").or(predicate::str::contains("Cursor")));
}

#[test]
fn ai_command_overrides_ai_provider() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // aiCommand must come from a TRUSTED channel — here the SPECSYNC_AI_COMMAND
    // env var (always honored, never sourced from a committed repo file). It
    // still overrides aiProvider: the "false" command exits 1, AI falls back to
    // template, and stderr shows it tried.
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "claude",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    // Create unspecced module so generation is triggered
    fs::create_dir_all(root.join("src/newmod")).unwrap();
    fs::write(root.join("src/newmod/lib.rs"), "pub fn hello() {}").unwrap();

    // The spec is still written from the template, but the run exits non-zero
    // and reports the AI failure prominently on stderr
    specsync()
        .env("SPECSYNC_AI_COMMAND", "false")
        .arg("generate")
        .arg("--provider")
        .arg("auto")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("AI generation failed"))
        .stderr(predicate::str::contains("template fallback was used"));
}

#[test]
fn cli_provider_overrides_config_provider() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Config says claude, CLI says cursor — cursor should win
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "claude",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("cursor")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cursor").or(predicate::str::contains("Cursor")));
}

// Note: `ollama` is now an API provider (corvid-ai, OpenAI-compatible HTTP),
// not a CLI shell-out. Its reclassification is covered by deterministic unit
// tests in `src/ai.rs` (`ollama_is_an_api_provider`, `ollama_resolves_keyless`)
// rather than a network-dependent integration test.

#[test]
fn auto_detect_defaults_to_local_ollama_without_keys() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Uncovered module so `generate` actually attempts AI resolution.
    fs::create_dir_all(root.join("src/widget")).unwrap();
    fs::write(root.join("src/widget/lib.rs"), "pub fn widget() {}").unwrap();

    // Low timeout bounds the test if a local Ollama happens to be running.
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiTimeout": 3,
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    // With every provider key cleared and no aiProvider configured, auto-detect
    // must fall back to local Ollama rather than erroring.
    let mut cmd = specsync();
    for key in [
        "OLLAMA_API_KEY",
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "OPENROUTER_API_KEY",
        "GEMINI_API_KEY",
        "DEEPSEEK_API_KEY",
        "GROQ_API_KEY",
        "MISTRAL_API_KEY",
        "XAI_API_KEY",
        "TOGETHER_API_KEY",
    ] {
        cmd.env_remove(key);
    }
    cmd.arg("generate")
        .arg("--provider")
        .arg("auto")
        .arg("--root")
        .arg(&root)
        .assert()
        // Robust whether or not a local Ollama daemon is reachable on the test
        // host: either "Auto-detected ... ollama" (probe hit) or "defaulting to
        // Ollama" (probe miss) — both resolve to Ollama, never an error/CLI.
        .stderr(predicate::str::contains("ollama").or(predicate::str::contains("Ollama")));
}

#[test]
fn config_ai_provider_triggers_ai_without_provider_flag() {
    // Regression: a configured `aiProvider` must invoke AI even without the
    // `--provider` flag (previously this silently fell back to templates).
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "anthropic",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    // No --provider flag — AI mode comes from config. With no key it fails fast
    // on the key check, proving AI (not template-only) was selected.
    specsync()
        .arg("generate")
        .arg("--root")
        .arg(&root)
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ANTHROPIC_API_KEY"));
}

#[test]
fn env_provider_overrides_config_provider() {
    // 12-factor `flag > env > config`: SPECSYNC_AI_PROVIDER must outrank the
    // `aiProvider` config value.
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "ollama",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    // Config says ollama, env says anthropic → env wins → fails on the missing
    // Anthropic key (proving it did NOT use the configured ollama).
    specsync()
        .arg("generate")
        .arg("--root")
        .arg(&root)
        .env("SPECSYNC_AI_PROVIDER", "anthropic")
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ANTHROPIC_API_KEY"));
}

// ─── 8. Direct API provider tests ───────────────────────────────────────

#[test]
fn anthropic_provider_requires_api_key() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // aiProvider=anthropic without ANTHROPIC_API_KEY should error
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "anthropic",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("auto")
        .arg("--root")
        .arg(&root)
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ANTHROPIC_API_KEY"));
}

#[test]
fn openai_provider_requires_api_key() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // aiProvider=openai without OPENAI_API_KEY should error
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "openai",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("auto")
        .arg("--root")
        .arg(&root)
        .env_remove("OPENAI_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("OPENAI_API_KEY"));
}

#[test]
fn provider_flag_anthropic_requires_api_key() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("anthropic")
        .arg("--root")
        .arg(&root)
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ANTHROPIC_API_KEY"));
}

#[test]
fn provider_flag_openai_requires_api_key() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("openai")
        .arg("--root")
        .arg(&root)
        .env_remove("OPENAI_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("OPENAI_API_KEY"));
}

#[test]
fn anthropic_api_alias_works() {
    // "anthropic-api" should be accepted as an alias for "anthropic"
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("anthropic-api")
        .arg("--root")
        .arg(&root)
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ANTHROPIC_API_KEY"));
}

#[test]
fn openai_api_alias_works() {
    // "openai-api" should be accepted as an alias for "openai"
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("openai-api")
        .arg("--root")
        .arg(&root)
        .env_remove("OPENAI_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("OPENAI_API_KEY"));
}

#[test]
fn ai_api_key_config_field_used_for_anthropic() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Create unspecced module so generation is triggered
    fs::create_dir_all(root.join("src/newmod")).unwrap();
    fs::write(root.join("src/newmod/lib.rs"), "pub fn hello() {}").unwrap();

    // Set aiProvider=anthropic with aiApiKey in config (fake key)
    // This should attempt the API call (and fail with auth error), proving
    // the key was picked up from config rather than erroring about missing key
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "anthropic",
        "aiApiKey": "sk-ant-test-fake-key",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    // The API call fails (bad key) — the template fallback is written but the
    // run exits non-zero so the failure is not silent
    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("auto")
        .arg("--root")
        .arg(&root)
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("AI generation failed"));
}

#[test]
fn unknown_provider_lists_api_options() {
    // Error message for unknown provider should mention anthropic and openai
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("bogus")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("anthropic").and(predicate::str::contains("openai")));
}
