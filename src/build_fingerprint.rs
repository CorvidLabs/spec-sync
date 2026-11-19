//! The compile configuration framed into the build fingerprint (#878, #981). `build.rs` includes
//! this file with `#[path]`, and the crate compiles it for its own tests.

/// The configuration variables Cargo gives a build script that can change what the binary
/// decides, as sorted `(name, value)` pairs: the target, the profile with its optimization and
/// debug settings, and every `cfg` and feature. Anything else, such as `PATH`, is left out.
pub(crate) fn configuration_frames(
    variables: impl IntoIterator<Item = (String, String)>,
) -> Vec<(String, String)> {
    let mut frames: Vec<(String, String)> = variables
        .into_iter()
        .filter(|(name, _)| {
            matches!(name.as_str(), "TARGET" | "PROFILE" | "OPT_LEVEL" | "DEBUG")
                || name.starts_with("CARGO_CFG_")
                || name.starts_with("CARGO_FEATURE_")
        })
        .collect();
    frames.sort();
    frames
}

/// The frames as one `NAME=value;NAME=value` line, the way `build.rs` records them.
pub(crate) fn configuration_record(frames: &[(String, String)]) -> String {
    frames
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(";")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variables(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
            .collect()
    }

    #[test]
    fn two_configurations_of_one_tree_frame_differently() {
        let linux = variables(&[
            ("TARGET", "x86_64-unknown-linux-gnu"),
            ("PROFILE", "debug"),
            ("CARGO_CFG_UNIX", ""),
        ]);
        let windows = variables(&[
            ("TARGET", "x86_64-pc-windows-msvc"),
            ("PROFILE", "debug"),
            ("CARGO_CFG_WINDOWS", ""),
        ]);
        let release = variables(&[
            ("TARGET", "x86_64-unknown-linux-gnu"),
            ("PROFILE", "release"),
            ("CARGO_CFG_UNIX", ""),
        ]);
        let featured = variables(&[
            ("TARGET", "x86_64-unknown-linux-gnu"),
            ("PROFILE", "debug"),
            ("CARGO_CFG_UNIX", ""),
            ("CARGO_FEATURE_EXTRA", "1"),
        ]);
        let framed = configuration_frames(linux);
        for other in [windows, release, featured] {
            assert_ne!(framed, configuration_frames(other));
        }
    }

    #[test]
    fn the_same_configuration_frames_the_same_whatever_else_is_set() {
        let one = variables(&[
            ("PROFILE", "debug"),
            ("TARGET", "t"),
            ("PATH", "/a"),
            ("CARGO_FEATURE_X", "1"),
        ]);
        let two = variables(&[
            ("CARGO_FEATURE_X", "1"),
            ("PATH", "/b"),
            ("TARGET", "t"),
            ("HOME", "/h"),
            ("PROFILE", "debug"),
        ]);
        assert_eq!(configuration_frames(one), configuration_frames(two));
        assert_eq!(
            configuration_record(&configuration_frames(variables(&[
                ("TARGET", "t"),
                ("PROFILE", "debug")
            ]))),
            "PROFILE=debug;TARGET=t"
        );
    }
}
