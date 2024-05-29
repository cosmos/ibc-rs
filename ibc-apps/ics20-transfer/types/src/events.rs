//! Defines all token transfer event types
use ibc_core::channel::types::acknowledgement::AcknowledgementStatus;
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;
use ibc_core::router::types::event::ModuleEvent;

use super::Memo;
use crate::{Amount, PrefixedDenom, MODULE_ID_STR};

const EVENT_TYPE_PACKET: &str = "fungible_token_packet";
const EVENT_TYPE_TIMEOUT: &str = "timeout";
const EVENT_TYPE_DENOM_TRACE: &str = "denomination_trace";
const EVENT_TYPE_TRANSFER: &str = "ibc_transfer";

/// Contains all events variants that can be emitted from the token transfer application
pub enum Event {
    Recv(RecvEvent),
    Ack(AckEvent),
    AckStatus(AckStatusEvent),
    Timeout(TimeoutEvent),
    DenomTrace(DenomTraceEvent),
    Transfer(TransferEvent),
}

/// Event emitted by the `onRecvPacket` module callback to indicate the that the
/// `RecvPacket` message was processed
pub struct RecvEvent {
    pub sender: Signer,
    pub receiver: Signer,
    pub denom: PrefixedDenom,
    pub amount: Amount,
    pub memo: Memo,
    pub success: bool,
}

impl From<RecvEvent> for ModuleEvent {
    fn from(ev: RecvEvent) -> Self {
        let RecvEvent {
            sender,
            receiver,
            denom,
            amount,
            memo,
            success,
        } = ev;
        Self {
            kind: EVENT_TYPE_PACKET.to_string(),
            attributes: vec![
                ("module", MODULE_ID_STR).into(),
                ("sender", sender).into(),
                ("receiver", receiver).into(),
                ("denom", denom).into(),
                ("amount", amount).into(),
                ("memo", memo).into(),
                ("success", success).into(),
            ],
        }
    }
}

/// Event emitted in the `onAcknowledgePacket` module callback
pub struct AckEvent {
    pub sender: Signer,
    pub receiver: Signer,
    pub denom: PrefixedDenom,
    pub amount: Amount,
    pub memo: Memo,
    pub acknowledgement: AcknowledgementStatus,
}

impl From<AckEvent> for ModuleEvent {
    fn from(ev: AckEvent) -> Self {
        let AckEvent {
            sender,
            receiver,
            denom,
            amount,
            memo,
            acknowledgement,
        } = ev;
        Self {
            kind: EVENT_TYPE_PACKET.to_string(),
            attributes: vec![
                ("module", MODULE_ID_STR).into(),
                ("sender", sender).into(),
                ("receiver", receiver).into(),
                ("denom", denom).into(),
                ("amount", amount).into(),
                ("memo", memo).into(),
                ("acknowledgement", acknowledgement).into(),
            ],
        }
    }
}

/// Event emitted in the `onAcknowledgePacket` module callback to indicate
/// whether the acknowledgement is a success or a failure
pub struct AckStatusEvent {
    pub acknowledgement: AcknowledgementStatus,
}

impl From<AckStatusEvent> for ModuleEvent {
    fn from(ev: AckStatusEvent) -> Self {
        let AckStatusEvent { acknowledgement } = ev;
        let attr_label = match acknowledgement {
            AcknowledgementStatus::Success(_) => "success",
            AcknowledgementStatus::Error(_) => "error",
        };

        Self {
            kind: EVENT_TYPE_PACKET.to_string(),
            attributes: vec![(attr_label, acknowledgement.to_string()).into()],
        }
    }
}

/// Event emitted in the `onTimeoutPacket` module callback
pub struct TimeoutEvent {
    pub refund_receiver: Signer,
    pub refund_denom: PrefixedDenom,
    pub refund_amount: Amount,
    pub memo: Memo,
}

impl From<TimeoutEvent> for ModuleEvent {
    fn from(ev: TimeoutEvent) -> Self {
        let TimeoutEvent {
            refund_receiver,
            refund_denom,
            refund_amount,
            memo,
        } = ev;
        Self {
            kind: EVENT_TYPE_TIMEOUT.to_string(),
            attributes: vec![
                ("module", MODULE_ID_STR).into(),
                ("refund_receiver", refund_receiver).into(),
                ("refund_denom", refund_denom).into(),
                ("refund_amount", refund_amount).into(),
                ("memo", memo).into(),
            ],
        }
    }
}

/// Event emitted in the `onRecvPacket` module callback when new tokens are minted
pub struct DenomTraceEvent {
    pub trace_hash: Option<String>,
    pub denom: PrefixedDenom,
}

impl From<DenomTraceEvent> for ModuleEvent {
    fn from(ev: DenomTraceEvent) -> Self {
        let DenomTraceEvent { trace_hash, denom } = ev;
        let mut ev = Self {
            kind: EVENT_TYPE_DENOM_TRACE.to_string(),
            attributes: vec![("denom", denom).into()],
        };
        if let Some(hash) = trace_hash {
            ev.attributes.push(("trace_hash", hash).into());
        }
        ev
    }
}

/// Event emitted after a successful `sendTransfer`
pub struct TransferEvent {
    pub sender: Signer,
    pub receiver: Signer,
    pub amount: Amount,
    pub denom: PrefixedDenom,
    pub memo: Memo,
}

impl From<TransferEvent> for ModuleEvent {
    fn from(ev: TransferEvent) -> Self {
        let TransferEvent {
            sender,
            receiver,
            amount,
            denom,
            memo,
        } = ev;

        Self {
            kind: EVENT_TYPE_TRANSFER.to_string(),
            attributes: vec![
                ("sender", sender).into(),
                ("receiver", receiver).into(),
                ("amount", amount).into(),
                ("denom", denom).into(),
                ("memo", memo).into(),
            ],
        }
    }
}

impl From<Event> for ModuleEvent {
    fn from(ev: Event) -> Self {
        match ev {
            Event::Recv(ev) => ev.into(),
            Event::Ack(ev) => ev.into(),
            Event::AckStatus(ev) => ev.into(),
            Event::Timeout(ev) => ev.into(),
            Event::DenomTrace(ev) => ev.into(),
            Event::Transfer(ev) => ev.into(),
        }
    }
}
