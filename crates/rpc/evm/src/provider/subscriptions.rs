use super::*;

impl CanonStateSubscriptions for WhirlpoolProvider {
    fn subscribe_to_canonical_state(&self) -> CanonStateNotifications<Self::Primitives> {
        self.canon_state_tx.subscribe()
    }
}

impl PersistedBlockSubscriptions for WhirlpoolProvider {
    fn subscribe_persisted_block(&self) -> PersistedBlockNotifications {
        PersistedBlockNotifications(self.persisted_block_tx.subscribe())
    }
}

impl ForkChoiceSubscriptions for WhirlpoolProvider {
    type Header = Header;

    fn subscribe_safe_block(&self) -> ForkChoiceNotifications<Self::Header> {
        ForkChoiceNotifications(self.safe_block_tx.subscribe())
    }

    fn subscribe_finalized_block(&self) -> ForkChoiceNotifications<Self::Header> {
        ForkChoiceNotifications(self.finalized_block_tx.subscribe())
    }
}
