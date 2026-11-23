//! `specsync history <module>` (REQ-cmd-change-037): print a module's whole history as one Change
//! Log table, the frozen table as written followed by one row per history fragment. A submodule of
//! the change commands, beside `doctor`; the fragments and the table live in `crate::change`.

use colored::Colorize;
use std::path::Path;
use std::process;

use crate::change;
use crate::types::OutputFormat;

/// Print `module`'s history in `format`. Exits 1 for an unknown module or for a history fragment
/// `specsync check` would refuse, naming it.
pub fn cmd_history(root: &Path, module: &str, format: OutputFormat) {
    match change::read_spec_history(root, module) {
        Ok(history) => match format {
            OutputFormat::Json => match serde_json::to_string_pretty(&history) {
                Ok(json) => println!("{json}"),
                Err(error) => {
                    eprintln!("{} {error}", "error:".red().bold());
                    process::exit(1);
                }
            },
            _ => print!(
                "## Change Log\n\n{}",
                change::render_spec_history_table(&history)
            ),
        },
        Err(error) => {
            match format {
                OutputFormat::Json => println!("{}", serde_json::json!({ "error": error })),
                _ => eprintln!("{} {error}", "error:".red().bold()),
            }
            process::exit(1);
        }
    }
}
