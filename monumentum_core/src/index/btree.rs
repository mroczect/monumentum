use crate::buffer_pool::BufferPool;
use crate::index::key::IndexKey;
use crate::page::{BTREE_NODE_HEADER_SIZE, PageType};
use monumentum_handler::error::DbError;

const MAX_KEYS_PER_NODE: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BTreeOnDisk {
    root_page_id: u32,
}

impl BTreeOnDisk {
    pub fn create(buffer_pool: &mut BufferPool) -> Result<Self, DbError> {
        let root_page_id = buffer_pool.allocate_page(PageType::Index)?;
        {
            let page = buffer_pool.get_page(root_page_id)?;
            page.header.page_type = PageType::Index;
            page.data[0] = 1;
            page.data[1..3].copy_from_slice(&0u16.to_le_bytes());
            page.data[3..7].copy_from_slice(&u32::MAX.to_le_bytes());
        }
        buffer_pool.unpin_page(root_page_id, true)?;
        Ok(Self { root_page_id })
    }

    #[must_use]
    pub const fn root_page_id(&self) -> u32 {
        self.root_page_id
    }

    pub fn insert_static(
        buffer_pool: &mut BufferPool,
        root_page_id: &mut u32,
        key: IndexKey,
        value: u64,
    ) -> Result<(), DbError> {
        let root = Self::load_node(buffer_pool, *root_page_id)?;
        if root.keys.len() >= MAX_KEYS_PER_NODE {
            Self::split_root(buffer_pool, root_page_id)?;
        }
        Self::insert_non_full(buffer_pool, *root_page_id, key, value)
    }

    pub fn lookup_static(
        buffer_pool: &mut BufferPool,
        root_page_id: u32,
        key: &IndexKey,
    ) -> Result<Option<u64>, DbError> {
        let mut node = Self::load_node(buffer_pool, root_page_id)?;
        loop {
            if node.is_leaf {
                return match node.keys.binary_search(key) {
                    Ok(idx) => node.values.get(idx).copied().map(Some).ok_or_else(|| {
                        DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "value missing at index",
                        ))
                    }),
                    Err(_) => Ok(None),
                };
            }

            let idx = node.keys.partition_point(|k| k <= key);
            let child_page_id = *node.children.get(idx).ok_or_else(|| {
                DbError::corruption(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "child page missing",
                ))
            })?;
            node = Self::load_node(buffer_pool, child_page_id)?;
        }
    }

    fn split_root(buffer_pool: &mut BufferPool, root_page_id: &mut u32) -> Result<(), DbError> {
        let new_root_page_id = Self::allocate_node(buffer_pool, false)?;
        let old_root_page_id = *root_page_id;

        {
            let mut new_root = Self::load_node(buffer_pool, new_root_page_id)?;
            new_root.children.push(old_root_page_id);
            Self::save_node(buffer_pool, &new_root)?;
        }

        Self::split_child(buffer_pool, new_root_page_id, 0)?;
        *root_page_id = new_root_page_id;
        Ok(())
    }

    fn split_child(
        buffer_pool: &mut BufferPool,
        parent_page_id: u32,
        child_idx: usize,
    ) -> Result<(), DbError> {
        let mut parent = Self::load_node(buffer_pool, parent_page_id)?;
        let child_page_id = *parent
            .children
            .get(child_idx)
            .ok_or_else(|| DbError::invalid_operation("child index out of bounds"))?;
        let mut child = Self::load_node(buffer_pool, child_page_id)?;

        let mid = child.keys.len() / 2;
        let mid_key = child
            .keys
            .get(mid)
            .ok_or_else(|| DbError::invalid_operation("mid key missing"))?
            .clone();

        let new_child_page_id = Self::allocate_node(buffer_pool, child.is_leaf)?;
        let mut new_child = Self::load_node(buffer_pool, new_child_page_id)?;
        new_child.is_leaf = child.is_leaf;
        new_child.parent_page_id = parent_page_id;

        if child.is_leaf {
            new_child.keys = child.keys.split_off(mid);
            new_child.values = child.values.split_off(mid);
        } else {
            let mid_plus_one = mid
                .checked_add(1)
                .ok_or_else(|| DbError::invalid_operation("mid index overflow"))?;
            new_child.keys = child.keys.split_off(mid_plus_one);
            new_child.children = child.children.split_off(mid_plus_one);
        }

        let insert_pos = child_idx
            .checked_add(1)
            .ok_or_else(|| DbError::invalid_operation("child index overflow"))?;
        parent.keys.insert(child_idx, mid_key);
        parent.children.insert(insert_pos, new_child_page_id);

        Self::save_node(buffer_pool, &child)?;
        Self::save_node(buffer_pool, &new_child)?;
        Self::save_node(buffer_pool, &parent)?;
        Ok(())
    }

    fn insert_non_full(
        buffer_pool: &mut BufferPool,
        page_id: u32,
        key: IndexKey,
        value: u64,
    ) -> Result<(), DbError> {
        let mut node = Self::load_node(buffer_pool, page_id)?;
        if node.is_leaf {
            match node.keys.binary_search(&key) {
                Ok(_) => {
                    return Err(DbError::constraint_violation(
                        monumentum_handler::error::ErrorKind::UniqueViolation,
                        "duplicate key",
                        None,
                        None,
                    ));
                }
                Err(idx) => {
                    node.keys.insert(idx, key);
                    node.values.insert(idx, value);
                }
            }
            Self::save_node(buffer_pool, &node)?;
            Ok(())
        } else {
            let idx = node.keys.partition_point(|k| k <= &key);
            let child_page_id = *node
                .children
                .get(idx)
                .ok_or_else(|| DbError::invalid_operation("child index out of bounds"))?;
            let child = Self::load_node(buffer_pool, child_page_id)?;
            if child.keys.len() >= MAX_KEYS_PER_NODE {
                Self::split_child(buffer_pool, page_id, idx)?;
                let parent_after = Self::load_node(buffer_pool, page_id)?;
                let new_idx = parent_after.keys.partition_point(|k| k <= &key);
                let new_child_page_id = *parent_after.children.get(new_idx).ok_or_else(|| {
                    DbError::invalid_operation("child index out of bounds after split")
                })?;
                return Self::insert_non_full(buffer_pool, new_child_page_id, key, value);
            }
            Self::insert_non_full(buffer_pool, child_page_id, key, value)
        }
    }

    fn allocate_node(buffer_pool: &mut BufferPool, is_leaf: bool) -> Result<u32, DbError> {
        let page_id = buffer_pool.allocate_page(PageType::Index)?;
        {
            let page = buffer_pool.get_page(page_id)?;
            page.header.page_type = PageType::Index;
            page.data[0] = u8::from(is_leaf);
            page.data[1..3].copy_from_slice(&0u16.to_le_bytes());
            page.data[3..7].copy_from_slice(&u32::MAX.to_le_bytes());
        }
        buffer_pool.unpin_page(page_id, true)?;
        Ok(page_id)
    }

    fn load_node(buffer_pool: &mut BufferPool, page_id: u32) -> Result<Node, DbError> {
        let page = buffer_pool.get_page(page_id)?;
        let is_leaf = page.data[0] == 1;
        let num_keys = u16::from_le_bytes([page.data[1], page.data[2]]) as usize;
        let parent_page_id = u32::from_le_bytes(
            page.data
                .get(3..7)
                .ok_or_else(|| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "missing parent page id",
                    ))
                })?
                .try_into()
                .map_err(|e| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid parent page id: {e}"),
                    ))
                })?,
        );

        let mut cursor = NodeCursor::new(&page.data[..], BTREE_NODE_HEADER_SIZE);
        let mut keys = Vec::with_capacity(num_keys);
        let mut values = Vec::with_capacity(num_keys);
        let mut children = Vec::with_capacity(if is_leaf {
            0
        } else {
            num_keys.checked_add(1).ok_or_else(|| {
                DbError::corruption(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "children capacity overflow",
                ))
            })?
        });

        for _ in 0..num_keys {
            let key_len = cursor.read_u16()? as usize;
            let key_bytes = cursor.read_bytes(key_len)?;
            let key = IndexKey::from_bytes(key_bytes)?;
            keys.push(key);
            if is_leaf {
                let value = cursor.read_u64()?;
                values.push(value);
            } else {
                let child = cursor.read_u32()?;
                children.push(child);
            }
        }

        if !is_leaf {
            let last_child = cursor.read_u32()?;
            children.push(last_child);
        }

        buffer_pool.unpin_page(page_id, false)?;
        Ok(Node {
            page_id,
            is_leaf,
            parent_page_id,
            keys,
            values,
            children,
        })
    }

    fn save_node(buffer_pool: &mut BufferPool, node: &Node) -> Result<(), DbError> {
        let page = buffer_pool.get_page(node.page_id)?;
        page.header.page_type = PageType::Index;
        page.data.fill(0);
        page.data[0] = u8::from(node.is_leaf);
        let num_keys = u16::try_from(node.keys.len())
            .map_err(|e| DbError::invalid_operation(format!("too many keys: {e}")))?;
        page.data[1..3].copy_from_slice(&num_keys.to_le_bytes());
        page.data[3..7].copy_from_slice(&node.parent_page_id.to_le_bytes());

        let mut cursor = NodeCursorMut::new(&mut page.data[..], BTREE_NODE_HEADER_SIZE);
        for (i, key) in node.keys.iter().enumerate() {
            let key_bytes = key.to_bytes()?;
            let key_len = u16::try_from(key_bytes.len())
                .map_err(|e| DbError::invalid_operation(format!("key too long: {e}")))?;
            cursor.write_u16(key_len)?;
            cursor.write_bytes(&key_bytes)?;
            if node.is_leaf {
                let value = node
                    .values
                    .get(i)
                    .ok_or_else(|| DbError::invalid_operation("missing value"))?;
                cursor.write_u64(*value)?;
            } else {
                let child = node
                    .children
                    .get(i)
                    .ok_or_else(|| DbError::invalid_operation("missing child"))?;
                cursor.write_u32(*child)?;
            }
        }

        if !node.is_leaf {
            let last_child = node.children.last().copied().unwrap_or(u32::MAX);
            cursor.write_u32(last_child)?;
        }

        buffer_pool.unpin_page(node.page_id, true)?;
        Ok(())
    }
}

#[derive(Debug)]
struct Node {
    page_id: u32,
    is_leaf: bool,
    parent_page_id: u32,
    keys: Vec<IndexKey>,
    values: Vec<u64>,
    children: Vec<u32>,
}

struct NodeCursor<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> NodeCursor<'a> {
    const fn new(data: &'a [u8], start: usize) -> Self {
        Self {
            data,
            offset: start,
        }
    }

    fn read_u16(&mut self) -> Result<u16, DbError> {
        let end = self.offset.checked_add(2).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "offset overflow",
            ))
        })?;
        let bytes = self.data.get(self.offset..end).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "not enough bytes for u16",
            ))
        })?;
        self.offset = end;
        Ok(u16::from_le_bytes(bytes.try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid u16: {e}"),
            ))
        })?))
    }

    fn read_u32(&mut self) -> Result<u32, DbError> {
        let end = self.offset.checked_add(4).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "offset overflow",
            ))
        })?;
        let bytes = self.data.get(self.offset..end).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "not enough bytes for u32",
            ))
        })?;
        self.offset = end;
        Ok(u32::from_le_bytes(bytes.try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid u32: {e}"),
            ))
        })?))
    }

    fn read_u64(&mut self) -> Result<u64, DbError> {
        let end = self.offset.checked_add(8).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "offset overflow",
            ))
        })?;
        let bytes = self.data.get(self.offset..end).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "not enough bytes for u64",
            ))
        })?;
        self.offset = end;
        Ok(u64::from_le_bytes(bytes.try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid u64: {e}"),
            ))
        })?))
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], DbError> {
        let end = self.offset.checked_add(len).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "byte length overflow",
            ))
        })?;
        let bytes = self.data.get(self.offset..end).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "not enough bytes",
            ))
        })?;
        self.offset = end;
        Ok(bytes)
    }
}

struct NodeCursorMut<'a> {
    data: &'a mut [u8],
    offset: usize,
}

impl<'a> NodeCursorMut<'a> {
    const fn new(data: &'a mut [u8], start: usize) -> Self {
        Self {
            data,
            offset: start,
        }
    }

    fn write_u16(&mut self, value: u16) -> Result<(), DbError> {
        let end = self
            .offset
            .checked_add(2)
            .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;
        let target = self
            .data
            .get_mut(self.offset..end)
            .ok_or_else(|| DbError::invalid_operation("insufficient space"))?;
        target.copy_from_slice(&value.to_le_bytes());
        self.offset = end;
        Ok(())
    }

    fn write_u32(&mut self, value: u32) -> Result<(), DbError> {
        let end = self
            .offset
            .checked_add(4)
            .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;
        let target = self
            .data
            .get_mut(self.offset..end)
            .ok_or_else(|| DbError::invalid_operation("insufficient space"))?;
        target.copy_from_slice(&value.to_le_bytes());
        self.offset = end;
        Ok(())
    }

    fn write_u64(&mut self, value: u64) -> Result<(), DbError> {
        let end = self
            .offset
            .checked_add(8)
            .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;
        let target = self
            .data
            .get_mut(self.offset..end)
            .ok_or_else(|| DbError::invalid_operation("insufficient space"))?;
        target.copy_from_slice(&value.to_le_bytes());
        self.offset = end;
        Ok(())
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), DbError> {
        let end = self
            .offset
            .checked_add(bytes.len())
            .ok_or_else(|| DbError::invalid_operation("byte length overflow"))?;
        let target = self
            .data
            .get_mut(self.offset..end)
            .ok_or_else(|| DbError::invalid_operation("insufficient space"))?;
        target.copy_from_slice(bytes);
        self.offset = end;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer_pool::BufferPool;
    use crate::pager::Pager;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        std::env::temp_dir().join(format!(
            "monumentum_btree_test_{}_{}.db",
            std::process::id(),
            nanos
        ))
    }

    fn to_u64(i: i64) -> Result<u64, DbError> {
        u64::try_from(i).map_err(|e| DbError::invalid_operation(format!("negative value: {e}")))
    }

    #[test]
    fn test_btree_insert_and_lookup() -> Result<(), DbError> {
        let path = temp_db_path();
        let pager = Pager::open(&path)?;
        let mut buffer_pool = BufferPool::new(pager, 10)?;
        let btree = BTreeOnDisk::create(&mut buffer_pool)?;
        let mut root_id = btree.root_page_id();

        for i in 0..10_i64 {
            let key = IndexKey::Integer(i);
            let value = to_u64(i)?;
            BTreeOnDisk::insert_static(&mut buffer_pool, &mut root_id, key, value)?;
        }

        for i in 0..10_i64 {
            let key = IndexKey::Integer(i);
            let value = to_u64(i)?;
            let result = BTreeOnDisk::lookup_static(&mut buffer_pool, root_id, &key)?;
            assert_eq!(result, Some(value));
        }

        let missing =
            BTreeOnDisk::lookup_static(&mut buffer_pool, root_id, &IndexKey::Integer(99))?;
        assert_eq!(missing, None);

        let _ = fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn test_btree_large_insert_causes_split() -> Result<(), DbError> {
        let path = temp_db_path();
        let pager = Pager::open(&path)?;
        let mut buffer_pool = BufferPool::new(pager, 50)?;
        let btree = BTreeOnDisk::create(&mut buffer_pool)?;
        let mut root_id = btree.root_page_id();

        for i in 0..150_i64 {
            let key = IndexKey::Integer(i);
            let value = to_u64(i)?;
            BTreeOnDisk::insert_static(&mut buffer_pool, &mut root_id, key, value)?;
        }

        for i in 0..150_i64 {
            let key = IndexKey::Integer(i);
            let value = to_u64(i)?;
            let result = BTreeOnDisk::lookup_static(&mut buffer_pool, root_id, &key)?;
            assert_eq!(result, Some(value));
        }

        let _ = fs::remove_file(&path);
        Ok(())
    }
}
