use crate::types::Path;

use ibc::core::host::types::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, ClientStatePath,
    CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
    UpgradeClientPath,
};

macro_rules! impl_into_path_for {
    ($($path:ty),+) => {
        $(impl From<$path> for Path {
            fn from(ibc_path: $path) -> Self {
                Self::try_from(ibc_path.to_string()).unwrap() // safety - `IbcPath`s are correct-by-construction
            }
        })+
    };
}

impl_into_path_for!(
    ClientStatePath,
    ClientConsensusStatePath,
    ConnectionPath,
    ClientConnectionPath,
    ChannelEndPath,
    SeqSendPath,
    SeqRecvPath,
    SeqAckPath,
    CommitmentPath,
    ReceiptPath,
    AckPath,
    UpgradeClientPath
);
