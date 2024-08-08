//! Implementation of a global context mock. Used in testing handlers of all IBC modules.

use alloc::sync::Arc;
use core::fmt::Debug;

use basecoin_store::context::{ProvableStore, Store};
use basecoin_store::impls::SharedStore;
use basecoin_store::types::{BinStore, JsonStore, ProtobufStore, TypedSet, TypedStore};
use ibc::core::channel::types::channel::ChannelEnd;
use ibc::core::channel::types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::client::types::Height;
use ibc::core::connection::types::ConnectionEnd;
use ibc::core::handler::types::events::IbcEvent;
use ibc::core::host::types::identifiers::{ConnectionId, Sequence};
use ibc::core::host::types::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, ClientStatePath,
    ClientUpdateHeightPath, ClientUpdateTimePath, CommitmentPath, ConnectionPath,
    NextChannelSequencePath, NextClientSequencePath, NextConnectionSequencePath, ReceiptPath,
    SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::channel::v1::Channel as RawChannelEnd;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;
use ibc_proto::ibc::core::connection::v1::ConnectionEnd as RawConnectionEnd;
use ibc_proto::ics23::CommitmentProof;
use parking_lot::Mutex;
use typed_builder::TypedBuilder;

use crate::context::{MockStore, TestContext};
use crate::fixtures::core::context::TestContextConfig;
use crate::hosts::{HostClientState, TestBlock, TestHeader, TestHost};
use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
pub const DEFAULT_BLOCK_TIME_SECS: u64 = 3;

pub type DefaultIbcStore = MockIbcStore<MockStore>;

/// An object that stores all IBC related data.
#[derive(Debug)]
pub struct MockIbcStore<S>
where
    S: ProvableStore + Debug,
{
    /// chain revision number,
    pub revision_number: Arc<Mutex<u64>>,

    /// Handle to store instance.
    /// The module is guaranteed exclusive access to all paths in the store key-space.
    pub store: SharedStore<S>,
    /// A typed-store for next client counter sequence
    pub client_counter: JsonStore<SharedStore<S>, NextClientSequencePath, u64>,
    /// A typed-store for next connection counter sequence
    pub conn_counter: JsonStore<SharedStore<S>, NextConnectionSequencePath, u64>,
    /// A typed-store for next channel counter sequence
    pub channel_counter: JsonStore<SharedStore<S>, NextChannelSequencePath, u64>,
    /// Tracks the processed time for client updates
    pub client_processed_times: JsonStore<SharedStore<S>, ClientUpdateTimePath, Timestamp>,
    /// A typed-store to track the processed height for client updates
    pub client_processed_heights:
        ProtobufStore<SharedStore<S>, ClientUpdateHeightPath, Height, RawHeight>,
    /// A typed-store for AnyClientState
    pub client_state_store: ProtobufStore<SharedStore<S>, ClientStatePath, AnyClientState, Any>,
    /// A typed-store for AnyConsensusState
    pub consensus_state_store:
        ProtobufStore<SharedStore<S>, ClientConsensusStatePath, AnyConsensusState, Any>,
    /// A typed-store for ConnectionEnd
    pub connection_end_store:
        ProtobufStore<SharedStore<S>, ConnectionPath, ConnectionEnd, RawConnectionEnd>,
    /// A typed-store for ConnectionIds
    pub connection_ids_store: JsonStore<SharedStore<S>, ClientConnectionPath, Vec<ConnectionId>>,
    /// A typed-store for ChannelEnd
    pub channel_end_store: ProtobufStore<SharedStore<S>, ChannelEndPath, ChannelEnd, RawChannelEnd>,
    /// A typed-store for send sequences
    pub send_sequence_store: JsonStore<SharedStore<S>, SeqSendPath, Sequence>,
    /// A typed-store for receive sequences
    pub recv_sequence_store: JsonStore<SharedStore<S>, SeqRecvPath, Sequence>,
    /// A typed-store for ack sequences
    pub ack_sequence_store: JsonStore<SharedStore<S>, SeqAckPath, Sequence>,
    /// A typed-store for packet commitments
    pub packet_commitment_store: BinStore<SharedStore<S>, CommitmentPath, PacketCommitment>,
    /// A typed-store for packet receipts
    pub packet_receipt_store: TypedSet<SharedStore<S>, ReceiptPath>,
    /// A typed-store for packet ack
    pub packet_ack_store: BinStore<SharedStore<S>, AckPath, AcknowledgementCommitment>,
    /// Map of host consensus states
    pub host_consensus_states: Arc<Mutex<BTreeMap<u64, AnyConsensusState>>>,
    /// Map of older ibc commitment proofs
    pub ibc_commiment_proofs: Arc<Mutex<BTreeMap<u64, CommitmentProof>>>,
    /// IBC Events
    pub events: Arc<Mutex<Vec<IbcEvent>>>,
    /// message logs
    pub logs: Arc<Mutex<Vec<String>>>,
}

impl<S> MockIbcStore<S>
where
    S: ProvableStore + Debug,
{
    pub fn new(revision_number: u64, store: S) -> Self {
        let shared_store = SharedStore::new(store);

        let mut client_counter = TypedStore::new(shared_store.clone());
        let mut conn_counter = TypedStore::new(shared_store.clone());
        let mut channel_counter = TypedStore::new(shared_store.clone());

        client_counter
            .set(NextClientSequencePath, 0)
            .expect("no error");

        conn_counter
            .set(NextConnectionSequencePath, 0)
            .expect("no error");

        channel_counter
            .set(NextChannelSequencePath, 0)
            .expect("no error");

        Self {
            revision_number: Arc::new(Mutex::new(revision_number)),
            client_counter,
            conn_counter,
            channel_counter,
            client_processed_times: TypedStore::new(shared_store.clone()),
            client_processed_heights: TypedStore::new(shared_store.clone()),
            host_consensus_states: Arc::new(Mutex::new(Default::default())),
            ibc_commiment_proofs: Arc::new(Mutex::new(Default::default())),
            client_state_store: TypedStore::new(shared_store.clone()),
            consensus_state_store: TypedStore::new(shared_store.clone()),
            connection_end_store: TypedStore::new(shared_store.clone()),
            connection_ids_store: TypedStore::new(shared_store.clone()),
            channel_end_store: TypedStore::new(shared_store.clone()),
            send_sequence_store: TypedStore::new(shared_store.clone()),
            recv_sequence_store: TypedStore::new(shared_store.clone()),
            ack_sequence_store: TypedStore::new(shared_store.clone()),
            packet_commitment_store: TypedStore::new(shared_store.clone()),
            packet_receipt_store: TypedStore::new(shared_store.clone()),
            packet_ack_store: TypedStore::new(shared_store.clone()),
            events: Arc::new(Mutex::new(Vec::new())),
            logs: Arc::new(Mutex::new(Vec::new())),
            store: shared_store,
        }
    }

    fn store_host_consensus_state(&mut self, height: u64, consensus_state: AnyConsensusState) {
        self.host_consensus_states
            .lock()
            .insert(height, consensus_state);
    }

    fn store_ibc_commitment_proof(&mut self, height: u64, proof: CommitmentProof) {
        self.ibc_commiment_proofs.lock().insert(height, proof);
    }

    pub fn begin_block(
        &mut self,
        height: u64,
        consensus_state: AnyConsensusState,
        proof: CommitmentProof,
    ) {
        assert_eq!(self.store.current_height(), height);
        self.store_host_consensus_state(height, consensus_state);
        self.store_ibc_commitment_proof(height, proof);
    }

    pub fn end_block(&mut self) -> Result<Vec<u8>, <SharedStore<S> as Store>::Error> {
        self.store.commit()
    }

    pub fn prune_host_consensus_states_till(&self, height: &Height) {
        assert!(height.revision_number() == *self.revision_number.lock());
        let mut history = self.host_consensus_states.lock();
        history.retain(|h, _| h > &height.revision_height());
        let mut commitment_proofs = self.ibc_commiment_proofs.lock();
        commitment_proofs.retain(|h, _| h > &height.revision_height());
    }
}

impl<S> Default for MockIbcStore<S>
where
    S: ProvableStore + Debug + Default,
{
    fn default() -> Self {
        // Note: this creates a MockIbcStore which has MockConsensusState as Host ConsensusState
        let mut ibc_store = Self::new(0, S::default());
        ibc_store.store.commit().expect("no error");
        ibc_store.store_host_consensus_state(
            ibc_store.store.current_height(),
            MockHeader::default()
                .with_current_timestamp()
                .into_consensus_state()
                .into(),
        );
        ibc_store.store_ibc_commitment_proof(
            ibc_store.store.current_height(),
            CommitmentProof::default(),
        );
        ibc_store
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::channel::types::acknowledgement::Acknowledgement;
    use ibc::core::channel::types::channel::{Counterparty, Order};
    use ibc::core::channel::types::error::{ChannelError, PacketError};
    use ibc::core::channel::types::packet::Packet;
    use ibc::core::channel::types::Version;
    use ibc::core::host::types::identifiers::{ChannelId, PortId};
    use ibc::core::primitives::Signer;
    use ibc::core::router::module::Module;
    use ibc::core::router::router::Router;
    use ibc::core::router::types::module::{ModuleExtras, ModuleId};

    use super::*;
    use crate::fixtures::core::channel::PacketConfig;
    use crate::fixtures::core::signer::dummy_bech32_account;
    use crate::testapp::ibc::core::router::MockRouter;

    #[test]
    fn test_router() {
        #[derive(Debug, Default)]
        struct FooModule {
            counter: u64,
        }

        impl Module for FooModule {
            fn on_chan_open_init_validate(
                &self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                version: &Version,
            ) -> Result<Version, ChannelError> {
                Ok(version.clone())
            }

            fn on_chan_open_init_execute(
                &mut self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                version: &Version,
            ) -> Result<(ModuleExtras, Version), ChannelError> {
                Ok((ModuleExtras::empty(), version.clone()))
            }

            fn on_chan_open_try_validate(
                &self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                counterparty_version: &Version,
            ) -> Result<Version, ChannelError> {
                Ok(counterparty_version.clone())
            }

            fn on_chan_open_try_execute(
                &mut self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                counterparty_version: &Version,
            ) -> Result<(ModuleExtras, Version), ChannelError> {
                Ok((ModuleExtras::empty(), counterparty_version.clone()))
            }

            fn on_recv_packet_execute(
                &mut self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> (ModuleExtras, Acknowledgement) {
                self.counter += 1;

                (
                    ModuleExtras::empty(),
                    Acknowledgement::try_from(vec![1u8]).expect("Never fails"),
                )
            }

            fn on_timeout_packet_validate(
                &self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> Result<(), PacketError> {
                Ok(())
            }

            fn on_timeout_packet_execute(
                &mut self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> (ModuleExtras, Result<(), PacketError>) {
                (ModuleExtras::empty(), Ok(()))
            }

            fn on_acknowledgement_packet_validate(
                &self,
                _packet: &Packet,
                _acknowledgement: &Acknowledgement,
                _relayer: &Signer,
            ) -> Result<(), PacketError> {
                Ok(())
            }

            fn on_acknowledgement_packet_execute(
                &mut self,
                _packet: &Packet,
                _acknowledgement: &Acknowledgement,
                _relayer: &Signer,
            ) -> (ModuleExtras, Result<(), PacketError>) {
                (ModuleExtras::empty(), Ok(()))
            }
        }

        #[derive(Debug, Default)]
        struct BarModule;

        impl Module for BarModule {
            fn on_chan_open_init_validate(
                &self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                version: &Version,
            ) -> Result<Version, ChannelError> {
                Ok(version.clone())
            }

            fn on_chan_open_init_execute(
                &mut self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                version: &Version,
            ) -> Result<(ModuleExtras, Version), ChannelError> {
                Ok((ModuleExtras::empty(), version.clone()))
            }

            fn on_chan_open_try_validate(
                &self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                counterparty_version: &Version,
            ) -> Result<Version, ChannelError> {
                Ok(counterparty_version.clone())
            }

            fn on_chan_open_try_execute(
                &mut self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                counterparty_version: &Version,
            ) -> Result<(ModuleExtras, Version), ChannelError> {
                Ok((ModuleExtras::empty(), counterparty_version.clone()))
            }

            fn on_recv_packet_execute(
                &mut self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> (ModuleExtras, Acknowledgement) {
                (
                    ModuleExtras::empty(),
                    Acknowledgement::try_from(vec![1u8]).expect("Never fails"),
                )
            }

            fn on_timeout_packet_validate(
                &self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> Result<(), PacketError> {
                Ok(())
            }

            fn on_timeout_packet_execute(
                &mut self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> (ModuleExtras, Result<(), PacketError>) {
                (ModuleExtras::empty(), Ok(()))
            }

            fn on_acknowledgement_packet_validate(
                &self,
                _packet: &Packet,
                _acknowledgement: &Acknowledgement,
                _relayer: &Signer,
            ) -> Result<(), PacketError> {
                Ok(())
            }

            fn on_acknowledgement_packet_execute(
                &mut self,
                _packet: &Packet,
                _acknowledgement: &Acknowledgement,
                _relayer: &Signer,
            ) -> (ModuleExtras, Result<(), PacketError>) {
                (ModuleExtras::empty(), Ok(()))
            }
        }

        let mut router = MockRouter::default();
        router
            .add_route(ModuleId::new("foomodule".to_string()), FooModule::default())
            .expect("Never fails");
        router
            .add_route(ModuleId::new("barmodule".to_string()), BarModule)
            .expect("Never fails");

        let mut on_recv_packet_result = |module_id: &'static str| {
            let module_id = ModuleId::new(module_id.to_string());
            let m = router.get_route_mut(&module_id).expect("Never fails");

            let packet = PacketConfig::builder().build();

            let result = m.on_recv_packet_execute(&packet, &dummy_bech32_account().into());
            (module_id, result)
        };

        let _results = [
            on_recv_packet_result("foomodule"),
            on_recv_packet_result("barmodule"),
        ];
    }
}

pub struct LightClientState<H: TestHost> {
    pub client_state: H::ClientState,
    pub consensus_states:
        BTreeMap<Height, <<H::Block as TestBlock>::Header as TestHeader>::ConsensusState>,
}

impl<H> Default for LightClientState<H>
where
    H: TestHost,
    HostClientState<H>: ClientStateValidation<DefaultIbcStore>,
{
    fn default() -> Self {
        let context = TestContext::<H>::default();
        LightClientBuilder::init().context(&context).build()
    }
}

impl<H> LightClientState<H>
where
    H: TestHost,
    HostClientState<H>: ClientStateValidation<DefaultIbcStore>,
{
    pub fn with_latest_height(height: Height) -> Self {
        let context = TestContextConfig::builder()
            .latest_height(height)
            .build::<TestContext<_>>();
        LightClientBuilder::init().context(&context).build()
    }
}

#[derive(TypedBuilder)]
#[builder(builder_method(name = init), build_method(into))]
pub struct LightClientBuilder<'a, H>
where
    H: TestHost,
    HostClientState<H>: ClientStateValidation<DefaultIbcStore>,
{
    context: &'a TestContext<H>,
    #[builder(default, setter(into))]
    consensus_heights: Vec<Height>,
    #[builder(default)]
    params: H::LightClientParams,
}

impl<'a, H> From<LightClientBuilder<'a, H>> for LightClientState<H>
where
    H: TestHost,
    HostClientState<H>: ClientStateValidation<DefaultIbcStore>,
{
    fn from(builder: LightClientBuilder<'a, H>) -> Self {
        let LightClientBuilder {
            context,
            consensus_heights,
            params,
        } = builder;

        context.generate_light_client(consensus_heights, &params)
    }
}
