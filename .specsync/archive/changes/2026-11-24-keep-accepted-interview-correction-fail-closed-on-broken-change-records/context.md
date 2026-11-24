# Context

The terminal evidence isolation in PR #1114 uses a roster that sets aside per-change failures for read-only lifecycle summaries. The same tolerant helper was also called by `change correct`, which writes new approval, correction and verification records. Route that writer through the existing fully validated roster. Keep read-only isolation and other writers’ rules intact.

Review finding: https://github.com/CorvidLabs/spec-sync/pull/1114#discussion_r4674008149
