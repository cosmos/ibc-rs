use ibc::apps::nft_transfer::types::{ClassData, ClassId, ClassUri, TokenData, TokenId, TokenUri};

#[derive(Debug)]
pub struct DummyNftTransferModule;

#[derive(Debug)]
pub struct DummyNft {
    pub class_id: ClassId,
    pub token_id: TokenId,
    pub token_uri: Option<TokenUri>,
    pub token_data: Option<TokenData>,
}

impl Default for DummyNft {
    fn default() -> Self {
        let class_id = "class_0".parse().expect("infallible");
        let token_id = "token_0".parse().expect("infallible");
        let token_uri = Some("http://example.com".parse().expect("infallible"));
        let data = r#"{"name":{"value":"Crypto Creatures"},"image":{"value":"binary","mime":"image/png"}}"#;
        let token_data = Some(data.parse().expect("infallible"));
        Self {
            class_id,
            token_id,
            token_uri,
            token_data,
        }
    }
}

#[derive(Debug)]
pub struct DummyNftClass {
    pub class_id: ClassId,
    pub class_uri: Option<ClassUri>,
    pub class_data: Option<ClassData>,
}

impl Default for DummyNftClass {
    fn default() -> Self {
        let class_id = "class_0".parse().expect("infallible");
        let class_uri = Some("http://example.com".parse().expect("infallible"));
        let data = r#"{"name":{"value":"Crypto Creatures"},"image":{"value":"binary","mime":"image/png"}}"#;
        let class_data = Some(data.parse().expect("infallible"));
        Self {
            class_id,
            class_uri,
            class_data,
        }
    }
}

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
