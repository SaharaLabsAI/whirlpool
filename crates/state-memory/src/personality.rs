use std::sync::RwLock;

use state::{
    PersonalityBySignerNonce, PersonalityLatestById, PersonalityStorage, StoredPersonality,
};

#[derive(Debug, thiserror::Error)]
pub enum InMemoryPersonalityStorageError {
    #[error("internal personality storage error: {0}")]
    Internal(String),
}

#[derive(Debug, Default)]
pub struct InMemoryPersonalityStorage {
    by_personality_id: RwLock<PersonalityLatestById>,
    by_signer_nonce: RwLock<PersonalityBySignerNonce>,
}

impl InMemoryPersonalityStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PersonalityStorage for InMemoryPersonalityStorage {
    type Error = InMemoryPersonalityStorageError;

    fn put(&self, entry: StoredPersonality) -> Result<(), Self::Error> {
        let personality_id = entry.personality_id.clone();
        let signer_nonce = (entry.signer.clone(), entry.nonce);

        let mut latest = self
            .by_personality_id
            .write()
            .map_err(|_| InMemoryPersonalityStorageError::Internal("latest map poisoned".into()))?;
        let mut by_nonce = self
            .by_signer_nonce
            .write()
            .map_err(|_| InMemoryPersonalityStorageError::Internal("signer nonce map poisoned".into()))?;

        latest.insert(personality_id, entry.clone());
        by_nonce.insert(signer_nonce, entry);
        Ok(())
    }

    fn get_latest(&self, personality_id: &[u8]) -> Result<Option<StoredPersonality>, Self::Error> {
        let latest = self
            .by_personality_id
            .read()
            .map_err(|_| InMemoryPersonalityStorageError::Internal("latest map poisoned".into()))?;
        Ok(latest.get(personality_id).cloned())
    }

    fn get_by_signer_nonce(
        &self,
        signer: &[u8],
        nonce: u64,
    ) -> Result<Option<StoredPersonality>, Self::Error> {
        let by_nonce = self
            .by_signer_nonce
            .read()
            .map_err(|_| InMemoryPersonalityStorageError::Internal("signer nonce map poisoned".into()))?;
        Ok(by_nonce.get(&(signer.to_vec(), nonce)).cloned())
    }

    fn len(&self) -> Result<usize, Self::Error> {
        let latest = self
            .by_personality_id
            .read()
            .map_err(|_| InMemoryPersonalityStorageError::Internal("latest map poisoned".into()))?;
        Ok(latest.len())
    }
}

#[cfg(test)]
mod tests {
    use super::InMemoryPersonalityStorage;
    use state::{PersonalityStorage, StoredPersonality};

    fn stored_personality(markdown: &str, nonce: u64) -> StoredPersonality {
        StoredPersonality {
            tx_hash: [nonce as u8; 32],
            block_height: nonce,
            signer: b"signer-1".to_vec(),
            personality_id: b"persona-1".to_vec(),
            nonce,
            markdown: markdown.as_bytes().to_vec(),
            markdown_hash: [markdown.len() as u8; 32],
        }
    }

    #[test]
    fn store_is_empty_before_finalized_writes() {
        let store = InMemoryPersonalityStorage::new();

        assert!(store.is_empty().expect("empty check must succeed"));
        assert_eq!(
            store
                .get_latest(b"persona-1")
                .expect("lookup must succeed"),
            None
        );
        assert_eq!(
            store
                .get_by_signer_nonce(b"signer-1", 1)
                .expect("nonce lookup must succeed"),
            None
        );
    }

    #[test]
    fn latest_finalized_write_replaces_existing_personality() {
        let store = InMemoryPersonalityStorage::new();
        let first = stored_personality("# First", 1);
        let replacement = stored_personality("# Replacement", 2);

        store.put(first.clone()).expect("first write must succeed");
        store
            .put(replacement.clone())
            .expect("replacement write must succeed");

        assert_eq!(store.len().expect("len must succeed"), 1);
        assert_eq!(
            store
                .get_latest(&replacement.personality_id)
                .expect("latest lookup must succeed"),
            Some(replacement.clone())
        );
        assert_eq!(
            store
                .get_by_signer_nonce(&first.signer, first.nonce)
                .expect("first nonce lookup must succeed"),
            Some(first)
        );
        assert_eq!(
            store
                .get_by_signer_nonce(&replacement.signer, replacement.nonce)
                .expect("replacement nonce lookup must succeed"),
            Some(replacement)
        );
    }

    #[test]
    fn in_memory_store_drops_state_across_instances() {
        let first_store = InMemoryPersonalityStorage::new();
        first_store
            .put(stored_personality("# First", 1))
            .expect("write must succeed");
        assert_eq!(first_store.len().expect("len must succeed"), 1);

        let replacement_store = InMemoryPersonalityStorage::new();
        assert!(
            replacement_store
                .is_empty()
                .expect("empty check must succeed"),
            "new in-memory store should not retain prior process state"
        );
        assert_eq!(
            replacement_store
                .get_latest(b"persona-1")
                .expect("latest lookup must succeed"),
            None
        );
    }
}
