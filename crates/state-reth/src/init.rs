// Database initialization helpers.

use std::path::Path;

use crate::db::RethStateDb;
use crate::error::RethStateError;

/// Open or create a state database at the given path.
///
/// This is the primary entry point for creating a `RethStateDb`.
/// The database is initialized with all required tables if they don't exist.
pub fn open_state_db(path: &Path) -> Result<RethStateDb, RethStateError> {
    RethStateDb::open(path)
}
