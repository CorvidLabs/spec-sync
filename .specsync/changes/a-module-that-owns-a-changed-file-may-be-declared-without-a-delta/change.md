---
id: a-module-that-owns-a-changed-file-may-be-declared-without-a-delta
state: draft
type: feature
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# A module that owns a changed file may be declared without a delta

## Intent

A module that owns a changed file may be declared without a delta

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a change declaring auth and sessions with a delta only for sessions, whose declared file src/auth.rs auth owns, is approved, materialized, verified and accepted, with the sessions delta applied and the auth spec untouched; the same scope over a file only sessions owns is refused at approval naming auth

## No-spec Rationale

Not applicable
