# Context

CHG-0157 removed `deny_unknown_fields` from seventeen persisted-evidence structs so that a
field added in, say, 6.4 would not force 7.0. An adversarial pass over it afterwards could not
break the digest-invariance claim — proven at codegen and at runtime, both suites green on both
revisions — but found that the valve was claimed in three places where it does not hold, and
that one code comment asserts the opposite of what the code does.

This is the sibling-site pattern again, the one this release has paid for repeatedly: the fix
landed where the report pointed, and a parallel implementation survived beside it. Here the
parallel site was `agents.rs`, kept strict on the strength of a sentence nobody checked.

CHG-0157 is already archived and merged at `7bec5b31`, so this is a follow-on rather than an
amendment. That ordering is deliberate: the bisect history reads "the valve landed, then what
it overclaimed was corrected", which is what happened.
