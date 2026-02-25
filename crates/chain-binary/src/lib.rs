pub mod app;
pub mod block;
pub mod config;

// Stub modules — compilation errors in incomplete implementations
// TODO: Complete these modules in future tasks
#[cfg(any(test, feature = "never_enable_this"))]
pub mod mailbox;
#[cfg(any(test, feature = "never_enable_this"))]
pub mod sink;
pub mod wire;
