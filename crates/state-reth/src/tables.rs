// Re-export the reth-db-api tables used by state-reth.
//
// These tables are defined by reth-db-api and are already created
// by `init_db`. We re-export them for convenient use within this crate.
pub use reth_db_api::tables::{
    BlockBodyIndices, Bytecodes, CanonicalHeaders, HashedAccounts, HashedStorages, HeaderNumbers,
    HeaderTerminalDifficulties, Headers, PlainAccountState, PlainStorageState, Receipts,
    TransactionBlocks, TransactionHashNumbers, Transactions,
};
