use colored::Colorize;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::validator::find_spec_files;
use crate::view;

pub fn cmd_view(root: &Path, role: &str, spec_filter: Option<&str>) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);
    let spec_files = find_spec_files(&specs_dir);

    if spec_files.is_empty() {
        eprintln!("No spec files found in {}/", config.specs_dir);
        process::exit(1);
    }

    let module_of = |spec_path: &Path| -> String {
        let name = spec_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        name.strip_suffix(".spec").unwrap_or(name).to_string()
    };

    // A filter that matches nothing used to leave the loop body unexecuted, so
    // the command returned normally: zero bytes, exit 0, indistinguishable from
    // a module that exists and renders empty. A script or agent fetching spec
    // context for a mistyped name got an empty payload and no signal to retry
    // (#551).
    let mut rendered = 0usize;
    let mut failed = 0usize;

    for spec_path in &spec_files {
        if let Some(filter) = spec_filter
            && module_of(spec_path) != filter
        {
            continue;
        }

        match view::view_spec(spec_path, role) {
            Ok(output) => {
                rendered += 1;
                println!("{output}");
                println!("---\n");
            }
            Err(e) => {
                // Reported but previously ignored by the exit code, so a caller
                // could not tell a rendered spec from an unrenderable one.
                failed += 1;
                eprintln!("{} {e}", "error:".red().bold());
            }
        }
    }

    if let Some(filter) = spec_filter
        && rendered == 0
        && failed == 0
    {
        let mut available: Vec<String> = spec_files.iter().map(|p| module_of(p)).collect();
        available.sort();
        eprintln!("{} no spec module named `{filter}`", "error:".red().bold());
        // Name what does exist: the whole failure is a typo, and the remedy is
        // one of these strings.
        let near: Vec<&String> = available
            .iter()
            .filter(|name| {
                let (a, b) = (name.to_lowercase(), filter.to_lowercase());
                a.contains(&b) || b.contains(&a)
            })
            .collect();
        if !near.is_empty() {
            eprintln!(
                "  did you mean: {}",
                near.iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        } else {
            eprintln!("  available: {}", available.join(", "));
        }
        process::exit(1);
    }

    if failed > 0 {
        process::exit(1);
    }
}
