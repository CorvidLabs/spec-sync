// Integration test suite for specsync
#[path = "integration/helpers.rs"]
pub mod helpers;

#[path = "integration/check.rs"]
mod check;

#[path = "integration/fix.rs"]
mod fix;

#[path = "integration/commands.rs"]
mod commands;

#[path = "integration/languages.rs"]
mod languages;

#[path = "integration/mcp.rs"]
mod mcp;

#[path = "integration/config.rs"]
mod config;

#[path = "integration/change.rs"]
mod change;

#[path = "integration/comment.rs"]
mod comment;

#[path = "integration/coverage_unmeasured.rs"]
mod coverage_unmeasured;

#[path = "integration/staleness_unmeasurable.rs"]
mod staleness_unmeasurable;
