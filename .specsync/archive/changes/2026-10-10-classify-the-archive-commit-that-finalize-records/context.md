---
change: classify-the-archive-commit-that-finalize-records
artifact: context
---

# Context

`change finalize` commits the archive move. The integration test still staged `.specsync` and diffed that index against HEAD. After the commit, that diff is empty, so the classifier reported `archive_only=false` and CI failed the test job.

The test now classifies `HEAD^..HEAD` for `.specsync` and checks that the archive commit's parent is the implementation commit.
