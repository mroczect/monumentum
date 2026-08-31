use crate::error::DbError;
use crate::store::wal::Wal;
use std::path::Path;

pub struct RecoveryResult {
    pub records: Vec<Vec<u8>>,
}

pub fn recover_wal(path: &Path) -> Result<RecoveryResult, DbError> {
    let mut wal = Wal::open(path)?;
    let records = wal.read_all()?;
    Ok(RecoveryResult { records })
}
