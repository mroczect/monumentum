use crate::page::{PAGE_SIZE, Page, PageType};
use monumentum_handler::error::DbError;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug)]
pub struct Pager {
    file: File,
    page_count: u32,
}

impl Pager {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        let file_len = file.metadata()?.len();
        let page_size_u64 = PAGE_SIZE as u64;
        if page_size_u64 == 0 {
            return Err(DbError::invalid_operation("page size cannot be zero"));
        }
        let remainder = file_len
            .checked_rem(page_size_u64)
            .ok_or_else(|| DbError::invalid_operation("failed to compute file size remainder"))?;
        if remainder != 0 {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "database file size is not a multiple of page size",
            )));
        }
        let page_count_u64 = file_len
            .checked_div(page_size_u64)
            .ok_or_else(|| DbError::invalid_operation("failed to compute page count"))?;
        let page_count = u32::try_from(page_count_u64)
            .map_err(|e| DbError::invalid_operation(format!("file too large: {e}")))?;
        Ok(Self { file, page_count })
    }

    pub fn read_page(&mut self, page_id: u32) -> Result<Page, DbError> {
        if page_id >= self.page_count {
            return Err(DbError::invalid_operation("page id out of bounds"));
        }
        let offset = u64::from(page_id)
            .checked_mul(PAGE_SIZE as u64)
            .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;
        let _ = self.file.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0u8; PAGE_SIZE];
        self.file.read_exact(&mut buf)?;
        Page::from_bytes(&buf)
    }

    pub fn write_page(&mut self, page: &Page) -> Result<(), DbError> {
        if page.header.page_id >= self.page_count {
            return Err(DbError::invalid_operation("page id out of bounds"));
        }
        let offset = u64::from(page.header.page_id)
            .checked_mul(PAGE_SIZE as u64)
            .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;
        let _ = self.file.seek(SeekFrom::Start(offset))?;
        let buf = page.as_bytes();
        self.file.write_all(&buf)?;
        self.file.sync_data()?;
        Ok(())
    }

    pub fn allocate_page(&mut self, page_type: PageType) -> Result<u32, DbError> {
        let new_page_id = self.page_count;
        let page = Page::new(new_page_id, page_type);
        let offset = u64::from(new_page_id)
            .checked_mul(PAGE_SIZE as u64)
            .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;
        let _ = self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&page.as_bytes())?;
        self.file.sync_data()?;
        self.page_count = self
            .page_count
            .checked_add(1)
            .ok_or_else(|| DbError::invalid_operation("page count overflow"))?;
        Ok(new_page_id)
    }

    pub fn free_page(&mut self, page_id: u32) -> Result<(), DbError> {
        if page_id >= self.page_count {
            return Err(DbError::invalid_operation("page id out of bounds"));
        }
        let mut page = self.read_page(page_id)?;
        page.header.page_type = PageType::Freelist;
        page.header.free_space_offset = crate::page::INITIAL_FREE_SPACE_OFFSET;
        page.header.cell_count = 0;
        self.write_page(&page)?;
        Ok(())
    }

    pub fn sync(&mut self) -> Result<(), DbError> {
        self.file.sync_all()?;
        Ok(())
    }

    #[must_use]
    pub const fn page_count(&self) -> u32 {
        self.page_count
    }
}
