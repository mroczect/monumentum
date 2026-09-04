use monumentum_handler::error::DbError;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_HEADER_SIZE: usize = 16;
pub const PAGE_BODY_SIZE: usize = PAGE_SIZE - PAGE_HEADER_SIZE;
pub const INITIAL_FREE_SPACE_OFFSET: u16 = 4096;
pub const META_PAGE_ID: u32 = 0;
pub const META_LSN_OFFSET: usize = 0;
pub const META_CATALOG_PAGE_OFFSET: usize = 8;
pub const META_LAST_CHECKPOINT_LSN_OFFSET: usize = 12;
pub const CATALOG_PAGE_HEADER_SIZE: usize = 8;
pub const CATALOG_CHUNK_SIZE: usize = PAGE_BODY_SIZE - CATALOG_PAGE_HEADER_SIZE;
pub const META_CATALOG_PAGE_COUNT_OFFSET: usize = 20;
pub const BTREE_NODE_HEADER_SIZE: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PageType {
    Meta = 0,
    Freelist = 1,
    TableMeta = 2,
    Data = 3,
    Index = 4,
    Overflow = 5,
}

impl TryFrom<u8> for PageType {
    type Error = DbError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Meta),
            1 => Ok(Self::Freelist),
            2 => Ok(Self::TableMeta),
            3 => Ok(Self::Data),
            4 => Ok(Self::Index),
            5 => Ok(Self::Overflow),
            _ => Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid page type",
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageHeader {
    pub page_id: u32,
    pub page_type: PageType,
    pub free_space_offset: u16,
    pub cell_count: u16,
    pub checksum: u32,
    pub flags: u32,
}

impl PageHeader {
    #[must_use]
    pub const fn new(page_id: u32, page_type: PageType) -> Self {
        Self {
            page_id,
            page_type,
            free_space_offset: INITIAL_FREE_SPACE_OFFSET,
            cell_count: 0,
            checksum: 0,
            flags: 0,
        }
    }

    #[must_use]
    pub fn to_bytes(&self) -> [u8; PAGE_HEADER_SIZE] {
        let mut buf = [0u8; PAGE_HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.page_id.to_le_bytes());
        buf[4] = self.page_type as u8;
        buf[5..7].copy_from_slice(&self.free_space_offset.to_le_bytes());
        buf[7..9].copy_from_slice(&self.cell_count.to_le_bytes());
        buf[9..13].copy_from_slice(&self.checksum.to_le_bytes());
        let flags_bytes = self.flags.to_le_bytes();
        buf[13..16].copy_from_slice(&flags_bytes[0..3]);
        buf
    }

    pub fn from_bytes(data: &[u8; PAGE_HEADER_SIZE]) -> Result<Self, DbError> {
        let page_id = u32::from_le_bytes(data[0..4].try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid slice: {e}"),
            ))
        })?);
        let page_type = PageType::try_from(data[4])?;
        let free_space_offset = u16::from_le_bytes(data[5..7].try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid slice: {e}"),
            ))
        })?);
        let cell_count = u16::from_le_bytes(data[7..9].try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid slice: {e}"),
            ))
        })?);
        let checksum = u32::from_le_bytes(data[9..13].try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid slice: {e}"),
            ))
        })?);
        let mut flags_bytes = [0u8; 4];
        flags_bytes[0..3].copy_from_slice(&data[13..16]);
        let flags = u32::from_le_bytes(flags_bytes);

        Ok(Self {
            page_id,
            page_type,
            free_space_offset,
            cell_count,
            checksum,
            flags,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    pub header: PageHeader,
    pub data: [u8; PAGE_BODY_SIZE],
}

impl Page {
    #[must_use]
    pub const fn new(page_id: u32, page_type: PageType) -> Self {
        Self {
            header: PageHeader::new(page_id, page_type),
            data: [0; PAGE_BODY_SIZE],
        }
    }

    #[must_use]
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(PAGE_SIZE);
        buf.extend_from_slice(&self.header.to_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DbError> {
        if bytes.len() != PAGE_SIZE {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid page size",
            )));
        }
        let header_bytes = bytes.get(..PAGE_HEADER_SIZE).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing header",
            ))
        })?;
        let mut header_array = [0u8; PAGE_HEADER_SIZE];
        header_array.copy_from_slice(header_bytes);
        let header = PageHeader::from_bytes(&header_array)?;

        let data_bytes = bytes.get(PAGE_HEADER_SIZE..).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing body",
            ))
        })?;
        let mut data = [0u8; PAGE_BODY_SIZE];
        data.copy_from_slice(data_bytes);

        Ok(Self { header, data })
    }
}
