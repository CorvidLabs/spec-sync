// Integration test suite for specsync
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

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

#[path = "integration/ai.rs"]
mod ai;

#[path = "integration/config.rs"]
mod config;
