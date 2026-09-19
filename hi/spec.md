---
hi: 1
families: [SPEC]
---

# The module contract

## Intent

A contract is a document a person wrote for other people, with just enough machine-checkable structure that it cannot rot. It should read like something worth handing to a new teammate — why the module exists, what it promises, what must stay true, how it fails — rather than a generated API dump. Scaffolding should get me to a real starting point without pretending to know things about my code that it cannot see.

## Criteria

- **SPEC-1**  A module's contract is a plain markdown file I can read, review and diff like any other file in the repository.
- **SPEC-2**  The contract says which source files it governs, so there is never a question about what it covers.
- **SPEC-3**  The contract lists the module's public surface in a table a reader can scan.
  - **SPEC-3.a**  The names in that table are the ones the code is held to.
  - **SPEC-3.b**  I can document things that are not exports — endpoints, configuration keys, component state — without the tool trying to match them to code.
- **SPEC-4**  The contract carries what a reader needs beyond the API: why the module exists, what must stay true, how it behaves and how it fails.
- **SPEC-5**  A project can decide for itself which sections a contract must have.
- **SPEC-6**  Beside every contract sit its companions: the intent, the work still to do, the background a newcomer needs, and the evidence.
  - **SPEC-6.a**  Requirements live in the intent file rather than inline in the contract.
- **SPEC-7**  Scaffolding a module gives me the contract and its companions already pointed at the source files it found.
  - **SPEC-7.a**  A scaffold fills in only what it can see and leaves the rest to me, rather than inventing facts about my code.
  - **SPEC-7.b**  A placeholder left behind by a scaffold is caught before it can ship.
  - **SPEC-7.c**  When I do not know what to write, I can be walked through a new contract question by question.
  - **SPEC-7.d**  A team's own template can stand in for the default one.
- **SPEC-8**  I can read a contract through the eyes of one audience — developer, QA, product or agent — instead of reading all of it.
- **SPEC-9**  A contract carries a version that moves whenever its content does.
  - **SPEC-9.a**  Every change to a contract leaves a note in the file saying what changed.
