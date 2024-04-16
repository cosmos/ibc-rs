use ibc_client_wasm_types::{SUBJECT_PREFIX, SUBSTITUTE_PREFIX};

/// The MigrationPrefix enumerates the prefix type used during migration mode.
/// The migration mode is activated when there is an incoming
/// `MigrateClientStore` message. It specifies the prefix key for either the
/// subject or substitute store, or none if the migration is not active.
#[derive(Clone, Debug)]
pub enum MigrationPrefix {
    Subject,
    Substitute,
    None,
}

impl MigrationPrefix {
    pub fn key(&self) -> &[u8] {
        match self {
            MigrationPrefix::Subject => SUBJECT_PREFIX,
            MigrationPrefix::Substitute => SUBSTITUTE_PREFIX,
            MigrationPrefix::None => b"",
        }
    }
}

/// Travel is an enum to represent the direction of travel in the context of
/// height.
#[derive(Clone, Debug)]
pub enum HeightTravel {
    Next,
    Prev,
}
