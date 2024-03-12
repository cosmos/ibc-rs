use ibc::clients::tendermint::types::client_type;
use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::error::{ClientError, UpgradeClientError};
use ibc::core::client::types::msgs::{ClientMsg, MsgUpgradeClient};
use ibc::core::client::types::Height;
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::error::ContextError;
use ibc::core::handler::types::events::{IbcEvent, MessageEvent};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::path::ClientConsensusStatePath;
use ibc_testkit::fixtures::clients::tendermint::{
    dummy_tendermint_header, dummy_tm_client_state_from_header,
};
use ibc_testkit::fixtures::core::client::dummy_msg_upgrade_client;
use ibc_testkit::fixtures::{Expect, Fixture};
use ibc_testkit::testapp::ibc::clients::mock::client_state::client_type as mock_client_type;
use ibc_testkit::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{MockClientConfig, MockContext};

enum Ctx {
    Default,
    WithClient,
}

enum Msg {
    Default,
    LowUpgradeHeight,
    UnknownUpgradedClientStateType,
}

fn msg_upgrade_client_fixture(ctx_variant: Ctx, msg_variant: Msg) -> Fixture<MsgUpgradeClient> {
    let client_id = mock_client_type().build_client_id(0);

    let ctx_default = MockContext::default();
    let ctx_with_client = ctx_default.clone().with_client_config(
        MockClientConfig::builder()
            .client_id(client_id.clone())
            .latest_height(Height::new(0, 42).unwrap())
            .build(),
    );
    let ctx = match ctx_variant {
        Ctx::Default => ctx_default,
        Ctx::WithClient => ctx_with_client,
    };

    let upgrade_height = Height::new(1, 26).unwrap();
    let msg_default = dummy_msg_upgrade_client(client_id.clone(), upgrade_height);

    let low_upgrade_height = Height::new(0, 26).unwrap();
    let msg_with_low_upgrade_height = dummy_msg_upgrade_client(client_id, low_upgrade_height);

    let msg_with_unknown_upgraded_cs = MsgUpgradeClient {
        upgraded_client_state: dummy_tm_client_state_from_header(dummy_tendermint_header()).into(),
        ..msg_default.clone()
    };

    let msg = match msg_variant {
        Msg::Default => msg_default,
        Msg::LowUpgradeHeight => msg_with_low_upgrade_height,
        Msg::UnknownUpgradedClientStateType => msg_with_unknown_upgraded_cs,
    };

    Fixture { ctx, msg }
}

fn upgrade_client_validate(fxt: &Fixture<MsgUpgradeClient>, expect: Expect) {
    let Fixture { ctx, msg } = fxt;
    let router = MockRouter::new_with_transfer();
    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));
    let res = validate(ctx, &router, msg_envelope);
    let err_msg = fxt.generate_error_msg(&expect, "validation", &res);

    match expect {
        Expect::Failure(_) => {
            assert!(res.is_err(), "{err_msg}");
        }
        Expect::Success => {
            assert!(res.is_ok(), "{err_msg}");
        }
    };
}

fn upgrade_client_execute(fxt: &mut Fixture<MsgUpgradeClient>, expect: Expect) {
    let mut router = MockRouter::new_with_transfer();
    let msg_envelope = MsgEnvelope::from(ClientMsg::from(fxt.msg.clone()));
    let res = execute(&mut fxt.ctx, &mut router, msg_envelope);
    let err_msg = fxt.generate_error_msg(&expect, "execution", &res);
    match expect {
        Expect::Failure(err) => {
            assert!(res.is_err(), "{err_msg}");
            assert_eq!(
                core::mem::discriminant(res.as_ref().unwrap_err()),
                core::mem::discriminant(&err.unwrap())
            );
        }
        Expect::Success => {
            assert!(res.is_ok(), "{err_msg}");
            let ibc_events = fxt.ctx.get_events();
            assert!(matches!(
                ibc_events[0],
                IbcEvent::Message(MessageEvent::Client)
            ));
            let IbcEvent::UpgradeClient(upgrade_client_event) = &ibc_events[1] else {
                panic!("unexpected event variant");
            };
            let plan_height = Height::new(1, 26).unwrap();

            assert_eq!(upgrade_client_event.client_id(), &fxt.msg.client_id);
            assert_eq!(upgrade_client_event.client_type(), &mock_client_type());
            assert_eq!(upgrade_client_event.consensus_height(), &plan_height);

            let client_state = fxt.ctx.client_state(&fxt.msg.client_id).unwrap();
            let msg_client_state: AnyClientState =
                fxt.msg.upgraded_client_state.clone().try_into().unwrap();
            assert_eq!(client_state, msg_client_state);

            let consensus_state = fxt
                .ctx
                .consensus_state(&ClientConsensusStatePath::new(
                    fxt.msg.client_id.clone(),
                    plan_height.revision_number(),
                    plan_height.revision_height(),
                ))
                .unwrap();
            let msg_consensus_state: AnyConsensusState =
                fxt.msg.upgraded_consensus_state.clone().try_into().unwrap();
            assert_eq!(consensus_state, msg_consensus_state);
        }
    };
}

#[test]
fn msg_upgrade_client_healthy() {
    let mut fxt = msg_upgrade_client_fixture(Ctx::WithClient, Msg::Default);
    upgrade_client_validate(&fxt, Expect::Success);
    upgrade_client_execute(&mut fxt, Expect::Success);
}

#[test]
fn upgrade_client_fail_nonexisting_client() {
    let fxt = msg_upgrade_client_fixture(Ctx::Default, Msg::Default);
    let expected_err = ContextError::ClientError(ClientError::ClientStateNotFound {
        client_id: fxt.msg.client_id.clone(),
    });
    upgrade_client_validate(&fxt, Expect::Failure(Some(expected_err)));
}

#[test]
fn upgrade_client_fail_low_upgrade_height() {
    let fxt: Fixture<MsgUpgradeClient> =
        msg_upgrade_client_fixture(Ctx::WithClient, Msg::LowUpgradeHeight);
    let expected_err: ClientError = UpgradeClientError::LowUpgradeHeight {
        upgraded_height: Height::new(0, 26).unwrap(),
        client_height: fxt.ctx.latest_height(),
    }
    .into();
    upgrade_client_validate(
        &fxt,
        Expect::Failure(Some(ContextError::from(expected_err))),
    );
}

#[test]
fn upgrade_client_fail_unknown_upgraded_client_state() {
    let fxt = msg_upgrade_client_fixture(Ctx::WithClient, Msg::UnknownUpgradedClientStateType);
    let expected_err = ContextError::ClientError(ClientError::UnknownClientStateType {
        client_state_type: client_type().to_string(),
    });
    upgrade_client_validate(&fxt, Expect::Failure(Some(expected_err)));
}
