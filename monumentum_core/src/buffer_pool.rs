use crate::page::{Page, PageType};
use crate::pager::Pager;
use monumentum_handler::error::DbError;
use std::collections::HashMap;

#[derive(Debug)]
struct PageEntry {
    page: Page,
    dirty: bool,
    pin_count: u32,
    last_used: u64,
}

#[derive(Debug)]
pub struct BufferPool {
    pager: Pager,
    capacity: usize,
    entries: HashMap<u32, PageEntry>,
    clock: u64,
}

impl BufferPool {
    pub fn new(pager: Pager, capacity: usize) -> Result<Self, DbError> {
        if capacity == 0 {
            return Err(DbError::invalid_operation(
                "buffer pool capacity must be greater than zero",
            ));
        }
        Ok(Self {
            pager,
            capacity,
            entries: HashMap::new(),
            clock: 0,
        })
    }

    pub fn get_page(&mut self, page_id: u32) -> Result<&mut Page, DbError> {
        if self.entries.contains_key(&page_id) {
            let clock = self.next_clock()?;
            let entry = self
                .entries
                .get_mut(&page_id)
                .ok_or_else(|| DbError::invalid_operation("page not found in buffer"))?;
            entry.pin_count = entry
                .pin_count
                .checked_add(1)
                .ok_or_else(|| DbError::invalid_operation("pin count overflow"))?;
            entry.last_used = clock;
            return Ok(&mut entry.page);
        }

        let page = self.pager.read_page(page_id)?;
        if self.entries.len() >= self.capacity {
            self.evict_one()?;
        }
        let clock = self.next_clock()?;
        let entry = PageEntry {
            page,
            dirty: false,
            pin_count: 1,
            last_used: clock,
        };
        let _ = self.entries.insert(page_id, entry);
        let entry = self
            .entries
            .get_mut(&page_id)
            .ok_or_else(|| DbError::invalid_operation("page not found after insert"))?;
        Ok(&mut entry.page)
    }

    pub fn unpin_page(&mut self, page_id: u32, dirty: bool) -> Result<(), DbError> {
        let clock = self.next_clock()?;
        let entry = self
            .entries
            .get_mut(&page_id)
            .ok_or_else(|| DbError::invalid_operation("page not in buffer"))?;
        entry.pin_count = entry
            .pin_count
            .checked_sub(1)
            .ok_or_else(|| DbError::invalid_operation("cannot unpin a page with zero pin count"))?;
        if dirty {
            entry.dirty = true;
        }
        entry.last_used = clock;
        Ok(())
    }

    pub fn mark_dirty(&mut self, page_id: u32) -> Result<(), DbError> {
        let clock = self.next_clock()?;
        let entry = self
            .entries
            .get_mut(&page_id)
            .ok_or_else(|| DbError::invalid_operation("page not in buffer"))?;
        entry.dirty = true;
        entry.last_used = clock;
        Ok(())
    }

    pub fn flush_page(&mut self, page_id: u32) -> Result<(), DbError> {
        let clock = self.next_clock()?;
        let entry = self
            .entries
            .get_mut(&page_id)
            .ok_or_else(|| DbError::invalid_operation("page not in buffer"))?;
        if entry.dirty {
            self.pager.write_page(&entry.page)?;
            entry.dirty = false;
            entry.last_used = clock;
        }
        Ok(())
    }

    pub fn flush_all(&mut self) -> Result<(), DbError> {
        let page_ids: Vec<u32> = self.entries.keys().copied().collect();
        for page_id in page_ids {
            self.flush_page(page_id)?;
        }
        self.pager.sync()?;
        Ok(())
    }

    pub fn allocate_page(&mut self, page_type: PageType) -> Result<u32, DbError> {
        let page_id = self.pager.allocate_page(page_type)?;
        let page = self.pager.read_page(page_id)?;
        if self.entries.len() >= self.capacity {
            self.evict_one()?;
        }
        let clock = self.next_clock()?;
        let entry = PageEntry {
            page,
            dirty: false,
            pin_count: 0,
            last_used: clock,
        };
        let _ = self.entries.insert(page_id, entry);
        Ok(page_id)
    }

    pub fn evict_one(&mut self) -> Result<(), DbError> {
        let mut candidate: Option<(u32, u64)> = None;
        for (&page_id, entry) in &self.entries {
            if entry.pin_count == 0 {
                if let Some((_, min_used)) = candidate {
                    if entry.last_used < min_used {
                        candidate = Some((page_id, entry.last_used));
                    }
                } else {
                    candidate = Some((page_id, entry.last_used));
                }
            }
        }

        let page_id = candidate
            .map(|(id, _)| id)
            .ok_or_else(|| DbError::invalid_operation("no unpinned page to evict"))?;

        if self
            .entries
            .get(&page_id)
            .ok_or_else(|| DbError::invalid_operation("page not found"))?
            .dirty
        {
            self.flush_page(page_id)?;
        }

        let _ = self.entries.remove(&page_id);
        Ok(())
    }

    fn next_clock(&mut self) -> Result<u64, DbError> {
        let next = self
            .clock
            .checked_add(1)
            .ok_or_else(|| DbError::invalid_operation("clock overflow"))?;
        self.clock = next;
        Ok(next)
    }

    #[must_use]
    pub const fn page_count(&self) -> u32 {
        self.pager.page_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        std::env::temp_dir().join(format!(
            "monumentum_test_{}_{}.db",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn test_buffer_pool_basic() -> Result<(), DbError> {
        let path = temp_db_path();
        let pager = Pager::open(&path)?;
        let mut pool = BufferPool::new(pager, 2)?;

        let page_id = pool.allocate_page(PageType::Data)?;
        {
            let page = pool.get_page(page_id)?;
            page.header.cell_count = 42;
        }
        pool.unpin_page(page_id, true)?;
        pool.flush_page(page_id)?;

        {
            let page = pool.get_page(page_id)?;
            assert_eq!(page.header.cell_count, 42);
        }
        pool.unpin_page(page_id, false)?;

        let _ = fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn test_buffer_pool_eviction() -> Result<(), DbError> {
        let path = temp_db_path();
        let pager = Pager::open(&path)?;
        let mut pool = BufferPool::new(pager, 1)?;

        let first_page_id = pool.allocate_page(PageType::Data)?;
        let second_page_id = pool.allocate_page(PageType::Data)?;

        assert!(!pool.entries.contains_key(&first_page_id));
        assert!(pool.entries.contains_key(&second_page_id));

        let _ = fs::remove_file(&path);
        Ok(())
    }
}
