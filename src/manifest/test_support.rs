use super::access::ManifestAccess;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) struct CountingManifestAccess {
    pub(super) texts: HashMap<PathBuf, String>,
    pub(super) directories: HashSet<PathBuf>,
    pub(super) children: HashMap<PathBuf, Vec<String>>,
    pub(super) text_reads: HashMap<PathBuf, usize>,
    pub(super) child_reads: HashMap<PathBuf, usize>,
    pub(super) entries: usize,
    entry_limit: usize,
}

impl CountingManifestAccess {
    pub(super) fn new(entry_limit: usize) -> Self {
        Self {
            texts: HashMap::new(),
            directories: HashSet::new(),
            children: HashMap::new(),
            text_reads: HashMap::new(),
            child_reads: HashMap::new(),
            entries: 0,
            entry_limit,
        }
    }
}

impl ManifestAccess for CountingManifestAccess {
    fn read_text(&mut self, relative: &Path, _label: &str) -> Result<Option<String>, String> {
        *self.text_reads.entry(relative.to_path_buf()).or_default() += 1;
        Ok(self.texts.get(relative).cloned())
    }

    fn directory_exists(&mut self, relative: &Path, _label: &str) -> Result<bool, String> {
        Ok(self.directories.contains(relative))
    }

    fn child_directories(&mut self, relative: &Path, _label: &str) -> Result<Vec<String>, String> {
        *self.child_reads.entry(relative.to_path_buf()).or_default() += 1;
        let children = self.children.get(relative).cloned().unwrap_or_default();
        self.charge_entries(children.len(), "test child directory")?;
        Ok(children)
    }

    fn charge_entries(&mut self, count: usize, _label: &str) -> Result<(), String> {
        if self.entries.saturating_add(count) > self.entry_limit {
            return Err(format!(
                "Retained manifest discovery exceeds the {}-entry limit",
                self.entry_limit
            ));
        }
        self.entries = self.entries.saturating_add(count);
        Ok(())
    }
}
