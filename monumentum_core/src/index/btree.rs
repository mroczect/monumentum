use crate::buffer_pool::BufferPool;
use crate::index::key::IndexKey;
use crate::page::{Page, PageType};
use monumentum_handler::error::DbError;
use std::cmp::Ordering;

const MAX_KEYS_PER_NODE: usize = 100; 

pub struct BTree {
    buffer_pool: BufferPool,
    root_page_id: u32,
    next_page_id: u32,
}

struct Node {
    page: Page,
    is_leaf: bool,
    keys: Vec<IndexKey>,
    values: Vec<u64>,       
    children: Vec<u32>,     
    parent_page_id: u32,
}

impl BTree {
    pub fn new(mut buffer_pool: BufferPool) -> Result<Self, DbError> {
        let root_page_id = buffer_pool.allocate_page(PageType::Index)?;
        {
            let root_page = buffer_pool.get_page(root_page_id)?;
            root_page.header.page_type = PageType::Index;
            root_page.data[0] = 1; 
            root_page.data[1..3].copy_from_slice(&0u16.to_le_bytes());
            root_page.data[3..7].copy_from_slice(&u32::MAX.to_le_bytes()); 
        }
        buffer_pool.unpin_page(root_page_id, true)?;

        Ok(Self {
            buffer_pool,
            root_page_id,
            next_page_id: root_page_id + 1,
        })
    }

    pub fn lookup(&mut self, key: &IndexKey) -> Result<Option<u64>, DbError> {
        let mut node = self.load_node(self.root_page_id)?;
        loop {
            if node.is_leaf {
                match node.keys.binary_search(key) {
                    Ok(idx) => return Ok(Some(node.values[idx])),
                    Err(_) => return Ok(None),
                }
            }
            let idx = node.keys.partition_point(|k| k < key);
            let child_page_id = node.children[idx];
            node = self.load_node(child_page_id)?;
        }
    }

    pub fn insert(&mut self, key: IndexKey, value: u64) -> Result<(), DbError> {
        let root = self.load_node(self.root_page_id)?;
        if root.keys.len() >= MAX_KEYS_PER_NODE {
            let new_root_page_id = self.allocate_node(true)?; 
            let old_root = root;
            let mut new_root = self.load_node(new_root_page_id)?;
            new_root.is_leaf = false;
            new_root.children.push(self.root_page_id);
            self.split_child(&mut new_root, 0)?;
            self.root_page_id = new_root_page_id;
            let mut old_root_node = self.load_node(self.root_page_id)?;
        }
        self.insert_non_full(self.root_page_id, key, value)
    }

    fn insert_non_full(&mut self, page_id: u32, key: IndexKey, value: u64) -> Result<(), DbError> {
        let mut node = self.load_node(page_id)?;
        if node.is_leaf {
            let idx = node.keys.binary_search(&key).map_or_else(|i| i, |_| return Err(DbError::constraint_violation(
                monumentum_handler::error::ErrorKind::UniqueViolation,
                "duplicate key",
                None,
                None,
            )));
            node.keys.insert(idx, key);
            node.values.insert(idx, value);
            self.save_node(&node)?;
            Ok(())
        } else {
            let idx = node.keys.partition_point(|k| k < &key);
            let child_page_id = node.children[idx];
            let child = self.load_node(child_page_id)?;
            if child.keys.len() >= MAX_KEYS_PER_NODE {
                self.split_child(&mut node, idx)?;
                let new_idx = node.keys.partition_point(|k| k < &key);
                return self.insert_non_full(node.children[new_idx], key, value);
            }
            self.insert_non_full(child_page_id, key, value)
        }
    }

    fn split_child(&mut self, parent: &mut Node, child_idx: usize) -> Result<(), DbError> {
        let child_page_id = parent.children[child_idx];
        let mut child = self.load_node(child_page_id)?;

        let mid = child.keys.len() / 2;
        let mid_key = child.keys[mid].clone();

        let new_child_page_id = self.allocate_node(child.is_leaf)?;
        let mut new_child = self.load_node(new_child_page_id)?;
        new_child.is_leaf = child.is_leaf;
        new_child.parent_page_id = parent.page.header.page_id;

        if child.is_leaf {
            new_child.keys = child.keys.split_off(mid);
            new_child.values = child.values.split_off(mid);
        } else {
            new_child.keys = child.keys.split_off(mid + 1);
            new_child.children = child.children.split_off(mid + 1);
        }

        parent.keys.insert(child_idx, mid_key);
        parent.children.insert(child_idx + 1, new_child_page_id);

        self.save_node(&child)?;
        self.save_node(&new_child)?;
        self.save_node(parent)?;
        Ok(())
    }

    fn load_node(&mut self, page_id: u32) -> Result<Node, DbError> {
        let page = self.buffer_pool.get_page(page_id)?;
        let is_leaf = page.data[0] == 1;
        let num_keys = u16::from_le_bytes([page.data[1], page.data[2]]) as usize;
        let parent_page_id = u32::from_le_bytes(page.data[3..7].try_into().unwrap_or([0;4]));

        let mut offset = 7;
        let mut keys = Vec::with_capacity(num_keys);
        let mut values = Vec::with_capacity(num_keys);
        let mut children = Vec::with_capacity(if is_leaf { 0 } else { num_keys + 1 });

        for _ in 0..num_keys {
            let key_len = u16::from_le_bytes(page.data[offset..offset+2].try_into().unwrap_or([0;2])) as usize;
            offset += 2;
            let key_bytes = page.data.get(offset..offset+key_len).ok_or_else(|| DbError::corruption(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad key length")))?;
            offset += key_len;
            let key = IndexKey::from_bytes(key_bytes)?;
            if is_leaf {
                let value = u64::from_le_bytes(page.data[offset..offset+8].try_into().unwrap_or([0;8]));
                offset += 8;
                values.push(value);
            } else {
                let child = u32::from_le_bytes(page.data[offset..offset+4].try_into().unwrap_or([0;4]));
                offset += 4;
                children.push(child);
            }
            keys.push(key);
        }
        if !is_leaf {
            let child = u32::from_le_bytes(page.data[offset..offset+4].try_into().unwrap_or([0;4]));
            children.push(child);
        }

        let node = Node {
            page: page.clone(),
            is_leaf,
            keys,
            values,
            children,
            parent_page_id,
        };
        self.buffer_pool.unpin_page(page_id, false)?;
        Ok(node)
    }

    fn save_node(&mut self, node: &Node) -> Result<(), DbError> {
        let page = self.buffer_pool.get_page(node.page.header.page_id)?;
        page.data.fill(0);
        page.data[0] = u8::from(node.is_leaf);
        page.data[1..3].copy_from_slice(&(node.keys.len() as u16).to_le_bytes());
        page.data[3..7].copy_from_slice(&node.parent_page_id.to_le_bytes());

        let mut offset = 7;
        for (i, key) in node.keys.iter().enumerate() {
            let key_bytes = key.to_bytes()?;
            let key_len = key_bytes.len() as u16;
            page.data[offset..offset+2].copy_from_slice(&key_len.to_le_bytes());
            offset += 2;
            page.data[offset..offset+key_bytes.len()].copy_from_slice(&key_bytes);
            offset += key_bytes.len();
            if node.is_leaf {
                let val = node.values[i];
                page.data[offset..offset+8].copy_from_slice(&val.to_le_bytes());
                offset += 8;
            } else {
                let child = node.children[i];
                page.data[offset..offset+4].copy_from_slice(&child.to_le_bytes());
                offset += 4;
            }
        }
        if !node.is_leaf {
            let last_child = node.children.last().copied().unwrap_or(u32::MAX);
            page.data[offset..offset+4].copy_from_slice(&last_child.to_le_bytes());
        }

        self.buffer_pool.unpin_page(node.page.header.page_id, true)?;
        Ok(())
    }

    fn allocate_node(&mut self, is_leaf: bool) -> Result<u32, DbError> {
        let page_id = self.buffer_pool.allocate_page(PageType::Index)?;
        let page = self.buffer_pool.get_page(page_id)?;
        page.header.page_type = PageType::Index;
        page.data[0] = u8::from(is_leaf);
        page.data[1..3].copy_from_slice(&0u16.to_le_bytes());
        page.data[3..7].copy_from_slice(&u32::MAX.to_le_bytes());
        self.buffer_pool.unpin_page(page_id, true)?;
        Ok(page_id)
    }
}
