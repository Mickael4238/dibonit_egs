//use std::collections::HashMap;
use crate::*;
// note: the maximum size in byte for the table is:
//    Rcapa * Rsize + 10 bytes (table structure) + columns structs todo(give the max size)
//     = 0xFFFE0001 + 0xA                        +
//     = 0xFFFE000B (included in u32)
pub type Rcapa = u16;           // row capacity (number of rows)
pub type Rsize = u16;           // row size (bytes)

pub type Tid = u16;              // type for table ids
#[derive(Clone, Debug)]
pub struct DBufTable {
    pub table_id: Tid,
    pub columns: Vec<DColumn>,
    pub row_count:    Rcapa,
    pub row_capacity: Rcapa,
    pub row_size: u16,
    pub has_index: bool,
}

impl DBufTable {
    pub fn new(table_id: Tid, columns: Vec<DColumn>, row_capacity: Rcapa, has_index: bool) -> Re<Self> {
        if table_id == 0 { return Err(DErr::RootError(Fe::NewDBufHeader,1,Rc::InvalidDBufId)); }
        if columns.len() > MAX_COLUMN_NUM as usize { return Err(DErr::RootError(Fe::NewDBufHeader,2,Rc::ColumnCountExceeded)); }
        let mut seen_ids = HashMap::new();
        for col in &columns {
            if seen_ids.contains_key(&col.id()) { return Err(DErr::RootError(Fe::NewDBufHeader,3,Rc::DuplicateColumnId(col.id()))); }
            seen_ids.insert(col.id(), ());
        }
        //let row_size = columns.iter().map(|c| c.size_of()).sum();
        let row_size :Rsize = columns.iter().try_fold(0 as Rsize, |acc, c| {
            acc.checked_add(c.size_of())
                .ok_or("Columns size overflowed u8 limit")
        }).map_err(|_| DErr::RootError(Fe::NewDBufHeader,4,Rc::RowSizeExceeded))?;

        Ok(Self { table_id, columns, row_count: 0, row_capacity, row_size, has_index })
    }
    pub(crate) fn from_bytes(p0: &Vec<u8>) -> Re<Self>  {
        todo!()
    }
    pub(crate) fn header_size(&self) -> usize {
        todo!()
    }
    /*
    pub fn header_size(&self) -> usize {
        let mut size = 10;
        if self.has_app_token {
            size += 2 + self.app_token.len();
        }
        if self.has_user_token {
            size += 2 + self.user_token.len();
        }
        for col in &self.columns { size += 2; if col.col_type.requires_size_field() { size += 2; } }
        size
    }


    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let col_count_high = (self.columns.len() as u16 >> 8) as u8 & 0x07;
        let mut byte0 = (ROLLING_VERSION & ROLLING_VERSION_MASK) | ((col_count_high << COL_COUNT_HIGH_SHIFT) & COL_COUNT_HIGH_MASK);
        if self.has_app_token { byte0 |= HAS_APP_TOKEN_MASK; }
        if self.has_user_token { byte0 |= HAS_USER_TOKEN_MASK; }
        if self.has_index { byte0 |= HAS_INDEX_MASK; }
        bytes.push(byte0);
        bytes.push(self.columns.len() as u8);
        bytes.extend_from_slice(&self.d_buf_id.to_be_bytes());
        for col in &self.columns {
            let col_id_high = (col.col_id >> 8) as u8 & 0x07;
            let col_type_id = col.col_type.type_id() & COL_TYPE_MASK;
            bytes.push(((col_id_high << COL_ID_HIGH_SHIFT) & COL_ID_HIGH_MASK) | (col_type_id & COL_TYPE_MASK));
            bytes.push(col.col_id as u8);
            if col.col_type.requires_size_field() {
                if let DColumn::Char(s) | DColumn::Ascii(s) = col.col_type { bytes.extend_from_slice(&(s as u16).to_be_bytes()); }
            }
        }
        bytes.extend_from_slice(&self.row_count.to_be_bytes());
        bytes.extend_from_slice(&self.row_capacity.to_be_bytes());
        bytes.extend_from_slice(&self.row_size.to_be_bytes());
        if self.has_app_token {
            bytes.extend_from_slice(&(self.app_token.len() as u16).to_be_bytes());
            bytes.extend_from_slice(&self.app_token);
        }
        if self.has_user_token {
            bytes.extend_from_slice(&(self.user_token.len() as u16).to_be_bytes());
            bytes.extend_from_slice(&self.user_token);
        }
        bytes
    }
    pub fn from_bytes(bytes: &[u8]) -> Re<DBufHeaderTOTO> {
        if bytes.len() < 10 { return Err(DErr::RootError(Fe::FromBytes,1,Rc::BufferTooSmall)); }
        let byte0 = bytes[0];
        let rolling_version = byte0 & ROLLING_VERSION_MASK;
        if rolling_version > MAX_ROLLING_VERSION { return Err(DErr::RootError(Fe::FromBytes,2,Rc::InvalidRollingVersion(rolling_version))); }
        let col_count_high = (byte0 & COL_COUNT_HIGH_MASK) >> COL_COUNT_HIGH_SHIFT;
        let col_count_low = bytes[1];
        let col_count = ((col_count_high as u16) << 8) | (col_count_low as u16);
        if col_count > MAX_COLUMN_COUNT { return Err(DErr::RootError(Fe::FromBytes,3,Rc::ColumnCountExceeded)); }
        let d_buf_id = u16::from_be_bytes([bytes[2], bytes[3]]);
        if d_buf_id == 0 { return Err(DErr::RootError(Fe::FromBytes,4,Rc::InvalidDBufId)); }
        let mut columns = Vec::with_capacity(col_count as usize);
        let mut offset = 4;
        for _ in 0..col_count {
            if offset + 2 > bytes.len() { return Err(DErr::RootError(Fe::FromBytes,5,Rc::BufferTooSmall)); }
            let byte_col = bytes[offset];
            let col_id_high = (byte_col & COL_ID_HIGH_MASK) >> COL_ID_HIGH_SHIFT;
            let col_type_id = byte_col & COL_TYPE_MASK;
            let col_id_low = bytes[offset + 1];
            let col_id = ((col_id_high as u16) << 8) | (col_id_low as u16);
            if col_id == 0 { return Err(DErr::RootError(Fe::FromBytes,6,Rc::ReservedColumnId)); }
            offset += 2;
            let size = if col_type_id == 12 || col_type_id == 13 {
                if offset + 2 > bytes.len() { return Err(DErr::RootError(Fe::FromBytes,7,Rc::BufferTooSmall)); }
                let s = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize; offset += 2; Some(s)
            } else { None };
            let col_type = DColumn::from_type_id(col_type_id, size)?;
            columns.push(DColumn::new(col_id, col_type )?);
        }
        if offset + 6 > bytes.len() { return Err(DErr::RootError(Fe::FromBytes,8,Rc::BufferTooSmall)); }
        let row_count = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        let row_capacity = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]);
        let row_size = u16::from_be_bytes([bytes[offset + 4], bytes[offset + 5]]);
        if row_count > row_capacity { return Err(DErr::RootError(Fe::FromBytes,9,Rc::RowCountExceedsCapacity)); }
        let calculated_row_size: usize = columns.iter().map(|c| c.size).sum();
        if calculated_row_size != row_size as usize { return Err(DErr::RootError(Fe::FromBytes,10,Rc::RowSizeMismatch)); }

        let has_app_token = (byte0 & HAS_APP_TOKEN_MASK) != 0;
        let has_user_token = (byte0 & HAS_USER_TOKEN_MASK) != 0;
        let has_index = (byte0 & HAS_INDEX_MASK) != 0;

        let mut offset_after_header = offset + 6;
        let mut app_token = Vec::new();
        let mut user_token = Vec::new();

        if has_app_token {
            if offset_after_header + 2 > bytes.len() { return Err(DErr::RootError(Fe::FromBytes,11,Rc::BufferTooSmall)); }
            let app_token_len = u16::from_be_bytes([bytes[offset_after_header], bytes[offset_after_header + 1]]) as usize;
            offset_after_header += 2;
            if offset_after_header + app_token_len > bytes.len() { return Err(DErr::RootError(Fe::FromBytes,12,Rc::BufferTooSmall)); }
            app_token = bytes[offset_after_header..offset_after_header + app_token_len].to_vec();
            offset_after_header += app_token_len;
        }

        if has_user_token {
            if offset_after_header + 2 > bytes.len() { return Err(DErr::RootError(Fe::FromBytes,13,Rc::BufferTooSmall)); }
            let user_token_len = u16::from_be_bytes([bytes[offset_after_header], bytes[offset_after_header + 1]]) as usize;
            offset_after_header += 2;
            if offset_after_header + user_token_len > bytes.len() { return Err(DErr::RootError(Fe::FromBytes,14,Rc::BufferTooSmall)); }
            user_token = bytes[offset_after_header..offset_after_header + user_token_len].to_vec();
        }

        Ok(DBufHeaderTOTO { d_buf_id, columns, row_count, row_capacity, row_size, has_app_token, has_user_token, has_index, app_token, user_token })
    }*/
}
