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

    for spec_path in &spec_files {
        // If a specific spec was requested, filter by module name
        if let Some(filter) = spec_filter {
            let name = spec_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            // Strip .spec suffix if present
            let module_name = name.strip_suffix(".spec").unwrap_or(name);
            if module_name != filter {
                continue;
            }
        }

        match view::view_spec(spec_path, role) {
            Ok(output) => {
                println!("{output}");
                println!("---\n");
            }
            Err(e) => {
                eprintln!("{} {e}", "error:".red().bold());
            }
        }
    }
}
