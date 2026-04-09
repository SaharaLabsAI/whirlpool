//! Error mapping from Commonware errors to P2P errors.

use network::P2pError;

/// Maps a send error from Commonware into a P2pError::SendFailed variant.
///
/// This function wraps any error type that implements `Display`
/// into a `P2pError::SendFailed` variant for send operations.
///
/// # Example
/// ```ignore
/// let cw_error = some_commonware_function().err();
/// let p2p_error = map_send_error(cw_error);
/// ```
pub fn map_send_error<E: std::fmt::Display>(e: E) -> P2pError {
    P2pError::SendFailed(e.to_string())
}

/// Maps a receive error from Commonware into a P2pError::ReceiveFailed variant.
///
/// This function wraps any error type that implements `Display`
/// into a `P2pError::ReceiveFailed` variant for receive operations.
///
/// # Example
/// ```ignore
/// let cw_error = some_commonware_function().err();
/// let p2p_error = map_recv_error(cw_error);
/// ```
pub fn map_recv_error<E: std::fmt::Display>(e: E) -> P2pError {
    P2pError::ReceiveFailed(e.to_string())
}
