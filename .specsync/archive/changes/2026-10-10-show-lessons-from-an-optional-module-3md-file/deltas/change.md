## MODIFIED

### REQUIREMENT REQ-change-106
The system SHALL show, when a change is opened, shown, or asked its status, the lessons already written for the modules it touches and the project's principles when a principles file is configured. The text result and the structured result SHALL carry the same briefing. A module context that holds only its generated scaffold SHALL NOT be presented as knowledge. The briefing SHALL be bounded and SHALL name the file for anything left unread. An unreadable context or principles file SHALL NOT fail the command.

Acceptance Criteria
- The briefing lists each affected module context that has substantive lines, with the path, the substantive line count, a bounded excerpt, and whether the excerpt was cut.
- When `specs/<module>/<module>.3md` exists, lessons come from every plane after the first, and the briefing names that file. The first plane is context. `context.md` is not the lesson source for that module.
- When that file does not exist, `context.md` remains the lesson source.
- An unreadable or invalid 3md file does not fail the command.
- The context path is the directory of the module's canonical spec, including a non-default specs directory or a registry mapping.
- A multiline HTML comment is not a lesson.
- A scaffold-only context is absent.
- A configured principles file is named, with a bounded excerpt, or marked missing when it cannot be read.
- No principles file produces no principles entry.
- The briefing is computed and is not stored on the change record.
