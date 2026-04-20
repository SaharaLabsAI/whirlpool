use super::*;

#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(test)]
static FAIL_NEXT_COMMIT_DELETE: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static FAIL_NEXT_INSERT_STORAGE_DELETE: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
pub fn inject_next_commit_delete_failure() {
    FAIL_NEXT_COMMIT_DELETE.store(true, Ordering::SeqCst);
}

#[cfg(test)]
pub fn inject_next_insert_storage_delete_failure() {
    FAIL_NEXT_INSERT_STORAGE_DELETE.store(true, Ordering::SeqCst);
}

pub fn maybe_inject_commit_delete_failure() -> Result<(), RethStateError> {
    #[cfg(test)]
    if FAIL_NEXT_COMMIT_DELETE.swap(false, Ordering::SeqCst) {
        return Err(RethStateError::Database(reth_db::DatabaseError::Other(
            "injected commit delete failure".to_string(),
        )));
    }
    Ok(())
}

pub fn maybe_inject_insert_storage_delete_failure() -> Result<(), RethStateError> {
    #[cfg(test)]
    if FAIL_NEXT_INSERT_STORAGE_DELETE.swap(false, Ordering::SeqCst) {
        return Err(RethStateError::Database(reth_db::DatabaseError::Other(
            "injected insert_storage delete failure".to_string(),
        )));
    }
    Ok(())
}
