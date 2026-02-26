//! Error mapping from Commonware errors to P2P errors.

use p2p::P2pError;

/// Maps an error from Commonware into our vendor-agnostic P2pError.
///
/// This function wraps any error type that implements `Display + Error + Send + Sync + 'static`
/// into a `P2pError::InvalidRecipients` variant for uniformity.
///
/// # Example
/// ```ignore
/// let cw_error = some_commonware_function().err();
/// let p2p_error = map_error(cw_error);
/// ```
pub fn map_error<E: std::fmt::Display + std::error::Error + Send + Sync + 'static>(
    err: E,
) -> P2pError {
    P2pError::InvalidRecipients(err.to_string())
}
