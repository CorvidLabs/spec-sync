## ADDED

### REQUIREMENT REQ-change-085

Terminal evidence SHALL be trusted only against the commit where that evidence entered history, and no later commit that re-introduces the same package SHALL be usable as its anchor.

Acceptance Criteria
- A commit that re-introduces a package cannot authenticate the evidence it carries, because the check compares committed bytes against the working tree and would otherwise be satisfied by any commit of the current state, whatever that state has become.
- The rule applies wherever a package can be re-introduced, not only where it is archived: a package moved back to an active workspace and archived again is re-introduced at a path SpecSync itself writes, and is covered.
- A package is identified for this purpose by the identity recorded inside its evidence, not by the name of the directory holding it, because the directory name is not part of a package's identity anywhere else.
- Relocating a package without altering it continues to authenticate, so history can be reorganised and the earlier evidence still stands.
- Every archive that authenticates before this rule is applied continues to authenticate after it.
