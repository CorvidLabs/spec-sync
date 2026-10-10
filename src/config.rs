//! Configuration entry points, backed by private discovery, parsing, and loading owners.

mod checked;
mod discovery;
mod json;
mod loading;
mod serialization;
mod toml_config;
mod toml_syntax;

pub(crate) use checked::{
    parse_config_content_checked, parse_config_content_checked_with_source_dirs,
    validate_config_file,
};
pub use discovery::{detect_source_dirs, detect_source_dirs_checked};
pub(crate) use discovery::{
    detect_source_dirs_with_confidence, is_detectable_source_file,
    source_detection_ignores_directory,
};
#[allow(unused_imports)]
pub use discovery::{discover_manifest_modules, discover_manifest_modules_checked};
pub(crate) use loading::CONFIG_PATH_CANDIDATES;
pub use loading::{
    is_legacy_layout, load_config, load_config_allowing_unloadable, load_config_from_path,
    read_config_file,
};
pub use serialization::{config_to_toml, config_to_toml_lossy_fields};

/// Default schema pattern for SQL table extraction.
pub fn default_schema_pattern() -> &'static str {
    r"CREATE (?:VIRTUAL )?TABLE(?:\s+IF NOT EXISTS)?\s+(\w+)"
}

#[cfg(test)]
mod tests;
