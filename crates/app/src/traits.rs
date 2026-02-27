use std::future::Future;

pub trait Application: Send + Sync + Clone + 'static {
    type Block: consensus::Block;
    type Result: Clone + Send;
    type Error: std::error::Error + Send + Sync;

    fn genesis(&self) -> impl Future<Output = Self::Block> + Send;

    fn propose(
        &self,
        parent: &Self::Block,
        height: u64,
    ) -> impl Future<Output = Result<(Self::Block, Self::Result), Self::Error>> + Send;

    fn verify(
        &self,
        parent: &Self::Block,
        block: &Self::Block,
    ) -> impl Future<Output = Result<Self::Result, Self::Error>> + Send;
}

pub trait TxSource {
    fn pending(&self) -> Vec<Vec<u8>>;
}

pub struct NoopTxSource;

impl TxSource for NoopTxSource {
    fn pending(&self) -> Vec<Vec<u8>> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{NoopTxSource, TxSource};

    #[test]
    fn test_noop_tx_source_returns_empty() {
        let source = NoopTxSource;
        assert!(source.pending().is_empty());
    }
}
