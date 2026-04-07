# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 3.x     | Yes       |
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

- Pin SpecSync to a specific version in CI (`uses: CorvidLabs/spec-sync@v3`)
- Review spec files from untrusted sources before running validation
- Use `--no-cross-project` if you don't need cross-project references in CI
