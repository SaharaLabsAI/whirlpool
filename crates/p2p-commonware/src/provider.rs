use p2p::{Channel, NetworkProvider, P2pError};
use std::collections::HashSet;
use commonware_p2p::{Sender as CwSender, Receiver as CwReceiver};
use commonware_cryptography::PublicKey;
use std::fmt::Debug;
use std::hash::Hash;
use crate::{CommonwareSender, CommonwareReceiver};

/// Network provider that uses a closure factory to create Commonware sender/receiver pairs.
///
/// The factory closure is called with a channel ID and should return a tuple of
/// (Sender, Receiver) from the Commonware P2P implementation.
pub struct CommonwareNetworkProvider<F> {
    factory: F,
    opened: HashSet<Channel>,
}

impl<F> CommonwareNetworkProvider<F> {
    /// Create a new provider with the given factory closure.
    ///
    /// The factory will be called when `start()` is invoked to create the
    /// underlying Commonware sender/receiver pair.
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            opened: HashSet::new(),
        }
    }
}

impl<F, S, R, P> NetworkProvider for CommonwareNetworkProvider<F>
where
    F: FnMut(u64) -> Result<(S, R), Box<dyn std::error::Error + Send + Sync>> + Send + 'static,
    S: CwSender<PublicKey = P> + Clone + Send + Sync + 'static,
    R: CwReceiver<PublicKey = P> + Send + 'static,
    P: PublicKey + Clone + Eq + Hash + Debug + Send + Sync + 'static,
{
    type PeerId = crate::CommonwarePeerId<P>;
    type Sender = CommonwareSender<S>;
    type Receiver = CommonwareReceiver<R>;

    fn start(mut self) -> Result<(Self::Sender, Self::Receiver), P2pError> {
        // Use channel 0 as default for simple start() API
        let (sender, receiver) = (self.factory)(0)
            .map_err(|e| P2pError::SendFailed(e.to_string()))?;
        
        Ok((
            CommonwareSender::new(sender),
            CommonwareReceiver::new(receiver),
        ))
    }
}
