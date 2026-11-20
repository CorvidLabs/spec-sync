## ADDED

### REQUIREMENT REQ-cmd-scaffold-009

`scaffold --template` SHALL read each template file in the folder the person names only through `read_typed_file` (REQ-util-011), with a bound of 8 MiB, so a FIFO, a device or a larger file is refused with the path and the reason, and nothing is scaffolded.

Acceptance Criteria
- A template folder whose `spec.md` is a FIFO is refused without waiting.
