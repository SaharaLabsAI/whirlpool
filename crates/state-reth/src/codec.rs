// Codec helpers for converting between reth-db types and our domain types.
//
// reth-db stores `Account` (nonce + balance + bytecode_hash) but our StateDb
// uses revm `AccountInfo` (nonce + balance + code_hash + code). This module
// bridges the two representations.

use revm::primitives::KECCAK_EMPTY;
use revm::state::AccountInfo;

/// Convert a reth-db `Account` to a revm `AccountInfo`.
///
/// Note: The `code` field is NOT populated here — callers must look up
/// the bytecode from the `Bytecodes` table separately.
pub fn account_to_info(account: &reth_primitives_traits::Account) -> AccountInfo {
    AccountInfo {
        balance: account.balance,
        nonce: account.nonce,
        code_hash: account.get_bytecode_hash(),
        code: None, // Must be loaded from Bytecodes table
        account_id: None,
    }
}

/// Convert a revm `AccountInfo` to a reth-db `Account`.
pub fn info_to_account(info: &AccountInfo) -> reth_primitives_traits::Account {
    reth_primitives_traits::Account {
        nonce: info.nonce,
        balance: info.balance,
        bytecode_hash: if info.code_hash == KECCAK_EMPTY {
            None
        } else {
            Some(info.code_hash)
        },
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{b256, U256};
    use revm::primitives::KECCAK_EMPTY;
    use revm::state::AccountInfo;

    use crate::codec::{account_to_info, info_to_account};

    #[test]
    fn test_account_roundtrip() {
        let info = AccountInfo {
            balance: U256::from(123_456u64),
            nonce: 42,
            code_hash: b256!("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
            code: None,
            account_id: None,
        };

        let reth_account = info_to_account(&info);
        let roundtrip = account_to_info(&reth_account);

        assert_eq!(roundtrip.balance, info.balance);
        assert_eq!(roundtrip.nonce, info.nonce);
        assert_eq!(roundtrip.code_hash, info.code_hash);
        assert_eq!(roundtrip.code, None);
        assert_eq!(roundtrip.account_id, None);
    }

    #[test]
    fn test_account_empty_code() {
        let info = AccountInfo {
            balance: U256::from(1u64),
            nonce: 1,
            code_hash: KECCAK_EMPTY,
            code: None,
            account_id: None,
        };

        let reth_account = info_to_account(&info);
        assert_eq!(reth_account.bytecode_hash, None);

        let roundtrip = account_to_info(&reth_account);
        assert_eq!(roundtrip.code_hash, KECCAK_EMPTY);
        assert_eq!(roundtrip.code, None);
        assert_eq!(roundtrip.account_id, None);
    }
}
