//! CommonwareNetworkProvider implementation using discovery::Network.

use std::collections::HashMap;
use std::num::NonZeroU32;
use commonware_cryptography::Signer;
use commonware_p2p::authenticated::discovery::{self, Oracle, Sender as DiscoverySender, Receiver as DiscoveryReceiver};
use commonware_runtime::{Clock, Metrics, Resolver, Spawner, Quota, Network};
use rand_core::CryptoRngCore;

use crate::{
    CommonwareReceiver, CommonwareSender, CommonwarePeerId, MultiplexReceiver, MultiplexSender,
};
use p2p::{Channel, NetworkProvider, P2pError};

/// Configuration for channel registration.
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    /// Maximum number of messages to buffer per channel before backpressure.
    pub backlog: usize,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self { backlog: 1024 }
    }
}

/// Network provider that uses commonware's discovery::Network.
/// 
/// Registers 3 channels (VOTE, CERTIFICATE, RESOLVER) and multiplexes them
/// through a single NetworkSender/NetworkReceiver interface.
pub struct CommonwareNetworkProvider<E, C>
where
    E: Spawner + Clock + CryptoRngCore + Network + Resolver + Metrics,
    C: Signer,
{
    network: discovery::Network<E, C>,
    oracle: Oracle<C::PublicKey>,
    channel_config: ChannelConfig,
}

impl<E, C> CommonwareNetworkProvider<E, C>
where
    E: Spawner + Clock + CryptoRngCore + Network + Resolver + Metrics,
    C: Signer,
{
    /// Create a new provider from a discovery network and oracle.
    pub fn new(
        network: discovery::Network<E, C>,
        oracle: Oracle<C::PublicKey>,
    ) -> Self {
        Self {
            network,
            oracle,
            channel_config: ChannelConfig::default(),
        }
    }

    /// Create a new provider with custom channel configuration.
    pub fn with_config(
        network: discovery::Network<E, C>,
        oracle: Oracle<C::PublicKey>,
        channel_config: ChannelConfig,
    ) -> Self {
        Self {
            network,
            oracle,
            channel_config,
        }
    }

    /// Get a reference to the oracle for peer set management.
    pub fn oracle(&self) -> &Oracle<C::PublicKey> {
        &self.oracle
    }
}

impl<E, C> NetworkProvider for CommonwareNetworkProvider<E, C>
where
    E: Spawner + Clock + CryptoRngCore + Network + Resolver + Metrics,
    C: Signer,
    C::PublicKey: Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static,
{
    type PeerId = CommonwarePeerId<C::PublicKey>;
    type Sender = MultiplexSender<DiscoverySender<C::PublicKey, E>>;
    type Receiver = MultiplexReceiver<DiscoveryReceiver<C::PublicKey>>;

    fn start(mut self) -> Result<(Self::Sender, Self::Receiver), P2pError> {
        let backlog = self.channel_config.backlog;
        // Create a default quota - 10,000 messages per second
        let quota = Quota::per_second(NonZeroU32::new(10000).unwrap());

        // Register VOTE channel (0)
        let (vote_sender, vote_receiver) = self.network.register(
            Channel::VOTE.0,
            quota.clone(),
            backlog,
        );

        // Register CERTIFICATE channel (1)
        let (cert_sender, cert_receiver) = self.network.register(
            Channel::CERTIFICATE.0,
            quota.clone(),
            backlog,
        );

        // Register RESOLVER channel (2)
        let (res_sender, res_receiver) = self.network.register(
            Channel::RESOLVER.0,
            quota.clone(),
            backlog,
        );

        // Start the network (returns handle that keeps network alive)
        let handle = self.network.start();

        // Build sender map
        let mut senders = HashMap::new();
        senders.insert(Channel::VOTE, CommonwareSender::new(vote_sender));
        senders.insert(Channel::CERTIFICATE, CommonwareSender::new(cert_sender));
        senders.insert(Channel::RESOLVER, CommonwareSender::new(res_sender));

        // Build receiver list
        let receivers = vec![
            (Channel::VOTE, CommonwareReceiver::new(vote_receiver)),
            (Channel::CERTIFICATE, CommonwareReceiver::new(cert_receiver)),
            (Channel::RESOLVER, CommonwareReceiver::new(res_receiver)),
        ];

        let multiplex_sender = MultiplexSender::new(senders);
        let multiplex_receiver = MultiplexReceiver::new(receivers, handle);

        Ok((multiplex_sender, multiplex_receiver))
    }
}
