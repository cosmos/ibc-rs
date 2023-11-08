//! Protocol logic specific to processing ICS3 messages of type `MsgConnectionOpenConfirm`.

use ibc_proto::protobuf::Protobuf;

use crate::core::context::ContextError;
use crate::core::events::{IbcEvent, MessageEvent};
use crate::core::ics02_client::client_state::{ClientStateCommon, ClientStateValidation};
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics03_connection::events::OpenConfirm;
use crate::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
use crate::core::ics24_host::path::{ClientConsensusStatePath, ConnectionPath, Path};
use crate::core::{ExecutionContext, ValidationContext};
use crate::prelude::*;

pub(crate) fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgConnectionOpenConfirm) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let vars = LocalVars::new(ctx_b, msg)?;
    validate_impl(ctx_b, msg, &vars)
}

fn validate_impl<Ctx>(
    ctx_b: &Ctx,
    msg: &MsgConnectionOpenConfirm,
    vars: &LocalVars,
) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_b.validate_message_signer(&msg.signer)?;

    let conn_end_on_b = vars.conn_end_on_b();

    conn_end_on_b.verify_state_matches(&State::TryOpen)?;

    let client_id_on_a = vars.client_id_on_a();
    let client_id_on_b = vars.client_id_on_b();
    let conn_id_on_a = vars.conn_id_on_a()?;

    // Verify proofs
    {
        let client_state_of_a_on_b = ctx_b.client_state(client_id_on_b)?;

        {
            let status = client_state_of_a_on_b
                .status(ctx_b.get_client_validation_context(), client_id_on_b)?;
            if !status.is_active() {
                return Err(ClientError::ClientNotActive { status }.into());
            }
        }
        client_state_of_a_on_b.validate_proof_height(msg.proof_height_on_a)?;

        let client_cons_state_path_on_b =
            ClientConsensusStatePath::new(client_id_on_b, &msg.proof_height_on_a);
        let consensus_state_of_a_on_b = ctx_b.consensus_state(&client_cons_state_path_on_b)?;

        let prefix_on_a = conn_end_on_b.counterparty().prefix();
        let prefix_on_b = ctx_b.commitment_prefix();

        let expected_conn_end_on_a = ConnectionEnd::new(
            State::Open,
            client_id_on_a.clone(),
            Counterparty::new(
                client_id_on_b.clone(),
                Some(msg.conn_id_on_b.clone()),
                prefix_on_b,
            ),
            conn_end_on_b.versions().to_vec(),
            conn_end_on_b.delay_period(),
        )?;

        client_state_of_a_on_b
            .verify_membership(
                prefix_on_a,
                &msg.proof_conn_end_on_a,
                consensus_state_of_a_on_b.root(),
                Path::Connection(ConnectionPath::new(conn_id_on_a)),
                expected_conn_end_on_a.encode_vec(),
            )
            .map_err(ConnectionError::VerifyConnectionState)?;
    }

    Ok(())
}

pub(crate) fn execute<Ctx>(
    ctx_b: &mut Ctx,
    msg: &MsgConnectionOpenConfirm,
) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let vars = LocalVars::new(ctx_b, msg)?;
    execute_impl(ctx_b, msg, vars)
}

fn execute_impl<Ctx>(
    ctx_b: &mut Ctx,
    msg: &MsgConnectionOpenConfirm,
    vars: LocalVars,
) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let client_id_on_a = vars.client_id_on_a();
    let client_id_on_b = vars.client_id_on_b();
    let conn_id_on_a = vars.conn_id_on_a()?;

    let event = IbcEvent::OpenConfirmConnection(OpenConfirm::new(
        msg.conn_id_on_b.clone(),
        client_id_on_b.clone(),
        conn_id_on_a.clone(),
        client_id_on_a.clone(),
    ));
    ctx_b.emit_ibc_event(IbcEvent::Message(MessageEvent::Connection))?;
    ctx_b.emit_ibc_event(event)?;
    ctx_b.log_message("success: conn_open_confirm verification passed".to_string())?;

    {
        let new_conn_end_on_b = {
            let mut new_conn_end_on_b = vars.conn_end_on_b;

            new_conn_end_on_b.set_state(State::Open);
            new_conn_end_on_b
        };

        ctx_b.store_connection(&ConnectionPath(msg.conn_id_on_b.clone()), new_conn_end_on_b)?;
    }

    Ok(())
}

struct LocalVars {
    conn_end_on_b: ConnectionEnd,
}

impl LocalVars {
    fn new<Ctx>(ctx_b: &Ctx, msg: &MsgConnectionOpenConfirm) -> Result<Self, ContextError>
    where
        Ctx: ValidationContext,
    {
        Ok(Self {
            conn_end_on_b: ctx_b.connection_end(&msg.conn_id_on_b)?,
        })
    }

    fn conn_end_on_b(&self) -> &ConnectionEnd {
        &self.conn_end_on_b
    }

    fn client_id_on_a(&self) -> &ClientId {
        self.conn_end_on_b.counterparty().client_id()
    }

    fn client_id_on_b(&self) -> &ClientId {
        self.conn_end_on_b.client_id()
    }

    fn conn_id_on_a(&self) -> Result<&ConnectionId, ConnectionError> {
        self.conn_end_on_b
            .counterparty()
            .connection_id()
            .ok_or(ConnectionError::InvalidCounterparty)
    }
}
