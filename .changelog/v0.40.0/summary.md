This release primarily consolidated the modules in the ibc-rs crate, removed many legacy items, and documented every item in the crate. This represents a big step towards v1.0. Very few items changed name; most were just moved to elsewhere in the module tree. Perhaps a good heuristic to fix the breaking changes is the remove the faulty `use` statements, and have your editor re-import the item.

There were also a few minor validation checks missing, which we added. These were pretty much the last remaining known ones.

There are breaking changes.
