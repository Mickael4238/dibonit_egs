use std::fmt;
use std::any::Any;
use std::collections::HashMap;
use d_err_lib::DErr;

const ROLLING_VERSION: u8 = 0;
const MAX_ROLLING_VERSION: u8 = 3;
const MAX_COLUMN_COUNT: u16 = 2047;
const MAX_COLUMN_ID: u16 = 2047;
const MAX_RECURSION_DEPTH: u8 = 16;
const ROLLING_VERSION_MASK: u8 = 0x03;
const COL_COUNT_HIGH_MASK: u8 = 0x1C;
const COL_COUNT_HIGH_SHIFT: u8 = 2;
const COL_ID_HIGH_MASK: u8 = 0xE0;
const COL_ID_HIGH_SHIFT: u8 = 5;
const COL_TYPE_MASK: u8 = 0x1F;
const HAS_APP_TOKEN_MASK: u8 = 0x20;
const HAS_USER_TOKEN_MASK: u8 = 0x40;
const HAS_INDEX_MASK: u8 = 0x80;

type Re<T> = Result<T, DErr<Fe, Rc>>;

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Boolean, U8, U16, U32, U64, I8, I16, I32, I64, F32, F64, UtcTime, Char(usize), Ascii(usize), Child,
}

impl ColumnType {
    pub fn type_id(&self) -> u8 {
        match self {
            ColumnType::Boolean => 0, ColumnType::U8 => 1, ColumnType::U16 => 2,
            ColumnType::U32 => 3, ColumnType::U64 => 4, ColumnType::I8 => 5,
            ColumnType::I16 => 6, ColumnType::I32 => 7, ColumnType::I64 => 8,
            ColumnType::F32 => 9, ColumnType::F64 => 10, ColumnType::UtcTime => 11,
            ColumnType::Char(_) => 12, ColumnType::Ascii(_) => 13, ColumnType::Child => 14,
        }
    }
    pub fn size_of(&self) -> usize {
        match self {
            ColumnType::Boolean | ColumnType::U8 | ColumnType::I8 => 1,
            ColumnType::U16 | ColumnType::I16 => 2,
            ColumnType::U32 | ColumnType::I32 | ColumnType::F32 => 4,
            ColumnType::U64 | ColumnType::I64 | ColumnType::F64 | ColumnType::UtcTime => 8,
            ColumnType::Char(s) | ColumnType::Ascii(s) => *s,
            ColumnType::Child => 6,
        }
    }
    pub fn requires_size_field(&self) -> bool {
        matches!(self, ColumnType::Char(_) | ColumnType::Ascii(_))
    }
    pub fn is_child(&self) -> bool { matches!(self, ColumnType::Child) }
    pub fn from_type_id(type_id: u8, size: Option<usize>) -> Re<ColumnType> {
        match type_id {
            0 => Ok(ColumnType::Boolean), 1 => Ok(ColumnType::U8), 2 => Ok(ColumnType::U16),
            3 => Ok(ColumnType::U32), 4 => Ok(ColumnType::U64), 5 => Ok(ColumnType::I8),
            6 => Ok(ColumnType::I16), 7 => Ok(ColumnType::I32), 8 => Ok(ColumnType::I64),
            9 => Ok(ColumnType::F32), 10 => Ok(ColumnType::F64), 11 => Ok(ColumnType::UtcTime),
            12 => match size { Some(0) => Err(DErr::RootError(Fe::FromTypeId,1,Rc::InvalidColumnSize)), Some(s) => Ok(ColumnType::Char(s)), None => Err(DErr::RootError(Fe::FromTypeId,2,Rc::MissingSizeForVariableType)) },
            13 => match size { Some(0) => Err(DErr::RootError(Fe::FromTypeId,1,Rc::InvalidColumnSize)), Some(s) => Ok(ColumnType::Ascii(s)), None => Err(DErr::RootError(Fe::FromTypeId,2,Rc::MissingSizeForVariableType)) },
            14 => Ok(ColumnType::Child),
            _ => Err(DErr::RootError(Fe::FromTypeId,3,Rc::InvalidColumnType(type_id))),
        }
    }
}

pub trait Downcast: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
impl<T: Any + Send + Sync> Downcast for T {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub trait IntoDBuf: Downcast + Sync {
    fn d_buf_id(&self) -> u16;
    fn columns(&self) -> Vec<ColumnHeader>;
    fn to_row_data(&self) -> Vec<u8>;
    fn children(&self) -> Vec<&dyn IntoDBuf>;
    fn child_column_id(&self) -> Option<u16>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnHeader { pub col_id: u16, pub col_type: ColumnType }

impl ColumnHeader {
    pub fn new(col_id: u16, col_type: ColumnType) -> Re<ColumnHeader> {
        if col_id == 0 { return Err(DErr::RootError(Fe::NewColumnHeader,1,Rc::ReservedColumnId)); }
        if col_id > MAX_COLUMN_ID { return Err(DErr::RootError(Fe::NewColumnHeader,2,Rc::ColumnIdExceeded)); }
        Ok(ColumnHeader { col_id, col_type })
    }
}

#[derive(Clone, Debug)]
pub struct Column { pub header: ColumnHeader, pub size: usize }
impl Column { pub fn new(header: ColumnHeader) -> Column { Column { header: header.clone(), size: header.col_type.size_of() } } }

#[derive(Clone, Debug)]
pub struct DBufHeader {
    pub d_buf_id: u16,
    pub columns: Vec<Column>,
    pub row_count: u16,
    pub row_capacity: u16,
    pub row_size: u16,
    pub has_app_token: bool,
    pub has_user_token: bool,
    pub has_index: bool,
    pub app_token: Vec<u8>,
    pub user_token: Vec<u8>,
}

impl DBufHeader {
    pub fn header_size(&self) -> usize {
        let mut size = 10;
        if self.has_app_token {
            size += 2 + self.app_token.len();
        }
        if self.has_user_token {
            size += 2 + self.user_token.len();
        }
        for col in &self.columns { size += 2; if col.header.col_type.requires_size_field() { size += 2; } }
        size
    }
    pub fn new(d_buf_id: u16, columns: Vec<Column>, row_capacity: u16, has_app_token: bool, has_user_token: bool, has_index: bool, app_token: Vec<u8>, user_token: Vec<u8>) -> Re<DBufHeader> {
        if d_buf_id == 0 { return Err(DErr::RootError(Fe::NewDBufHeader,1,Rc::InvalidDBufId)); }
        if columns.len() > MAX_COLUMN_COUNT as usize { return Err(DErr::RootError(Fe::NewDBufHeader,2,Rc::ColumnCountExceeded)); }
        let mut seen_ids = HashMap::new();
        for col in &columns {
            if seen_ids.contains_key(&col.header.col_id) { return Err(DErr::RootError(Fe::NewDBufHeader,3,Rc::DuplicateColumnId(col.header.col_id))); }
            seen_ids.insert(col.header.col_id, ());
        }
        let row_size: usize = columns.iter().map(|c| c.size).sum();
        if row_size > u16::MAX as usize { return Err(DErr::RootError(Fe::NewDBufHeader,4,Rc::RowSizeExceeded)); }
        if has_app_token && app_token.len() > u16::MAX as usize { return Err(DErr::RootError(Fe::NewDBufHeader,5,Rc::TokenTooLarge)); }
        if has_user_token && user_token.len() > u16::MAX as usize { return Err(DErr::RootError(Fe::NewDBufHeader,6,Rc::TokenTooLarge)); }
        Ok(DBufHeader { d_buf_id, columns, row_count: 0, row_capacity, row_size: row_size as u16, has_app_token, has_user_token, has_index, app_token, user_token })
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
            let col_id_high = (col.header.col_id >> 8) as u8 & 0x07;
            let col_type_id = col.header.col_type.type_id() & COL_TYPE_MASK;
            bytes.push(((col_id_high << COL_ID_HIGH_SHIFT) & COL_ID_HIGH_MASK) | (col_type_id & COL_TYPE_MASK));
            bytes.push(col.header.col_id as u8);
            if col.header.col_type.requires_size_field() {
                if let ColumnType::Char(s) | ColumnType::Ascii(s) = col.header.col_type { bytes.extend_from_slice(&(s as u16).to_be_bytes()); }
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
    pub fn from_bytes(bytes: &[u8]) -> Re<DBufHeader> {
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
            let col_type = ColumnType::from_type_id(col_type_id, size)?;
            columns.push(Column::new(ColumnHeader { col_id, col_type }));
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
        
        Ok(DBufHeader { d_buf_id, columns, row_count, row_capacity, row_size, has_app_token, has_user_token, has_index, app_token, user_token })
    }
}

#[derive(Clone, Debug)]
pub struct DBuf { pub header: DBufHeader, pub raw_values: Vec<u8> }

impl DBuf {
    pub fn new(d_buf_id: u16, columns: Vec<ColumnHeader>, row_capacity: u16) -> Re<DBuf> {
        let cols: Vec<Column> = columns.into_iter().map(Column::new).collect();
        let header = DBufHeader::new(d_buf_id, cols, row_capacity, false, false, false, Vec::new(), Vec::new())?;
        let mut raw_values = header.to_bytes();
        raw_values.reserve(header.row_capacity as usize * header.row_size as usize);
        Ok(DBuf { header, raw_values })
    }
    pub fn new_with_tokens(d_buf_id: u16, columns: Vec<ColumnHeader>, row_capacity: u16, has_app_token: bool, has_user_token: bool, has_index: bool, app_token: Vec<u8>, user_token: Vec<u8>) -> Re<DBuf> {
        let cols: Vec<Column> = columns.into_iter().map(Column::new).collect();
        let header = DBufHeader::new(d_buf_id, cols, row_capacity, has_app_token, has_user_token, has_index, app_token, user_token)?;
        let mut raw_values = header.to_bytes();
        raw_values.reserve(header.row_capacity as usize * header.row_size as usize);
        Ok(DBuf { header, raw_values })
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Re<DBuf> {
        let header = DBufHeader::from_bytes(&bytes)?;
        let header_size = header.header_size();
        if bytes.len() < header_size { return Err(DErr::RootError(Fe::FromBytesDBuf,1,Rc::BufferTooSmall)); }
        let expected_data_size = header.row_count as usize * header.row_size as usize;
        let actual_data_size = bytes.len() - header_size;
        if actual_data_size < expected_data_size { return Err(DErr::RootError(Fe::FromBytesDBuf,2,Rc::BufferTooSmall)); }
        Ok(DBuf { header, raw_values: bytes })
    }
    pub fn build_dbuff_from_rawdata(bytes: Vec<u8>) -> Re<DBuf> {
        DBuf::from_bytes(bytes)
    }
    
    pub fn compare_structure(
        &self,
        other: &DBuf,
        check_id: bool,
        check_user_token: bool,
        check_app_token: bool,
        check_children: bool,
        all_columns_are_checked: bool,
    ) -> Re<()> {
        // Check d_buf_id if requested
        if check_id && self.header.d_buf_id != other.header.d_buf_id {
            return Err(DErr::RootError(Fe::CompareStructure, 1, Rc::DBufIdMismatch));
        }

        // Check app_token if requested
        if check_app_token {
            if self.header.has_app_token != other.header.has_app_token {
                return Err(DErr::RootError(Fe::CompareStructure, 2, Rc::ColumnTypeMismatch));
            }
            if self.header.has_app_token && self.header.app_token != other.header.app_token {
                return Err(DErr::RootError(Fe::CompareStructure, 3, Rc::ColumnTypeMismatch));
            }
        }

        // Check user_token if requested
        if check_user_token {
            if self.header.has_user_token != other.header.has_user_token {
                return Err(DErr::RootError(Fe::CompareStructure, 4, Rc::ColumnTypeMismatch));
            }
            if self.header.has_user_token && self.header.user_token != other.header.user_token {
                return Err(DErr::RootError(Fe::CompareStructure, 5, Rc::ColumnTypeMismatch));
            }
        }

        // Check columns: ensure all columns in other exist in self (other can't have more info)
        for col_other in &other.header.columns {
            let col_self_opt = self
                .header
                .columns
                .iter()
                .find(|c| c.header.col_id == col_other.header.col_id);
            
            if let Some(col_self) = col_self_opt {
                // Check column type
                if col_self.header.col_type != col_other.header.col_type {
                    return Err(DErr::RootError(Fe::CompareStructure, 6, Rc::ColumnTypeMismatch));
                }
                // Check column size
                if col_self.size != col_other.size {
                    return Err(DErr::RootError(Fe::CompareStructure, 7, Rc::ColumnSizeMismatch));
                }
            } else {
                // Column in other not found in self - always error
                return Err(DErr::RootError(Fe::CompareStructure, 8, Rc::ExtraColumnInSecondDBuf));
            }
        }

        // If all_columns_are_checked, ensure self doesn't have extra columns
        if all_columns_are_checked {
            for col_self in &self.header.columns {
                if !other
                    .header
                    .columns
                    .iter()
                    .any(|c| c.header.col_id == col_self.header.col_id)
                {
                    return Err(DErr::RootError(Fe::CompareStructure, 9, Rc::MissingColumnInSecondDBuf));
                }
            }
        }

        // Check children if requested
        if check_children {
            // Find child columns in both
            let child_col_self = self
                .header
                .columns
                .iter()
                .find(|c| c.header.col_type.is_child());
            let child_col_other = other
                .header
                .columns
                .iter()
                .find(|c| c.header.col_type.is_child());

            if let Some(col_self) = child_col_self {
                if let Some(col_other) = child_col_other {
                    if col_self.header.col_id != col_other.header.col_id {
                        return Err(DErr::RootError(Fe::CompareStructure, 10, Rc::ColumnTypeMismatch));
                    }
                    // Compare children for each row
                    for row_idx in 0..self.header.row_count.min(other.header.row_count) {
                        let child_self_result = self.get_child(row_idx, col_self.header.col_id);
                        let child_other_result = other.get_child(row_idx, col_other.header.col_id);
                        
                        if let (Ok(child_self), Ok(child_other)) = (child_self_result, child_other_result) {
                            child_self.compare_structure(
                                &child_other,
                                check_id,
                                check_user_token,
                                check_app_token,
                                check_children,
                                all_columns_are_checked,
                            )?;
                        }
                    }
                }
            } else if child_col_other.is_some() {
                // other has child column but self doesn't
                return Err(DErr::RootError(Fe::CompareStructure, 11, Rc::ExtraColumnInSecondDBuf));
            }
        }

        Ok(())
    }
    pub fn header_size(&self) -> usize { self.header.header_size() }
    pub fn data_offset(&self) -> usize { self.header_size() }
    pub fn add_rows(&mut self, rows_data: &[u8]) -> Re<()> {
        if rows_data.len() % self.header.row_size as usize != 0 { return Err(DErr::RootError(Fe::AddRows,1,Rc::DataSizeMismatch)); }
        let num_rows = rows_data.len() / self.header.row_size as usize;
        if self.header.row_count + num_rows as u16 > self.header.row_capacity { return Err(DErr::RootError(Fe::AddRows,2,Rc::CapacityExceeded)); }
        let current_data_len = self.raw_values.len().saturating_sub(self.data_offset());
        let new_data_len = current_data_len + rows_data.len();
        let max_data_len = self.header.row_capacity as usize * self.header.row_size as usize;
        if new_data_len > max_data_len { return Err(DErr::RootError(Fe::AddRows,3,Rc::CapacityExceeded)); }
        self.raw_values.extend_from_slice(rows_data);
        self.header.row_count += num_rows as u16;
        let header_bytes = self.header.to_bytes();
        self.raw_values[..header_bytes.len()].copy_from_slice(&header_bytes);
        Ok(())
    }
    pub fn get_row(&self, row_index: u16) -> Re<Vec<u8>> {
        if row_index >= self.header.row_count { return Err(DErr::RootError(Fe::GetRow,1,Rc::IndexOutOfBounds)); }
        let start = self.data_offset() + (row_index as usize * self.header.row_size as usize);
        let end = start + self.header.row_size as usize;
        if end > self.raw_values.len() { return Err(DErr::RootError(Fe::GetRow,2,Rc::BufferTooSmall)); }
        Ok(self.raw_values[start..end].to_vec())
    }
    pub fn get_col(&self, row_index: u16, col_id: u16) -> Re<Vec<u8>> {
        if row_index >= self.header.row_count { return Err(DErr::RootError(Fe::GetCol,1,Rc::IndexOutOfBounds)); }
        let col = self.header.columns.iter().find(|c| c.header.col_id == col_id).ok_or_else(|| DErr::RootError(Fe::GetCol,2,Rc::ColumnNotFound(col_id)))?;
        let row_start = self.data_offset() + (row_index as usize * self.header.row_size as usize);
        let mut col_offset = 0;
        for c in &self.header.columns { if c.header.col_id == col_id { break; } col_offset += c.size; }
        let start = row_start + col_offset; let end = start + col.size;
        if end > self.raw_values.len() { return Err(DErr::RootError(Fe::GetCol,3,Rc::BufferTooSmall)); }
        Ok(self.raw_values[start..end].to_vec())
    }
    pub fn get_d_buf_id(&self) -> u16 { self.header.d_buf_id }
    pub fn get_column_metadata(&self, col_id: u16) -> Re<(&ColumnHeader, usize)> {
        let col = self.header.columns.iter().find(|c| c.header.col_id == col_id).ok_or_else(|| DErr::RootError(Fe::GetColumnMetadata,1,Rc::ColumnNotFound(col_id)))?;
        Ok((&col.header, col.size))
    }
    pub fn get_all_rows(&self) -> Vec<Vec<u8>> { (0..self.header.row_count).filter_map(|i| self.get_row(i).ok()).collect() }
    pub fn into_raw_bytes(self) -> Vec<u8> { self.raw_values }
    pub fn as_raw_bytes(&self) -> &[u8] { &self.raw_values }
    pub fn iter_columns(&self) -> impl Iterator<Item = &Column> { self.header.columns.iter() }
    pub fn find_column(&self, col_id: u16) -> Re<&Column> {
        self.header.columns.iter().find(|c| c.header.col_id == col_id).ok_or_else(|| DErr::RootError(Fe::FindColumn,1,Rc::ColumnNotFound(col_id)))
    }
    pub fn pretty_print(&self) -> String {
        self.pretty_print_with_depth(0)
    }
    fn pretty_print_with_depth(&self, depth: usize) -> String {
        let mut result = String::new();
        let tabs = "\t".repeat(depth);
        
        result.push_str(&format!("{}Header:\n", tabs));
        result.push_str(&format!("{}\tDBuf ID: {}\n", tabs, self.header.d_buf_id));
        result.push_str(&format!("{}\tRow Count: {}\n", tabs, self.header.row_count));
        result.push_str(&format!("{}\tRow Capacity: {}\n", tabs, self.header.row_capacity));
        result.push_str(&format!("{}\tRow Size: {}\n", tabs, self.header.row_size));
        result.push_str(&format!("{}\tColumns:\n", tabs));
        
        for col in &self.header.columns {
            let col_type_str = match &col.header.col_type {
                ColumnType::Boolean => "Boolean".to_string(),
                ColumnType::U8 => "U8".to_string(),
                ColumnType::U16 => "U16".to_string(),
                ColumnType::U32 => "U32".to_string(),
                ColumnType::U64 => "U64".to_string(),
                ColumnType::I8 => "I8".to_string(),
                ColumnType::I16 => "I16".to_string(),
                ColumnType::I32 => "I32".to_string(),
                ColumnType::I64 => "I64".to_string(),
                ColumnType::F32 => "F32".to_string(),
                ColumnType::F64 => "F64".to_string(),
                ColumnType::UtcTime => "UtcTime".to_string(),
                ColumnType::Char(s) => format!("Char({})", s),
                ColumnType::Ascii(s) => format!("Ascii({})", s),
                ColumnType::Child => "Child".to_string(),
            };
            let col_id_char = char::from_u32(col.header.col_id as u32).unwrap_or('?');
            result.push_str(&format!("{}\t\tID: {} ('{}'), Type: {}, Size: {}\n", tabs, col.header.col_id, col_id_char, col_type_str, col.size));
        }
        
        result.push_str(&format!("\n{}Data:\n", tabs));
        
        for row_idx in 0..self.header.row_count {
            let row_data = self.get_row(row_idx).unwrap();
            let mut col_offset = 0;
            let mut row_values = Vec::new();
            
            for col in &self.header.columns {
                let end = col_offset + col.size;
                let cell_data = &row_data[col_offset..end];
                
                let cell_str = match &col.header.col_type {
                    ColumnType::Boolean => format!("{}", cell_data[0] != 0),
                    ColumnType::U8 => format!("{}", cell_data[0]),
                    ColumnType::U16 => format!("{}", u16::from_be_bytes([cell_data[0], cell_data[1]])),
                    ColumnType::U32 => format!("{}", u32::from_be_bytes([cell_data[0], cell_data[1], cell_data[2], cell_data[3]])),
                    ColumnType::U64 => format!("{}", u64::from_be_bytes([cell_data[0], cell_data[1], cell_data[2], cell_data[3], cell_data[4], cell_data[5], cell_data[6], cell_data[7]])),
                    ColumnType::I8 => format!("{}", cell_data[0] as i8),
                    ColumnType::I16 => format!("{}", i16::from_be_bytes([cell_data[0], cell_data[1]])),
                    ColumnType::I32 => format!("{}", i32::from_be_bytes([cell_data[0], cell_data[1], cell_data[2], cell_data[3]])),
                    ColumnType::I64 => format!("{}", i64::from_be_bytes([cell_data[0], cell_data[1], cell_data[2], cell_data[3], cell_data[4], cell_data[5], cell_data[6], cell_data[7]])),
                    ColumnType::F32 => format!("{}", f32::from_be_bytes([cell_data[0], cell_data[1], cell_data[2], cell_data[3]])),
                    ColumnType::F64 => format!("{}", f64::from_be_bytes([cell_data[0], cell_data[1], cell_data[2], cell_data[3], cell_data[4], cell_data[5], cell_data[6], cell_data[7]])),
                    ColumnType::UtcTime => format!("{}", u64::from_be_bytes([cell_data[0], cell_data[1], cell_data[2], cell_data[3], cell_data[4], cell_data[5], cell_data[6], cell_data[7]])),
                    ColumnType::Char(_) | ColumnType::Ascii(_) => {
                        let s = String::from_utf8_lossy(cell_data);
                        format!("\"{}\"", s.trim_end_matches(char::from(0)))
                    },
                    ColumnType::Child => {
                        let child_header_offset = u16::from_be_bytes([cell_data[0], cell_data[1]]);
                        let first_child_row = u16::from_be_bytes([cell_data[2], cell_data[3]]);
                        let last_child_row = u16::from_be_bytes([cell_data[4], cell_data[5]]);
                        format!("Child(offset={}, first_row={}, last_row={})", child_header_offset, first_child_row, last_child_row)
                    },
                };
                row_values.push(cell_str);
                col_offset = end;
            }
            
            result.push_str(&format!("{}\tRow {}: {}\n", tabs, row_idx, row_values.join("\t")));
            
            for col in &self.header.columns {
                if col.header.col_type.is_child() {
                    let child_result = self.get_child(row_idx, col.header.col_id);
                    if let Ok(child_dbuf) = child_result {
                        result.push_str(&format!("\n"));
                        result.push_str(&child_dbuf.pretty_print_with_depth(depth + 1));
                    }
                }
            }
        }
        
        result
    }
    pub fn add_row_with_child(&mut self, row_data: Vec<u8>, child_d_buf: DBuf, child_col_id: u16, first_child_row: u16, last_child_row: u16) -> Re<()> {
        if row_data.len() != self.header.row_size as usize { return Err(DErr::RootError(Fe::AddRowWithChild,1,Rc::DataSizeMismatch)); }
        let child_col = self.find_column(child_col_id)?;
        if !child_col.header.col_type.is_child() { return Err(DErr::RootError(Fe::AddRowWithChild,2,Rc::ColumnNotChild(child_col_id))); }
        let child_bytes = child_d_buf.into_raw_bytes();
        let data_start = self.data_offset();
        let new_row_start = data_start + (self.header.row_count as usize * self.header.row_size as usize);
        let child_abs_offset = new_row_start + self.header.row_size as usize;
        let child_header_offset_u16 = child_abs_offset as u16;
        let mut final_row_data = row_data;
        let mut col_offset = 0;
        for col in &self.header.columns {
            if col.header.col_id == child_col_id {
                final_row_data[col_offset..col_offset+2].copy_from_slice(&child_header_offset_u16.to_be_bytes());
                final_row_data[col_offset+2..col_offset+4].copy_from_slice(&first_child_row.to_be_bytes());
                final_row_data[col_offset+4..col_offset+6].copy_from_slice(&last_child_row.to_be_bytes());
                break;
            }
            col_offset += col.size;
        }
        self.raw_values.extend_from_slice(&final_row_data);
        self.raw_values.extend_from_slice(&child_bytes);
        self.header.row_count += 1;
        let header_bytes = self.header.to_bytes();
        self.raw_values[..header_bytes.len()].copy_from_slice(&header_bytes);
        Ok(())
    }
    pub fn get_child(&self, row_index: u16, child_col_id: u16) -> Re<DBuf> {
        if row_index >= self.header.row_count { return Err(DErr::RootError(Fe::GetChild,1,Rc::IndexOutOfBounds)); }
        let child_col = self.find_column(child_col_id)?;
        if !child_col.header.col_type.is_child() { return Err(DErr::RootError(Fe::GetChild,2,Rc::ColumnNotChild(child_col_id))); }
        let row_start = self.data_offset() + (row_index as usize * self.header.row_size as usize);
        let mut col_offset = 0;
        for col in &self.header.columns {
            if col.header.col_id == child_col_id { break; }
            col_offset += col.size;
        }
        let child_data_start = row_start + col_offset;
        let child_data = &self.raw_values[child_data_start..child_data_start+6];
        let child_header_offset = u16::from_be_bytes([child_data[0], child_data[1]]);
        let _first_child_row_index = u16::from_be_bytes([child_data[2], child_data[3]]);
        let _last_child_row_index = u16::from_be_bytes([child_data[4], child_data[5]]);
        let absolute_child_offset = child_header_offset as usize;
        if absolute_child_offset >= self.raw_values.len() { return Err(DErr::RootError(Fe::GetChild,3,Rc::InvalidChildOffset(child_header_offset))); }
        let child_header = DBufHeader::from_bytes(&self.raw_values[absolute_child_offset..])?;
        let child_row_count = child_header.row_count;
        let child_data_size = child_row_count as usize * child_header.row_size as usize;
        let child_total_size = child_header.header_size() + child_data_size;
        let child_end = absolute_child_offset + child_total_size;
        if child_end > self.raw_values.len() { return Err(DErr::RootError(Fe::GetChild,4,Rc::InvalidChildOffset(child_header_offset))); }
        let child_bytes = self.raw_values[absolute_child_offset..child_end].to_vec();
        DBuf::from_bytes(child_bytes)
    }
    pub fn get_child_typed<T: IntoDBuf + 'static + Clone>(&self, row_index: u16, child_col_id: u16, expected_d_buf_id: u16) -> Re<T> {
        let child_d_buf = self.get_child(row_index, child_col_id)?;
        if child_d_buf.header.d_buf_id != expected_d_buf_id {
            return Err(DErr::RootError(Fe::GetChild,5,Rc::DBufNotFound(expected_d_buf_id)));
        }
        let child_any = child_d_buf;
        let child_any_ref = &child_any as &dyn Any;
        match child_any_ref.downcast_ref::<T>() {
            Some(t) => Ok(t.clone()),
            None => Err(DErr::RootError(Fe::GetChild,6,Rc::DBufNotFound(expected_d_buf_id))),
        }
    }
}

pub fn build_d_buf_from_structure<T: IntoDBuf + 'static + Clone>(data: &[T], max_depth: u8) -> Re<DBuf> {
    if max_depth > MAX_RECURSION_DEPTH { return Err(DErr::RootError(Fe::MaxDepthExceeded,1,Rc::MaxDepthExceeded)); }
    let root = data.first().ok_or_else(|| DErr::RootError(Fe::DBufNotFound,1,Rc::BufferTooSmall))?;
    let _d_buf_id = root.d_buf_id();
    let _columns = root.columns();
    let _child_col_id = root.child_column_id();
    let all_items: Vec<&dyn IntoDBuf> = data.iter().map(|x| x as &dyn IntoDBuf).collect();
    let mut depth = 0u8;
    let all_d_buf_defs = collect_d_buf_definitions(&all_items, &mut depth, max_depth)?;
    let root_d_buf = fill_d_bufs(&all_d_buf_defs, &all_items, 0, &mut depth, max_depth)?;
    Ok(root_d_buf)
}

fn collect_d_buf_definitions(items: &[&dyn IntoDBuf], current_depth: &mut u8, max_depth: u8) -> Re<Vec<DBufDefinition>> {
    *current_depth += 1;
    if *current_depth > max_depth { return Err(DErr::RootError(Fe::MaxDepthExceeded,2,Rc::MaxDepthExceeded)); }
    let mut definitions = Vec::new();
    let mut seen_ids = HashMap::new();
    for item in items {
        let d_buf_id = item.d_buf_id();
        if seen_ids.contains_key(&d_buf_id) { continue; }
        seen_ids.insert(d_buf_id, true);
        let columns = item.columns();
        let child_col_id = item.child_column_id();
        let mut child_definitions = Vec::new();
        let children: Vec<&dyn IntoDBuf> = item.children();
        if !children.is_empty() {
            let mut child_depth = *current_depth;
            let child_defs = collect_d_buf_definitions(&children, &mut child_depth, max_depth)?;
            child_definitions.extend(child_defs);
        }
        let capacity = items.iter().filter(|&&x| x.d_buf_id() == d_buf_id).count() as u16;
        definitions.push(DBufDefinition { d_buf_id, columns, capacity, child_col_id, child_definitions });
    }
    *current_depth -= 1;
    Ok(definitions)
}

#[derive(Debug, Clone)]
struct DBufDefinition {
    d_buf_id: u16,
    columns: Vec<ColumnHeader>,
    capacity: u16,
    child_col_id: Option<u16>,
    child_definitions: Vec<DBufDefinition>,
}

fn fill_d_bufs(definitions: &[DBufDefinition], items: &[&dyn IntoDBuf], def_index: usize, current_depth: &mut u8, max_depth: u8) -> Re<DBuf> {
    *current_depth += 1;
    if *current_depth > max_depth { return Err(DErr::RootError(Fe::MaxDepthExceeded,3,Rc::MaxDepthExceeded)); }
    let def = &definitions[def_index];
    let mut d_buf = DBuf::new(def.d_buf_id, def.columns.clone(), def.capacity)?;
    let relevant_items: Vec<&dyn IntoDBuf> = items.iter().filter(|&&x| x.d_buf_id() == def.d_buf_id).cloned().collect();
    let mut child_d_bufs: HashMap<u16, DBuf> = HashMap::new();
    for (_child_idx, child_def) in def.child_definitions.iter().enumerate() {
        let child_items: Vec<&dyn IntoDBuf> = relevant_items.iter()
            .flat_map(|x| x.children())
            .filter(|c| c.d_buf_id() == child_def.d_buf_id)
            .collect();
        if child_items.is_empty() { continue; }
        let child_d_buf = fill_d_bufs(definitions, &child_items, 0, current_depth, max_depth)?;
        child_d_bufs.insert(child_def.d_buf_id, child_d_buf);
    }
    let mut rows_to_add = Vec::new();
    let mut child_bytes_map: HashMap<u16, Vec<u8>> = HashMap::new();
    
    for item in &relevant_items {
        let row_data = item.to_row_data();
        let children: Vec<&dyn IntoDBuf> = item.children();
        let mut final_row_data = row_data.clone();
        if let Some(child_col_id) = def.child_col_id {
            if !children.is_empty() {
                let first_child = children.first().unwrap();
                let child_d_buf = child_d_bufs.get(&first_child.d_buf_id()).ok_or_else(|| DErr::RootError(Fe::DBufNotFound,2,Rc::DBufNotFound(first_child.d_buf_id())))?;
                let child_bytes = child_d_buf.clone().into_raw_bytes();
                let child_header_offset = (d_buf.raw_values.len() + (d_buf.header.row_count as usize + rows_to_add.len() + 1) * d_buf.header.row_size as usize) as u16;
                let child_row_count = child_d_buf.header.row_count;
                let last_idx = if child_row_count > 0 { child_row_count - 1 } else { 0 };
                let mut col_offset = 0;
                for col in &d_buf.header.columns {
                    if col.header.col_id == child_col_id {
                        final_row_data[col_offset..col_offset+2].copy_from_slice(&child_header_offset.to_be_bytes());
                        final_row_data[col_offset+2..col_offset+4].copy_from_slice(&0u16.to_be_bytes());
                        final_row_data[col_offset+4..col_offset+6].copy_from_slice(&last_idx.to_be_bytes());
                        break;
                    }
                    col_offset += col.size;
                }
                child_bytes_map.insert(first_child.d_buf_id(), child_bytes);
            }
        }
        rows_to_add.push(final_row_data);
    }
    
    for row_data in &rows_to_add {
        d_buf.add_rows(row_data)?;
    }
    
    for (_child_id, child_bytes) in &child_bytes_map {
        d_buf.raw_values.extend_from_slice(child_bytes);
    }
    *current_depth -= 1;
    Ok(d_buf)
}

#[derive(Debug)]
pub enum Rc {
    DuplicateColumnId(u16), ReservedColumnId, InvalidColumnType(u8), InvalidColumnSize, ColumnIdExceeded, InvalidDBufId, ColumnCountExceeded, RowSizeExceeded, MissingSizeForVariableType,
    CapacityExceeded, RowCapacityZero, DataSizeMismatch, IndexOutOfBounds, ColumnNotFound(u16), BufferTooSmall,
    HeaderCorruption, RowCountExceedsCapacity, RowSizeMismatch, InvalidRollingVersion(u8),
    InvalidChildOffset(u16), ColumnNotChild(u16), MaxDepthExceeded, DBufNotFound(u16), TokenTooLarge,
    DBufIdMismatch, ColumnTypeMismatch, ColumnSizeMismatch, ExtraColumnInSecondDBuf, MissingColumnInSecondDBuf,
}

impl fmt::Display for Rc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rc::DuplicateColumnId(id) => write!(f, "DuplicateColumnId({})", id),
            Rc::ReservedColumnId => write!(f, "ReservedColumnId"),
            Rc::InvalidColumnType(id) => write!(f, "InvalidColumnType({})", id),
            Rc::InvalidColumnSize => write!(f, "InvalidColumnSize"),
            Rc::ColumnIdExceeded => write!(f, "ColumnIdExceeded"),
            Rc::InvalidDBufId => write!(f, "InvalidDBufId"),
            Rc::ColumnCountExceeded => write!(f, "ColumnCountExceeded"),
            Rc::RowSizeExceeded => write!(f, "RowSizeExceeded"),
            Rc::MissingSizeForVariableType => write!(f, "MissingSizeForVariableType"),
            Rc::CapacityExceeded => write!(f, "CapacityExceeded"),
            Rc::RowCapacityZero => write!(f, "RowCapacityZero"),
            Rc::DataSizeMismatch => write!(f, "DataSizeMismatch"),
            Rc::IndexOutOfBounds => write!(f, "IndexOutOfBounds"),
            Rc::ColumnNotFound(id) => write!(f, "ColumnNotFound({})", id),
            Rc::BufferTooSmall => write!(f, "BufferTooSmall"),
            Rc::HeaderCorruption => write!(f, "HeaderCorruption"),
            Rc::RowCountExceedsCapacity => write!(f, "RowCountExceedsCapacity"),
            Rc::RowSizeMismatch => write!(f, "RowSizeMismatch"),
            Rc::InvalidRollingVersion(v) => write!(f, "InvalidRollingVersion({})", v),
            Rc::InvalidChildOffset(o) => write!(f, "InvalidChildOffset({})", o),
            Rc::ColumnNotChild(id) => write!(f, "ColumnNotChild({})", id),
            Rc::MaxDepthExceeded => write!(f, "MaxDepthExceeded"),
            Rc::DBufNotFound(id) => write!(f, "DBufNotFound({})", id),
            Rc::TokenTooLarge => write!(f, "TokenTooLarge"),
            Rc::DBufIdMismatch => write!(f, "DBufIdMismatch"),
            Rc::ColumnTypeMismatch => write!(f, "ColumnTypeMismatch"),
            Rc::ColumnSizeMismatch => write!(f, "ColumnSizeMismatch"),
            Rc::ExtraColumnInSecondDBuf => write!(f, "ExtraColumnInSecondDBuf"),
            Rc::MissingColumnInSecondDBuf => write!(f, "MissingColumnInSecondDBuf"),
        }
    }
}

#[derive(Debug)]
pub enum Fe {
    NewDBuf, NewColumnHeader, NewDBufHeader, FromTypeId, FromBytes, FromBytesDBuf, AddRows, AddRowWithChild,
    GetRow, GetCol, GetChild, GetColumnMetadata, FindColumn, MaxDepthExceeded, DBufNotFound,
    CompareStructure,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c() -> Vec<ColumnHeader> { vec![ColumnHeader::new(1, ColumnType::U8).unwrap(), ColumnHeader::new(2, ColumnType::U16).unwrap(), ColumnHeader::new(3, ColumnType::U32).unwrap()] }
    fn d() -> DBuf { DBuf::new(1, c(), 100).unwrap() }
    #[test] fn t1() { assert_eq!(ColumnType::Boolean.size_of(), 1); assert_eq!(ColumnType::Child.size_of(), 6); }
    #[test] fn t2() { assert_eq!(ColumnType::Child.type_id(), 14); }
    #[test] fn t3() { assert!(ColumnType::Child.is_child()); }
    #[test] fn t4() { assert!(!ColumnType::Child.requires_size_field()); }
    #[test] fn t5() { assert_eq!(ColumnType::from_type_id(14, None).unwrap(), ColumnType::Child); }
    #[test] fn t6() { assert!(ColumnHeader::new(4, ColumnType::Child).is_ok()); }
    #[test] fn t7() { let cols: Vec<Column> = c().into_iter().map(Column::new).collect(); let h = DBufHeader::new(1, cols, 100, false, false, false, Vec::new(), Vec::new()).unwrap(); assert_eq!(h.row_size, 7); }
    #[test] fn t8() { let cols = vec![Column::new(ColumnHeader::new(1, ColumnType::U32).unwrap()), Column::new(ColumnHeader::new(2, ColumnType::Child).unwrap())]; let h = DBufHeader::new(1, cols, 100, false, false, false, Vec::new(), Vec::new()).unwrap(); assert_eq!(h.row_size, 10); }
    #[test] fn t9() { let d = d(); assert_eq!(d.header.d_buf_id, 1); }
    #[test] fn t23_token_flags_in_header() {
        let cols: Vec<Column> = c().into_iter().map(Column::new).collect();
        let app_token = vec![1u8, 2, 3, 4];
        let user_token = vec![5u8, 6, 7, 8, 9];
        let h = DBufHeader::new(1, cols, 100, true, true, true, app_token.clone(), user_token.clone()).unwrap();
        assert_eq!(h.has_app_token, true);
        assert_eq!(h.has_user_token, true);
        assert_eq!(h.has_index, true);
        assert_eq!(h.app_token, app_token);
        assert_eq!(h.user_token, user_token);
    }
    #[test] fn t24_token_serialization() {
        let cols: Vec<Column> = c().into_iter().map(Column::new).collect();
        let app_token = vec![1u8, 2, 3, 4];
        let user_token = vec![5u8, 6, 7, 8, 9];
        let h = DBufHeader::new(1, cols, 100, true, true, false, app_token.clone(), user_token.clone()).unwrap();
        let bytes = h.to_bytes();
        let h2 = DBufHeader::from_bytes(&bytes).unwrap();
        assert_eq!(h2.has_app_token, true);
        assert_eq!(h2.has_user_token, true);
        assert_eq!(h2.has_index, false);
        assert_eq!(h2.app_token, app_token);
        assert_eq!(h2.user_token, user_token);
    }
    #[test] fn t25_no_tokens() {
        let cols: Vec<Column> = c().into_iter().map(Column::new).collect();
        let h = DBufHeader::new(1, cols, 100, false, false, false, Vec::new(), Vec::new()).unwrap();
        let bytes = h.to_bytes();
        let h2 = DBufHeader::from_bytes(&bytes).unwrap();
        assert_eq!(h2.has_app_token, false);
        assert_eq!(h2.has_user_token, false);
        assert_eq!(h2.has_index, false);
        assert_eq!(h2.app_token.len(), 0);
        assert_eq!(h2.user_token.len(), 0);
    }
    #[test] fn t26_only_app_token() {
        let cols: Vec<Column> = c().into_iter().map(Column::new).collect();
        let app_token = vec![10u8, 20, 30];
        let h = DBufHeader::new(1, cols, 100, true, false, false, app_token.clone(), Vec::new()).unwrap();
        let bytes = h.to_bytes();
        let h2 = DBufHeader::from_bytes(&bytes).unwrap();
        assert_eq!(h2.has_app_token, true);
        assert_eq!(h2.has_user_token, false);
        assert_eq!(h2.app_token, app_token);
    }
    #[test] fn t10() { let mut d = d(); d.add_rows(&vec![1,0,1,0,0,1,0]).unwrap(); assert_eq!(d.header.row_count, 1); }
    #[test] fn t11() { let mut d = d(); let rd = vec![1,0,1,0,0,1,0]; d.add_rows(&rd).unwrap(); assert_eq!(d.get_row(0).unwrap(), rd); }
    #[test] fn t12() { let mut d = d(); d.add_rows(&vec![1,0,1,0,0,1,0]).unwrap(); assert_eq!(d.get_col(0,1).unwrap(), vec![1]); assert_eq!(d.get_col(0,2).unwrap(), vec![0,1]); }
    #[test] fn t13() { let mut d = d(); let rd = vec![1,0,1,0,0,1,0]; d.add_rows(&rd).unwrap(); let hb = d.header.to_bytes(); d.raw_values[..hb.len()].copy_from_slice(&hb); let r = DBuf::from_bytes(d.raw_values.clone()).unwrap(); assert_eq!(r.header.row_count, 1); }
    #[test] fn t14() { let cols = vec![ColumnHeader::new(1, ColumnType::U32).unwrap(), ColumnHeader::new(2, ColumnType::Child).unwrap()]; let mut p = DBuf::new(1, cols, 10).unwrap(); let ccols = vec![ColumnHeader::new(1, ColumnType::U32).unwrap()]; let mut c = DBuf::new(2, ccols, 5).unwrap(); c.add_rows(&vec![100,0,0,0]).unwrap(); let prd = vec![1,0,0,0, 0,0,0,0,0,0]; p.add_row_with_child(prd, c, 2, 0, 0).unwrap(); assert_eq!(p.header.row_count, 1); }
    #[test] fn t15() { 
        let cols = vec![ColumnHeader::new(1, ColumnType::U32).unwrap(), ColumnHeader::new(2, ColumnType::Child).unwrap()]; 
        let mut p = DBuf::new(1, cols, 10).unwrap(); 
        let ccols = vec![ColumnHeader::new(1, ColumnType::U32).unwrap()]; 
        let mut c = DBuf::new(2, ccols, 5).unwrap(); 
        c.add_rows(&vec![100,0,0,0]).unwrap(); 
        let prd = vec![1,0,0,0, 0,0,0,0,0,0]; 
        p.add_row_with_child(prd, c, 2, 0, 0).unwrap(); 
        let rc = p.get_child(0, 2).unwrap(); 
        assert_eq!(rc.header.d_buf_id, 2); 
        assert_eq!(rc.header.row_count, 1); 
        assert_eq!(rc.header.row_size, 4);
    }
    #[test] fn t16() { let d = d(); assert!(d.get_child(0, 1).is_err()); }
    struct SP { id: u32, name: String }
    impl IntoDBuf for SP {
        fn d_buf_id(&self) -> u16 { 100 }
        fn columns(&self) -> Vec<ColumnHeader> { vec![ColumnHeader::new(1, ColumnType::U32).unwrap(), ColumnHeader::new(2, ColumnType::Char(50)).unwrap()] }
        fn to_row_data(&self) -> Vec<u8> { let mut b = self.id.to_be_bytes().to_vec(); b.extend_from_slice(self.name.as_bytes()); while b.len() < 54 { b.push(0); } b }
        fn children(&self) -> Vec<&dyn IntoDBuf> { Vec::new() }
        fn child_column_id(&self) -> Option<u16> { None }
    }
    
    #[derive(Clone)]
    struct Product { id: u32, name: String, price: f32 }
    impl IntoDBuf for Product {
        fn d_buf_id(&self) -> u16 { 1000 }
        fn columns(&self) -> Vec<ColumnHeader> { 
            vec![
                ColumnHeader::new(1, ColumnType::U32).unwrap(),
                ColumnHeader::new(2, ColumnType::Char(50)).unwrap(),
                ColumnHeader::new(3, ColumnType::F32).unwrap(),
            ]
        }
        fn to_row_data(&self) -> Vec<u8> { 
            let mut b = self.id.to_be_bytes().to_vec();
            b.extend_from_slice(self.name.as_bytes());
            while b.len() < 54 { b.push(0); }
            b.extend_from_slice(&self.price.to_be_bytes());
            b
        }
        fn children(&self) -> Vec<&dyn IntoDBuf> { Vec::new() }
        fn child_column_id(&self) -> Option<u16> { None }
    }
    
    #[derive(Clone)]
    struct Order { id: u32, date: u64, products: Vec<Product> }
    impl IntoDBuf for Order {
        fn d_buf_id(&self) -> u16 { 1001 }
        fn columns(&self) -> Vec<ColumnHeader> { 
            vec![
                ColumnHeader::new(1, ColumnType::U32).unwrap(),
                ColumnHeader::new(2, ColumnType::U64).unwrap(),
                ColumnHeader::new(3, ColumnType::Child).unwrap(),
            ]
        }
        fn to_row_data(&self) -> Vec<u8> { 
            let mut b = self.id.to_be_bytes().to_vec();
            b.extend_from_slice(&self.date.to_be_bytes());
            b.extend_from_slice(&[0u8; 6]);
            b
        }
        fn children(&self) -> Vec<&dyn IntoDBuf> { 
            self.products.iter().map(|p| p as &dyn IntoDBuf).collect()
        }
        fn child_column_id(&self) -> Option<u16> { Some(3) }
    }
    
    #[derive(Clone)]
    struct User { id: u32, name: String, orders: Vec<Order> }
    impl IntoDBuf for User {
        fn d_buf_id(&self) -> u16 { 1002 }
        fn columns(&self) -> Vec<ColumnHeader> { 
            vec![
                ColumnHeader::new(1, ColumnType::U32).unwrap(),
                ColumnHeader::new(2, ColumnType::Char(50)).unwrap(),
                ColumnHeader::new(3, ColumnType::Child).unwrap(),
            ]
        }
        fn to_row_data(&self) -> Vec<u8> { 
            let mut b = self.id.to_be_bytes().to_vec();
            b.extend_from_slice(self.name.as_bytes());
            while b.len() < 54 { b.push(0); }
            b.extend_from_slice(&[0u8; 6]);
            b
        }
        fn children(&self) -> Vec<&dyn IntoDBuf> { 
            self.orders.iter().map(|o| o as &dyn IntoDBuf).collect()
        }
        fn child_column_id(&self) -> Option<u16> { Some(3) }
    }
    
    #[test] fn t17_single_level_hierarchy() {
        let products = vec![
            Product { id: 1, name: "Laptop".to_string(), price: 999.99 },
            Product { id: 2, name: "Mouse".to_string(), price: 25.50 },
        ];
        let d_buf = build_d_buf_from_structure(&products, 16).unwrap();
        assert_eq!(d_buf.header.d_buf_id, 1000);
        assert_eq!(d_buf.header.row_count, 2);
        assert_eq!(d_buf.header.row_size, 58);
    }
    
    #[test] fn t18_multi_level_hierarchy() {
        let products = vec![
            Product { id: 1, name: "Laptop".to_string(), price: 999.99 },
        ];
        let orders = vec![
            Order { id: 1, date: 1234567890, products: products.clone() },
        ];
        let users = vec![
            User { id: 1, name: "Alice".to_string(), orders: orders.clone() },
        ];
        let d_buf = build_d_buf_from_structure(&users, 16).unwrap();
        assert_eq!(d_buf.header.d_buf_id, 1002);
        assert_eq!(d_buf.header.row_count, 1);
    }
    
    #[test] fn t19_empty_children() {
        let users = vec![
            User { id: 1, name: "Bob".to_string(), orders: vec![] },
        ];
        let d_buf = build_d_buf_from_structure(&users, 16).unwrap();
        assert_eq!(d_buf.header.d_buf_id, 1002);
        assert_eq!(d_buf.header.row_count, 1);
    }
    
    #[test] fn t20_multiple_children() {
        let productsAB = vec![
            Product { id: 1, name: "A".to_string(), price: 1.0 },
            Product { id: 2, name: "B".to_string(), price: 2.0 },
        ];

        let productsCD = vec![
            Product { id: 3, name: "C".to_string(), price: 1.0 },
            Product { id: 4, name: "D".to_string(), price: 2.0 },
        ];

        let productsEF = vec![
            Product { id: 1, name: "A".to_string(), price: 1.0 },
            Product { id: 2, name: "B".to_string(), price: 2.0 },
        ];

        let productsGH = vec![
            Product { id: 3, name: "C".to_string(), price: 1.0 },
            Product { id: 4, name: "D".to_string(), price: 2.0 },
        ];
        let orders1 = vec![
            Order { id: 1, date: 100, products: productsAB.clone() },
            Order { id: 2, date: 200, products: productsCD.clone() },
        ];

        let orders2 = vec![
            Order { id: 3, date: 100, products: productsEF.clone() },
            Order { id: 4, date: 200, products: productsGH.clone() },
        ];
        let users = vec![
            User { id: 1, name: "Charlie".to_string(),orders: orders1},
            User { id: 2, name: "Charlie".to_string(),orders:  orders2 },
        ];
        let d_buf = build_d_buf_from_structure(&users, 16).unwrap();
        assert_eq!(d_buf.header.d_buf_id, 1002);
        assert_eq!(d_buf.header.row_count, 2);
        
        let raw_data = d_buf.clone().into_raw_bytes();
        let d_buf_from_raw = DBuf::build_dbuff_from_rawdata(raw_data).unwrap();
        assert_eq!(d_buf.header.d_buf_id, d_buf_from_raw.header.d_buf_id);
        assert_eq!(d_buf.header.row_count, d_buf_from_raw.header.row_count);
        assert_eq!(d_buf.header.row_size, d_buf_from_raw.header.row_size);
        assert_eq!(d_buf.header.columns.len(), d_buf_from_raw.header.columns.len());
        
        for (orig_col, new_col) in d_buf.header.columns.iter().zip(d_buf_from_raw.header.columns.iter()) {
            assert_eq!(orig_col.header.col_id, new_col.header.col_id);
            assert_eq!(orig_col.header.col_type, new_col.header.col_type);
        }
        
        let pretty = d_buf.pretty_print();
        let path = std::path::Path::new("testresults/t20_multiple_children.txt");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, pretty).unwrap();
    }
    
    #[test] fn t21_max_depth_validation() {
        let products = vec![Product { id: 1, name: "X".to_string(), price: 1.0 }];
        let result = build_d_buf_from_structure(&products, 0);
        assert!(result.is_err());
        
        let products2 = vec![Product { id: 1, name: "Y".to_string(), price: 2.0 }];
        let d_buf = build_d_buf_from_structure(&products2, 16).unwrap();
        
        let raw_data = d_buf.clone().into_raw_bytes();
        let d_buf_from_raw = DBuf::build_dbuff_from_rawdata(raw_data).unwrap();
        assert_eq!(d_buf.header.d_buf_id, d_buf_from_raw.header.d_buf_id);
        assert_eq!(d_buf.header.row_count, d_buf_from_raw.header.row_count);
        
        let pretty = d_buf.pretty_print();
        let path = std::path::Path::new("testresults/t21_max_depth_validation.txt");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, pretty).unwrap();
    }
    
    #[test] fn t22_get_child_typed() {
        let cols = vec![ColumnHeader::new(1, ColumnType::U32).unwrap(), ColumnHeader::new(2, ColumnType::Child).unwrap()];
        let mut p = DBuf::new(1, cols, 10).unwrap();
        let ccols = vec![ColumnHeader::new(1, ColumnType::U32).unwrap()];
        let mut c = DBuf::new(2, ccols, 5).unwrap();
        c.add_rows(&vec![100,0,0,0]).unwrap();
        let prd = vec![1,0,0,0, 0,0,0,0,0,0];
        p.add_row_with_child(prd, c, 2, 0, 0).unwrap();
        let rc = p.get_child(0, 2).unwrap();
        assert_eq!(rc.header.d_buf_id, 2);
        assert_eq!(rc.header.row_count, 1);
        
        let raw_data = p.clone().into_raw_bytes();
        let p_from_raw = DBuf::build_dbuff_from_rawdata(raw_data).unwrap();
        assert_eq!(p.header.d_buf_id, p_from_raw.header.d_buf_id);
        assert_eq!(p.header.row_count, p_from_raw.header.row_count);
        
        let pretty = p.pretty_print();
        let path = std::path::Path::new("testresults/t22_get_child_typed.txt");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, pretty).unwrap();
    }
    
    #[test] fn t27_compare_structure_basic() {
        let cols = vec![
            ColumnHeader::new(1, ColumnType::U32).unwrap(),
            ColumnHeader::new(2, ColumnType::U16).unwrap(),
        ];
        let d1 = DBuf::new(100, cols.clone(), 10).unwrap();
        let d2 = DBuf::new(100, cols, 10).unwrap();
        
        assert!(d1.compare_structure(&d2, true, true, true, true, true).is_ok());
    }
    
    #[test] fn t28_compare_structure_id_mismatch() {
        let cols = vec![
            ColumnHeader::new(1, ColumnType::U32).unwrap(),
        ];
        let d1 = DBuf::new(100, cols.clone(), 10).unwrap();
        let d2 = DBuf::new(200, cols, 10).unwrap();
        
        assert!(d1.compare_structure(&d2, true, true, true, true, true).is_err());
        assert!(d1.compare_structure(&d2, false, true, true, true, true).is_ok());
    }
    
    #[test] fn t29_compare_structure_extra_column() {
        let cols1 = vec![
            ColumnHeader::new(1, ColumnType::U32).unwrap(),
        ];
        let cols2 = vec![
            ColumnHeader::new(1, ColumnType::U32).unwrap(),
            ColumnHeader::new(2, ColumnType::U16).unwrap(),
        ];
        let d1 = DBuf::new(100, cols1, 10).unwrap();
        let d2 = DBuf::new(100, cols2, 10).unwrap();
        
        // d2 has extra column - should always fail
        assert!(d1.compare_structure(&d2, true, true, true, true, true).is_err());
    }
    
    #[test] fn t30_compare_structure_missing_column() {
        let cols1 = vec![
            ColumnHeader::new(1, ColumnType::U32).unwrap(),
            ColumnHeader::new(2, ColumnType::U16).unwrap(),
        ];
        let cols2 = vec![
            ColumnHeader::new(1, ColumnType::U32).unwrap(),
        ];
        let d1 = DBuf::new(100, cols1, 10).unwrap();
        let d2 = DBuf::new(100, cols2, 10).unwrap();
        
        // With all_columns_are_checked=true, should fail
        assert!(d1.compare_structure(&d2, true, true, true, true, true).is_err());
        // With all_columns_are_checked=false, should succeed
        assert!(d1.compare_structure(&d2, true, true, true, true, false).is_ok());
    }
    
    #[test] fn t31_compare_structure_column_type_mismatch() {
        let cols1 = vec![
            ColumnHeader::new(1, ColumnType::U32).unwrap(),
        ];
        let cols2 = vec![
            ColumnHeader::new(1, ColumnType::U16).unwrap(),
        ];
        let d1 = DBuf::new(100, cols1, 10).unwrap();
        let d2 = DBuf::new(100, cols2, 10).unwrap();
        
        assert!(d1.compare_structure(&d2, true, true, true, true, true).is_err());
    }
}
