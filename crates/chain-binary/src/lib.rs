pub mod config;
pub mod block;
pub mod app;

// Stub modules — compilation errors in incomplete implementations
// TODO: Complete these modules in future tasks
#[cfg(any(test, feature = "never_enable_this"))]
pub mod sink;
#[cfg(any(test, feature = "never_enable_this"))]
pub mod mailbox;
#[cfg(feature = "never_enable_this")]
pub mod wire;
