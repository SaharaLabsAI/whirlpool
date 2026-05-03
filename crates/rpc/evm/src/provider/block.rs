use std::ops::{RangeBounds, RangeInclusive};

use alloy_consensus::Header;
use alloy_eips::{BlockHashOrNumber, BlockId};
use alloy_primitives::{BlockHash, BlockNumber, TxNumber, B256};
use reth_chainspec::ChainInfo;
use reth_db_api::models::StoredBlockBodyIndices;
use reth_ethereum_primitives::{Block, EthPrimitives};
use reth_primitives_traits::{RecoveredBlock, SealedHeader};
use reth_storage_api::{
    BlockBodyIndicesProvider, BlockHashReader, BlockIdReader, BlockNumReader, BlockReader,
    BlockReaderIdExt, BlockSource, HeaderProvider, NodePrimitivesProvider, TransactionVariant,
};
use reth_storage_errors::provider::ProviderResult;

use crate::provider_impl::{map_db_err, range_to_exclusive_bounds, WhirlpoolProvider};

impl NodePrimitivesProvider for WhirlpoolProvider {
    type Primitives = EthPrimitives;
}

impl BlockHashReader for WhirlpoolProvider {
    fn block_hash(&self, number: u64) -> ProviderResult<Option<B256>> {
        self.state_db
            .rpc_reader()
            .blocks()
            .canonical()
            .block_hash(number)
            .map_err(map_db_err)
    }

    fn canonical_hashes_range(
        &self,
        start: BlockNumber,
        end: BlockNumber,
    ) -> ProviderResult<Vec<B256>> {
        self.state_db
            .rpc_reader()
            .blocks()
            .canonical()
            .canonical_hashes_range(start, end)
            .map_err(map_db_err)
    }
}

impl BlockNumReader for WhirlpoolProvider {
    fn chain_info(&self) -> ProviderResult<ChainInfo> {
        let tip = self
            .state_db
            .rpc_reader()
            .blocks()
            .canonical()
            .canonical_tip()
            .map_err(map_db_err)?;
        Ok(match tip {
            Some(tip) => ChainInfo {
                best_hash: tip.best_hash,
                best_number: tip.best_number,
            },
            None => ChainInfo {
                best_hash: B256::default(),
                best_number: 0,
            },
        })
    }

    fn best_block_number(&self) -> ProviderResult<BlockNumber> {
        Ok(self.chain_info()?.best_number)
    }

    fn last_block_number(&self) -> ProviderResult<BlockNumber> {
        self.best_block_number()
    }

    fn block_number(&self, hash: B256) -> ProviderResult<Option<BlockNumber>> {
        self.state_db
            .rpc_reader()
            .blocks()
            .headers()
            .lookup()
            .block_number(hash)
            .map_err(map_db_err)
    }
}

impl BlockIdReader for WhirlpoolProvider {
    fn pending_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        Ok(None)
    }

    fn safe_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        Ok(None)
    }

    fn finalized_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        Ok(None)
    }
}

impl HeaderProvider for WhirlpoolProvider {
    type Header = Header;

    fn header(&self, block_hash: BlockHash) -> ProviderResult<Option<Self::Header>> {
        self.state_db
            .rpc_reader()
            .blocks()
            .headers()
            .lookup()
            .header_by_hash(block_hash)
            .map_err(map_db_err)
    }

    fn header_by_number(&self, num: u64) -> ProviderResult<Option<Self::Header>> {
        self.state_db
            .rpc_reader()
            .blocks()
            .headers()
            .lookup()
            .header_by_number(num)
            .map_err(map_db_err)
    }

    fn headers_range(
        &self,
        range: impl RangeBounds<BlockNumber>,
    ) -> ProviderResult<Vec<Self::Header>> {
        let (start, end) = range_to_exclusive_bounds(range);
        if start >= end {
            return Ok(Vec::new());
        }

        self.state_db
            .rpc_reader()
            .blocks()
            .headers()
            .ranges()
            .headers_range(start, end)
            .map_err(map_db_err)
    }

    fn sealed_header(
        &self,
        number: BlockNumber,
    ) -> ProviderResult<Option<SealedHeader<Self::Header>>> {
        let Some((header, hash)) = self
            .state_db
            .rpc_reader()
            .blocks()
            .headers()
            .ranges()
            .header_with_hash(number)
            .map_err(map_db_err)?
        else {
            return Ok(None);
        };
        Ok(Some(SealedHeader::new(header, hash)))
    }

    fn sealed_headers_while(
        &self,
        range: impl RangeBounds<BlockNumber>,
        mut predicate: impl FnMut(&SealedHeader<Self::Header>) -> bool,
    ) -> ProviderResult<Vec<SealedHeader<Self::Header>>> {
        let (start, end) = range_to_exclusive_bounds(range);
        if start >= end {
            return Ok(Vec::new());
        }

        let reader = self.state_db.rpc_reader();
        let mut headers = Vec::new();
        for number in start..end {
            let Some((header, hash)) = reader
                .blocks()
                .headers()
                .ranges()
                .header_with_hash(number)
                .map_err(map_db_err)?
            else {
                continue;
            };
            let sealed = SealedHeader::new(header, hash);
            if !predicate(&sealed) {
                break;
            }
            headers.push(sealed);
        }
        Ok(headers)
    }
}

impl BlockReader for WhirlpoolProvider {
    type Block = Block;

    fn find_block_by_hash(
        &self,
        hash: B256,
        _source: BlockSource,
    ) -> ProviderResult<Option<Self::Block>> {
        let Some(number) = self
            .state_db
            .rpc_reader()
            .blocks()
            .headers()
            .lookup()
            .block_number(hash)
            .map_err(map_db_err)?
        else {
            return Ok(None);
        };
        self.block(BlockHashOrNumber::Number(number))
    }

    fn block(&self, id: BlockHashOrNumber) -> ProviderResult<Option<Self::Block>> {
        let number = match id {
            BlockHashOrNumber::Hash(hash) => self
                .state_db
                .rpc_reader()
                .blocks()
                .headers()
                .lookup()
                .block_number(hash)
                .map_err(map_db_err)?,
            BlockHashOrNumber::Number(number) => Some(number),
        };

        let Some(number) = number else {
            return Ok(None);
        };
        self.read_block_by_number(number)
    }

    fn pending_block(&self) -> ProviderResult<Option<RecoveredBlock<Self::Block>>> {
        Ok(None)
    }

    fn pending_block_and_receipts(
        &self,
    ) -> ProviderResult<Option<(RecoveredBlock<Self::Block>, Vec<Self::Receipt>)>> {
        Ok(None)
    }

    fn recovered_block(
        &self,
        id: BlockHashOrNumber,
        _transaction_kind: TransactionVariant,
    ) -> ProviderResult<Option<RecoveredBlock<Self::Block>>> {
        let Some(number) = self.convert_hash_or_number(id)? else {
            return Ok(None);
        };
        self.recovered_block_by_number(number, false)
    }

    fn sealed_block_with_senders(
        &self,
        id: BlockHashOrNumber,
        _transaction_kind: TransactionVariant,
    ) -> ProviderResult<Option<RecoveredBlock<Self::Block>>> {
        let Some(number) = self.convert_hash_or_number(id)? else {
            return Ok(None);
        };
        self.recovered_block_by_number(number, true)
    }

    fn block_range(&self, range: RangeInclusive<BlockNumber>) -> ProviderResult<Vec<Self::Block>> {
        let mut blocks = Vec::new();
        for number in range {
            if let Some(block) = self.read_block_by_number(number)? {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    fn block_with_senders_range(
        &self,
        range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<Vec<RecoveredBlock<Self::Block>>> {
        let mut blocks = Vec::new();
        for number in range {
            if let Some(block) = self.recovered_block_by_number(number, false)? {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    fn recovered_block_range(
        &self,
        range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<Vec<RecoveredBlock<Self::Block>>> {
        let mut blocks = Vec::new();
        for number in range {
            if let Some(block) = self.recovered_block_by_number(number, true)? {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    fn block_by_transaction_id(&self, id: TxNumber) -> ProviderResult<Option<BlockNumber>> {
        self.state_db
            .rpc_reader()
            .transactions()
            .meta()
            .block_number_by_transaction_id(id)
            .map_err(map_db_err)
    }
}

impl BlockReaderIdExt for WhirlpoolProvider {
    fn block_by_id(&self, id: BlockId) -> ProviderResult<Option<Self::Block>> {
        match self.block_number_for_id(id)? {
            Some(number) => self.read_block_by_number(number),
            None => Ok(None),
        }
    }

    fn sealed_header_by_id(
        &self,
        id: BlockId,
    ) -> ProviderResult<Option<SealedHeader<Self::Header>>> {
        match self.block_number_for_id(id)? {
            Some(number) => self.sealed_header(number),
            None => Ok(None),
        }
    }

    fn header_by_id(&self, id: BlockId) -> ProviderResult<Option<Self::Header>> {
        match self.block_number_for_id(id)? {
            Some(number) => self.header_by_number(number),
            None => Ok(None),
        }
    }
}

impl BlockBodyIndicesProvider for WhirlpoolProvider {
    fn block_body_indices(&self, num: u64) -> ProviderResult<Option<StoredBlockBodyIndices>> {
        self.state_db
            .rpc_reader()
            .blocks()
            .bodies()
            .block_body_indices(num)
            .map(|indices| indices.map(Into::into))
            .map_err(map_db_err)
    }

    fn block_body_indices_range(
        &self,
        range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<Vec<StoredBlockBodyIndices>> {
        let start = *range.start();
        let end = *range.end();
        self.state_db
            .rpc_reader()
            .blocks()
            .bodies()
            .block_body_indices_range(start, end)
            .map(|indices| indices.into_iter().map(Into::into).collect())
            .map_err(map_db_err)
    }
}
