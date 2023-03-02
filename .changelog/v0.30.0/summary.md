This release contains an overhaul of the `send_packet()` and `send_transfer()` architecture.
The main gain is to separate into `send_packet_{validate,execute}()`, and similarly for 
`send_transfer()`.
