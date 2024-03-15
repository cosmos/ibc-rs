extern crate alloc;
use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use ibc_apps::transfer::types::msgs::transfer::MsgTransfer;
use ibc_apps::transfer::types::packet::PacketData;
use ibc_apps::transfer::types::{Amount, BaseDenom, PrefixedCoin, PrefixedDenom, TracePath};
use ibc_core::host::types::identifiers::*;
use thiserror::Error;

#[cw_serde]
pub struct Msg {
    pub test: String,
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: Msg,
) -> Result<Response, ContractError> {
    let _serde: (
        PrefixedCoin,
        Amount,
        ChannelId,
        ChainId,
        ConnectionId,
        MsgTransfer,
        PacketData,
    ) = serde_json::from_str(&msg.test).expect("test");
    let a = serde_json::to_string(&ChannelId::from_str(msg.test.as_str()).unwrap()).expect("test");
    let b = serde_json::to_string(&PortId::from_str(msg.test.as_str()).unwrap()).expect("test");
    let c = serde_json::to_string(&ClientId::from_str(msg.test.as_str()).unwrap()).expect("test");
    let d =
        serde_json::to_string(&ConnectionId::from_str(msg.test.as_str()).unwrap()).expect("test");
    let e = serde_json::to_string(&PrefixedDenom {
        trace_path: TracePath::empty(),
        base_denom: BaseDenom::from_str(msg.test.as_str()).unwrap(),
    })
    .expect("test");
    let f = serde_json::to_string(&ConnectionId::from_str(msg.test.as_str()).expect("test"))
        .expect("test");
    Ok(Response::new()
        .add_attribute(a, b)
        .add_attribute(c, d)
        .add_attribute(e, f)
        .add_attribute(msg.test.clone(), msg.test.clone()))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Msg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cw_serde]
#[derive(Error)]
pub enum ContractError {}
