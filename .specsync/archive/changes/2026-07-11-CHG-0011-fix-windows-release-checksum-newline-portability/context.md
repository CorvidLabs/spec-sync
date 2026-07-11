---
change: CHG-0011-fix-windows-release-checksum-newline-portability
artifact: context
---

# Context

The v5.0.0 Windows release checksum was correct but ended with CRLF because PowerShell `Out-File`
used the platform newline. Unix `shasum -c` treated the carriage return as part of the archive
filename and failed. The published v5.0.0 asset was repaired manually; this change prevents the
workflow from reproducing the defect.

Keep the existing five-target build and archive layout. Generate the Windows checksum with an
explicit ASCII LF byte sequence, then verify every platform's archive/checksum pair before upload.
