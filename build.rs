//! Build fingerprint (#878).
//!
//! A cached result is only as good as the rules that produced it, and the package version does not
//! name those rules: every build of an unreleased tree reports the same version. This digest of
//! every source and manifest that can change what SpecSync decides is compiled in as
//! `SPECSYNC_BUILD_FINGERPRINT`, so a cache entry is replayed only by the build that wrote it.

#[path = "src/build_fingerprint.rs"]
mod build_fingerprint;

use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Files outside `src/` whose bytes decide behavior: dependency versions and this script.
const ROOT_INPUTS: [&str; 3] = ["Cargo.toml", "Cargo.lock", "build.rs"];

fn main() {
    let root = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR"),
    );
    let mut inputs = Vec::new();
    collect_files(&root.join("src"), &mut inputs);
    inputs.extend(
        ROOT_INPUTS
            .iter()
            .map(|name| root.join(name))
            .filter(|path| path.is_file()),
    );
    let mut framed: Vec<(String, PathBuf)> = inputs
        .into_iter()
        .map(|path| {
            let relative = path
                .strip_prefix(&root)
                .expect("inputs are collected under the manifest directory")
                .to_string_lossy()
                .replace('\\', "/");
            (relative, path)
        })
        .collect();
    framed.sort();

    let mut hasher = Sha256::new();
    for (relative, path) in &framed {
        let bytes = fs::read(path).unwrap_or_else(|error| panic!("read {relative}: {error}"));
        hasher.update((relative.len() as u64).to_le_bytes());
        hasher.update(relative.as_bytes());
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    // The compile configuration is part of the build: another target, profile or feature set of
    // the same tree is another build, so it never replays this one's cached results (#981).
    let configuration = build_fingerprint::configuration_frames(std::env::vars());
    hasher.update(b"configuration");
    for (name, value) in &configuration {
        hasher.update((name.len() as u64).to_le_bytes());
        hasher.update(name.as_bytes());
        hasher.update((value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    }
    println!(
        "cargo:rustc-env=SPECSYNC_BUILD_FINGERPRINT={:x}",
        hasher.finalize()
    );
    println!(
        "cargo:rustc-env=SPECSYNC_BUILD_CONFIGURATION={}",
        build_fingerprint::configuration_record(&configuration)
    );
    // A directory is scanned recursively for any modification.
    println!("cargo:rerun-if-changed=src");
    for name in ROOT_INPUTS {
        println!("cargo:rerun-if-changed={name}");
    }
}

/// Every regular file under `dir`, recursively. Symlinks are followed as Cargo would compile them.
fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|error| panic!("read {}: {error}", dir.display()));
    for entry in entries {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            collect_files(&path, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
}
