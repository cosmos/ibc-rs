This patch release resolves two issues. It corrects the packet sequence number
encoding within Timeout message handlers to align with the big-endian format and
addresses a recursive call error during the conversion from connection `State`
to `i32`.

There are no consensus-breaking changes.
