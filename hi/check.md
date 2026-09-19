---
hi: 1
families: [CHECK]
---

# Catching drift

## Intent

The point of the whole product: a document that says what a module does should not be able to quietly stop being true. One command compares every contract to the code it claims to describe, in both directions, and says plainly which side is wrong. It should be fast enough to run without thinking about it, honest about the difference between a lie and an omission, and useful from the first run without any configuration, network or account.

## Criteria

- **CHECK-1**  One command tells me whether every module's written contract still matches the code it claims to describe.
  - **CHECK-1.a**  The check reads only what is on disk, so it never needs a network call, an account or a model.
  - **CHECK-1.b**  Re-running it on a tree I have not touched is fast enough to keep in my inner loop.
  - **CHECK-1.c**  I can point the check at a single module while I am working inside it.
- **CHECK-2**  A contract that names something the code no longer has is an error, because the document is lying.
- **CHECK-3**  Code that exports something the contract never mentions is a warning, because the document is merely incomplete.
  - **CHECK-3.a**  I can promote every warning to a failure when nothing undocumented should slip through.
- **CHECK-4**  A source file the contract claims and the repository no longer has fails the check.
- **CHECK-5**  A section the project requires and the contract leaves out fails the check.
- **CHECK-6**  A database table or column the contract documents and the schema lacks fails the check.
- **CHECK-7**  Public surface is recognised in whatever language a file happens to be written in, without me teaching it the language.
- **CHECK-8**  The dependencies a contract declares between my own modules are held to the imports in the code.
  - **CHECK-8.a**  A cycle between modules is reported rather than left for someone to trip over.
  - **CHECK-8.b**  An import that no contract declares is reported.
- **CHECK-9**  Every failure names the file, the symbol and the fix, so I never have to read the tool's own source to understand it.
- **CHECK-10**  When the code is right and the document is behind, the missing rows can be written into the contract for me.
  - **CHECK-10.a**  I can see exactly what that would write before anything touches a file.
  - **CHECK-10.b**  A copy of the original is kept when a contract is rewritten for me.
- **CHECK-11**  I can leave the check running so drift shows up the moment a file changes.
- **CHECK-12**  The same problems appear in my editor on the lines they belong to, without me switching to a terminal.
  - **CHECK-12.a**  Coverage and quality are readable from inside the editor too, not only from a terminal.
