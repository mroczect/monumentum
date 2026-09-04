use crate::buffer_pool::BufferPool;
use crate::page::{DATA_PAGE_HEADER_SIZE, PAGE_BODY_SIZE, PageType};
use crate::serde::{decode_row, encode_row};
use monumentum_handler::core::row::Row;
use monumentum_handler::error::DbError;

#[derive(Debug)]
pub struct TableStorage {
    buffer_pool: BufferPool,
    first_data_page_id: u32,
}

impl TableStorage {
    pub fn new(mut buffer_pool: BufferPool) -> Result<Self, DbError> {
        let first_page_id = buffer_pool.allocate_page(PageType::Data)?;
        {
            let page = buffer_pool.get_page(first_page_id)?;
            page.header.page_type = PageType::Data;
            page.data[0..4].copy_from_slice(&0u32.to_le_bytes());
            page.data[4..8].copy_from_slice(&0u32.to_le_bytes());
        }
        buffer_pool.unpin_page(first_page_id, true)?;

        Ok(Self {
            buffer_pool,
            first_data_page_id: first_page_id,
        })
    }

    pub fn insert_row(&mut self, row: &Row) -> Result<(), DbError> {
        let encoded = encode_row(row)?;
        let mut current_page_id = self.first_data_page_id;

        loop {
            let current_page = self.buffer_pool.get_page(current_page_id)?;
            let used_len = Self::get_used_len(current_page)?;
            let free_space = PAGE_BODY_SIZE
                .checked_sub(DATA_PAGE_HEADER_SIZE)
                .and_then(|v| v.checked_sub(used_len))
                .ok_or_else(|| DbError::invalid_operation("free space underflow"))?;

            if free_space >= encoded.len() {
                let start = DATA_PAGE_HEADER_SIZE
                    .checked_add(used_len)
                    .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;
                let end = start
                    .checked_add(encoded.len())
                    .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;
                current_page
                    .data
                    .get_mut(start..end)
                    .ok_or_else(|| DbError::invalid_operation("insufficient page space"))?
                    .copy_from_slice(&encoded);
                let new_used_len = used_len
                    .checked_add(encoded.len())
                    .ok_or_else(|| DbError::invalid_operation("used_len overflow"))?;
                Self::set_used_len(current_page, new_used_len)?;
                let page_id = current_page.header.page_id;
                self.buffer_pool.unpin_page(page_id, true)?;
                return Ok(());
            }

            let next_page_id = Self::get_next_page_id(current_page)?;
            let current_id = current_page.header.page_id;
            self.buffer_pool.unpin_page(current_id, false)?;

            if next_page_id == 0 {
                let new_page_id = self.buffer_pool.allocate_page(PageType::Data)?;
                {
                    let page = self.buffer_pool.get_page(new_page_id)?;
                    page.header.page_type = PageType::Data;
                    page.data[0..4].copy_from_slice(&0u32.to_le_bytes());
                    page.data[4..8].copy_from_slice(&0u32.to_le_bytes());
                }
                self.buffer_pool.unpin_page(new_page_id, true)?;

                {
                    let prev_page = self.buffer_pool.get_page(current_id)?;
                    prev_page.data[0..4].copy_from_slice(&new_page_id.to_le_bytes());
                }
                self.buffer_pool.unpin_page(current_id, true)?;

                current_page_id = new_page_id;
            } else {
                current_page_id = next_page_id;
            }
        }
    }

    pub fn get_row(&mut self, row_idx: usize) -> Result<Option<Row>, DbError> {
        let mut current_page_id = self.first_data_page_id;
        let mut current_row_idx = 0_usize;

        loop {
            let current_page = self.buffer_pool.get_page(current_page_id)?;
            let used_len = Self::get_used_len(current_page)?;
            let mut offset = DATA_PAGE_HEADER_SIZE;
            let end = offset
                .checked_add(used_len)
                .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;

            while offset < end {
                let len_end = offset
                    .checked_add(4)
                    .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;
                let len_bytes = current_page.data.get(offset..len_end).ok_or_else(|| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "row length missing",
                    ))
                })?;
                let len = u32::from_le_bytes(len_bytes.try_into().map_err(|e| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid row length: {e}"),
                    ))
                })?) as usize;

                let row_start = offset
                    .checked_add(4)
                    .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;
                let row_end = row_start
                    .checked_add(len)
                    .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;

                if current_row_idx == row_idx {
                    let row_bytes = current_page.data.get(row_start..row_end).ok_or_else(|| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "row bytes missing",
                        ))
                    })?;
                    let row = decode_row(row_bytes)?;
                    self.buffer_pool.unpin_page(current_page_id, false)?;
                    return Ok(Some(row));
                }

                offset = row_end;
                current_row_idx = current_row_idx
                    .checked_add(1)
                    .ok_or_else(|| DbError::invalid_operation("row index overflow"))?;
            }

            let next_page_id = Self::get_next_page_id(current_page)?;
            let current_id = current_page.header.page_id;
            self.buffer_pool.unpin_page(current_id, false)?;

            if next_page_id == 0 {
                return Ok(None);
            }
            current_page_id = next_page_id;
        }
    }

    fn get_next_page_id(page: &crate::page::Page) -> Result<u32, DbError> {
        Ok(u32::from_le_bytes(
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
        ))
    }

    fn get_used_len(page: &crate::page::Page) -> Result<usize, DbError> {
        let used = u32::from_le_bytes(
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
        );
        usize::try_from(used).map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("used_len too large: {e}"),
            ))
        })
    }

    fn set_used_len(page: &mut crate::page::Page, len: usize) -> Result<(), DbError> {
        let len_u32 = u32::try_from(len)
            .map_err(|e| DbError::invalid_operation(format!("used_len overflow: {e}")))?;
        page.data[4..8].copy_from_slice(&len_u32.to_le_bytes());
        Ok(())
    }
}
