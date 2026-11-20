## ADDED

### REQUIREMENT REQ-cmd-change-041

`change correct-owner --manifest` SHALL read the manifest the person names only through `read_typed_file` (REQ-util-011), with a bound of 4 MiB, so a FIFO, a device or a larger file is refused with the path and the reason instead of being waited on or read whole.

Acceptance Criteria
- A FIFO given as the manifest is refused without waiting.
