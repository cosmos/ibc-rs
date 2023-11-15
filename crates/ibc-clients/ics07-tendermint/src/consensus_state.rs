impl ConsensusStateTrait for ConsensusState {
    fn root(&self) -> &CommitmentRoot {
        &self.root
    }

    fn timestamp(&self) -> Timestamp {
        self.timestamp.into()
    }

    fn encode_vec(self) -> Vec<u8> {
        <Self as Protobuf<Any>>::encode_vec(self)
    }
}