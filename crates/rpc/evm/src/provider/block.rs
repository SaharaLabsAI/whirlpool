use super::*;

impl NodePrimitivesProvider for WhirlpoolProvider {
    type Primitives = EthPrimitives;
}

impl BlockHashReader for WhirlpoolProvider {
    fn block_hash(&self, number: u64) -> ProviderResult<Option<B256>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        tx.get::<CanonicalHeaders>(number).map_err(map_db_err)
    }

    fn canonical_hashes_range(
        &self,
        start: BlockNumber,
        end: BlockNumber,
    ) -> ProviderResult<Vec<B256>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let mut hashes = Vec::new();
        for number in start..end {
            if let Some(hash) = tx.get::<CanonicalHeaders>(number).map_err(map_db_err)? {
                hashes.push(hash);
            }
        }
        Ok(hashes)
    }
}

impl BlockNumReader for WhirlpoolProvider {
    fn chain_info(&self) -> ProviderResult<ChainInfo> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let best_number = tx
            .cursor_read::<CanonicalHeaders>()
            .map_err(map_db_err)?
            .last()
            .map_err(map_db_err)?
            .map(|(number, _)| number)
            .unwrap_or(0);
        let best_hash = tx
            .get::<CanonicalHeaders>(best_number)
            .map_err(map_db_err)?
            .unwrap_or_default();
        Ok(ChainInfo {
            best_hash,
            best_number,
        })
    }

    fn best_block_number(&self) -> ProviderResult<BlockNumber> {
        Ok(self.chain_info()?.best_number)
    }

    fn last_block_number(&self) -> ProviderResult<BlockNumber> {
        self.best_block_number()
    }

    fn block_number(&self, hash: B256) -> ProviderResult<Option<BlockNumber>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        tx.get::<HeaderNumbers>(hash).map_err(map_db_err)
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
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let Some(number) = tx.get::<HeaderNumbers>(block_hash).map_err(map_db_err)? else {
            return Ok(None);
        };
        tx.get::<Headers>(number).map_err(map_db_err)
    }

    fn header_by_number(&self, num: u64) -> ProviderResult<Option<Self::Header>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        tx.get::<Headers>(num).map_err(map_db_err)
    }

    fn headers_range(
        &self,
        range: impl RangeBounds<BlockNumber>,
    ) -> ProviderResult<Vec<Self::Header>> {
        let (start, end) = range_to_exclusive_bounds(range);
        if start >= end {
            return Ok(Vec::new());
        }

        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let mut headers = Vec::new();
        for number in start..end {
            if let Some(header) = tx.get::<Headers>(number).map_err(map_db_err)? {
                headers.push(header);
            }
        }
        Ok(headers)
    }

    fn sealed_header(
        &self,
        number: BlockNumber,
    ) -> ProviderResult<Option<SealedHeader<Self::Header>>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let Some(header) = tx.get::<Headers>(number).map_err(map_db_err)? else {
            return Ok(None);
        };
        let Some(hash) = tx.get::<CanonicalHeaders>(number).map_err(map_db_err)? else {
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

        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let mut headers = Vec::new();
        for number in start..end {
            let Some(header) = tx.get::<Headers>(number).map_err(map_db_err)? else {
                continue;
            };
            let Some(hash) = tx.get::<CanonicalHeaders>(number).map_err(map_db_err)? else {
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
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let Some(number) = tx.get::<HeaderNumbers>(hash).map_err(map_db_err)? else {
            return Ok(None);
        };
        self.block(BlockHashOrNumber::Number(number))
    }

    fn block(&self, id: BlockHashOrNumber) -> ProviderResult<Option<Self::Block>> {
        let number = match id {
            BlockHashOrNumber::Hash(hash) => {
                let tx = self.state_db.inner().tx().map_err(map_db_err)?;
                tx.get::<HeaderNumbers>(hash).map_err(map_db_err)?
            }
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
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let mut cursor = tx.cursor_read::<TransactionBlocks>().map_err(map_db_err)?;
        let entry = cursor.seek(id).map_err(map_db_err)?;
        Ok(entry.map(|(_, block_number)| block_number))
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
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        tx.get::<BlockBodyIndices>(num).map_err(map_db_err)
    }

    fn block_body_indices_range(
        &self,
        range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<Vec<StoredBlockBodyIndices>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let mut indices = Vec::new();
        for number in range {
            if let Some(body_indices) = tx.get::<BlockBodyIndices>(number).map_err(map_db_err)? {
                indices.push(body_indices);
            }
        }
        Ok(indices)
    }
}
