//use std::collections::HashMap;
use crate::*;
#[derive(Clone, Debug)]
pub struct DBufHeaderTOTO {
    pub d_buf_id: u16,
    pub columns: Vec<ColumnHeaderTOTO>,
    pub row_count: u16,
    pub row_capacity: u16,
    pub row_size: u16,
    pub has_app_token: bool,
    pub has_user_token: bool,
    pub has_index: bool,
    pub app_token: Vec<u8>,
    pub user_token: Vec<u8>,
}

impl DBufHeaderTOTO {
    pub(crate) fn build_child_headers<T: ToDBufTOTO>(&self, p0: &&mut DBuftBuilderTOTO, p1: &&T) {
        todo!()
    }
}

impl DBufHeaderTOTO {
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
    pub fn new(d_buf_id: u16, columns: Vec<ColumnHeaderTOTO>, row_capacity: u16, has_app_token: bool, has_user_token: bool, has_index: bool, app_token: Vec<u8>, user_token: Vec<u8>) -> Re<DBufHeaderTOTO> {
        if d_buf_id == 0 { return Err(DErr::RootError(Fe::NewDBufHeader,1,Rc::InvalidDBufId)); }
        if columns.len() > MAX_COLUMN_COUNT as usize { return Err(DErr::RootError(Fe::NewDBufHeader,2,Rc::ColumnCountExceeded)); }
        let mut seen_ids = HashMap::new();
        for col in &columns {
            if seen_ids.contains_key(&col.col_id) { return Err(DErr::RootError(Fe::NewDBufHeader,3,Rc::DuplicateColumnId(col.col_id))); }
            seen_ids.insert(col.col_id, ());
        }
        let row_size: usize = columns.iter().map(|c| c.size).sum();
        if row_size > u16::MAX as usize { return Err(DErr::RootError(Fe::NewDBufHeader,4,Rc::RowSizeExceeded)); }
        if has_app_token && app_token.len() > u16::MAX as usize { return Err(DErr::RootError(Fe::NewDBufHeader,5,Rc::TokenTooLarge)); }
        if has_user_token && user_token.len() > u16::MAX as usize { return Err(DErr::RootError(Fe::NewDBufHeader,6,Rc::TokenTooLarge)); }
        Ok(DBufHeaderTOTO { d_buf_id, columns, row_count: 0, row_capacity, row_size: row_size as u16, has_app_token, has_user_token, has_index, app_token, user_token })
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
                if let ColumnTypeTOTO::Char(s) | ColumnTypeTOTO::Ascii(s) = col.col_type { bytes.extend_from_slice(&(s as u16).to_be_bytes()); }
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
            let col_type = ColumnTypeTOTO::from_type_id(col_type_id, size)?;
            columns.push(ColumnHeaderTOTO::new(col_id, col_type )?);
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
    }
}

