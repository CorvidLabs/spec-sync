//! Registered entry point for topical CLI command integration regressions.
//!
//! Keep shared Windows junction fixtures here: issue discovery, retained-root
//! coverage, and the sibling MCP integration module all use the same helpers.

#[cfg(windows)]
use std::fs;

#[path = "commands/coverage.rs"]
mod coverage;
#[path = "commands/dependencies.rs"]
mod dependencies;
#[path = "commands/generation.rs"]
mod generation;
#[path = "commands/history.rs"]
mod history;
#[path = "commands/issues.rs"]
mod issues;
#[path = "commands/maintenance.rs"]
mod maintenance;
#[path = "commands/migration.rs"]
mod migration;
#[path = "commands/project_setup.rs"]
mod project_setup;
#[path = "commands/quality_gates.rs"]
mod quality_gates;

#[cfg(windows)]
pub(super) fn create_windows_junction(
    junction: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), String> {
    run_windows_junction_script(
        "$ErrorActionPreference = 'Stop'; New-Item -ItemType Junction \
         -Path $env:SPECSYNC_TEST_JUNCTION \
         -Target $env:SPECSYNC_TEST_TARGET | Out-Null",
        junction,
        target,
    )
}

#[cfg(windows)]
fn retarget_windows_junction(
    junction: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), String> {
    fs::remove_dir(junction)
        .map_err(|error| format!("failed to remove Windows junction fixture: {error}"))?;
    create_windows_junction(junction, target)
}

#[cfg(windows)]
fn run_windows_junction_script(
    script: &str,
    junction: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), String> {
    let junction = junction
        .to_str()
        .ok_or_else(|| "junction fixture path must be valid Unicode".to_string())?;
    let target = target
        .to_str()
        .ok_or_else(|| "junction target path must be valid Unicode".to_string())?;
    let mut unavailable = Vec::new();
    for executable in ["powershell.exe", "pwsh.exe"] {
        let output = match std::process::Command::new(executable)
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                script,
            ])
            .env("SPECSYNC_TEST_JUNCTION", junction)
            .env("SPECSYNC_TEST_TARGET", target)
            .output()
        {
            Ok(output) => output,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                unavailable.push(executable);
                continue;
            }
            Err(error) => {
                return Err(format!(
                    "failed to launch {executable} junction fixture: {error}"
                ));
            }
        };
        if output.status.success() {
            return Ok(());
        }
        return Err(format!(
            "{executable} junction fixture exited with {:?}; stdout: {}; stderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Err(format!(
        "failed to launch a PowerShell junction fixture; unavailable executables: {}",
        unavailable.join(", ")
    ))
}
