# CosmWasm Contract

This crate showcases a barebones CosmWasm contract implementation utilizing the `ibc-client-cw`
crate, which exposes the requisite interfaces needed to implement contracts on top of `ibc-rs`.

The following template can be used to get you started:

```rs
#[derive(Clone, Debug)]
pub struct TendermintClient;

impl<'a> ClientType<'a> for TendermintClient {
    type ClientState = ClientState;
    type ConsensusState = ConsensusState;
}

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
