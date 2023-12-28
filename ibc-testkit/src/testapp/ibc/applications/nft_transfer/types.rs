#[derive(Debug)]
pub struct DummyNftTransferModule;

#[derive(Debug, Default)]
pub struct DummyNft;

#[derive(Debug, Default)]
pub struct DummyNftClass;

impl DummyNftTransferModule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DummyNftTransferModule {
    fn default() -> Self {
        Self::new()
    }
}
