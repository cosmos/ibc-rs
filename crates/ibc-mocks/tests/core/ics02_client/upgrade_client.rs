#[cfg(test)]
mod tests {
    use ibc::clients::ics07_tendermint::client_state::ClientState as TmClientState;
    use ibc::clients::ics07_tendermint::client_type;
    use ibc::clients::ics07_tendermint::header::test_util::get_dummy_tendermint_header;
    use ibc::core::events::{IbcEvent, MessageEvent};
    use ibc::core::ics02_client::error::{ClientError, UpgradeClientError};
    use ibc::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
    use ibc::core::ics02_client::msgs::ClientMsg;
    use ibc::core::ics24_host::identifier::ClientId;
    use ibc::core::ics24_host::path::ClientConsensusStatePath;
    use ibc::core::{execute, validate, ContextError, MsgEnvelope, ValidationContext};
    use ibc::mock::client_state::client_type as mock_client_type;
    use ibc::prelude::*;
    use ibc::{downcast, Height};
    use ibc_mocks::core::definition::{AnyClientState, AnyConsensusState, MockContext};
    use ibc_mocks::router::definition::MockRouter;
    use ibc_mocks::utils::fixture::{Expect, Fixture};

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
        let client_id = ClientId::new(mock_client_type(), 0).unwrap();

        let ctx_default = MockContext::default();
        let ctx_with_client = ctx_default
            .clone()
            .with_client(&client_id, Height::new(0, 42).unwrap());
        let ctx = match ctx_variant {
            Ctx::Default => ctx_default,
            Ctx::WithClient => ctx_with_client,
        };

        let upgrade_height = Height::new(1, 26).unwrap();
        let msg_default = MsgUpgradeClient::new_dummy(upgrade_height);

        let low_upgrade_height = Height::new(0, 26).unwrap();
        let msg_with_low_upgrade_height = MsgUpgradeClient::new_dummy(low_upgrade_height);

        let msg_with_unknown_upgraded_cs = MsgUpgradeClient {
            upgraded_client_state: TmClientState::new_dummy_from_header(
                get_dummy_tendermint_header(),
            )
            .into(),
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
                assert!(matches!(
                    fxt.ctx.events[0],
                    IbcEvent::Message(MessageEvent::Client)
                ));
                let upgrade_client_event =
                    downcast!(&fxt.ctx.events[1] => IbcEvent::UpgradeClient).unwrap();
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
                        &fxt.msg.client_id,
                        &plan_height,
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
        upgrade_client_validate(&fxt, Expect::Failure(Some(expected_err.into())));
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
            Expect::Failure(Some(ContextError::from(expected_err).into())),
        );
    }

    #[test]
    fn upgrade_client_fail_unknown_upgraded_client_state() {
        let fxt = msg_upgrade_client_fixture(Ctx::WithClient, Msg::UnknownUpgradedClientStateType);
        let expected_err = ContextError::ClientError(ClientError::UnknownClientStateType {
            client_state_type: client_type().to_string(),
        });
        upgrade_client_validate(&fxt, Expect::Failure(Some(expected_err.into())));
    }
}
