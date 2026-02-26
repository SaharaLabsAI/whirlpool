//! Commonware P2P bridge crate.
//!
//! This crate provides adapter types that bridge our vendor-agnostic `p2p` trait system
//! to the Commonware P2P implementation.
pub mod provider;


mod peer_id;
mod error;

#[cfg(test)]
mod tests;

pub use peer_id::CommonwarePeerId;
pub use error::{map_send_error, map_recv_error};

pub mod sender;
pub mod receiver;
pub use sender::CommonwareSender;
pub use receiver::CommonwareReceiver;
pub use provider::CommonwareNetworkProvider;

// TODO: Implement in Task 3
pub struct MultiplexSender<S> {
    _phantom: std::marker::PhantomData<S>,
}

// TODO: Implement in Task 4  
pub struct MultiplexReceiver<R> {
    _phantom: std::marker::PhantomData<R>,
}
