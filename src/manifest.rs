//! Manifest-aware module detection.
//!
//! Parses language-specific manifest files (Package.swift, Cargo.toml,
//! build.gradle.kts, package.json, etc.) to discover targets, source paths,
//! and module names instead of relying on directory scanning alone.

include!("manifest/max_gradle_manifest_bytes.rs");
include!("manifest/tests.rs");
