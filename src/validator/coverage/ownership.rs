use super::super::mappings::normalize_source_mapping;
use super::CoverageSourceSnapshot;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(super) fn coverage_source_modules(
    snapshot: &CoverageSourceSnapshot,
    configured: &str,
) -> Result<Vec<String>, String> {
    snapshot
        .immediate_modules
        .get(configured)
        .into_iter()
        .flatten()
        .map(|name| {
            name.to_str()
                .map(str::to_string)
                .ok_or_else(|| "Coverage source module name is not valid UTF-8".to_string())
        })
        .collect()
}

/// A configured source directory as a `/`-joined project-relative prefix, with
/// `.` becoming the empty root prefix.
pub(super) fn normalized_coverage_source_dir(configured: &str) -> Result<String, String> {
    if configured == "." {
        return Ok(String::new());
    }
    normalize_source_mapping(configured).ok_or_else(|| {
        format!("Coverage source directory must remain beneath the project root: {configured}")
    })
}

/// Join a source-directory prefix with a child directory name.
pub(super) fn coverage_module_directory(source_dir: &str, child: &str) -> String {
    if source_dir.is_empty() {
        child.to_string()
    } else {
        format!("{source_dir}/{child}")
    }
}

/// How many discovered source files a candidate module owns, and how many of
/// those no spec maps.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ModuleFileOwnership {
    owned: usize,
    unmapped: usize,
}

impl ModuleFileOwnership {
    fn observe(&mut self, mapped: bool) {
        self.owned += 1;
        if !mapped {
            self.unmapped += 1;
        }
    }

    fn absorb(&mut self, other: Self) {
        self.owned += other.owned;
        self.unmapped += other.unmapped;
    }

    /// Whether this module still has a coverage gap worth reporting.
    ///
    /// Owning NO discovered source file is not evidence of coverage: nothing
    /// was measured, so the module keeps its report rather than being declared
    /// covered by default. Only a module whose files were all looked at, and
    /// all found mapped, is silent.
    pub(super) fn is_uncovered(self) -> bool {
        self.owned == 0 || self.unmapped > 0
    }
}

/// Which discovered source files each candidate module name owns.
///
/// Built in one pass over the measured file list so a candidate can be answered
/// without rescanning it, and keyed only by the directories and stems a
/// candidate can actually name — an unbounded ancestor index would grow with
/// tree depth for names nothing ever asks about.
#[derive(Debug, Default)]
pub(super) struct CoverageModuleOwnership {
    directories: HashMap<String, ModuleFileOwnership>,
    flat_stems: HashMap<String, HashMap<String, ModuleFileOwnership>>,
}

impl CoverageModuleOwnership {
    pub(super) fn index(
        source_files: &[String],
        specced_files: &HashSet<String>,
        candidate_directories: &HashSet<String>,
        source_roots: &HashSet<String>,
    ) -> Self {
        let mut ownership = Self {
            directories: candidate_directories
                .iter()
                .map(|directory| (directory.clone(), ModuleFileOwnership::default()))
                .collect(),
            flat_stems: HashMap::new(),
        };
        for file in source_files {
            let mapped = specced_files.contains(file.as_str());
            let (parent, name) = match file.rsplit_once('/') {
                Some((parent, name)) => (parent, name),
                None => ("", file.as_str()),
            };
            if source_roots.contains(parent) {
                // Keyed with the same derivation the flat-module block uses, so
                // a stem always finds its own files.
                let stem = Path::new(name)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or(name);
                ownership
                    .flat_stems
                    .entry(parent.to_string())
                    .or_default()
                    .entry(stem.to_string())
                    .or_default()
                    .observe(mapped);
            }
            let mut prefix = String::new();
            if let Some(entry) = ownership.directories.get_mut(&prefix) {
                entry.observe(mapped);
            }
            for component in parent.split('/').filter(|component| !component.is_empty()) {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(component);
                if let Some(entry) = ownership.directories.get_mut(&prefix) {
                    entry.observe(mapped);
                }
            }
        }
        ownership
    }

    pub(super) fn directory(&self, directory: &str) -> ModuleFileOwnership {
        self.directories.get(directory).copied().unwrap_or_default()
    }

    /// Ownership across every source path a manifest declares for a module.
    /// Paths that do not normalize contribute nothing, so a module whose
    /// declared sources cannot be located keeps its report.
    pub(super) fn declared_directories(&self, source_paths: &[String]) -> ModuleFileOwnership {
        let mut total = ModuleFileOwnership::default();
        for path in source_paths {
            let Some(normalized) = normalize_source_mapping(path) else {
                continue;
            };
            total.absorb(self.directory(&normalized));
        }
        total
    }

    pub(super) fn flat_stem(&self, source_dir: &str, stem: &str) -> ModuleFileOwnership {
        self.flat_stems
            .get(source_dir)
            .and_then(|stems| stems.get(stem))
            .copied()
            .unwrap_or_default()
    }
}

/// Ownership over the explicit file list a configured module declares. A file
/// the traversal never discovered is not counted as owned, so a declaration
/// pointing at nothing keeps its report.
pub(super) fn declared_files_ownership(
    files: &[String],
    discovered_files: &HashSet<&str>,
    specced_files: &HashSet<String>,
) -> ModuleFileOwnership {
    let mut ownership = ModuleFileOwnership::default();
    for file in files {
        let Some(normalized) = normalize_source_mapping(file) else {
            continue;
        };
        if !discovered_files.contains(normalized.as_str()) {
            continue;
        }
        ownership.observe(specced_files.contains(&normalized));
    }
    ownership
}

#[cfg(test)]
#[path = "tests/ownership.rs"]
mod tests;
