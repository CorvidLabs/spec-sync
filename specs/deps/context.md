# deps — Context

The deps module was added in v3.3.0 (#139) to validate cross-module dependency declarations. It builds a dependency graph from spec frontmatter `depends_on` fields and cross-references them against actual import statements found in source code for Rust, TypeScript, and Python.
