use ibc_eureka_core_channel_types::commitment::compute_packet_commitment;
use ibc_eureka_core_channel_types::error::ChannelError;
use ibc_eureka_core_channel_types::msgs::MsgTimeoutOnClose;
use ibc_eureka_core_client::context::prelude::*;
use ibc_eureka_core_host::types::path::{
    ClientConsensusStatePath, CommitmentPath, Path, ReceiptPath,
};
use ibc_eureka_core_host::ValidationContext;
use ibc_primitives::prelude::*;

pub fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgTimeoutOnClose) -> Result<(), ChannelError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    let packet = &msg.packet;
    let payload = &packet.payloads[0];

    let (_, source_port) = &payload.header.source_port;
    let channel_target_client_on_source = &packet.header.target_client_on_source;
    let (target_prefix, target_port) = &payload.header.target_port;
    let channel_source_client_on_target = &packet.header.source_client_on_target;
    let seq_on_a = &packet.header.seq_on_a;
    let data = &payload.data;

    let commitment_path_on_a =
        CommitmentPath::new(source_port, channel_target_client_on_source, *seq_on_a);

    //verify the packet was sent, check the store
    let Ok(commitment_on_a) = ctx_a.get_packet_commitment(&commitment_path_on_a) else {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    let expected_commitment_on_a = compute_packet_commitment(
        data,
        &packet.header.timeout_height_on_b,
        &packet.header.timeout_timestamp_on_b,
    );
    if commitment_on_a != expected_commitment_on_a {
        return Err(ChannelError::MismatchedPacketCommitment {
            expected: expected_commitment_on_a,
            actual: commitment_on_a,
        });
    }

    // Verify proofs
    {
        let id_target_client_on_source = channel_target_client_on_source.as_ref();
        let client_val_ctx_a = ctx_a.get_client_validation_context();
        let target_client_on_source = client_val_ctx_a.client_state(id_target_client_on_source)?;

        target_client_on_source
            .status(
                ctx_a.get_client_validation_context(),
                id_target_client_on_source,
            )?
            .verify_is_active()?;

        target_client_on_source.validate_proof_height(msg.proof_height_on_b)?;

        let client_cons_state_path_on_a = ClientConsensusStatePath::new(
            id_target_client_on_source.clone(),
            msg.proof_height_on_b.revision_number(),
            msg.proof_height_on_b.revision_height(),
        );
        let consensus_state_of_b_on_a =
            client_val_ctx_a.consensus_state(&client_cons_state_path_on_a)?;

        let next_seq_recv_verification_result = {
            let receipt_path_on_b =
                ReceiptPath::new(target_port, channel_source_client_on_target, *seq_on_a);

            target_client_on_source.verify_non_membership(
                target_prefix,
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                Path::Receipt(receipt_path_on_b),
            )
        };

        next_seq_recv_verification_result?;
    };

    Ok(())
}
