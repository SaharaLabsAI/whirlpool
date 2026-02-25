use crate::block::Block;
use std::future::Future;

/// Events emitted by the consensus engine.
#[derive(Debug)]
pub enum ConsensusEvent<B: Block> {
    /// A block has been finalized by consensus.
    Finalized {
        block: B,
        height: u64,
        /// Opaque finalization proof bytes.
        proof: Vec<u8>,
    },
    /// A block is tentatively accepted but not yet finalized.
    PreFinalized {
        block: B,
        height: u64,
    },
    /// A consensus fault was detected.
    Fault {
        /// Identifier of the faulty participant.
        offender: Vec<u8>,
        /// Opaque evidence bytes.
        evidence: Vec<u8>,
    },
}

/// Sink that receives consensus events.
pub trait EventSink: Send + Sync + 'static {
    /// The block type carried by events.
    type Block: Block;

    /// Handle an incoming consensus event.
    fn handle(
        &self,
        event: ConsensusEvent<Self::Block>,
    ) -> impl Future<Output = ()> + Send;
}
