# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 5.x     | Yes       |
| 4.x     | No        |
| 3.x     | No        |
| < 3.0   | No        |

## Reporting a Vulnerability

If you discover a security vulnerability in SpecSync, please report it responsibly.

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please email: **security@corvidlabs.com**

Include:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### What to Expect

- **Acknowledgment** within 48 hours
- **Assessment** within 1 week
- **Fix or mitigation** for confirmed vulnerabilities as soon as possible
- Credit in the release notes (unless you prefer anonymity)

### Scope

SpecSync is a local CLI tool and GitHub Action. The primary security concerns are:

- **Path traversal** — spec file paths or cross-project references escaping the intended directory
- **Code injection** — malicious spec content causing unexpected behavior during parsing
- **GitHub Action security** — token exposure, unsafe input handling in CI
- **Dependency vulnerabilities** — issues in upstream Rust crates

### Out of Scope

- Bugs that require physical access to the machine running SpecSync
- Issues in dependencies that have already been reported upstream
- Denial of service via extremely large files (SpecSync is a local tool)

## Security Best Practices for Users

- Pin SpecSync to the current major version in CI (`uses: CorvidLabs/spec-sync@v6`), or to a full release tag/commit for stronger reproducibility
- Review spec files from untrusted sources before running validation
- Remote cross-project validation is opt-in; only pass `resolve --remote` or `resolve --verify` when network-backed reference checks are required
