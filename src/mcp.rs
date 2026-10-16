//! Model Context Protocol server.
//!
//! Private modules and the calls the split actually makes:
//!
//! - `protocol` calls `tools`, `snapshot`, and `confinement`.
//! - `tools` calls `protocol`, `snapshot`, `confinement`, `preflight`, and `publication`.
//! - `snapshot` calls `protocol`, `snapshot_manifest`, and `confinement`.
//! - `snapshot_manifest` calls `snapshot`, `confinement`, and `preflight`.
//! - `confinement` calls `protocol`, `tools`, `snapshot`, `preflight`, and `publication`.
//! - `preflight` calls `snapshot_manifest` and `confinement`.
//! - `publication` calls `tools`, `snapshot`, and `confinement`.
//!
//! Snapshot discovery and confinement preflight call back into the readers they
//! share. Those edges stay `pub(super)`. `run_mcp_server` and
//! `mcp_tool_definitions` are the only functions visible outside this module.
//!
//! `server_name.rs` and `publish_staged_file_with_verification_hooks.rs` stay
//! so `src/github.rs` can keep including those historical paths. They are not
//! part of this module.

mod confinement;
mod preflight;
mod protocol;
mod publication;
mod snapshot;
mod snapshot_manifest;
mod tools;

#[cfg(test)]
mod test_support;

use std::path::Path;

pub fn run_mcp_server(root: &Path, allow_write: bool) -> Result<(), String> {
    protocol::run_mcp_server(root, allow_write)
}

pub fn mcp_tool_definitions(allow_write: bool) -> Vec<serde_json::Value> {
    protocol::mcp_tool_definitions(allow_write)
}

#[cfg(test)]
mod gh_process_absence {
    /// The GitHub read-path scan include_str!s three historical MCP paths and
    /// cannot be extended from this change. This test covers every file the
    /// implementation moved into.
    #[test]
    fn mcp_sources_do_not_construct_a_gh_process() {
        let command = ["Com", "mand::new(\"gh\")"].concat();
        let process = ["process::Com", "mand::new(\"gh\")"].concat();
        let sources: &[(&str, &str)] = &[
            ("src/mcp.rs", include_str!("mcp.rs")),
            ("src/mcp/protocol.rs", include_str!("mcp/protocol.rs")),
            (
                "src/mcp/protocol/tests.rs",
                include_str!("mcp/protocol/tests.rs"),
            ),
            ("src/mcp/tools.rs", include_str!("mcp/tools.rs")),
            ("src/mcp/tools/tests.rs", include_str!("mcp/tools/tests.rs")),
            ("src/mcp/snapshot.rs", include_str!("mcp/snapshot.rs")),
            (
                "src/mcp/snapshot/tests.rs",
                include_str!("mcp/snapshot/tests.rs"),
            ),
            (
                "src/mcp/snapshot_manifest.rs",
                include_str!("mcp/snapshot_manifest.rs"),
            ),
            (
                "src/mcp/snapshot_manifest/tests.rs",
                include_str!("mcp/snapshot_manifest/tests.rs"),
            ),
            ("src/mcp/confinement.rs", include_str!("mcp/confinement.rs")),
            (
                "src/mcp/confinement/tests.rs",
                include_str!("mcp/confinement/tests.rs"),
            ),
            ("src/mcp/preflight.rs", include_str!("mcp/preflight.rs")),
            (
                "src/mcp/preflight/tests.rs",
                include_str!("mcp/preflight/tests.rs"),
            ),
            ("src/mcp/publication.rs", include_str!("mcp/publication.rs")),
            (
                "src/mcp/publication/tests.rs",
                include_str!("mcp/publication/tests.rs"),
            ),
            (
                "src/mcp/test_support.rs",
                include_str!("mcp/test_support.rs"),
            ),
            ("src/mcp/server_name.rs", include_str!("mcp/server_name.rs")),
            (
                "src/mcp/publish_staged_file_with_verification_hooks.rs",
                include_str!("mcp/publish_staged_file_with_verification_hooks.rs"),
            ),
        ];
        for (path, source) in sources {
            assert!(
                !source.contains(command.as_str()) && !source.contains(process.as_str()),
                "{path} must not construct a gh process"
            );
        }
    }
}
