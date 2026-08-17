---
change: CHG-0139-declaring-a-module-must-never-reduce-the-verification-a-change-receives
artifact: design
---

# Design

## The change

Track the declared modules that have **no** routing entry, and let their presence re-enable the
project-wide list:

    let mut unrouted_modules = Vec::new();
    for module in &record.affected_specs {
        match routing.component_verification_commands.get(module) {
            Some(component) => commands.extend(component.iter().cloned()),
            None => unrouted_modules.push(module.as_str()),
        }
    }
    if commands.is_empty() || !unrouted_modules.is_empty() {
        commands.extend(policy.verification_commands.iter().cloned());
    }

Four lines. The condition moves from per-change to per-module: `commands.is_empty()` still covers
"no module declared at all", and `!unrouted_modules.is_empty()` covers "a declared module nobody
routed".

## Why not simply make the project list a floor

Always appending `policy.verification_commands` would satisfy the monotonicity property and
`REQ-change-015`'s literal wording, and it would delete targeted verification — the feature
routing exists for. `change check`'s own description is "run targeted verification for one
change". A change scoped entirely to routed modules should still get its fast path.

The distinction the old code failed to draw is between *"this module has a faster equivalent"* and
*"nobody has said anything about this module"*. The first is an optimisation; the second is an
absence of information, and this release exists to stop absence being read as a positive result.

## The monotonicity property, stated directly

The test asserts a **superset relation** rather than a specific command list:

    for command in unrouted_only.iter().chain(routed_only.iter()) {
        assert!(both.contains(command));
    }

That catches any future regression of this shape, not only the instance found. Asserting the exact
expected list would pass while a differently-shaped suppression was introduced elsewhere in the
same function.

## The vacuity control

`a_fully_routed_change_still_runs_only_its_component_commands` asserts a routed-only scope gets
**exactly** its component command and specifically not `project-wide`. Without it, "always append
the project list" passes the monotonicity test while removing the optimisation — the naive fix,
green.

It passes on the unfixed binary too, which is what makes it a control: a control that
discriminates is not a control.

## Consequence, stated rather than discovered

Most changes now run more. 49 of this repository's 62 spec modules have no routing entry, so any
change naming one of them pulls in the full `cargo test` plus both release validators where it
previously did not. `change check` gets slower for those changes.

That is the correct trade — it was fast because it was not checking — but it is a real behaviour
change and belongs in the CHANGELOG rather than being found as a regression.
