---
change: answer-archived-change-status-change-list-and-change-status-within-2-seconds-on-raven
artifact: testing
---

# Testing

REQ-change-119 is covered by archived_status_git_processes_do_not_grow_with_other_archives (failed first on main: three more archived changes added 237 Git processes to one read, 210 with two archives and 447 with five), cached_archived_reads_answer_exactly_like_live_reads, cached_archived_history_still_refuses_a_later_hidden_downgrade, cached_archived_reads_recheck_an_edited_archive_and_a_moved_head and path_is_state_candidate_matches_only_one_change_s_state_paths. raven_sized_archived_status_benchmark is the timed benchmark (`fledge run bench-archived-reads`). Raven timings before and after are recorded in the pull request.
