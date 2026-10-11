---
change: split-source-files-so-none-is-longer-than-5-000-lines
artifact: plan
---

# Plan

## Approach

Split every source file longer than 5,000 lines at a top-level item boundary and include the parts from the original module file.

## Out of scope

No API change, no behavior change, and no rewrite of the lifecycle.

## Steps

Cut the files, keep the tests in the same module, update the spec file lists, and compile.

## Risks

An include path can point at the wrong file. The compile and the existing tests catch that.

## Constraints consulted

The program must behave the same. GitHub read paths must not construct a gh process.
