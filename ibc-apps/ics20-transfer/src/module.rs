use ibc_app_transfer_types::error::TokenTransferError;
use ibc_app_transfer_types::events::{AckEvent, AckStatusEvent, RecvEvent, TimeoutEvent};
use ibc_app_transfer_types::packet::PacketData;
use ibc_app_transfer_types::{ack_success_b64, VERSION};
use ibc_core::channel::types::acknowledgement::{Acknowledgement, AcknowledgementStatus};
use ibc_core::channel::types::channel::{Counterparty, Order};
use ibc_core::channel::types::packet::Packet;
use ibc_core::channel::types::Version;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;
use ibc_core::router::types::module::ModuleExtras;

use crate::context::{TokenTransferExecutionContext, TokenTransferValidationContext};
use crate::handler::{
    process_recv_packet_execute, refund_packet_token_execute, refund_packet_token_validate,
};

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

    if !version.is_empty() {
        version
            .verify_is_expected(Version::new(VERSION.to_string()))
            .map_err(ContextError::from)?;
    }

    Ok(())
}

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

    counterparty_version
        .verify_is_expected(Version::new(VERSION.to_string()))
        .map_err(ContextError::from)?;

    Ok(())
}

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
    counterparty_version
        .verify_is_expected(Version::new(VERSION.to_string()))
        .map_err(ContextError::from)?;

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
    let Ok(data) = serde_json::from_slice::<PacketData>(&packet.data) else {
        let ack =
            AcknowledgementStatus::error(TokenTransferError::PacketDataDeserialization.into());
        return (ModuleExtras::empty(), ack.into());
    };

    let (mut extras, ack) = match process_recv_packet_execute(ctx_b, packet, data.clone()) {
        Ok(extras) => (extras, AcknowledgementStatus::success(ack_success_b64())),
        Err((extras, error)) => (extras, AcknowledgementStatus::error(error.into())),
    };

    let recv_event = RecvEvent {
        sender: data.sender,
        receiver: data.receiver,
        denom: data.token.denom,
        amount: data.token.amount,
        memo: data.memo,
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

    let acknowledgement = serde_json::from_slice::<AcknowledgementStatus>(acknowledgement.as_ref())
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
    let Ok(data) = serde_json::from_slice::<PacketData>(&packet.data) else {
        return (
            ModuleExtras::empty(),
            Err(TokenTransferError::PacketDataDeserialization),
        );
    };

    let Ok(acknowledgement) =
        serde_json::from_slice::<AcknowledgementStatus>(acknowledgement.as_ref())
    else {
        return (
            ModuleExtras::empty(),
            Err(TokenTransferError::AckDeserialization),
        );
    };

    if !acknowledgement.is_successful() {
        if let Err(err) = refund_packet_token_execute(ctx, packet, &data) {
            return (ModuleExtras::empty(), Err(err));
        }
    }

    let ack_event = AckEvent {
        sender: data.sender,
        receiver: data.receiver,
        denom: data.token.denom,
        amount: data.token.amount,
        memo: data.memo,
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
    let Ok(data) = serde_json::from_slice::<PacketData>(&packet.data) else {
        return (
            ModuleExtras::empty(),
            Err(TokenTransferError::PacketDataDeserialization),
        );
    };

    if let Err(err) = refund_packet_token_execute(ctx, packet, &data) {
        return (ModuleExtras::empty(), Err(err));
    }

    let timeout_event = TimeoutEvent {
        refund_receiver: data.sender,
        refund_denom: data.token.denom,
        refund_amount: data.token.amount,
        memo: data.memo,
    };

    let extras = ModuleExtras {
        events: vec![timeout_event.into()],
        log: Vec::new(),
    };

    (extras, Ok(()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ack_ser() {
        fn ser_json_assert_eq(ack: AcknowledgementStatus, json_str: &str) {
            let ser = serde_json::to_string(&ack).unwrap();
            assert_eq!(ser, json_str)
        }

        ser_json_assert_eq(
            AcknowledgementStatus::success(ack_success_b64()),
            r#"{"result":"AQ=="}"#,
        );
        ser_json_assert_eq(
            AcknowledgementStatus::error(TokenTransferError::PacketDataDeserialization.into()),
            r#"{"error":"failed to deserialize packet data"}"#,
        );
    }

    #[test]
    fn test_ack_success_to_vec() {
        let ack_success: Vec<u8> = AcknowledgementStatus::success(ack_success_b64()).into();

        // Check that it's the same output as ibc-go
        // Note: this also implicitly checks that the ack bytes are non-empty,
        // which would make the conversion to `Acknowledgement` panic
        assert_eq!(ack_success, br#"{"result":"AQ=="}"#);
    }

    #[test]
    fn test_ack_error_to_vec() {
        let ack_error: Vec<u8> =
            AcknowledgementStatus::error(TokenTransferError::PacketDataDeserialization.into())
                .into();

        // Check that it's the same output as ibc-go
        // Note: this also implicitly checks that the ack bytes are non-empty,
        // which would make the conversion to `Acknowledgement` panic
        assert_eq!(
            ack_error,
            br#"{"error":"failed to deserialize packet data"}"#
        );
    }

    #[test]
    fn test_ack_de() {
        fn de_json_assert_eq(json_str: &str, ack: AcknowledgementStatus) {
            let de = serde_json::from_str::<AcknowledgementStatus>(json_str).unwrap();
            assert_eq!(de, ack)
        }

        de_json_assert_eq(
            r#"{"result":"AQ=="}"#,
            AcknowledgementStatus::success(ack_success_b64()),
        );
        de_json_assert_eq(
            r#"{"error":"failed to deserialize packet data"}"#,
            AcknowledgementStatus::error(TokenTransferError::PacketDataDeserialization.into()),
        );

        assert!(serde_json::from_str::<AcknowledgementStatus>(r#"{"success":"AQ=="}"#).is_err());
    }
}
