use crate::traits::Application;
use consensus::{traits::ConsensusApp, ConsensusError};

#[derive(Clone)]
pub struct ApplicationAdapter<A: Application> {
    inner: A,
}

impl<A: Application> ApplicationAdapter<A> {
    pub fn new(app: A) -> Self {
        Self { inner: app }
    }

    pub fn inner(&self) -> &A {
        &self.inner
    }
}

#[allow(clippy::manual_async_fn)]
impl<A> ConsensusApp for ApplicationAdapter<A>
where
    A: Application,
{
    type Block = A::Block;

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
    use consensus::{
        traits::{Block as CoreBlock, ConsensusApp},
        ConsensusError,
    };
    use futures::executor::block_on;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockBlock {
        id: [u8; 32],
        parent_id: [u8; 32],
        height: u64,
    }

    impl CoreBlock for MockBlock {
        type Id = [u8; 32];

        fn id(&self) -> Self::Id {
            self.id
        }

        fn parent_id(&self) -> Self::Id {
            self.parent_id
        }

        fn height(&self) -> u64 {
            self.height
        }
    }

    #[derive(Clone)]
    struct MockApplication {
        genesis: MockBlock,
        should_verify_fail: bool,
    }

    impl MockApplication {
        fn new(genesis: MockBlock) -> Self {
            Self {
                genesis,
                should_verify_fail: false,
            }
        }

        fn with_verify_failure(genesis: MockBlock) -> Self {
            Self {
                genesis,
                should_verify_fail: true,
            }
        }
    }

    impl Application for MockApplication {
        type Block = MockBlock;
        type Result = ();
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

            async move { Ok((block, ())) }
        }

        fn verify(
            &self,
            _parent: &Self::Block,
            _block: &Self::Block,
        ) -> impl std::future::Future<Output = Result<Self::Result, Self::Error>> + Send {
            let should_fail = self.should_verify_fail;

            async move {
                if should_fail {
                    Err(ApplicationError::Verification(
                        "mock verification failed".to_string(),
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }

    fn sample_genesis() -> MockBlock {
        MockBlock {
            id: [9u8; 32],
            parent_id: [0u8; 32],
            height: 0,
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
        assert_eq!(CoreBlock::id(&actual), CoreBlock::id(&expected));
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
