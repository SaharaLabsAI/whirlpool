# Marshal networking channels

Marshal (payload) networking needs **two** additional P2P channels beyond the three Simplex
consensus channels.

These channels are marshal-specific and are not part of the consensus-agnostic `core` network
boundary.

```rust
// Pseudocode — constants and wiring shape.

// Payload broadcast lane (high bandwidth, best-effort).
//
// Used by a broadcast/buffer engine to disseminate block bytes to peers.
const BROADCASTER_CHANNEL: core::network::ChannelId = core::network::ChannelId(3);

// Payload backfill lane (request/response).
//
// Used by the marshal resolver to fetch missing payload bytes by commitment.
const MARSHAL_CHANNEL: core::network::ChannelId = core::network::ChannelId(4);

pub struct MarshalNetworkConfig {
  pub broadcast: core::network::ChannelConfig,
  pub backfill: core::network::ChannelConfig,
}

pub struct MarshalResolverConfig<P, M, B> {
  pub public_key: P,
  pub manager: M,
  pub blocker: B,
  pub mailbox_size: usize,
  pub initial: Duration,
  pub timeout: Duration,
  pub fetch_retry_timeout: Duration,
  pub priority_requests: bool,
  pub priority_responses: bool,
}

fn build_marshal_networking<Net: core::network::Network>(
  network: &mut Net,
  net_cfg: MarshalNetworkConfig,
  resolver_cfg: MarshalResolverConfig<Net::PeerId, impl commonware_p2p::Manager<PublicKey = Net::PeerId>, impl commonware_p2p::Blocker<PublicKey = Net::PeerId>>,
) -> (
  // Passed to `buffer.start(...)`.
  (Net::Sender, Net::Receiver),
  // Passed to `marshal.start(...)`.
  (tokio::sync::mpsc::Receiver<commonware_consensus::marshal::ingress::handler::Message<Block>>,
   commonware_consensus::marshal::resolver::p2p::Mailbox<
     commonware_consensus::marshal::ingress::handler::Request<Block>,
     Net::PeerId,
   >),
) {
  // 1) Broadcast channel pair.
  let broadcast = network.register_channel(BROADCASTER_CHANNEL, net_cfg.broadcast);

  // 2) Backfill channel pair used by the marshal resolver.
  let backfill = network.register_channel(MARSHAL_CHANNEL, net_cfg.backfill);

  let (ingress_rx, resolver_mailbox) = commonware_consensus::marshal::resolver::p2p::init(
    /* ctx */,
    commonware_consensus::marshal::resolver::p2p::Config {
      public_key: resolver_cfg.public_key,
      manager: resolver_cfg.manager,
      blocker: resolver_cfg.blocker,
      mailbox_size: resolver_cfg.mailbox_size,
      initial: resolver_cfg.initial,
      timeout: resolver_cfg.timeout,
      fetch_retry_timeout: resolver_cfg.fetch_retry_timeout,
      priority_requests: resolver_cfg.priority_requests,
      priority_responses: resolver_cfg.priority_responses,
    },
    backfill,
  );

  (broadcast, (ingress_rx, resolver_mailbox))
}
```

Notes:

- These channel IDs are placeholders aligned to Alto wiring.
- The exact message types for marshal ingress/requests live in commonware marshal modules.
