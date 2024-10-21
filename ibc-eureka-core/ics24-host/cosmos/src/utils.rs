use ibc_app_transfer_types::VERSION;
use ibc_core_host_types::identifiers::{ChannelId, PortId};
use ibc_primitives::prelude::*;
use sha2::{Digest, Sha256};

/// Helper function to generate an escrow address for a given port and channel
/// ids according to the format specified in the Cosmos SDK
/// [`ADR-028`](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-028-public-key-addresses.md)
pub fn cosmos_adr028_escrow_address(port_id: &PortId, channel_id: &ChannelId) -> Vec<u8> {
    let contents = format!("{port_id}/{channel_id}");

    let mut hasher = Sha256::new();
    hasher.update(VERSION.as_bytes());
    hasher.update([0]);
    hasher.update(contents.as_bytes());

    let mut hash = hasher.finalize().to_vec();
    hash.truncate(20);
    hash
}

#[cfg(test)]
mod tests {
    use subtle_encoding::bech32;

    use super::*;

    #[test]
    fn test_cosmos_escrow_address() {
        fn assert_eq_escrow_address(port_id: &str, channel_id: &str, address: &str) {
            let port_id = port_id.parse().unwrap();
            let channel_id = channel_id.parse().unwrap();
            let gen_address = {
                let addr = cosmos_adr028_escrow_address(&port_id, &channel_id);
                bech32::encode("cosmos", addr)
            };
            assert_eq!(gen_address, address.to_owned())
        }

        // addresses obtained using `gaiad query ibc-transfer escrow-address [port-id] [channel-id]`
        assert_eq_escrow_address(
            "transfer",
            "channel-141",
            "cosmos1x54ltnyg88k0ejmk8ytwrhd3ltm84xehrnlslf",
        );
        assert_eq_escrow_address(
            "transfer",
            "channel-207",
            "cosmos1ju6tlfclulxumtt2kglvnxduj5d93a64r5czge",
        );
        assert_eq_escrow_address(
            "transfer",
            "channel-187",
            "cosmos177x69sver58mcfs74x6dg0tv6ls4s3xmmcaw53",
        );
    }
}
