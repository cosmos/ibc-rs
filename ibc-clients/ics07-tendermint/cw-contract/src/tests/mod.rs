pub mod fixture;
pub mod helper;

use cosmwasm_std::from_json;
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use ibc_client_cw::types::{
    ContractResult, MigrateClientStoreMsg, UpdateStateMsgRaw, UpdateStateOnMisbehaviourMsgRaw,
};
use ibc_core::client::types::Status;

use crate::entrypoint::{instantiate, sudo};
use crate::tests::fixture::Fixture;
use crate::tests::helper::dummy_msg_info;

#[test]
fn happy_cw_create_client() {
    let fxt = Fixture::default();

    let mut deps = mock_dependencies();

    let instantiate_msg = fxt.dummy_instantiate_msg();

    let resp = instantiate(deps.as_mut(), mock_env(), dummy_msg_info(), instantiate_msg).unwrap();

    assert_eq!(0, resp.messages.len());

    let contract_result: ContractResult = from_json(resp.data.unwrap()).unwrap();

    assert!(contract_result.heights.is_none());

    fxt.check_client_status(deps.as_ref(), Status::Active);
}

#[test]
fn happy_cw_update_client() {
    let fxt = Fixture::default();

    let mut deps = mock_dependencies();

    // ------------------- Create client -------------------

    let instantiate_msg = fxt.dummy_instantiate_msg();

    instantiate(deps.as_mut(), mock_env(), dummy_msg_info(), instantiate_msg).unwrap();

    // ------------------- Verify and Update client -------------------

    let client_message = fxt.dummy_client_message();

    fxt.verify_client_message(deps.as_ref(), client_message.clone());

    let resp = sudo(
        deps.as_mut(),
        mock_env(),
        UpdateStateMsgRaw { client_message }.into(),
    )
    .unwrap();

    // ------------------- Check response -------------------

    assert_eq!(0, resp.messages.len());

    let contract_result: ContractResult = from_json(resp.data.unwrap()).unwrap();

    assert_eq!(contract_result.heights, Some(vec![fxt.target_height]));

    fxt.check_client_status(deps.as_ref(), Status::Active);
}

#[test]
fn happy_cw_recovery_client() {
    let fxt = Fixture::default().migration_mode();

    let mut deps = mock_dependencies();

    let mut ctx = fxt.ctx_mut(deps.as_mut());

    // ------------------- Create subject client -------------------

    let instantiate_msg = fxt.dummy_instantiate_msg();

    ctx.instantiate(instantiate_msg.clone()).unwrap();

    // ------------------- Freeze subject client -------------------

    let client_message = fxt.dummy_misbehaviour_message();

    fxt.check_for_misbehaviour(deps.as_ref(), client_message.clone());

    let mut ctx = fxt.ctx_mut(deps.as_mut());

    ctx.sudo(UpdateStateOnMisbehaviourMsgRaw { client_message }.into())
        .unwrap();

    // ------------------- Create substitute client -------------------

    ctx.set_substitute_prefix();

    ctx.instantiate(instantiate_msg).unwrap();

    // ------------------- Recover subject client -------------------

    let resp = sudo(deps.as_mut(), mock_env(), MigrateClientStoreMsg {}.into()).unwrap();

    assert_eq!(0, resp.messages.len());

    fxt.check_client_status(deps.as_ref(), Status::Active);
}
