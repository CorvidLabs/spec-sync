---
change: the-persistent-read-cache-stays-bounded
artifact: testing
---

# Testing

- `the_read_cache_holds_no_approvals_bytes_and_reads_them_back_unchanged`: two archived changes;
  a live index, a cold scoped index and a warm scoped index give the same ledgers per
  introduction, and the cache file holds `archive_introductions` but no `approved_scope` text.
  Red on main: the cache file holds the approvals JSON inline.
- `the_read_cache_keeps_at_most_4096_archive_introductions`: a flush of 4097 entries keeps at
  most 4096, and keeps the one the command used even though it sorts first. Red on main: 4097
  kept.
- `a_read_cache_file_over_8_mib_starts_fresh`: a valid cache file over 8 MiB is not loaded. Red
  on main: its 90,000 entries load.
- Existing archive, finalize, reopen and read-cache tests, including the warm and cold
  Git-process bounds, stay green; fmt and clippy clean.
