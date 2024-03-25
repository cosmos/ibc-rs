use ibc::core::primitives::prelude::*;
use ibc_proto::cosmos::base::query::v1beta1::{
    PageRequest as RawPageRequest, PageResponse as RawPageResponse,
};

pub type Proof = Vec<u8>;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PageRequest {
    /// key is a value returned in PageResponse.next_key to begin
    /// querying the next page most efficiently. Only one of offset or key
    /// should be set.
    pub key: Vec<u8>,
    /// offset is a numeric offset that can be used when key is unavailable.
    /// It is less efficient than using key. Only one of offset or key should
    /// be set.
    pub offset: u64,
    /// limit is the total number of results to be returned in the result page.
    /// If left empty it will default to a value to be set by each app.
    pub limit: u64,
    /// count_total is set to true  to indicate that the result set should include
    /// a count of the total number of items available for pagination in UIs.
    /// count_total is only respected when offset is used. It is ignored when key
    /// is set.
    pub count_total: bool,
    /// reverse is set to true if results are to be returned in the descending order.
    pub reverse: bool,
}

impl PageRequest {
    pub fn all() -> Self {
        Self {
            limit: u64::MAX,
            ..Default::default()
        }
    }
}

impl From<PageRequest> for RawPageRequest {
    fn from(request: PageRequest) -> Self {
        Self {
            key: request.key,
            offset: request.offset,
            limit: request.limit,
            count_total: request.count_total,
            reverse: request.reverse,
        }
    }
}

impl From<RawPageRequest> for PageRequest {
    fn from(request: RawPageRequest) -> Self {
        Self {
            key: request.key,
            offset: request.offset,
            limit: request.limit,
            count_total: request.count_total,
            reverse: request.reverse,
        }
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PageResponse {
    /// next_key is the key to be passed to PageRequest.key to
    /// query the next page most efficiently. It will be empty if
    /// there are no more results.
    pub next_key: Vec<u8>,
    /// total is total number of results available if PageRequest.count_total
    /// was set, its value is undefined otherwise
    pub total: u64,
}

impl From<PageResponse> for RawPageResponse {
    fn from(response: PageResponse) -> Self {
        Self {
            next_key: response.next_key,
            total: response.total,
        }
    }
}

impl From<RawPageResponse> for PageResponse {
    fn from(response: RawPageResponse) -> Self {
        Self {
            next_key: response.next_key,
            total: response.total,
        }
    }
}
