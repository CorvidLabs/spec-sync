use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::commands::load_and_discover;
use crate::parser;
use crate::registry;
use crate::validator;

pub(crate) fn cmd_resolve(root: &Path, remote: bool) {
    let (_config, spec_files) = load_and_discover(root, false);
    let mut cross_refs: Vec<(String, String, String)> = Vec::new();
    let mut local_refs: Vec<(String, String, bool)> = Vec::new();

    for spec_file in &spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let parsed = match parser::parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let spec_path = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .to_string();

        for dep in &parsed.frontmatter.depends_on {
            if validator::is_cross_project_ref(dep) {
                if let Some((repo, module)) = validator::parse_cross_project_ref(dep) {
                    cross_refs.push((spec_path.clone(), repo.to_string(), module.to_string()));
                }
            } else {
                let exists = root.join(dep).exists();
                local_refs.push((spec_path.clone(), dep.clone(), exists));
            }
        }
    }

    println!(
        "\n--- {} ------------------------------------------------",
        "Dependency Resolution".bold()
    );

    if local_refs.is_empty() && cross_refs.is_empty() {
        println!("\n  No dependencies declared in any spec.");
        return;
    }

    if !local_refs.is_empty() {
        println!("\n  {} Local dependencies:", "Local".bold());
        for (spec, dep, exists) in &local_refs {
            if *exists {
                println!("    {} {spec} -> {dep}", "✓".green());
            } else {
                println!("    {} {spec} -> {dep} (not found)", "✗".red());
            }
        }
    }

    if !cross_refs.is_empty() {
        println!("\n  {} Cross-project references:", "Remote".bold());

        if remote {
            // Fetch remote registries to verify cross-project refs
            let mut remote_errors = 0;
            // Group refs by repo to avoid duplicate fetches
            let mut repos: std::collections::HashMap<String, Option<registry::RemoteRegistry>> =
                std::collections::HashMap::new();

            for (_spec, repo, _module) in &cross_refs {
                repos
                    .entry(repo.clone())
                    .or_insert_with(|| match registry::fetch_remote_registry(repo) {
                        Ok(reg) => Some(reg),
                        Err(e) => {
                            eprintln!(
                                "    {} Failed to fetch registry for {repo}: {e}",
                                "!".yellow()
                            );
                            None
                        }
                    });
            }

            for (spec, repo, module) in &cross_refs {
                match repos.get(repo) {
                    Some(Some(reg)) => {
                        if reg.has_spec(module) {
                            println!("    {} {spec} -> {repo}@{module}", "✓".green());
                        } else {
                            println!(
                                "    {} {spec} -> {repo}@{module} (module not in registry)",
                                "✗".red()
                            );
                            remote_errors += 1;
                        }
                    }
                    Some(None) => {
                        println!(
                            "    {} {spec} -> {repo}@{module} (registry fetch failed)",
                            "?".yellow()
                        );
                    }
                    None => {
                        println!(
                            "    {} {spec} -> {repo}@{module} (no registry)",
                            "?".yellow()
                        );
                    }
                }
            }

            if remote_errors > 0 {
                println!(
                    "\n  {} {remote_errors} cross-project ref(s) could not be verified",
                    "Warning:".yellow()
                );
            }
        } else {
            for (spec, repo, module) in &cross_refs {
                println!("    {} {spec} -> {repo}@{module}", "→".cyan());
            }
            println!(
                "\n  {} Cross-project refs are not verified by default.",
                "Tip:".cyan()
            );
            println!("  Use --remote to fetch registries and verify they exist.");
        }
    }
}
