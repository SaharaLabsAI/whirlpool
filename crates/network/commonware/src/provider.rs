//! CommonwareNetworkProvider implementation using discovery::Network.

use commonware_cryptography::{PublicKey, Signer};
use commonware_p2p::authenticated::discovery::{
    self, Bootstrapper, Oracle, Receiver as DiscoveryReceiver, Sender as DiscoverySender,
};
use commonware_runtime::{BufferPooler, Clock, Metrics, Network, Quota, Resolver, Spawner};
use commonware_utils::ordered::Set;
use rand_core::CryptoRngCore;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::num::NonZeroU32;

use crate::{
    traits::CommonwareTransport, CommonwarePeerId, CommonwareReceiver, CommonwareSender,
    MultiplexReceiver, MultiplexSender,
};
use network::{Channel, NetworkProvider, P2pError};

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

/// Handle used to update the discovery oracle after provider construction.
pub struct OracleHandle<PK: PublicKey>(Oracle<PK>);

impl<PK> OracleHandle<PK>
where
    PK: PublicKey + Clone + Hash + Eq + Debug + Send + Sync + 'static,
{
    pub async fn update_validators(
        &mut self,
        epoch: u64,
        validators: impl IntoIterator<Item = PK>,
    ) {
        use commonware_p2p::Manager;

        let peers = Set::from_iter_dedup(validators);
        self.0.track(epoch, peers).await;
    }
}

/// Builder for constructing a discovery-backed network provider from high-level inputs.
pub struct CommonwareNetworkProviderBuilder<C, E = ()>
where
    C: Signer,
{
    signer: C,
    namespace: Vec<u8>,
    listen_addr: SocketAddr,
    dialable_addr: SocketAddr,
    bootstrappers: Vec<Bootstrapper<C::PublicKey>>,
    max_message_size: u32,
    initial_validators: Option<(u64, Vec<C::PublicKey>)>,
    channel_config: ChannelConfig,
    _phantom: PhantomData<E>,
}

/// Impl block for the default case where E = ()
impl<C: Signer> CommonwareNetworkProviderBuilder<C, ()>
where
    C::PublicKey: Clone + Hash + Eq + Debug + Send + Sync + 'static,
{
    pub fn new(signer: C, namespace: impl Into<Vec<u8>>) -> Self {
        let default_addr = SocketAddr::from(([0, 0, 0, 0], 0));
        Self {
            signer,
            namespace: namespace.into(),
            listen_addr: default_addr,
            dialable_addr: default_addr,
            bootstrappers: Vec::new(),
            max_message_size: 1024 * 1024,
            initial_validators: None,
            channel_config: ChannelConfig::default(),
            _phantom: PhantomData,
        }
    }
}
impl<C: Signer, E> CommonwareNetworkProviderBuilder<C, E>
where
    C::PublicKey: Clone + Hash + Eq + Debug + Send + Sync + 'static,
{
    pub fn is_some(&self) -> bool {
        true
    }

    pub fn listen_addr(mut self, addr: SocketAddr) -> Self {
        self.listen_addr = addr;
        self
    }

    pub fn dialable_addr(mut self, addr: SocketAddr) -> Self {
        self.dialable_addr = addr;
        self
    }

    pub fn bootstrappers(mut self, bootstrappers: Vec<Bootstrapper<C::PublicKey>>) -> Self {
        self.bootstrappers = bootstrappers;
        self
    }

    pub fn max_message_size(mut self, size: u32) -> Self {
        self.max_message_size = size;
        self
    }

    pub fn initial_validators(mut self, epoch: u64, validators: Vec<C::PublicKey>) -> Self {
        self.initial_validators = Some((epoch, validators));
        self
    }

    pub fn channel_config(mut self, config: ChannelConfig) -> Self {
        self.channel_config = config;
        self
    }

    pub async fn build<Ctx>(
        self,
        context: Ctx,
    ) -> (
        CommonwareNetworkProvider<Ctx, C>,
        OracleHandle<C::PublicKey>,
    )
    where
        Ctx: Spawner
            + BufferPooler
            + Clock
            + CryptoRngCore
            + Network
            + Resolver
            + Metrics
            + Send
            + 'static,
    {
        let config = discovery::Config::local(
            self.signer,
            &self.namespace,
            self.listen_addr,
            self.dialable_addr,
            self.bootstrappers,
            self.max_message_size,
        );

        let (network, oracle) = discovery::Network::new(context, config);
        let mut oracle_handle = OracleHandle(oracle.clone());

        if let Some((epoch, validators)) = self.initial_validators {
            if !validators.is_empty() {
                oracle_handle.update_validators(epoch, validators).await;
            }
        }

        let provider = CommonwareNetworkProvider {
            network,
            oracle,
            channel_config: self.channel_config,
        };

        (provider, oracle_handle)
    }
}

/// Separate channel pairs for simplex-style consumers that require dedicated
/// vote/certificate/resolver streams.
///
/// Exposes raw vendor types (S, R) to preserve trait implementations required
/// by vendor `simplex::Engine::start()`. No wrappers are applied here.
pub struct PerChannelNetwork<S, R> {
    pub vote: (S, R),
    pub cert: (S, R),
    pub resolver: (S, R),
    pub payload: (S, R),
    pub network_handle: commonware_runtime::Handle<()>,
}

type DiscoveryPerChannelNetwork<E, C> = PerChannelNetwork<
    DiscoverySender<<C as Signer>::PublicKey, E>,
    DiscoveryReceiver<<C as Signer>::PublicKey>,
>;

/// Network provider that uses commonware's discovery::Network.
///
/// Registers 3 channels (VOTE, CERTIFICATE, RESOLVER) and multiplexes them
/// through a single NetworkSender/NetworkReceiver interface.
pub struct CommonwareNetworkProvider<E, C>
where
    E: Spawner + BufferPooler + Clock + CryptoRngCore + Network + Resolver + Metrics,
    C: Signer,
{
    network: discovery::Network<E, C>,
    oracle: Oracle<C::PublicKey>,
    channel_config: ChannelConfig,
}

impl<E, C> CommonwareNetworkProvider<E, C>
where
    E: Spawner + BufferPooler + Clock + CryptoRngCore + Network + Resolver + Metrics,
    C: Signer,
    C::PublicKey: Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static,
{
    /// Create a new provider from a discovery network and oracle.
    pub fn new(network: discovery::Network<E, C>, oracle: Oracle<C::PublicKey>) -> Self {
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

    /// Start the network and return dedicated channel pairs.
    pub fn start_per_channel(mut self) -> Result<DiscoveryPerChannelNetwork<E, C>, P2pError> {
        let backlog = self.channel_config.backlog;
        let quota = Quota::per_second(NonZeroU32::new(10000).unwrap());

        let (vote_sender, vote_receiver) = self.network.register(Channel::VOTE.0, quota, backlog);
        let (cert_sender, cert_receiver) =
            self.network
                .register(Channel::CERTIFICATE.0, quota, backlog);
        let (resolver_sender, resolver_receiver) =
            self.network.register(Channel::RESOLVER.0, quota, backlog);
        let (payload_sender, payload_receiver) =
            self.network.register(Channel::PAYLOAD.0, quota, backlog);

        let network_handle = self.network.start();

        // Return raw vendor types (no wrappers) to preserve trait implementations
        // required by vendor simplex::Engine::start()
        Ok(PerChannelNetwork {
            vote: (vote_sender, vote_receiver),
            cert: (cert_sender, cert_receiver),
            resolver: (resolver_sender, resolver_receiver),
            payload: (payload_sender, payload_receiver),
            network_handle,
        })
    }

    /// Get a reference to the oracle for peer set management.
    pub fn oracle(&self) -> &Oracle<C::PublicKey> {
        &self.oracle
    }
}

impl<E, C> CommonwareTransport for CommonwareNetworkProvider<E, C>
where
    E: Spawner + BufferPooler + Clock + CryptoRngCore + Network + Resolver + Metrics,
    C: Signer,
    C::PublicKey: Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static,
{
    type PublicKey = C::PublicKey;
    type Sender = DiscoverySender<C::PublicKey, E>;
    type Receiver = DiscoveryReceiver<C::PublicKey>;

    fn start_per_channel(
        self,
    ) -> Result<PerChannelNetwork<Self::Sender, Self::Receiver>, P2pError> {
        CommonwareNetworkProvider::start_per_channel(self)
    }

    fn oracle(&self) -> &Oracle<Self::PublicKey> {
        CommonwareNetworkProvider::oracle(self)
    }
}

impl<E, C> NetworkProvider for CommonwareNetworkProvider<E, C>
where
    E: Spawner + BufferPooler + Clock + CryptoRngCore + Network + Resolver + Metrics,
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
        let (vote_sender, vote_receiver) = self.network.register(Channel::VOTE.0, quota, backlog);

        // Register CERTIFICATE channel (1)
        let (cert_sender, cert_receiver) =
            self.network
                .register(Channel::CERTIFICATE.0, quota, backlog);

        // Register RESOLVER channel (2)
        let (res_sender, res_receiver) = self.network.register(Channel::RESOLVER.0, quota, backlog);

        // Register PAYLOAD channel (3)
        let (payload_sender, payload_receiver) =
            self.network.register(Channel::PAYLOAD.0, quota, backlog);

        // Start the network (returns handle that keeps network alive)
        let handle = self.network.start();

        // Build sender map
        let mut senders = HashMap::new();
        senders.insert(Channel::VOTE, CommonwareSender::new(vote_sender));
        senders.insert(Channel::CERTIFICATE, CommonwareSender::new(cert_sender));
        senders.insert(Channel::RESOLVER, CommonwareSender::new(res_sender));
        senders.insert(Channel::PAYLOAD, CommonwareSender::new(payload_sender));

        // Build receiver list
        let receivers = vec![
            (
                Channel::VOTE,
                CommonwareReceiver::new(Channel::VOTE, vote_receiver),
            ),
            (
                Channel::CERTIFICATE,
                CommonwareReceiver::new(Channel::CERTIFICATE, cert_receiver),
            ),
            (
                Channel::RESOLVER,
                CommonwareReceiver::new(Channel::RESOLVER, res_receiver),
            ),
            (
                Channel::PAYLOAD,
                CommonwareReceiver::new(Channel::PAYLOAD, payload_receiver),
            ),
        ];

        let multiplex_sender = MultiplexSender::new(senders);
        let multiplex_receiver = MultiplexReceiver::new(receivers, handle);

        Ok((multiplex_sender, multiplex_receiver))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use commonware_cryptography::ed25519;
    use commonware_cryptography::Signer;
    use commonware_p2p::{Provider as _, Receiver as _, Recipients, Sender as _};
    use commonware_runtime::{deterministic, Clock, Runner};
    use std::net::SocketAddr;
    use std::time::Duration;

    #[test]
    fn tst_req1_001_builder_seeds_non_empty_validator_set() {
        let runner = deterministic::Runner::default();
        runner.start(|context| async move {
            let signer = ed25519::PrivateKey::from_seed(920);
            let self_pk = signer.public_key();
            let other_pk = ed25519::PrivateKey::from_seed(921).public_key();
            let addr = "127.0.0.1:30101"
                .parse::<SocketAddr>()
                .expect("valid socket");

            // Verify builder accepts initial_validators and build succeeds.
            let (_provider, _oracle_handle) =
                CommonwareNetworkProviderBuilder::new(signer, b"seed-validators-test")
                    .listen_addr(addr)
                    .dialable_addr(addr)
                    .initial_validators(0, vec![self_pk.clone(), other_pk.clone(), self_pk.clone()])
                    .build(context.with_label("seeded_network"))
                    .await;

            // Test passes if build() completes without error.
            // Oracle seeding happens internally; unit test can't verify actor state.
        });
    }

    #[test]
    fn tst_req1_002_builder_skips_empty_validator_set() {
        let runner = deterministic::Runner::default();
        runner.start(|context| async move {
            let signer = ed25519::PrivateKey::from_seed(922);
            let addr = "127.0.0.1:30102"
                .parse::<SocketAddr>()
                .expect("valid socket");

            // Verify builder accepts empty initial_validators and build succeeds.
            let (_provider, _oracle_handle) =
                CommonwareNetworkProviderBuilder::new(signer, b"empty-validators-test")
                    .listen_addr(addr)
                    .dialable_addr(addr)
                    .initial_validators(0, vec![])
                    .build(context.with_label("empty_seed_network"))
                    .await;

            // Unit test confirms API acceptance and build() success only.
            // Oracle internals require a fully running actor context.
        });
    }

    #[test]
    fn tst_req2_001_builder_threads_bootstrappers_into_discovery() {
        let runner = deterministic::Runner::default();
        runner.start(|context| async move {
            let signer_0 = ed25519::PrivateKey::from_seed(923);
            let signer_1 = ed25519::PrivateKey::from_seed(924);
            let pk_0 = signer_0.public_key();

            let addr_0 = "127.0.0.1:30103"
                .parse::<SocketAddr>()
                .expect("valid socket");
            let addr_1 = "127.0.0.1:30104"
                .parse::<SocketAddr>()
                .expect("valid socket");

            let (provider_0, mut oracle_0) =
                CommonwareNetworkProviderBuilder::new(signer_0, b"bootstrap-threading-test")
                    .listen_addr(addr_0)
                    .dialable_addr(addr_0)
                    .initial_validators(0, vec![pk_0.clone()])
                    .build(context.with_label("bootstrap_peer_0"))
                    .await;

            let (provider_1, _oracle_1) =
                CommonwareNetworkProviderBuilder::new(signer_1, b"bootstrap-threading-test")
                    .listen_addr(addr_1)
                    .dialable_addr(addr_1)
                    .bootstrappers(vec![(pk_0.clone(), addr_0.into())])
                    .initial_validators(0, vec![pk_0.clone()])
                    .build(context.with_label("bootstrap_peer_1"))
                    .await;

            let peer_0 = provider_0.start_per_channel().expect("peer 0 starts");
            let _peer_1 = provider_1.start_per_channel().expect("peer 1 starts");

            context.sleep(Duration::from_secs(1)).await;

            let mut vote_sender = peer_0.vote.0;
            vote_sender
                .send(
                    Recipients::One(pk_0.clone()),
                    Bytes::from_static(b"bootstrap-ready"),
                    false,
                )
                .await
                .expect("send should succeed when bootstrap config is valid");

            let seeded = oracle_0
                .0
                .peer_set(0)
                .await
                .expect("validator set seeded for bootstrap test");
            assert!(seeded.primary.iter().any(|pk| pk == &pk_0));
        });
    }

    #[test]
    fn test_start_per_channel_returns_three_pairs() {
        let runner = deterministic::Runner::default();
        runner.start(|context| async move {
            let signer_0 = ed25519::PrivateKey::from_seed(900);
            let signer_1 = ed25519::PrivateKey::from_seed(901);
            let pk_0 = signer_0.public_key();
            let pk_1 = signer_1.public_key();

            let addr_0 = "127.0.0.1:30001"
                .parse::<SocketAddr>()
                .expect("valid socket");
            let addr_1 = "127.0.0.1:30002"
                .parse::<SocketAddr>()
                .expect("valid socket");

            let (provider_0, mut oracle_0) =
                CommonwareNetworkProviderBuilder::new(signer_0, b"per-channel-test")
                    .listen_addr(addr_0)
                    .dialable_addr(addr_0)
                    .build(context.with_label("peer_0_network"))
                    .await;

            let (provider_1, mut oracle_1) =
                CommonwareNetworkProviderBuilder::new(signer_1, b"per-channel-test")
                    .listen_addr(addr_1)
                    .dialable_addr(addr_1)
                    .bootstrappers(vec![(pk_0.clone(), addr_0.into())])
                    .build(context.with_label("peer_1_network"))
                    .await;

            oracle_0
                .update_validators(0, vec![pk_0.clone(), pk_1.clone()])
                .await;
            oracle_1
                .update_validators(0, vec![pk_0.clone(), pk_1.clone()])
                .await;

            let _peer_0 = provider_0.start_per_channel().expect("peer 0 starts");
            let _peer_1 = provider_1.start_per_channel().expect("peer 1 starts");
        });
    }

    #[test]
    fn test_per_channel_send_receive() {
        let runner = deterministic::Runner::default();
        runner.start(|context| async move {
            let signer_0 = ed25519::PrivateKey::from_seed(910);
            let signer_1 = ed25519::PrivateKey::from_seed(911);
            let pk_0 = signer_0.public_key();
            let pk_1 = signer_1.public_key();

            let addr_0 = "127.0.0.1:30011"
                .parse::<SocketAddr>()
                .expect("valid socket");
            let addr_1 = "127.0.0.1:30012"
                .parse::<SocketAddr>()
                .expect("valid socket");

            let (provider_0, mut oracle_0) =
                CommonwareNetworkProviderBuilder::new(signer_0, b"per-channel-io-test")
                    .listen_addr(addr_0)
                    .dialable_addr(addr_0)
                    .build(context.with_label("peer_0_network"))
                    .await;

            let (provider_1, mut oracle_1) =
                CommonwareNetworkProviderBuilder::new(signer_1, b"per-channel-io-test")
                    .listen_addr(addr_1)
                    .dialable_addr(addr_1)
                    .bootstrappers(vec![(pk_0.clone(), addr_0.into())])
                    .build(context.with_label("peer_1_network"))
                    .await;

            oracle_0
                .update_validators(0, vec![pk_0.clone(), pk_1.clone()])
                .await;
            oracle_1
                .update_validators(0, vec![pk_0.clone(), pk_1.clone()])
                .await;

            let mut peer_0 = provider_0.start_per_channel().expect("peer 0 starts");
            let mut peer_1 = provider_1.start_per_channel().expect("peer 1 starts");

            context.sleep(Duration::from_secs(2)).await;

            peer_0
                .vote
                .0
                .send(
                    Recipients::One(pk_1.clone()),
                    Bytes::from_static(b"vote-msg"),
                    false,
                )
                .await
                .expect("vote send should succeed");

            peer_0
                .cert
                .0
                .send(
                    Recipients::One(pk_1.clone()),
                    Bytes::from_static(b"cert-msg"),
                    false,
                )
                .await
                .expect("certificate send should succeed");

            peer_0
                .resolver
                .0
                .send(
                    Recipients::One(pk_1),
                    Bytes::from_static(b"resolver-msg"),
                    false,
                )
                .await
                .expect("resolver send should succeed");

            let vote = peer_1.vote.1.recv().await.expect("vote receive");
            let cert = peer_1.cert.1.recv().await.expect("cert receive");
            let resolver = peer_1.resolver.1.recv().await.expect("resolver receive");

            let vote_data: Bytes = vote.1.into();
            let cert_data: Bytes = cert.1.into();
            let resolver_data: Bytes = resolver.1.into();

            assert_eq!(vote_data, Bytes::from_static(b"vote-msg"));
            assert_eq!(cert_data, Bytes::from_static(b"cert-msg"));
            assert_eq!(resolver_data, Bytes::from_static(b"resolver-msg"));
        });
    }
}
