// Mailbox Bridge — Automaton/CertifiableAutomaton/Relay traits for simplex engine
//
// This module bridges the gap between ConsensusApp and the simplex consensus engine.
// The simplex engine requires Automaton/Relay traits, but ConsensusApp doesn't provide them.
// Mailbox implements these traits and delegates to an actor that handles the actual work.
//
// The Relay::broadcast implementation looks up the proposed block by digest in the shared
// BlockStore, encodes it as a PayloadRelayMessage (digest ++ encoded-block), and forwards the
// bytes through an mpsc channel.  The receiving end (spawned in engine.rs) pushes them onto the
// PAYLOAD P2P channel so remote validators can obtain the block before voting.

use bytes::Bytes;
use commonware_actor::Feedback;
use commonware_codec::Encode;
use commonware_consensus::simplex::{types::Context, Plan};
use commonware_consensus::{Automaton, CertifiableAutomaton, Relay};
use commonware_cryptography::ed25519::PublicKey;
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::Digestible;
use commonware_utils::channel::oneshot;
use futures::channel::mpsc;
use futures::SinkExt;

use crate::BlockStore;

// Message types for actor channel
pub enum Message {
    Propose {
        context: Context<Digest, PublicKey>,
        response: oneshot::Sender<Digest>,
    },
    Verify {
        context: Context<Digest, PublicKey>,
        digest: Digest,
        response: oneshot::Sender<bool>,
    },
}

/// Mailbox implements Automaton/Relay traits for simplex consensus engine.
///
/// When constructed with [`Mailbox::with_relay`], the [`Relay::broadcast`]
/// implementation encodes the proposed block as a [`PayloadRelayMessage`]
/// and pushes it through an mpsc channel.  The consuming end (wired in
/// `engine.rs`) forwards the bytes onto the PAYLOAD P2P channel so that
/// remote validators receive the full block payload before voting.
///
/// When constructed with [`Mailbox::new`] (no relay wiring), broadcast
/// remains a silent no-op for backward-compatible single-node setups.
#[derive(Clone)]
pub struct Mailbox<B> {
    sender: mpsc::Sender<Message>,
    /// Shared block store — used by `broadcast` to look up the full block
    /// payload that corresponds to the digest the vendor engine provides.
    block_store: Option<BlockStore<B>>,
    /// Channel for outbound payload relay messages.  `None` ⇒ no-op relay.
    payload_tx: Option<mpsc::UnboundedSender<Bytes>>,
}

impl<B> Mailbox<B> {
    /// Create a mailbox **without** relay capability (broadcast is a no-op).
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self {
            sender,
            block_store: None,
            payload_tx: None,
        }
    }

    /// Create a mailbox **with** relay capability.
    ///
    /// `block_store` is shared with `MailboxActor` so that the relay can look
    /// up the full block for a given digest.  `payload_tx` is the sender-half
    /// of an unbounded channel whose receiver is consumed by a forwarding task
    /// in `engine.rs`.
    pub fn with_relay(
        sender: mpsc::Sender<Message>,
        block_store: BlockStore<B>,
        payload_tx: mpsc::UnboundedSender<Bytes>,
    ) -> Self {
        Self {
            sender,
            block_store: Some(block_store),
            payload_tx: Some(payload_tx),
        }
    }
}

// Implement Automaton trait (async methods matching vendor pattern)
impl<B: Clone + Send + Sync + 'static> Automaton for Mailbox<B> {
    type Context = Context<Digest, PublicKey>;
    type Digest = Digest;

    async fn propose(&mut self, context: Self::Context) -> oneshot::Receiver<Self::Digest> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(Message::Propose { context, response })
            .await
            .expect("Failed to send propose");
        receiver
    }

    async fn verify(
        &mut self,
        context: Self::Context,
        digest: Self::Digest,
    ) -> oneshot::Receiver<bool> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(Message::Verify {
                context,
                digest,
                response,
            })
            .await
            .expect("Failed to send verify");
        receiver
    }
}

// Implement CertifiableAutomaton trait (uses default certify)
impl<B: Clone + Send + Sync + 'static> CertifiableAutomaton for Mailbox<B> {}

// Implement Relay trait — broadcast block payloads to peers via PAYLOAD channel
impl<B: Clone + Encode + Send + Sync + 'static> Relay for Mailbox<B>
where
    B: Digestible<Digest = Digest>,
{
    type Digest = Digest;
    type PublicKey = PublicKey;
    type Plan = Plan<Self::PublicKey>;

    fn broadcast(&mut self, digest: Self::Digest, _plan: Self::Plan) -> Feedback {
        let (Some(ref block_store), Some(ref payload_tx)) = (&self.block_store, &self.payload_tx)
        else {
            // No relay wiring — silent no-op (single-node mode).
            return Feedback::Ok;
        };

        // Look up the full block by its digest. The lookup is synchronous
        // (Relay::broadcast no longer returns a future); skip if the shared
        // store is momentarily held by a writer.
        let Ok(guard) = block_store.try_read() else {
            tracing::debug!(
                ?digest,
                "relay broadcast: block store busy, skipping payload relay"
            );
            return Feedback::Backoff;
        };
        let block = guard.get(&digest).cloned();
        drop(guard);

        let Some(block) = block else {
            tracing::warn!(
                ?digest,
                "relay broadcast: digest not found in block store, skipping"
            );
            return Feedback::Ok;
        };

        // Encode as PayloadRelayMessage: [32-byte digest][encoded block]
        let msg = PayloadRelayMessage::new(digest, block.encode());
        let wire = msg.encode_wire();

        if let Err(e) = payload_tx.unbounded_send(wire) {
            tracing::warn!(
                ?digest,
                error = %e,
                "relay broadcast: failed to enqueue payload message"
            );
            return Feedback::Backoff;
        }
        Feedback::Ok
    }
}

mod actor;
mod payload;
#[cfg(test)]
mod tests;

pub use actor::MailboxActor;
pub use payload::{PayloadRelayMessage, DIGEST_SIZE};

// Helper functions
fn compute_digest<B>(block: &B) -> Digest
where
    B: Digestible<Digest = Digest>,
{
    block.digest()
}

#[allow(dead_code)]
fn digest_to_block_id(digest: Digest) -> [u8; 32] {
    let bytes: &[u8] = digest.as_ref();
    let mut id = [0u8; 32];
    id.copy_from_slice(bytes);
    id
}
