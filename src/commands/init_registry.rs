use colored::Colorize;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process;

use crate::config::validate_config_file;
use crate::output::csv_field;
use crate::registry;
use crate::types::OutputFormat;

#[derive(Debug, Serialize)]
struct InitRegistryReport {
    command: &'static str,
    success: bool,
    created: bool,
    unchanged: bool,
    registry: String,
    name: Option<String>,
    error: Option<String>,
}

impl InitRegistryReport {
    fn failure(registry: String, error: String) -> Self {
        Self {
            command: "init-registry",
            success: false,
            created: false,
            unchanged: true,
            registry,
            name: None,
            error: Some(error),
        }
    }
}

pub fn cmd_init_registry(root: &Path, name: Option<String>, format: OutputFormat) {
    match execute_init_registry(root, name) {
        Ok(report) => render_init_registry_report(&report, format),
        Err(report) => {
            render_init_registry_report(&report, format);
            process::exit(1);
        }
    }
}

fn execute_init_registry(
    root: &Path,
    name: Option<String>,
) -> Result<InitRegistryReport, InitRegistryReport> {
    // Respect the project layout: v4 projects get .specsync/registry.toml,
    // un-migrated 3.x projects keep the legacy root-level file.
    let registry_path = registry::local_registry_path(root);
    // Normalize to forward slashes so output matches the tool's path style on every platform.
    let rel_display = registry_path
        .strip_prefix(root)
        .unwrap_or(&registry_path)
        .display()
        .to_string()
        .replace('\\', "/");
    let project_name = name.unwrap_or_else(|| {
        root.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("project")
            .to_string()
    });
    if project_name.trim().is_empty() {
        return Err(InitRegistryReport::failure(
            rel_display,
            "registry name must not be empty; pass --name with a non-empty project name"
                .to_string(),
        ));
    }

    if registry_path.exists() {
        return match registry::load_local_registry(root) {
            Ok(Some(existing)) => Ok(InitRegistryReport {
                command: "init-registry",
                success: true,
                created: false,
                unchanged: true,
                registry: rel_display,
                name: Some(existing.name),
                error: None,
            }),
            Ok(None) => Err(InitRegistryReport::failure(
                rel_display,
                "registry already exists but is inert; repair or remove it explicitly".to_string(),
            )),
            Err(error) => Err(InitRegistryReport::failure(rel_display, error)),
        };
    }

    if let Some(config_path) = existing_config_path(root)
        && let Err(error) = validate_config_file(&config_path)
    {
        return Err(InitRegistryReport::failure(rel_display, error));
    }
    let config = crate::config::load_config_allowing_unloadable(root);
    let content = registry::generate_registry(root, &project_name, &config.specs_dir);
    if let Err(error) = toml::from_str::<toml::Value>(&content) {
        return Err(InitRegistryReport::failure(
            rel_display,
            format!("generated registry is invalid TOML: {error}"),
        ));
    }
    if let Some(parent) = registry_path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        return Err(InitRegistryReport::failure(
            rel_display,
            format!("failed to create {}: {error}", parent.display()),
        ));
    }

    let write_result = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&registry_path)
        .and_then(|mut file| file.write_all(content.as_bytes()));
    match write_result {
        Ok(()) => Ok(InitRegistryReport {
            command: "init-registry",
            success: true,
            created: true,
            unchanged: false,
            registry: rel_display,
            name: Some(project_name),
            error: None,
        }),
        Err(error) => Err(InitRegistryReport::failure(
            rel_display,
            format!("failed to write registry: {error}"),
        )),
    }
}

fn existing_config_path(root: &Path) -> Option<std::path::PathBuf> {
    [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync.toml",
        "specsync.json",
    ]
    .into_iter()
    .map(|relative| root.join(relative))
    .find(|path| path.exists())
}

fn render_init_registry_report(report: &InitRegistryReport, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string(report).expect("init-registry report must serialize")
            );
        }
        OutputFormat::Markdown | OutputFormat::Github => {
            println!("## SpecSync init-registry\n");
            println!(
                "- **Status:** {}",
                if report.success { "success" } else { "failure" }
            );
            println!("- **Created:** {}", report.created);
            println!("- **Unchanged:** {}", report.unchanged);
            println!("- **Registry:** `{}`", report.registry);
            if let Some(name) = &report.name {
                println!("- **Name:** `{name}`");
            }
            if let Some(error) = &report.error {
                println!("- **Error:** {error}");
            }
        }
        OutputFormat::Table => {
            println!("FIELD\tVALUE");
            println!(
                "status\t{}",
                if report.success { "success" } else { "failure" }
            );
            println!("created\t{}", report.created);
            println!("unchanged\t{}", report.unchanged);
            println!("registry\t{}", report.registry);
            if let Some(name) = &report.name {
                println!("name\t{name}");
            }
            if let Some(error) = &report.error {
                println!("error\t{error}");
            }
        }
        OutputFormat::Csv => {
            println!("command,success,created,unchanged,registry,name,error");
            println!(
                "init-registry,{},{},{},{},{},{}",
                report.success,
                report.created,
                report.unchanged,
                csv_field(&report.registry),
                csv_field(report.name.as_deref().unwrap_or_default()),
                csv_field(report.error.as_deref().unwrap_or_default())
            );
        }
        OutputFormat::Text => {
            if !report.success {
                eprintln!(
                    "{} {}",
                    "Error:".red(),
                    report
                        .error
                        .as_deref()
                        .unwrap_or("registry initialization failed")
                );
            } else if report.created {
                println!("{} Created {}", "✓".green(), report.registry);
            } else {
                println!("{} already exists — nothing to do", report.registry);
                println!("  Delete it first (or edit it by hand) to regenerate.");
            }
        }
    }
}
