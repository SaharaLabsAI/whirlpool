mod decode;
mod header;

use reth_ethereum_primitives::TransactionSigned;
use reth_primitives_traits::Recovered;

pub type RecoveredTx = Recovered<TransactionSigned>;

pub use decode::{decode_evm_transaction, decode_evm_transactions};
pub use header::build_header_from_evm_block;
