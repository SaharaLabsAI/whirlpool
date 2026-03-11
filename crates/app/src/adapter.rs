use crate::traits::Application;
use crate::types::EvmBlock;
use consensus::{traits::ConsensusApp, ConsensusError};

#[derive(Clone)]
pub struct ApplicationAdapter<A: Application<Block = EvmBlock>> {
    inner: A,
}

impl<A: Application<Block = EvmBlock>> ApplicationAdapter<A> {
    pub fn new(app: A) -> Self {
        Self { inner: app }
    }

    pub fn inner(&self) -> &A {
        &self.inner
    }
}

impl<A> ConsensusApp for ApplicationAdapter<A>
where
    A: Application<Block = EvmBlock>,
{
    type Block = EvmBlock;

    fn genesis(&self) -> impl std::future::Future<Output = Self::Block> + Send {
        self.inner.genesis()
    }

    fn propose(
        &self,
        parent: &Self::Block,
        height: u64,
    ) -> impl std::future::Future<Output = Option<Self::Block>> + Send {
        async move {
            match self.inner.propose(parent, height).await {
                Ok((block, _)) => Some(block),
                Err(_) => None,
            }
        }
    }

    fn verify(
        &self,
        parent: &Self::Block,
        block: &Self::Block,
    ) -> impl std::future::Future<Output = Result<(), ConsensusError>> + Send {
        async move {
            match self.inner.verify(parent, block).await {
                Ok(_) => Ok(()),
                Err(err) => Err(ConsensusError::InvalidBlock(err.to_string())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ApplicationAdapter;
    use crate::error::ApplicationError;
    use crate::traits::Application;
    use crate::types::{EvmBlock, ExecutionResult};
    use consensus::{
        traits::{Block as CoreBlock, ConsensusApp},
        ConsensusError,
    };
    use futures::executor::block_on;

    #[derive(Clone)]
    struct MockApplication {
        genesis: EvmBlock,
        should_verify_fail: bool,
    }

    impl MockApplication {
        fn new(genesis: EvmBlock) -> Self {
            Self {
                genesis,
                should_verify_fail: false,
            }
        }

        fn with_verify_failure(genesis: EvmBlock) -> Self {
            Self {
                genesis,
                should_verify_fail: true,
            }
        }
    }

    impl Application for MockApplication {
        type Block = EvmBlock;
        type Result = ExecutionResult;
        type Error = ApplicationError;

        fn genesis(&self) -> impl std::future::Future<Output = Self::Block> + Send {
            let block = self.genesis.clone();
            async move { block }
        }

        fn propose(
            &self,
            _parent: &Self::Block,
            height: u64,
        ) -> impl std::future::Future<Output = Result<(Self::Block, Self::Result), Self::Error>> + Send
        {
            let mut block = self.genesis.clone();
            block.height = height;

            let result = ExecutionResult {
                state_root: block.state_root,
                receipts_root: block.receipts_root,
                gas_used: block.gas_used,
                receipt_count: block.transactions.len(),
            };

            async move { Ok((block, result)) }
        }

        fn verify(
            &self,
            _parent: &Self::Block,
            block: &Self::Block,
        ) -> impl std::future::Future<Output = Result<Self::Result, Self::Error>> + Send {
            let should_fail = self.should_verify_fail;
            let result = ExecutionResult {
                state_root: block.state_root,
                receipts_root: block.receipts_root,
                gas_used: block.gas_used,
                receipt_count: block.transactions.len(),
            };

            async move {
                if should_fail {
                    Err(ApplicationError::Verification(
                        "mock verification failed".to_string(),
                    ))
                } else {
                    Ok(result)
                }
            }
        }
    }

    fn sample_genesis() -> EvmBlock {
        EvmBlock {
            height: 0,
            parent_id: [0u8; 32],
            state_root: [1u8; 32],
            transactions_root: [2u8; 32],
            receipts_root: [3u8; 32],
            gas_used: 0,
            base_fee_per_gas: 1_000_000_000,
            timestamp: 0,
            transactions: vec![],
        }
    }

    #[test]
    fn test_adapter_wrapping() {
        let app = MockApplication::new(sample_genesis());
        let adapter = ApplicationAdapter::new(app);
        let _ = adapter.inner();
    }

    #[test]
    fn test_adapter_genesis_passthrough() {
        let expected = sample_genesis();
        let app = MockApplication::new(expected.clone());
        let adapter = ApplicationAdapter::new(app);

        let actual = block_on(adapter.genesis());
        assert_eq!(CoreBlock::height(&actual), 0);
        assert_eq!(CoreBlock::parent_id(&actual), [0u8; 32]);
        assert_eq!(actual.state_root, expected.state_root);
    }

    #[test]
    fn test_adapter_verify_maps_error_to_invalid_block() {
        let genesis = sample_genesis();
        let app = MockApplication::with_verify_failure(genesis.clone());
        let adapter = ApplicationAdapter::new(app);

        let result = block_on(adapter.verify(&genesis, &genesis));
        assert!(matches!(result, Err(ConsensusError::InvalidBlock(_))));
    }
}
