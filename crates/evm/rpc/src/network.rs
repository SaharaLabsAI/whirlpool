//! Minimal network adapter for the reth RPC stack.
//!
//! Whirlpool is a single-node system — there is no real P2P layer. [`WhirlpoolNetwork`]
//! satisfies the [`NetworkInfo`] + [`Peers`] bounds required by [`reth_rpc_builder::RpcModuleBuilder`]
//! by returning static / empty values for every query.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use enr::secp256k1::SecretKey;
use reth_eth_wire_types::{Capability, DisconnectReason};
use reth_network_api::{
    NetworkError, NetworkInfo, NetworkStatus, PeerKind, Peers, PeersInfo, Reputation,
    ReputationChangeKind,
};
use reth_network_peers::{NodeRecord, PeerId};

/// Static listen address returned by [`WhirlpoolNetwork`].
const LISTEN_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 30303);

/// Minimal network adapter that satisfies reth RPC builder bounds.
///
/// Every method returns a sensible empty/default value because whirlpool
/// does not maintain a peer-to-peer network layer.
#[derive(Debug, Clone)]
pub struct WhirlpoolNetwork {
    chain_id: u64,
}

impl WhirlpoolNetwork {
    /// Create a new [`WhirlpoolNetwork`] for the given chain ID.
    pub fn new(chain_id: u64) -> Self {
        Self { chain_id }
    }
}

// ---------------------------------------------------------------------------
// NetworkInfo
// ---------------------------------------------------------------------------

impl NetworkInfo for WhirlpoolNetwork {
    fn local_addr(&self) -> SocketAddr {
        LISTEN_ADDR
    }

    async fn network_status(&self) -> Result<NetworkStatus, NetworkError> {
        #[expect(deprecated)]
        Ok(NetworkStatus {
            client_version: String::from("whirlpool/0.1.0"),
            protocol_version: 5,
            eth_protocol_info: reth_network_api::EthProtocolInfo {
                network: self.chain_id,
                difficulty: None,
                genesis: Default::default(),
                config: Default::default(),
                head: Default::default(),
            },
            capabilities: vec![Capability::new_static("eth", 68)],
        })
    }

    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    fn is_syncing(&self) -> bool {
        false
    }

    fn is_initially_syncing(&self) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// PeersInfo  (super-trait of Peers)
// ---------------------------------------------------------------------------

impl PeersInfo for WhirlpoolNetwork {
    fn num_connected_peers(&self) -> usize {
        0
    }

    fn local_node_record(&self) -> NodeRecord {
        NodeRecord::new(LISTEN_ADDR, PeerId::random())
    }

    fn local_enr(&self) -> enr::Enr<SecretKey> {
        // Deterministic dummy key — never used for real cryptographic purposes.
        let key =
            SecretKey::from_slice(&[0xcd; 32]).expect("static 32-byte slice is a valid secret key");
        enr::Enr::builder()
            .build(&key)
            .expect("building an empty ENR should not fail")
    }
}

// ---------------------------------------------------------------------------
// Peers
// ---------------------------------------------------------------------------

impl Peers for WhirlpoolNetwork {
    fn add_trusted_peer_id(&self, _peer: PeerId) {}

    fn add_peer_kind(
        &self,
        _peer: PeerId,
        _kind: PeerKind,
        _tcp_addr: SocketAddr,
        _udp_addr: Option<SocketAddr>,
    ) {
    }

    async fn get_peers_by_kind(
        &self,
        _kind: PeerKind,
    ) -> Result<Vec<reth_network_api::PeerInfo>, NetworkError> {
        Ok(vec![])
    }

    async fn get_all_peers(&self) -> Result<Vec<reth_network_api::PeerInfo>, NetworkError> {
        Ok(vec![])
    }

    async fn get_peer_by_id(
        &self,
        _peer_id: PeerId,
    ) -> Result<Option<reth_network_api::PeerInfo>, NetworkError> {
        Ok(None)
    }

    async fn get_peers_by_id(
        &self,
        _peer_ids: Vec<PeerId>,
    ) -> Result<Vec<reth_network_api::PeerInfo>, NetworkError> {
        Ok(vec![])
    }

    fn remove_peer(&self, _peer: PeerId, _kind: PeerKind) {}

    fn disconnect_peer(&self, _peer: PeerId) {}

    fn disconnect_peer_with_reason(&self, _peer: PeerId, _reason: DisconnectReason) {}

    fn connect_peer_kind(
        &self,
        _peer: PeerId,
        _kind: PeerKind,
        _tcp_addr: SocketAddr,
        _udp_addr: Option<SocketAddr>,
    ) {
    }

    fn reputation_change(&self, _peer_id: PeerId, _kind: ReputationChangeKind) {}

    async fn reputation_by_id(&self, _peer_id: PeerId) -> Result<Option<Reputation>, NetworkError> {
        Ok(None)
    }
}
