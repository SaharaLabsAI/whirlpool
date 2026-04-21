use crate::error::RethStateError;

#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(test)]
static FAIL_NEXT_COMMIT_DELETE: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static FAIL_NEXT_INSERT_STORAGE_DELETE: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
const COMMIT_DELETE_FAILURE: &str = "injected commit delete failure";
#[cfg(test)]
const INSERT_STORAGE_DELETE_FAILURE: &str = "injected insert_storage delete failure";

#[cfg(test)]
pub enum DeleteFailureTarget {
    Commit,
    InsertStorage,
}

#[cfg(test)]
pub fn inject_next_delete_failure(target: DeleteFailureTarget) {
    match target {
        DeleteFailureTarget::Commit => FAIL_NEXT_COMMIT_DELETE.store(true, Ordering::SeqCst),
        DeleteFailureTarget::InsertStorage => {
            FAIL_NEXT_INSERT_STORAGE_DELETE.store(true, Ordering::SeqCst)
        }
    };
}

pub fn maybe_inject_commit_delete_failure() -> Result<(), RethStateError> {
    #[cfg(test)]
    {
        maybe_inject_delete_failure(&FAIL_NEXT_COMMIT_DELETE, COMMIT_DELETE_FAILURE)?;
    }

    Ok(())
}

pub fn maybe_inject_insert_storage_delete_failure() -> Result<(), RethStateError> {
    #[cfg(test)]
    {
        maybe_inject_delete_failure(
            &FAIL_NEXT_INSERT_STORAGE_DELETE,
            INSERT_STORAGE_DELETE_FAILURE,
        )?;
    }

    Ok(())
}

#[cfg(test)]
fn maybe_inject_delete_failure(
    failure_flag: &AtomicBool,
    message: &'static str,
) -> Result<(), RethStateError> {
    if failure_flag.swap(false, Ordering::SeqCst) {
        return Err(RethStateError::Database(reth_db::DatabaseError::Other(
            message.to_string(),
        )));
    }

    Ok(())
}
