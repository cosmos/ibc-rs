#[derive(Debug)]
pub struct DummyNftTransferModule;

#[derive(Debug)]
pub struct DummyNft;

#[derive(Debug)]
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
