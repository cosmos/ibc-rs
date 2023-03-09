use crate::prelude::*;

use sha2::{Digest, Sha256};

use super::error::TokenTransferError;
use crate::applications::transfer::acknowledgement::TokenTransferAcknowledgement;
use crate::applications::transfer::events::{AckEvent, AckStatusEvent, RecvEvent, TimeoutEvent};
use crate::applications::transfer::packet::PacketData;
use crate::applications::transfer::relay::refund_packet_token_execute;
use crate::applications::transfer::relay::{
    on_recv_packet::process_recv_packet_execute, refund_packet_token_validate,
};
use crate::applications::transfer::{PrefixedCoin, PrefixedDenom, VERSION};
use crate::core::ics04_channel::channel::{Counterparty, Order};
use crate::core::ics04_channel::context::{
    SendPacketExecutionContext, SendPacketValidationContext,
};
use crate::core::ics04_channel::handler::ModuleExtras;
use crate::core::ics04_channel::msgs::acknowledgement::Acknowledgement;
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics04_channel::Version;
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::signer::Signer;

pub trait TokenTransferValidationContext: SendPacketValidationContext {
    type AccountId: TryFrom<Signer>;

    /// get_port returns the portID for the transfer module.
    fn get_port(&self) -> Result<PortId, TokenTransferError>;

    /// Returns the escrow account id for a port and channel combination
    fn get_escrow_account(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Self::AccountId, TokenTransferError>;

    /// Returns Ok() if the host chain supports sending coins.
    fn can_send_coins(&self) -> Result<(), TokenTransferError>;

    /// Returns Ok() if the host chain supports receiving coins.
    fn can_receive_coins(&self) -> Result<(), TokenTransferError>;

    /// Validates the sender and receiver accounts and the coin inputs
    fn send_coins_validate(
        &self,
        from_account: &Self::AccountId,
        to_account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// Validates the receiver account and the coin input
    fn mint_coins_validate(
        &self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// Validates the sender account and the coin input
    fn burn_coins_validate(
        &self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// Returns a hash of the prefixed denom.
    /// Implement only if the host chain supports hashed denominations.
    fn denom_hash_string(&self, _denom: &PrefixedDenom) -> Option<String> {
        None
    }
}

pub trait TokenTransferExecutionContext:
    TokenTransferValidationContext + SendPacketExecutionContext
{
    /// This function should enable sending ibc fungible tokens from one account to another
    fn send_coins_execute(
        &mut self,
        from_account: &Self::AccountId,
        to_account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// This function to enable minting ibc tokens to a user account
    fn mint_coins_execute(
        &mut self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// This function should enable burning of minted tokens in a user account
    fn burn_coins_execute(
        &mut self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;
}

// https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-028-public-key-addresses.md
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

#[allow(clippy::too_many_arguments)]
pub fn on_chan_open_init_validate(
    ctx: &impl TokenTransferValidationContext,
    order: Order,
    _connection_hops: &[ConnectionId],
    port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    version: &Version,
) -> Result<(), TokenTransferError> {
    if order != Order::Unordered {
        return Err(TokenTransferError::ChannelNotUnordered {
            expect_order: Order::Unordered,
            got_order: order,
        });
    }
    let bound_port = ctx.get_port()?;
    if port_id != &bound_port {
        return Err(TokenTransferError::InvalidPort {
            port_id: port_id.clone(),
            exp_port_id: bound_port,
        });
    }

    if !version.is_empty() && version != &Version::new(VERSION.to_string()) {
        return Err(TokenTransferError::InvalidVersion {
            expect_version: Version::new(VERSION.to_string()),
            got_version: version.clone(),
        });
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn on_chan_open_init_execute(
    _ctx: &mut impl TokenTransferExecutionContext,
    _order: Order,
    _connection_hops: &[ConnectionId],
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    _version: &Version,
) -> Result<(ModuleExtras, Version), TokenTransferError> {
    Ok((ModuleExtras::empty(), Version::new(VERSION.to_string())))
}

#[allow(clippy::too_many_arguments)]
pub fn on_chan_open_try_validate(
    _ctx: &impl TokenTransferValidationContext,
    order: Order,
    _connection_hops: &[ConnectionId],
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    counterparty_version: &Version,
) -> Result<(), TokenTransferError> {
    if order != Order::Unordered {
        return Err(TokenTransferError::ChannelNotUnordered {
            expect_order: Order::Unordered,
            got_order: order,
        });
    }
    if counterparty_version != &Version::new(VERSION.to_string()) {
        return Err(TokenTransferError::InvalidCounterpartyVersion {
            expect_version: Version::new(VERSION.to_string()),
            got_version: counterparty_version.clone(),
        });
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn on_chan_open_try_execute(
    _ctx: &mut impl TokenTransferExecutionContext,
    _order: Order,
    _connection_hops: &[ConnectionId],
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    _counterparty_version: &Version,
) -> Result<(ModuleExtras, Version), TokenTransferError> {
    Ok((ModuleExtras::empty(), Version::new(VERSION.to_string())))
}

pub fn on_chan_open_ack_validate(
    _ctx: &impl TokenTransferValidationContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
    counterparty_version: &Version,
) -> Result<(), TokenTransferError> {
    if counterparty_version != &Version::new(VERSION.to_string()) {
        return Err(TokenTransferError::InvalidCounterpartyVersion {
            expect_version: Version::new(VERSION.to_string()),
            got_version: counterparty_version.clone(),
        });
    }

    Ok(())
}

pub fn on_chan_open_ack_execute(
    _ctx: &mut impl TokenTransferExecutionContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty_version: &Version,
) -> Result<ModuleExtras, TokenTransferError> {
    Ok(ModuleExtras::empty())
}

pub fn on_chan_open_confirm_validate(
    _ctx: &impl TokenTransferValidationContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<(), TokenTransferError> {
    Ok(())
}

pub fn on_chan_open_confirm_execute(
    _ctx: &mut impl TokenTransferExecutionContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<ModuleExtras, TokenTransferError> {
    Ok(ModuleExtras::empty())
}

pub fn on_chan_close_init_validate(
    _ctx: &impl TokenTransferValidationContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<(), TokenTransferError> {
    Err(TokenTransferError::CantCloseChannel)
}
pub fn on_chan_close_init_execute(
    _ctx: &mut impl TokenTransferExecutionContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<ModuleExtras, TokenTransferError> {
    Err(TokenTransferError::CantCloseChannel)
}

pub fn on_chan_close_confirm_validate(
    _ctx: &impl TokenTransferValidationContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<(), TokenTransferError> {
    Ok(())
}

pub fn on_chan_close_confirm_execute(
    _ctx: &mut impl TokenTransferExecutionContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<ModuleExtras, TokenTransferError> {
    Ok(ModuleExtras::empty())
}

pub fn on_recv_packet_execute(
    ctx_b: &mut impl TokenTransferExecutionContext,
    packet: &Packet,
) -> (ModuleExtras, Acknowledgement) {
    let data = match serde_json::from_slice::<PacketData>(&packet.data) {
        Ok(data) => data,
        Err(_) => {
            let ack = TokenTransferAcknowledgement::Error(
                TokenTransferError::PacketDataDeserialization.to_string(),
            );
            return (ModuleExtras::empty(), ack.into());
        }
    };

    let (mut extras, ack) = match process_recv_packet_execute(ctx_b, packet, data.clone()) {
        Ok(extras) => (extras, TokenTransferAcknowledgement::success()),
        Err((extras, error)) => (extras, TokenTransferAcknowledgement::from_error(error)),
    };

    let recv_event = RecvEvent {
        receiver: data.receiver,
        denom: data.token.denom,
        amount: data.token.amount,
        success: ack.is_successful(),
    };
    extras.events.push(recv_event.into());

    (extras, ack.into())
}

pub fn on_acknowledgement_packet_validate<Ctx>(
    ctx: &Ctx,
    packet: &Packet,
    acknowledgement: &Acknowledgement,
    _relayer: &Signer,
) -> Result<(), TokenTransferError>
where
    Ctx: TokenTransferValidationContext,
{
    let data = serde_json::from_slice::<PacketData>(&packet.data)
        .map_err(|_| TokenTransferError::PacketDataDeserialization)?;

    let acknowledgement =
        serde_json::from_slice::<TokenTransferAcknowledgement>(acknowledgement.as_ref())
            .map_err(|_| TokenTransferError::AckDeserialization)?;

    if !acknowledgement.is_successful() {
        refund_packet_token_validate(ctx, packet, &data)?;
    }

    Ok(())
}

pub fn on_acknowledgement_packet_execute(
    ctx: &mut impl TokenTransferExecutionContext,
    packet: &Packet,
    acknowledgement: &Acknowledgement,
    _relayer: &Signer,
) -> (ModuleExtras, Result<(), TokenTransferError>) {
    let data = match serde_json::from_slice::<PacketData>(&packet.data) {
        Ok(data) => data,
        Err(_) => {
            return (
                ModuleExtras::empty(),
                Err(TokenTransferError::PacketDataDeserialization),
            );
        }
    };

    let acknowledgement =
        match serde_json::from_slice::<TokenTransferAcknowledgement>(acknowledgement.as_ref()) {
            Ok(ack) => ack,
            Err(_) => {
                return (
                    ModuleExtras::empty(),
                    Err(TokenTransferError::AckDeserialization),
                );
            }
        };

    if !acknowledgement.is_successful() {
        if let Err(err) = refund_packet_token_execute(ctx, packet, &data) {
            return (ModuleExtras::empty(), Err(err));
        }
    }

    let ack_event = AckEvent {
        receiver: data.receiver,
        denom: data.token.denom,
        amount: data.token.amount,
        acknowledgement: acknowledgement.clone(),
    };

    let extras = ModuleExtras {
        events: vec![ack_event.into(), AckStatusEvent { acknowledgement }.into()],
        log: Vec::new(),
    };

    (extras, Ok(()))
}

pub fn on_timeout_packet_validate<Ctx>(
    ctx: &Ctx,
    packet: &Packet,
    _relayer: &Signer,
) -> Result<(), TokenTransferError>
where
    Ctx: TokenTransferValidationContext,
{
    let data = serde_json::from_slice::<PacketData>(&packet.data)
        .map_err(|_| TokenTransferError::PacketDataDeserialization)?;

    refund_packet_token_validate(ctx, packet, &data)?;

    Ok(())
}

pub fn on_timeout_packet_execute(
    ctx: &mut impl TokenTransferExecutionContext,
    packet: &Packet,
    _relayer: &Signer,
) -> (ModuleExtras, Result<(), TokenTransferError>) {
    let data = match serde_json::from_slice::<PacketData>(&packet.data) {
        Ok(data) => data,
        Err(_) => {
            return (
                ModuleExtras::empty(),
                Err(TokenTransferError::PacketDataDeserialization),
            );
        }
    };

    if let Err(err) = refund_packet_token_execute(ctx, packet, &data) {
        return (ModuleExtras::empty(), Err(err));
    }

    let timeout_event = TimeoutEvent {
        refund_receiver: data.sender,
        refund_denom: data.token.denom,
        refund_amount: data.token.amount,
    };

    let extras = ModuleExtras {
        events: vec![timeout_event.into()],
        log: Vec::new(),
    };

    (extras, Ok(()))
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use subtle_encoding::bech32;

    use crate::applications::transfer::context::cosmos_adr028_escrow_address;
    use crate::core::ics04_channel::channel::{Counterparty, Order};
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
    use crate::mock::context::MockContext;

    fn get_defaults() -> (
        MockContext,
        Order,
        Vec<ConnectionId>,
        PortId,
        ChannelId,
        Counterparty,
    ) {
        let ctx = MockContext::default();
        let order = Order::Unordered;
        let connection_hops = vec![ConnectionId::new(1)];
        let port_id = PortId::transfer();
        let channel_id = ChannelId::new(1);
        let counterparty = Counterparty::new(port_id.clone(), Some(channel_id.clone()));

        (
            ctx,
            order,
            connection_hops,
            port_id,
            channel_id,
            counterparty,
        )
    }

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

    /// If the relayer passed "", indicating that it wants us to return the versions we support.
    /// We currently only support ics20
    #[test]
    fn test_on_chan_open_init_empty_version() {
        let (mut ctx, order, connection_hops, port_id, channel_id, counterparty) = get_defaults();

        let in_version = Version::new("".to_string());

        let (_, out_version) = on_chan_open_init_execute(
            &mut ctx,
            order,
            &connection_hops,
            &port_id,
            &channel_id,
            &counterparty,
            &in_version,
        )
        .unwrap();

        assert_eq!(out_version, Version::new(VERSION.to_string()));
    }

    /// If the relayer passed in the only supported version (ics20), then return ics20
    #[test]
    fn test_on_chan_open_init_ics20_version() {
        let (mut ctx, order, connection_hops, port_id, channel_id, counterparty) = get_defaults();

        let in_version = Version::new(VERSION.to_string());
        let (_, out_version) = on_chan_open_init_execute(
            &mut ctx,
            order,
            &connection_hops,
            &port_id,
            &channel_id,
            &counterparty,
            &in_version,
        )
        .unwrap();

        assert_eq!(out_version, Version::new(VERSION.to_string()));
    }

    /// If the relayer passed in an unsupported version, then fail
    #[test]
    fn test_on_chan_open_init_incorrect_version() {
        let (ctx, order, connection_hops, port_id, channel_id, counterparty) = get_defaults();

        let in_version = Version::new("some-unsupported-version".to_string());
        let res = on_chan_open_init_validate(
            &ctx,
            order,
            &connection_hops,
            &port_id,
            &channel_id,
            &counterparty,
            &in_version,
        );

        assert!(res.is_err());
    }

    /// If the counterparty supports ics20, then return ics20
    #[test]
    fn test_on_chan_open_try_counterparty_correct_version() {
        let (mut ctx, order, connection_hops, port_id, channel_id, counterparty) = get_defaults();

        let counterparty_version = Version::new(VERSION.to_string());

        let (_, out_version) = on_chan_open_try_execute(
            &mut ctx,
            order,
            &connection_hops,
            &port_id,
            &channel_id,
            &counterparty,
            &counterparty_version,
        )
        .unwrap();

        assert_eq!(out_version, Version::new(VERSION.to_string()));
    }

    /// If the counterparty doesn't support ics20, then fail
    #[test]
    fn test_on_chan_open_try_counterparty_incorrect_version() {
        let (ctx, order, connection_hops, port_id, channel_id, counterparty) = get_defaults();

        let counterparty_version = Version::new("some-unsupported-version".to_string());

        let res = on_chan_open_try_validate(
            &ctx,
            order,
            &connection_hops,
            &port_id,
            &channel_id,
            &counterparty,
            &counterparty_version,
        );

        assert!(res.is_err());
    }
}
