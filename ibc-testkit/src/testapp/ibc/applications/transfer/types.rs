#[derive(Debug)]
pub struct DummyTransferModule;

impl DummyTransferModule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DummyTransferModule {
    fn default() -> Self {
        Self::new()
    }
}
