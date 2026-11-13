---
change: the-delivery-blob-deadline-bounds-each-git-read-not-the-manifest-work-between-reads
artifact: context
---

# Context

`DeliveryBlobStream` (src/change/consider_accepted_evidence_anchor.rs) reads a change's clean
delivery blobs one at a time through one `git cat-file --batch` child (REQ-change-144). Its
watchdog was armed once, for the stream's whole life, with the 120 s Git command deadline. The
acceptance manifest keeps the stream open across its loop and does config, owner and manifest
work between reads, so a large scope whose own processing took over 120 s got the child killed
while Git answered promptly, and the next clean file failed with a Git deadline error (#991).
The whole-scope read the stream replaced bounded only Git's I/O.
