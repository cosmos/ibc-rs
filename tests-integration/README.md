Integration tests that make use of the types exposed by `ibc-testkit`. These
tests also depend upon the other IBC crates. They live in a separate crate, that
is not meant to be published, to avoid circular dependencies.
