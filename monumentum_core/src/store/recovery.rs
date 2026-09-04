use crate::store::wal::Wal;
use monumentum_handler::error::DbError;
use std::path::Path;

#[derive(Debug)]
pub struct RecoveryResult {
    pub records: Vec<Vec<u8>>,
}

pub fn recover_wal(path: &Path) -> Result<RecoveryResult, DbError> {
    let mut wal = Wal::open(path)?;
    let records = wal.read_all()?;
    Ok(RecoveryResult { records })
}
