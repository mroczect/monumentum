use crate::buffer_pool::BufferPool;
use crate::catalog::Catalog;
use crate::page::{
    CATALOG_CHUNK_SIZE, CATALOG_PAGE_HEADER_SIZE, META_CATALOG_PAGE_OFFSET,
    META_LAST_CHECKPOINT_LSN_OFFSET, META_LSN_OFFSET, META_PAGE_ID, Page, PageType,
};
use crate::pager::Pager;
use crate::serde::{decode_catalog, encode_catalog};
use crate::store::append_log::WalRecordType;
use crate::store::wal::Wal;
use crate::table::Table;
use crate::table_storage::TableStorage;
use alloc::collections::BTreeMap;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;
use std::path::Path;

#[derive(Debug)]
pub struct FileStorage {
    wal: Wal,
    buffer_pool: BufferPool,
    catalog: Catalog,
    current_lsn: u64,
    catalog_page_id: u32,
    last_checkpoint_lsn: u64,
    table_data: BTreeMap<String, TableStorage>,
}

impl FileStorage {
    pub fn open(path: &Path, cache_capacity: usize) -> Result<Self, DbError> {
        let wal_path = path.with_extension("wal");
        let wal = Wal::open(&wal_path)?;

        let pager = Pager::open(path)?;
        let mut buffer_pool = BufferPool::new(pager, cache_capacity)?;

        let (current_lsn, catalog_page_id, last_checkpoint_lsn) =
            Self::load_or_init_meta(&mut buffer_pool)?;
        let catalog = Self::load_catalog(&mut buffer_pool, catalog_page_id)?;

        let mut table_data = BTreeMap::new();
        for (name, table) in catalog.tables() {
            if let Some(id) = table.data_page_id() {
                let _ = table_data.insert(name.to_string(), TableStorage::from_first_page_id(id));
            }
        }

        let mut storage = Self {
            wal,
            buffer_pool,
            catalog,
            current_lsn,
            catalog_page_id,
            last_checkpoint_lsn,
            table_data,
        };

        storage.apply_wal_records()?;
        storage.checkpoint()?;

        Ok(storage)
    }

    fn load_or_init_meta(buffer_pool: &mut BufferPool) -> Result<(u64, u32, u64), DbError> {
        if buffer_pool.page_count() == 0 {
            let meta_page_id = buffer_pool.allocate_page(PageType::Meta)?;
            debug_assert_eq!(meta_page_id, META_PAGE_ID);
            let catalog_page_id = buffer_pool.allocate_page(PageType::Data)?;
            {
                let meta_page = buffer_pool.get_page(META_PAGE_ID)?;
                meta_page.header.page_type = PageType::Meta;
                meta_page.data[META_LSN_OFFSET..META_LSN_OFFSET + 8]
                    .copy_from_slice(&0u64.to_le_bytes());
                meta_page.data[META_CATALOG_PAGE_OFFSET..META_CATALOG_PAGE_OFFSET + 4]
                    .copy_from_slice(&catalog_page_id.to_le_bytes());
                meta_page.data
                    [META_LAST_CHECKPOINT_LSN_OFFSET..META_LAST_CHECKPOINT_LSN_OFFSET + 8]
                    .copy_from_slice(&0u64.to_le_bytes());
            }
            buffer_pool.unpin_page(META_PAGE_ID, true)?;

            let empty_catalog = Catalog::new();
            let encoded = encode_catalog(&empty_catalog)?;
            let chunk = encoded.as_slice();
            let page = buffer_pool.get_page(catalog_page_id)?;
            page.header.page_type = PageType::Data;
            page.data.fill(0);
            page.data[0..4].copy_from_slice(&0u32.to_le_bytes());
            let used_len = u32::try_from(chunk.len()).map_err(|e| {
                DbError::invalid_operation(format!("catalog chunk length overflow: {e}"))
            })?;
            page.data[4..8].copy_from_slice(&used_len.to_le_bytes());
            let start = CATALOG_PAGE_HEADER_SIZE;
            let end = start
                .checked_add(chunk.len())
                .ok_or_else(|| DbError::invalid_operation("catalog chunk too large"))?;
            page.data
                .get_mut(start..end)
                .ok_or_else(|| DbError::invalid_operation("catalog chunk does not fit"))?
                .copy_from_slice(chunk);
            buffer_pool.unpin_page(catalog_page_id, true)?;

            Ok((0, catalog_page_id, 0))
        } else {
            let meta_page = buffer_pool.get_page(META_PAGE_ID)?;
            let lsn = u64::from_le_bytes(
                meta_page
                    .data
                    .get(META_LSN_OFFSET..META_LSN_OFFSET + 8)
                    .ok_or_else(|| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "meta page too small",
                        ))
                    })?
                    .try_into()
                    .map_err(|e| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("invalid LSN slice: {e}"),
                        ))
                    })?,
            );
            let cat_page_id = u32::from_le_bytes(
                meta_page
                    .data
                    .get(META_CATALOG_PAGE_OFFSET..META_CATALOG_PAGE_OFFSET + 4)
                    .ok_or_else(|| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "meta page too small",
                        ))
                    })?
                    .try_into()
                    .map_err(|e| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("invalid catalog page slice: {e}"),
                        ))
                    })?,
            );
            let last_checkpoint = u64::from_le_bytes(
                meta_page
                    .data
                    .get(META_LAST_CHECKPOINT_LSN_OFFSET..META_LAST_CHECKPOINT_LSN_OFFSET + 8)
                    .ok_or_else(|| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "meta page too small",
                        ))
                    })?
                    .try_into()
                    .map_err(|e| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("invalid last checkpoint LSN slice: {e}"),
                        ))
                    })?,
            );
            buffer_pool.unpin_page(META_PAGE_ID, false)?;
            Ok((lsn, cat_page_id, last_checkpoint))
        }
    }

    fn load_catalog(buffer_pool: &mut BufferPool, first_page_id: u32) -> Result<Catalog, DbError> {
        let mut data = Vec::new();
        let mut current_page_id = first_page_id;
        loop {
            let page = buffer_pool.get_page(current_page_id)?;
            let next_page_id = u32::from_le_bytes(
                page.data
                    .get(0..4)
                    .ok_or_else(|| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "missing next_page_id",
                        ))
                    })?
                    .try_into()
                    .map_err(|e| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("invalid next_page_id: {e}"),
                        ))
                    })?,
            );
            let used_len = u32::from_le_bytes(
                page.data
                    .get(4..8)
                    .ok_or_else(|| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "missing used_len",
                        ))
                    })?
                    .try_into()
                    .map_err(|e| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("invalid used_len: {e}"),
                        ))
                    })?,
            ) as usize;
            if used_len > CATALOG_CHUNK_SIZE {
                return Err(DbError::corruption(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "used_len exceeds chunk size",
                )));
            }
            let start = CATALOG_PAGE_HEADER_SIZE;
            let end = start.checked_add(used_len).ok_or_else(|| {
                DbError::corruption(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "used_len overflow",
                ))
            })?;
            data.extend_from_slice(page.data.get(start..end).ok_or_else(|| {
                DbError::corruption(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "catalog chunk missing",
                ))
            })?);
            buffer_pool.unpin_page(current_page_id, false)?;

            if next_page_id == 0 {
                break;
            }
            current_page_id = next_page_id;
        }
        decode_catalog(&data)
    }

    fn apply_wal_records(&mut self) -> Result<(), DbError> {
        let wal_records = self.wal.read_wal_records()?;
        for record in wal_records {
            if record.lsn <= self.last_checkpoint_lsn {
                continue;
            }
            match record.record_type {
                WalRecordType::PageWrite => self.apply_page_write(&record)?,
                WalRecordType::Snapshot => {
                    let (lsn, catalog) = decode_snapshot(&record.data)?;
                    self.catalog = catalog;
                    self.current_lsn = lsn;
                }
            }
        }
        Ok(())
    }

    fn apply_page_write(
        &mut self,
        record: &crate::store::append_log::WalRecord,
    ) -> Result<(), DbError> {
        let min_len = 4_usize
            .checked_add(crate::page::PAGE_SIZE)
            .ok_or_else(|| DbError::invalid_operation("PageWrite length overflow"))?;
        if record.data.len() < min_len {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid PageWrite record size",
            )));
        }

        let page_id = u32::from_le_bytes(
            record
                .data
                .get(0..4)
                .ok_or_else(|| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "missing page_id",
                    ))
                })?
                .try_into()
                .map_err(|e| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid page_id slice: {e}"),
                    ))
                })?,
        );
        let page_bytes = record.data.get(4..).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing page data",
            ))
        })?;
        let page = Page::from_bytes(page_bytes)?;
        let buffered_page = self.buffer_pool.get_page(page_id)?;
        *buffered_page = page;
        self.buffer_pool.unpin_page(page_id, true)?;
        self.current_lsn = record.lsn;
        Ok(())
    }

    pub fn checkpoint(&mut self) -> Result<(), DbError> {
        self.buffer_pool.flush_all()?;
        {
            let meta_page = self.buffer_pool.get_page(META_PAGE_ID)?;
            meta_page.data[META_LSN_OFFSET..META_LSN_OFFSET + 8]
                .copy_from_slice(&self.current_lsn.to_le_bytes());
            meta_page.data[META_CATALOG_PAGE_OFFSET..META_CATALOG_PAGE_OFFSET + 4]
                .copy_from_slice(&self.catalog_page_id.to_le_bytes());
            meta_page.data[META_LAST_CHECKPOINT_LSN_OFFSET..META_LAST_CHECKPOINT_LSN_OFFSET + 8]
                .copy_from_slice(&self.current_lsn.to_le_bytes());
        }
        self.buffer_pool.unpin_page(META_PAGE_ID, true)?;
        self.buffer_pool.flush_page(META_PAGE_ID)?;
        self.buffer_pool.flush_all()?;
        self.wal.truncate()?;
        self.last_checkpoint_lsn = self.current_lsn;
        Ok(())
    }

    fn write_catalog_to_pages(&mut self, catalog: &Catalog) -> Result<(), DbError> {
        let encoded = encode_catalog(catalog)?;
        let chunks: Vec<&[u8]> = encoded.chunks(CATALOG_CHUNK_SIZE).collect();
        let needed_pages = chunks.len();

        let existing_pages = self.collect_existing_catalog_pages()?;

        for _ in existing_pages.len()..needed_pages {
            let _ = self.buffer_pool.allocate_page(PageType::Data)?;
        }

        let all_pages = self.collect_existing_catalog_pages()?;

        for (i, chunk) in chunks.iter().enumerate() {
            let page_id = *all_pages
                .get(i)
                .ok_or_else(|| DbError::invalid_operation("page index out of bounds"))?;
            let next_index = i
                .checked_add(1)
                .ok_or_else(|| DbError::invalid_operation("index overflow"))?;
            let next_page_id = if next_index < needed_pages {
                *all_pages
                    .get(next_index)
                    .ok_or_else(|| DbError::invalid_operation("next page index out of bounds"))?
            } else {
                0
            };
            self.write_catalog_chunk(page_id, next_page_id, chunk)?;
            self.append_page_write_wal(page_id)?;
        }

        for extra_page_id in all_pages.iter().skip(needed_pages) {
            let page = self.buffer_pool.get_page(*extra_page_id)?;
            page.header.page_type = PageType::Freelist;
            page.data.fill(0);
            self.buffer_pool.unpin_page(*extra_page_id, true)?;
            self.append_page_write_wal(*extra_page_id)?;
        }

        self.catalog_page_id = *all_pages
            .first()
            .ok_or_else(|| DbError::invalid_operation("no catalog pages"))?;

        Ok(())
    }

    fn collect_existing_catalog_pages(&mut self) -> Result<Vec<u32>, DbError> {
        let mut pages = Vec::new();
        let mut current = self.catalog_page_id;
        while current != 0 {
            pages.push(current);
            let page = self.buffer_pool.get_page(current)?;
            let next = u32::from_le_bytes(
                page.data
                    .get(0..4)
                    .ok_or_else(|| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "missing next_page_id",
                        ))
                    })?
                    .try_into()
                    .map_err(|e| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("invalid next_page_id: {e}"),
                        ))
                    })?,
            );
            self.buffer_pool.unpin_page(current, false)?;
            current = next;
        }
        Ok(pages)
    }

    fn write_catalog_chunk(
        &mut self,
        page_id: u32,
        next_page_id: u32,
        chunk: &[u8],
    ) -> Result<(), DbError> {
        let page = self.buffer_pool.get_page(page_id)?;
        page.header.page_type = PageType::Data;
        page.data.fill(0);
        page.data[0..4].copy_from_slice(&next_page_id.to_le_bytes());
        let chunk_len = u32::try_from(chunk.len())
            .map_err(|e| DbError::invalid_operation(format!("chunk length overflow: {e}")))?;
        page.data[4..8].copy_from_slice(&chunk_len.to_le_bytes());
        let start = CATALOG_PAGE_HEADER_SIZE;
        let end = start
            .checked_add(chunk.len())
            .ok_or_else(|| DbError::invalid_operation("chunk size overflow"))?;
        page.data
            .get_mut(start..end)
            .ok_or_else(|| DbError::invalid_operation("chunk too large for page"))?
            .copy_from_slice(chunk);
        self.buffer_pool.unpin_page(page_id, true)?;
        Ok(())
    }

    fn append_page_write_wal(&mut self, page_id: u32) -> Result<(), DbError> {
        let page = self.buffer_pool.get_page(page_id)?;
        let page_bytes = page.as_bytes();
        self.buffer_pool.unpin_page(page_id, false)?;

        let wal_data_len = 4_usize
            .checked_add(page_bytes.len())
            .ok_or_else(|| DbError::invalid_operation("WAL data size overflow"))?;
        let mut wal_data = Vec::with_capacity(wal_data_len);
        wal_data.extend_from_slice(&page_id.to_le_bytes());
        wal_data.extend_from_slice(&page_bytes);
        let new_lsn = self
            .current_lsn
            .checked_add(1)
            .ok_or_else(|| DbError::invalid_operation("LSN overflow"))?;
        self.wal
            .append_wal_record(new_lsn, WalRecordType::PageWrite, &wal_data)?;
        self.current_lsn = new_lsn;
        Ok(())
    }

    pub fn sync(&mut self) -> Result<(), DbError> {
        self.wal.sync()
    }

    pub fn close(mut self) -> Result<(), DbError> {
        self.checkpoint()?;
        self.wal.unlock()?;
        Ok(())
    }

    pub fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError> {
        self.catalog = catalog.clone();
        self.write_catalog_to_pages(catalog)
    }

    #[must_use]
    pub const fn get_catalog(&self) -> &Catalog {
        &self.catalog
    }

    #[must_use]
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.catalog.get_table(name)
    }
}

impl StorageEngine for FileStorage {
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError> {
        let name = schema.name().to_string();
        self.catalog.create_table(schema)?;

        let table_storage = TableStorage::new(&mut self.buffer_pool)?;
        let first_page_id = table_storage.first_data_page_id();
        let _ = self.table_data.insert(name.clone(), table_storage);

        if let Some(table) = self.catalog.get_table_mut(&name) {
            table.set_data_page_id(first_page_id);
        }

        let catalog = self.catalog.clone();
        self.write_catalog_to_pages(&catalog)
    }

    fn drop_table(&mut self, name: &str) -> Result<(), DbError> {
        self.catalog.drop_table(name)?;
        let _ = self.table_data.remove(name);
        let catalog = self.catalog.clone();
        self.write_catalog_to_pages(&catalog)
    }

    fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError> {
        self.catalog.rename_table(old_name, new_name)?;
        if let Some(storage) = self.table_data.remove(old_name) {
            let _ = self.table_data.insert(new_name.to_string(), storage);
        }
        let catalog = self.catalog.clone();
        self.write_catalog_to_pages(&catalog)
    }

    fn insert_row(&mut self, table: &str, row: &Row) -> Result<(), DbError> {
        let first_page_id = self
            .table_data
            .get(table)
            .map(TableStorage::first_data_page_id)
            .ok_or_else(|| DbError::table_not_found(table))?;
        TableStorage::insert_row_static(&mut self.buffer_pool, first_page_id, row)
    }

    fn get_row(&mut self, table: &str, row_idx: usize) -> Result<Option<Row>, DbError> {
        let first_page_id = self
            .table_data
            .get(table)
            .map(TableStorage::first_data_page_id)
            .ok_or_else(|| DbError::table_not_found(table))?;
        TableStorage::get_row_static(&mut self.buffer_pool, first_page_id, row_idx)
    }

    fn set_cell(
        &mut self,
        _table: &str,
        _row_idx: usize,
        _col_idx: usize,
        _value: Value,
    ) -> Result<(), DbError> {
        Err(DbError::unsupported("row operations not yet integrated"))
    }

    fn replace_rows(&mut self, _table: &str, _rows: Vec<Row>) -> Result<(), DbError> {
        Err(DbError::unsupported("row operations not yet integrated"))
    }

    fn checkpoint(&mut self) -> Result<(), DbError> {
        Self::checkpoint(self)
    }
}

#[derive(Debug, Default)]
pub struct InMemoryStorage {
    catalog: Catalog,
}

impl InMemoryStorage {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError> {
        self.catalog = catalog.clone();
        Ok(())
    }

    #[must_use]
    pub const fn get_catalog(&self) -> &Catalog {
        &self.catalog
    }

    #[must_use]
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.catalog.get_table(name)
    }
}

impl StorageEngine for InMemoryStorage {
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError> {
        self.catalog.create_table(schema)
    }

    fn drop_table(&mut self, name: &str) -> Result<(), DbError> {
        self.catalog.drop_table(name)
    }

    fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError> {
        self.catalog.rename_table(old_name, new_name)
    }

    fn insert_row(&mut self, _table: &str, _row: &Row) -> Result<(), DbError> {
        Err(DbError::unsupported(
            "row operations not supported in InMemoryStorage",
        ))
    }

    fn get_row(&mut self, _table: &str, _row_idx: usize) -> Result<Option<Row>, DbError> {
        Err(DbError::unsupported(
            "row operations not supported in InMemoryStorage",
        ))
    }

    fn set_cell(
        &mut self,
        _table: &str,
        _row_idx: usize,
        _col_idx: usize,
        _value: Value,
    ) -> Result<(), DbError> {
        Err(DbError::unsupported(
            "row operations not supported in InMemoryStorage",
        ))
    }

    fn replace_rows(&mut self, _table: &str, _rows: Vec<Row>) -> Result<(), DbError> {
        Err(DbError::unsupported(
            "row operations not supported in InMemoryStorage",
        ))
    }

    fn checkpoint(&mut self) -> Result<(), DbError> {
        Ok(())
    }
}

fn decode_snapshot(data: &[u8]) -> Result<(u64, Catalog), DbError> {
    use crate::serde::Decode;
    use std::io::Cursor;
    let mut cursor = Cursor::new(data);
    let seq = u64::decode(&mut cursor)?;
    let catalog = Catalog::decode(&mut cursor)?;
    Ok((seq, catalog))
}
