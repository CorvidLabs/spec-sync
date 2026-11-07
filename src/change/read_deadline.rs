// A deadline for read-only lifecycle commands (REQ-change-137, REQ-cmd-change-032).
//
// `change status`, `show`, `list` and `ship-status` may be given `--timeout <secs>` (or
// `SPECSYNC_TIMEOUT`). Long phases call `check_read_deadline` between items. Once the deadline has
// passed, the command's own thread flushes every open read cache, so the verdicts it completed are
// kept, prints one report naming the phase, its progress and the exact command that resumes it,
// and exits 124. Nothing after that point is printed, so a read cut short can never be reported as
// a verdict. A prefetch worker that sees the deadline stops taking work and lets the command's
// thread report. Lifecycle writes never take a deadline.

/// Exit status of a read-only lifecycle command stopped by its deadline.
pub const READ_TIMEOUT_EXIT_CODE: i32 = 124;

/// When a read must stop, and how long it was given.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ReadDeadline {
    until: Instant,
    limit_secs: u64,
}

/// How the command that set the deadline reports a timeout.
#[derive(Clone, Debug)]
struct ReadDeadlineReport {
    command: String,
    resume: String,
    json: bool,
}

thread_local! {
    static READ_DEADLINE: RefCell<Option<(ReadDeadline, Option<ReadDeadlineReport>)>> =
        const { RefCell::new(None) };
}

/// Clears this thread's read deadline when dropped.
pub struct ReadDeadlineGuard {
    previous: Option<(ReadDeadline, Option<ReadDeadlineReport>)>,
}

impl Drop for ReadDeadlineGuard {
    fn drop(&mut self) {
        let previous = self.previous.take();
        READ_DEADLINE.with(|slot| *slot.borrow_mut() = previous);
    }
}

/// The read timeout a command was given: `--timeout` when present, otherwise
/// `SPECSYNC_TIMEOUT` when it is a whole number of seconds, otherwise none.
pub fn read_timeout_secs(flag: Option<u64>) -> Option<u64> {
    flag.or_else(|| {
        std::env::var("SPECSYNC_TIMEOUT")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
    })
}

/// Give this thread's read a deadline of `limit_secs` from now. `command` names the command in
/// the report, and `resume` is the exact command that continues the work.
pub fn begin_read_deadline(
    limit_secs: u64,
    command: &str,
    resume: &str,
    json: bool,
) -> ReadDeadlineGuard {
    // A limit the clock can't hold is longer than any read, so it is no deadline at all, not a
    // panic (REQ-change-149).
    let Some(until) = Instant::now().checked_add(Duration::from_secs(limit_secs)) else {
        let previous = READ_DEADLINE.with(|slot| slot.borrow().clone());
        return ReadDeadlineGuard { previous };
    };
    let deadline = ReadDeadline { until, limit_secs };
    let report = ReadDeadlineReport {
        command: command.to_string(),
        resume: resume.to_string(),
        json,
    };
    let previous = READ_DEADLINE.with(|slot| slot.borrow_mut().replace((deadline, Some(report))));
    ReadDeadlineGuard { previous }
}

/// This thread's deadline, to hand to a worker thread.
pub(crate) fn current_read_deadline() -> Option<ReadDeadline> {
    READ_DEADLINE.with(|slot| slot.borrow().as_ref().map(|(deadline, _)| *deadline))
}

/// Give a worker thread the command's deadline. A worker never reports; it stops taking work.
pub(crate) fn begin_worker_read_deadline(deadline: Option<ReadDeadline>) -> ReadDeadlineGuard {
    let previous = READ_DEADLINE.with(|slot| {
        let mut slot = slot.borrow_mut();
        match deadline {
            Some(deadline) => slot.replace((deadline, None)),
            None => slot.take(),
        }
    });
    ReadDeadlineGuard { previous }
}

/// Whether this thread's deadline has passed.
pub(crate) fn read_deadline_passed() -> bool {
    READ_DEADLINE.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|(deadline, _)| Instant::now() >= deadline.until)
    })
}

/// Call between items of a long read phase. `done` of `total` items are complete (`total` 0 when
/// unknown). Past the deadline, the command's thread flushes its read caches, reports and exits
/// 124; a worker gets `Err` and stops.
pub(crate) fn check_read_deadline(phase: &str, done: usize, total: usize) -> Result<(), String> {
    if !read_deadline_passed() {
        return Ok(());
    }
    let Some((deadline, report)) = READ_DEADLINE.with(|slot| slot.borrow().clone()) else {
        return Ok(());
    };
    let Some(report) = report else {
        return Err(format!("read deadline passed during {phase}"));
    };
    flush_open_read_caches();
    let progress = (total > 0).then(|| format!("{done}/{total}"));
    if report.json {
        let body = serde_json::json!({
            "error": "timeout",
            "command": report.command,
            "timeout_secs": deadline.limit_secs,
            "phase": phase,
            "progress": progress,
            "wrote": false,
            "resume": report.resume,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&body).unwrap_or_else(|_| body.to_string())
        );
    } else {
        let at = progress
            .map(|progress| format!(" ({progress})"))
            .unwrap_or_default();
        eprintln!(
            "specsync: {} timed out after {}s during {phase}{at}. Nothing was written. \
             Completed work is cached, so rerunning continues from there:\n  {}",
            report.command, deadline.limit_secs, report.resume
        );
    }
    std::process::exit(READ_TIMEOUT_EXIT_CODE);
}
