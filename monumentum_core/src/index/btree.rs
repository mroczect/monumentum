use crate::buffer_pool::BufferPool;
use crate::index::key::IndexKey;
use crate::page::{BTREE_NODE_HEADER_SIZE, PageType};
use monumentum_handler::error::DbError;

const MAX_KEYS_PER_NODE: usize = 100;

#[derive(Debug)]
pub struct BTreeOnDisk {
    buffer_pool: BufferPool,
    root_page_id: u32,
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

impl BTreeOnDisk {
    pub fn new(mut buffer_pool: BufferPool) -> Result<Self, DbError> {
        let root_page_id = buffer_pool.allocate_page(PageType::Index)?;
        {
            let page = buffer_pool.get_page(root_page_id)?;
            page.header.page_type = PageType::Index;
            page.data[0] = 1;
            page.data[1..3].copy_from_slice(&0u16.to_le_bytes());
            page.data[3..7].copy_from_slice(&u32::MAX.to_le_bytes());
        }
        buffer_pool.unpin_page(root_page_id, true)?;

        Ok(Self {
            buffer_pool,
            root_page_id,
        })
    }

    pub fn lookup(&mut self, key: &IndexKey) -> Result<Option<u64>, DbError> {
        let mut node = self.load_node(self.root_page_id)?;
        loop {
            if node.is_leaf {
                return match node.keys.binary_search(key) {
                    Ok(idx) => {
                        let value = node.values.get(idx).ok_or_else(|| {
                            DbError::corruption(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "value missing at index",
                            ))
                        })?;
                        Ok(Some(*value))
                    }
                    Err(_) => Ok(None),
                };
            }

            let idx = node.keys.partition_point(|k| k < key);
            let child_page_id = *node.children.get(idx).ok_or_else(|| {
                DbError::corruption(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "child page missing",
                ))
            })?;
            node = self.load_node(child_page_id)?;
        }
    }

    pub fn insert(&mut self, key: IndexKey, value: u64) -> Result<(), DbError> {
        let root = self.load_node(self.root_page_id)?;
        if root.keys.len() >= MAX_KEYS_PER_NODE {
            self.split_root()?;
        }
        self.insert_non_full(self.root_page_id, key, value)
    }

    fn split_root(&mut self) -> Result<(), DbError> {
        let new_root_page_id = self.allocate_node(false)?;
        let old_root_page_id = self.root_page_id;

        {
            let mut new_root = self.load_node(new_root_page_id)?;
            new_root.children.push(old_root_page_id);
            self.save_node(&new_root)?;
        }

        self.split_child(new_root_page_id, 0)?;
        self.root_page_id = new_root_page_id;
        Ok(())
    }

    fn split_child(&mut self, parent_page_id: u32, child_idx: usize) -> Result<(), DbError> {
        let mut parent = self.load_node(parent_page_id)?;
        let child_page_id = *parent
            .children
            .get(child_idx)
            .ok_or_else(|| DbError::invalid_operation("child index out of bounds"))?;
        let mut child = self.load_node(child_page_id)?;

        let mid = child.keys.len() / 2;
        let mid_key = child
            .keys
            .get(mid)
            .ok_or_else(|| DbError::invalid_operation("mid key missing"))?
            .clone();

        let new_child_page_id = self.allocate_node(child.is_leaf)?;
        let mut new_child = self.load_node(new_child_page_id)?;
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

        self.save_node(&child)?;
        self.save_node(&new_child)?;
        self.save_node(&parent)?;
        Ok(())
    }

    fn insert_non_full(&mut self, page_id: u32, key: IndexKey, value: u64) -> Result<(), DbError> {
        let mut node = self.load_node(page_id)?;
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
            self.save_node(&node)?;
            Ok(())
        } else {
            let idx = node.keys.partition_point(|k| k < &key);
            let child_page_id = *node
                .children
                .get(idx)
                .ok_or_else(|| DbError::invalid_operation("child index out of bounds"))?;
            let child = self.load_node(child_page_id)?;
            if child.keys.len() >= MAX_KEYS_PER_NODE {
                self.split_child(page_id, idx)?;
                let parent_after = self.load_node(page_id)?;
                let new_idx = parent_after.keys.partition_point(|k| k < &key);
                let new_child_page_id = *parent_after.children.get(new_idx).ok_or_else(|| {
                    DbError::invalid_operation("child index out of bounds after split")
                })?;
                return self.insert_non_full(new_child_page_id, key, value);
            }
            self.insert_non_full(child_page_id, key, value)
        }
    }

    fn allocate_node(&mut self, is_leaf: bool) -> Result<u32, DbError> {
        let page_id = self.buffer_pool.allocate_page(PageType::Index)?;
        {
            let page = self.buffer_pool.get_page(page_id)?;
            page.header.page_type = PageType::Index;
            page.data[0] = u8::from(is_leaf);
            page.data[1..3].copy_from_slice(&0u16.to_le_bytes());
            page.data[3..7].copy_from_slice(&u32::MAX.to_le_bytes());
        }
        self.buffer_pool.unpin_page(page_id, true)?;
        Ok(page_id)
    }

    fn load_node(&mut self, page_id: u32) -> Result<Node, DbError> {
        let page = self.buffer_pool.get_page(page_id)?;
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

        self.buffer_pool.unpin_page(page_id, false)?;
        Ok(Node {
            page_id,
            is_leaf,
            parent_page_id,
            keys,
            values,
            children,
        })
    }

    fn save_node(&mut self, node: &Node) -> Result<(), DbError> {
        let page = self.buffer_pool.get_page(node.page_id)?;
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

        self.buffer_pool.unpin_page(node.page_id, true)?;
        Ok(())
    }
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
