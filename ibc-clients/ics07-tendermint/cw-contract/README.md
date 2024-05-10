# `ibc-client-tendermint-cw` crate

This crate showcases how to reuse `ibc-rs` light clients as a
[CosmWasm contract](https://github.com/cosmos/ibc/blob/main/spec/client/ics-008-wasm-client/README.md)
utilizing the `ibc-client-cw` crate.

The `ibc-client-cw` crate exposes the requisite types and traits needed to reuse
the `ibc-rs` light clients. Notably, it offers a
[`ClientType`](https://docs.rs/ibc-client-cw/latest/ibc_client_cw/api/trait.ClientType.html)
trait, which requires two associated types: `ClientState` and `ConsensusState`.
These types take any type that implements the
[`ClientStateExecution`](https://docs.rs/ibc-core/latest/ibc_core/client/context/client_state/trait.ClientStateExecution.html)
and
[`ConsensusState`](https://docs.rs/ibc-core/latest/ibc_core/client/context/consensus_state/trait.ConsensusState.html)
traits from the `ibc-core` crate.

For example, to reuse the existing
[`ibc-client-tendermint`](https://docs.rs/ibc-client-tendermint/latest/ibc_client_tendermint/):

```rs
use ibc_client_cw::api::ClientType;
use ibc_client_tendermint::client_state::ClientState;
use ibc_client_tendermint::consensus_state::ConsensusState;

#[derive(Clone, Debug)]
pub struct TendermintClient;

impl<'a> ClientType<'a> for TendermintClient {
    type ClientState = ClientState;
    type ConsensusState = ConsensusState;
}
```

Once the `ClientType` trait is implemented, the `ibc-client-cw` crate can be
used to complete the entry points for the CosmWasm contract:

```rs
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use ibc_client_cw::context::Context;
use ibc_client_cw::types::{ContractError, InstantiateMsg, QueryMsg, SudoMsg};

pub type TendermintContext<'a> = Context<'a, TendermintClient>;

#[entry_point]
pub fn instantiate(
    deps: DepsMut<'_>,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut ctx = TendermintContext::new_mut(deps, env)?;
    let data = ctx.instantiate(msg)?;
    Ok(Response::default().set_data(data))
}

#[entry_point]
pub fn sudo(deps: DepsMut<'_>, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let mut ctx = TendermintContext::new_mut(deps, env)?;
    let data = ctx.sudo(msg)?;
    Ok(Response::default().set_data(data))
}

#[entry_point]
pub fn query(deps: Deps<'_>, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let ctx = TendermintContext::new_ref(deps, env)?;
    ctx.query(msg)
}
```

The above snippets compile into a fully working CosmWasm contract implementing
Tendermint IBC light client.
