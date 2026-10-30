# Design

Retain exact bin target files in ManifestModule.source_paths while retaining their parent in source_dirs. Merge target paths with a same-name package rather than replacing its source root. Coverage counts exact file ownership; generation unions same-name sources, preserves configured mappings, and excludes exact targets from guessed aliases.
