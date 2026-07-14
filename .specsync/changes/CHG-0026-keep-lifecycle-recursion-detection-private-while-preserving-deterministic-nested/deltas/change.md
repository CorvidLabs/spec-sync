## MODIFIED

### SPEC SECTION Public API

**Exported Constants**

| Name | Description |
|------|-------------|
| `SDD_VERSION` | Current SDD project-layout version written by initialization |

**Exported Types**

| Type | Description |
|------|-------------|
| `ChangeState` | Six-state delivery lifecycle: draft, approved, implementing, verifying, accepted, archived |
| `ChangeKind` | Deterministic policy classification for feature, bug fix, refactor, migration, documentation, and operations work |
| `ArtifactKind` | Built-in or custom adaptive companion artifact selection |
| `SddPolicy` | Versioned enforcement, path, verification-command, template, and principles configuration |
| `ChangeRecord` | Durable machine state for one change workspace |
| `CreateChangeRequest` | Validated creation inputs grouped for CLI, imports, and agent clients |
| `ApprovalRecord` | Actor, timestamp, gate, digest, and optional note for one approval |
| `ReopenRecord` | Immutable audit event preserving superseded closing approval, prior verification, actor, reason, transition, and stale/current input digests |
| `ReopenResult` | Deterministic change-plus-audit result returned by the reopen transition |
| `ApprovalLedger` | Ordered portable approval and reopen history |
| `CommandEvidence` | Exit evidence for one configured verification command |
| `VerificationRecord` | Commit-bound verification result, contract digest, command results, and requirement coverage |
| `InterviewQuestion` | Stable deterministic question with choices and recommendation |
| `ChangeSummary` | Human/agent status projection with gate health and next action |
| `SddCheckReport` | Unified lifecycle validation errors, warnings, and checked-change count |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `load_policy` | `root: &Path` | `Option<SddPolicy>` | Load `.specsync/sdd.json`; absence leaves existing projects unenforced |
| `write_default_policy` | `root: &Path, verification_commands: Vec<String>` | `Result<(), String>` | Write new-project/adoption policy without overwriting existing policy |
| `create_change` | `root: &Path, request: CreateChangeRequest` | `Result<ChangeRecord, String>` | Create a sequential draft workspace and adaptive artifacts |
| `load_change` | `root: &Path, id: &str` | `Result<ChangeRecord, String>` | Load active or archived change state |
| `list_changes` | `root: &Path` | `Vec<ChangeRecord>` | List active changes in stable ID order |
| `next_questions` | `record: &ChangeRecord` | `Vec<InterviewQuestion>` | Return deterministic unanswered interview questions |
| `answer_question` | `root, id, question, answer` | `Result<ChangeRecord, String>` | Persist an interview answer and update adaptive artifacts |
| `add_dependency` | `root, id, dependency` | `Result<ChangeRecord, String>` | Declare ordering between active changes and invalidate stale approval digests |
| `approve_definition` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Validate and record mandatory definition approval |
| `start_implementation` | `root, id` | `Result<ChangeRecord, String>` | Enter implementation after approval and conflict validation |
| `verify_change` | `root, id` | `Result<VerificationRecord, String>` | Run configured tests and record commit/contract evidence |
| `reopen_change` | `root, id, actor, reason` | `Result<ReopenResult, String>` | Move stale accepted evidence to verifying and append an immutable supersession audit event |
| `accept_change` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Record closing approval and atomically apply semantic deltas |
| `archive_change` | `root, id` | `Result<PathBuf, String>` | Move an accepted workspace into the dated archive |
| `summarize_change` | `root, record` | `ChangeSummary` | Project gate health and next action for clients |
| `check_project` | `root: &Path` | `SddCheckReport` | Validate lifecycle state, approvals, conflicts, deltas, and path coverage |
| `check_project_quiet` | `root: &Path` | `SddCheckReport` | Run the same fail-closed lifecycle check while suppressing configured command output for machine-consumable report protocols |
| `adopt` | `root, dry_run, source` | `Result<Vec<String>, String>` | Preview or enable SDD and import OpenSpec or Spec Kit artifacts |
| `detect_verification_commands` | `root: &Path` | `Vec<String>` | Detect explicit fledge, Cargo, Bun, or Swift test commands |

**Exported Methods**

| Method | Description |
|--------|-------------|
| `as_str` | Return the stable serialized name for a change state or kind |
| `parse` | Parse user-facing change-kind or artifact names into typed values |
| `file_name` | Resolve an adaptive artifact to its safe Markdown filename |

Acceptance Criteria

- Nested lifecycle commands still fail once with the established deterministic contextual error.
- The process marker and diagnostic helper remain private binary implementation details.
- The canonical exported-function table contains no recursion helper.
