## ADDED

### REQUIREMENT REQ-cli-010

SpecSync SHALL remain correct for repositories authored on any host platform, including platforms it publishes no binary for, and the set of platforms it publishes binaries for SHALL NOT be read as the set of content it must handle.

Acceptance Criteria
- The published binary set as of 6.0 is Linux and macOS. Windows is not a supported target, and the packaged Action refuses a Windows runner with a message naming WSL rather than requesting a release asset that is not published.
- Dropping a published platform does not license removing handling for content authored on it. CRLF line endings, Windows-reserved filenames, Windows-invalid characters, and backslash path separators are read by Linux and macOS users whenever a repository has one contributor on Windows, so that handling is scoped to the content, not to the binary set.
- The crate continues to build from source on Windows. What ends at 6.0 is the prebuilt executable, not the platform's ability to compile one.
