---
change: CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c
artifact: docs
---

# Docs

Document batch `change correct-owner` beside the single-path form:

```text
specsync change correct-owner <id> \
  --path src/a.rs --path src/b.rs --spec owning_module \
  --actor "<human>" --reason "<text>"

specsync change correct-owner <id> \
  --manifest owners.json \
  --actor "<human>" --reason "<text>"

specsync change correct-owner <id> \
  --all-missing --spec owning_module \
  --actor "<human>" --reason "<text>"
```

Emphasize: every entry is still an independent audited correction; failure applies nothing; one
fresh approve → verify → accept cycle follows the whole batch.
