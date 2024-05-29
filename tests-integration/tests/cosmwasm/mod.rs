pub mod fixture;
pub mod helper;

use std::time::Duration;

use cosmwasm_std::from_json;
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use fixture::Fixture;
use ibc::core::client::types::{Height, Status};
use ibc_client_cw::types::{
    ContractResult, MigrateClientStoreMsg, MigrationPrefix, VerifyClientMessageRaw,
};
use ibc_client_tendermint_cw::entrypoint::sudo;

#[test]
fn test_cw_create_client_ok() {
    let fxt = Fixture::default();

    let mut deps = mock_dependencies();

    let resp = fxt.create_client(deps.as_mut()).unwrap();

    assert_eq!(0, resp.messages.len());

    let contract_result: ContractResult = from_json(resp.data.unwrap()).unwrap();

    assert!(contract_result.heights.is_none());

    fxt.check_client_status(deps.as_ref(), Status::Active);
}

#[test]
fn test_cw_update_client_ok() {
    let fxt = Fixture::default();

    let mut deps = mock_dependencies();

    // ------------------- Create client -------------------

    fxt.create_client(deps.as_mut()).unwrap();

    // ------------------- Verify and Update client -------------------

    let target_height = Height::new(0, 10).unwrap();

    let resp = fxt.update_client(deps.as_mut(), target_height).unwrap();

    // ------------------- Check response -------------------

    assert_eq!(0, resp.messages.len());

    let contract_result: ContractResult = from_json(resp.data.unwrap()).unwrap();

    assert_eq!(contract_result.heights, Some(vec![target_height]));

    fxt.check_client_status(deps.as_ref(), Status::Active);
}

#[test]
fn test_cw_recovery_client_ok() {
    let mut fxt = Fixture::default();

    let mut deps = mock_dependencies();

    // ------------------- Create subject client -------------------

    fxt.set_migration_prefix(MigrationPrefix::Subject);

    fxt.create_client(deps.as_mut()).unwrap();

    // ------------------- Freeze subject client -------------------

    fxt.update_client_on_misbehaviour(deps.as_mut());

    // ------------------- Create substitute client -------------------

    fxt.set_migration_prefix(MigrationPrefix::Substitute);

    fxt.create_client(deps.as_mut()).unwrap();

    // ------------------- Recover subject client -------------------

    let resp = sudo(deps.as_mut(), mock_env(), MigrateClientStoreMsg {}.into()).unwrap();

    assert_eq!(0, resp.messages.len());

    fxt.check_client_status(deps.as_ref(), Status::Active);
}

#[test]
fn test_cw_client_expiry() {
    let fxt = Fixture::default();

    let mut deps = mock_dependencies();

    // ------------------- Create client -------------------

    fxt.create_client(deps.as_mut()).unwrap();

    // ------------------- Expire client -------------------

    std::thread::sleep(Duration::from_millis(1200));

    // ------------------- Try update client -------------------

    let target_height = Height::new(0, 10).unwrap();

    let client_message = fxt.dummy_client_message(target_height);

    let resp = fxt.query(
        deps.as_ref(),
        VerifyClientMessageRaw {
            client_message: client_message.clone(),
        }
        .into(),
    );

    assert!(resp.is_err());

    // ------------------- Check client status -------------------

    fxt.check_client_status(deps.as_ref(), Status::Expired);
}
