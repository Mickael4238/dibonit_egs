use std::fmt;
use d_err_lib::DErr;

/// Current rolling version of DBuf format
const ROLLING_VERSION: u8 = 0;
/// Maximum supported rolling version (we can handle versions 0-3)
const MAX_ROLLING_VERSION: u8 = 3;
/// Maximum column count (2^11 - 1)
const MAX_COLUMN_COUNT: u16 = 2047;
/// Maximum column ID (same as column count)
const MAX_COLUMN_ID: u16 = 2047;
/// Mask for rolling version bits
const ROLLING_VERSION_MASK: u8 = 0x03;
/// Mask for column count high bits
const COL_COUNT_HIGH_MASK: u8 = 0x1C; // 0b00011100
/// Shift for column count high bits
const COL_COUNT_HIGH_SHIFT: u8 = 2;
/// Mask for column ID high bits
const COL_ID_HIGH_MASK: u8 = 0xE0; // 0b11100000
/// Shift for column ID high bits
const COL_ID_HIGH_SHIFT: u8 = 5;
/// Mask for column type bits
const COL_TYPE_MASK: u8 = 0x1F; // 0b00011111

/// A specialized `Result` type for operations in this module.
type Re<T> = Result<T, DErr<Fe, Rc>>;

/// Column types supported by DBuf
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Boolean,      // 0 - 1 byte
    U8,           // 1 - 1 byte
    U16,          // 2 - 2 bytes
    U32,          // 3 - 4 bytes
    U64,          // 4 - 8 bytes
    I8,           // 5 - 1 byte
    I16,          // 6 - 2 bytes
    I32,          // 7 - 4 bytes
    I64,          // 8 - 8 bytes
    F32,          // 9 - 4 bytes
    F64,          // 10 - 8 bytes
    UtcTime,      // 11 - 8 bytes
    Char(usize),  // 12 - variable size
    Ascii(usize), // 13 - variable size
}

impl ColumnType {
    /// Get the type ID (0-13)
    pub fn type_id(&self) -> u8 {
        match self {
            ColumnType::Boolean => 0,
            ColumnType::U8 => 1,
            ColumnType::U16 => 2,
            ColumnType::U32 => 3,
            ColumnType::U64 => 4,
            ColumnType::I8 => 5,
            ColumnType::I16 => 6,
            ColumnType::I32 => 7,
            ColumnType::I64 => 8,
            ColumnType::F32 => 9,
            ColumnType::F64 => 10,
            ColumnType::UtcTime => 11,
            ColumnType::Char(_) => 12,
            ColumnType::Ascii(_) => 13,
        }
    }

    /// Get the size in bytes for this column type
    pub fn size_of(&self) -> usize {
        match self {
            ColumnType::Boolean => 1,
            ColumnType::U8 => 1,
            ColumnType::U16 => 2,
            ColumnType::U32 => 4,
            ColumnType::U64 => 8,
            ColumnType::I8 => 1,
            ColumnType::I16 => 2,
            ColumnType::I32 => 4,
            ColumnType::I64 => 8,
            ColumnType::F32 => 4,
            ColumnType::F64 => 8,
            ColumnType::UtcTime => 8,
            ColumnType::Char(size) => *size,
            ColumnType::Ascii(size) => *size,
        }
    }

    /// Check if this type requires a size field in the header
    pub fn requires_size_field(&self) -> bool {
        matches!(self, ColumnType::Char(_) | ColumnType::Ascii(_))
    }

    /// Create ColumnType from type_id and optional size
    pub fn from_type_id(type_id: u8, size: Option<usize>) -> Re<ColumnType> {
        match type_id {
            0 => Ok(ColumnType::Boolean),
            1 => Ok(ColumnType::U8),
            2 => Ok(ColumnType::U16),
            3 => Ok(ColumnType::U32),
            4 => Ok(ColumnType::U64),
            5 => Ok(ColumnType::I8),
            6 => Ok(ColumnType::I16),
            7 => Ok(ColumnType::I32),
            8 => Ok(ColumnType::I64),
            9 => Ok(ColumnType::F32),
            10 => Ok(ColumnType::F64),
            11 => Ok(ColumnType::UtcTime),
            12 => {
                if let Some(size) = size {
                    if size == 0 {
                        Err(DErr::RootError(Fe::FromTypeId, 1, Rc::InvalidColumnSize))
                    } else {
                        Ok(ColumnType::Char(size))
                    }
                } else {
                    Err(DErr::RootError(Fe::FromTypeId, 2, Rc::MissingSizeForVariableType))
                }
            }
            13 => {
                if let Some(size) = size {
                    if size == 0 {
                        Err(DErr::RootError(Fe::FromTypeId, 1, Rc::InvalidColumnSize))
                    } else {
                        Ok(ColumnType::Ascii(size))
                    }
                } else {
                    Err(DErr::RootError(Fe::FromTypeId, 2, Rc::MissingSizeForVariableType))
                }
            }
            _ => Err(DErr::RootError(Fe::FromTypeId, 3, Rc::InvalidColumnType(type_id))),
        }
    }
}

/// Column header with u16 ID and type
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnHeader {
    pub col_id: u16,
    pub col_type: ColumnType,
}

impl ColumnHeader {
    pub fn new(col_id: u16, col_type: ColumnType) -> Re<ColumnHeader> {
        if col_id == 0 {
            return Err(DErr::RootError(Fe::NewColumnHeader, 1, Rc::ReservedColumnId));
        }
        if col_id > MAX_COLUMN_ID {
            return Err(DErr::RootError(Fe::NewColumnHeader, 2, Rc::ColumnIdExceeded));
        }
        Ok(ColumnHeader { col_id, col_type })
    }
}

/// Column with header and computed size
#[derive(Clone, Debug)]
pub struct Column {
    pub header: ColumnHeader,
    pub size: usize,
}

impl Column {
    pub fn new(header: ColumnHeader) -> Column {
        Column {
            header: header.clone(),
            size: header.col_type.size_of(),
        }
    }
}

/// DBuf header containing metadata
#[derive(Clone, Debug)]
pub struct DBufHeader {
    pub d_buf_id: u16,
    pub columns: Vec<Column>,
    pub row_count: u16,
    pub row_capacity: u16,
    pub row_size: u16,
}

impl DBufHeader {
    /// Calculate the total header size in bytes
    pub fn header_size(&self) -> usize {
        // Fixed header: version+col_count(2) + d_buf_id(2) + row_count(2) + row_capacity(2) + row_size(2) = 10 bytes
        let mut size = 10;
        // Column definitions: each column takes 2 bytes (col_id_high+col_type, col_id_low) + optional 2 bytes for size
        for col in &self.columns {
            size += 2; // Always 2 bytes for id and type
            if col.header.col_type.requires_size_field() {
                size += 2; // Additional 2 bytes for size
            }
        }
        size
    }

    /// Create a new DBufHeader
    pub fn new(d_buf_id: u16, columns: Vec<Column>, row_capacity: u16) -> Re<DBufHeader> {
        if d_buf_id == 0 {
            return Err(DErr::RootError(Fe::NewDBufHeader, 1, Rc::InvalidDBufId));
        }
        if columns.len() > MAX_COLUMN_COUNT as usize {
            return Err(DErr::RootError(Fe::NewDBufHeader, 2, Rc::ColumnCountExceeded));
        }
        
        // Check for duplicate column IDs
        let mut seen_ids = std::collections::HashSet::new();
        for col in &columns {
            if !seen_ids.insert(col.header.col_id) {
                return Err(DErr::RootError(Fe::NewDBufHeader, 3, Rc::DuplicateColumnId(col.header.col_id)));
            }
        }
        
        // Calculate row size
        let row_size: usize = columns.iter().map(|c| c.size).sum();
        if row_size > u16::MAX as usize {
            return Err(DErr::RootError(Fe::NewDBufHeader, 4, Rc::RowSizeExceeded));
        }
        
        Ok(DBufHeader {
            d_buf_id,
            columns,
            row_count: 0,
            row_capacity,
            row_size: row_size as u16,
        })
    }

    /// Encode header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // First byte: rolling_version (2b) + col_count_high (3b) + unused (3b)
        let col_count_high = (self.columns.len() as u16 >> 8) as u8 & 0x07; // Only 3 bits
        let byte0 = (ROLLING_VERSION & ROLLING_VERSION_MASK) | ((col_count_high << COL_COUNT_HIGH_SHIFT) & COL_COUNT_HIGH_MASK);
        bytes.push(byte0);
        
        // Second byte: col_count_low (8b)
        bytes.push(self.columns.len() as u8);
        
        // DBuf ID (2 bytes, big-endian)
        bytes.extend_from_slice(&self.d_buf_id.to_be_bytes());
        
        // Column definitions
        for col in &self.columns {
            // Pack col_id_high (3b) + col_type (5b) into one byte
            let col_id_high = (col.header.col_id >> 8) as u8 & 0x07; // Only 3 bits
            let col_type_id = col.header.col_type.type_id() & COL_TYPE_MASK;
            let byte_col = ((col_id_high << COL_ID_HIGH_SHIFT) & COL_ID_HIGH_MASK) | (col_type_id & COL_TYPE_MASK);
            bytes.push(byte_col);
            
            // col_id_low (8b)
            bytes.push(col.header.col_id as u8);
            
            // Optional size field for Char/ASCII
            if col.header.col_type.requires_size_field() {
                if let ColumnType::Char(size) | ColumnType::Ascii(size) = col.header.col_type {
                    bytes.extend_from_slice(&(size as u16).to_be_bytes());
                }
            }
        }
        
        // Row count (2 bytes)
        bytes.extend_from_slice(&self.row_count.to_be_bytes());
        
        // Row capacity (2 bytes)
        bytes.extend_from_slice(&self.row_capacity.to_be_bytes());
        
        // Row size (2 bytes)
        bytes.extend_from_slice(&self.row_size.to_be_bytes());
        
        bytes
    }

    /// Decode header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Re<DBufHeader> {
        if bytes.len() < 10 {
            return Err(DErr::RootError(Fe::FromBytes, 1, Rc::BufferTooSmall));
        }
        
        // Parse rolling version
        let byte0 = bytes[0];
        let rolling_version = byte0 & ROLLING_VERSION_MASK;
        if rolling_version > MAX_ROLLING_VERSION {
            return Err(DErr::RootError(Fe::FromBytes, 2, Rc::InvalidRollingVersion(rolling_version)));
        }
        
        // Parse column count (11 bits: 3 high + 8 low)
        let col_count_high = (byte0 & COL_COUNT_HIGH_MASK) >> COL_COUNT_HIGH_SHIFT;
        let col_count_low = bytes[1];
        let col_count = ((col_count_high as u16) << 8) | (col_count_low as u16);
        
        if col_count > MAX_COLUMN_COUNT {
            return Err(DErr::RootError(Fe::FromBytes, 3, Rc::ColumnCountExceeded));
        }
        
        // Parse DBuf ID
        let d_buf_id = u16::from_be_bytes([bytes[2], bytes[3]]);
        if d_buf_id == 0 {
            return Err(DErr::RootError(Fe::FromBytes, 4, Rc::InvalidDBufId));
        }
        
        // Parse column definitions
        let mut columns = Vec::with_capacity(col_count as usize);
        let mut offset = 4; // After version, col_count, d_buf_id
        
        for _ in 0..col_count {
            if offset + 2 > bytes.len() {
                return Err(DErr::RootError(Fe::FromBytes, 5, Rc::BufferTooSmall));
            }
            
            // Parse col_id_high (3b) + col_type (5b)
            let byte_col = bytes[offset];
            let col_id_high = (byte_col & COL_ID_HIGH_MASK) >> COL_ID_HIGH_SHIFT;
            let col_type_id = byte_col & COL_TYPE_MASK;
            
            // Parse col_id_low (8b)
            let col_id_low = bytes[offset + 1];
            let col_id = ((col_id_high as u16) << 8) | (col_id_low as u16);
            
            if col_id == 0 {
                return Err(DErr::RootError(Fe::FromBytes, 6, Rc::ReservedColumnId));
            }
            
            offset += 2;
            
            // Parse optional size field for Char/ASCII
            let size = if col_type_id == 12 || col_type_id == 13 {
                if offset + 2 > bytes.len() {
                    return Err(DErr::RootError(Fe::FromBytes, 7, Rc::BufferTooSmall));
                }
                let s = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
                offset += 2;
                Some(s)
            } else {
                None
            };
            
            // Create column type
            let col_type = ColumnType::from_type_id(col_type_id, size)?; 
            
            // Create column header and column
            let col_header = ColumnHeader { col_id, col_type };
            columns.push(Column::new(col_header));
        }
        
        // Parse row metadata
        if offset + 6 > bytes.len() {
            return Err(DErr::RootError(Fe::FromBytes, 8, Rc::BufferTooSmall));
        }
        
        let row_count = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        let row_capacity = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]);
        let row_size = u16::from_be_bytes([bytes[offset + 4], bytes[offset + 5]]);
        
        // Validate row count <= capacity
        if row_count > row_capacity {
            return Err(DErr::RootError(Fe::FromBytes, 9, Rc::RowCountExceedsCapacity));
        }
        
        // Validate row size matches sum of column sizes
        let calculated_row_size: usize = columns.iter().map(|c| c.size).sum();
        if calculated_row_size != row_size as usize {
            return Err(DErr::RootError(Fe::FromBytes, 10, Rc::RowSizeMismatch));
        }
        
        Ok(DBufHeader {
            d_buf_id,
            columns,
            row_count,
            row_capacity,
            row_size,
        })
    }
}

/// Dibonit Buffer - a structure representing a tabular data buffer.
#[derive(Clone, Debug)]
pub struct DBuf {
    pub header: DBufHeader,
    pub raw_values: Vec<u8>,
}

impl DBuf {
    /// Create a new DBuf with specified ID, columns, and capacity
    pub fn new(d_buf_id: u16, columns: Vec<ColumnHeader>, row_capacity: u16) -> Re<DBuf> {
        // Convert ColumnHeader to Column
        let cols: Vec<Column> = columns.into_iter().map(Column::new).collect();
        
        let header = DBufHeader::new(d_buf_id, cols, row_capacity)?;
        
        // Initialize raw_values with header bytes
        let mut raw_values = header.to_bytes();
        
        // Pre-allocate capacity for data
        let data_capacity = header.row_capacity as usize * header.row_size as usize;
        raw_values.reserve(data_capacity);
        
        Ok(DBuf {
            header,
            raw_values,
        })
    }

    /// Create DBuf from raw bytes (header + data)
    pub fn from_bytes(bytes: Vec<u8>) -> Re<DBuf> {
        let header = DBufHeader::from_bytes(&bytes)?;
        let header_size = header.header_size();
        
        if bytes.len() < header_size {
            return Err(DErr::RootError(Fe::FromBytesDBuf, 1, Rc::BufferTooSmall));
        }
        
        // Validate data size
        let expected_data_size = header.row_count as usize * header.row_size as usize;
        let actual_data_size = bytes.len() - header_size;
        
        if actual_data_size < expected_data_size {
            return Err(DErr::RootError(Fe::FromBytesDBuf, 2, Rc::BufferTooSmall));
        }
        
        Ok(DBuf {
            header,
            raw_values: bytes,
        })
    }

    /// Get the header size in bytes
    pub fn header_size(&self) -> usize {
        self.header.header_size()
    }

    /// Get the data offset (start of raw values)
    pub fn data_offset(&self) -> usize {
        self.header_size()
    }

    /// Add rows to the DBuf
    pub fn add_rows(&mut self, rows_data: &[u8]) -> Re<()> {
        if rows_data.len() % self.header.row_size as usize != 0 {
            return Err(DErr::RootError(Fe::AddRows, 1, Rc::DataSizeMismatch));
        }
        
        let num_rows = rows_data.len() / self.header.row_size as usize;
        
        if self.header.row_count + num_rows as u16 > self.header.row_capacity {
            return Err(DErr::RootError(Fe::AddRows, 2, Rc::CapacityExceeded));
        }
        
        // Ensure we have enough capacity in raw_values
        let current_data_len = self.raw_values.len().saturating_sub(self.data_offset());
        let new_data_len = current_data_len + rows_data.len();
        let max_data_len = self.header.row_capacity as usize * self.header.row_size as usize;
        
        if new_data_len > max_data_len {
            return Err(DErr::RootError(Fe::AddRows, 3, Rc::CapacityExceeded));
        }
        
        // Extend raw_values with new data
        self.raw_values.extend_from_slice(rows_data);
        self.header.row_count += num_rows as u16;
        
        Ok(())
    }

    /// Get a row by index
    pub fn get_row(&self, row_index: u16) -> Re<Vec<u8>> {
        if row_index >= self.header.row_count {
            return Err(DErr::RootError(Fe::GetRow, 1, Rc::IndexOutOfBounds));
        }
        
        let start = self.data_offset() + (row_index as usize * self.header.row_size as usize);
        let end = start + self.header.row_size as usize;
        
        if end > self.raw_values.len() {
            return Err(DErr::RootError(Fe::GetRow, 2, Rc::BufferTooSmall));
        }
        
        Ok(self.raw_values[start..end].to_vec())
    }

    /// Get a specific column value from a row by column ID
    pub fn get_col(&self, row_index: u16, col_id: u16) -> Re<Vec<u8>> {
        if row_index >= self.header.row_count {
            return Err(DErr::RootError(Fe::GetCol, 1, Rc::IndexOutOfBounds));
        }
        
        // Find column by ID
        let col = self.header.columns.iter()
            .find(|c| c.header.col_id == col_id)
            .ok_or_else(|| DErr::RootError(Fe::GetCol, 2, Rc::ColumnNotFound(col_id)))?;
        
        // Calculate offset within row
        let row_start = self.data_offset() + (row_index as usize * self.header.row_size as usize);
        
        // Calculate column offset within row
        let mut col_offset = 0;
        for c in &self.header.columns {
            if c.header.col_id == col_id {
                break;
            }
            col_offset += c.size;
        }
        
        let start = row_start + col_offset;
        let end = start + col.size;
        
        if end > self.raw_values.len() {
            return Err(DErr::RootError(Fe::GetCol, 3, Rc::BufferTooSmall));
        }
        
        Ok(self.raw_values[start..end].to_vec())
    }

    /// Get DBuf ID
    pub fn get_d_buf_id(&self) -> u16 {
        self.header.d_buf_id
    }

    /// Get column metadata by column ID
    pub fn get_column_metadata(&self, col_id: u16) -> Re<(&ColumnHeader, usize)> {
        let col = self.header.columns.iter()
            .find(|c| c.header.col_id == col_id)
            .ok_or_else(|| DErr::RootError(Fe::GetColumnMetadata, 1, Rc::ColumnNotFound(col_id)))?;
        
        Ok((&col.header, col.size))
    }

    /// Get all rows
    pub fn get_all_rows(&self) -> Vec<Vec<u8>> {
        (0..self.header.row_count)
            .filter_map(|i| self.get_row(i).ok())
            .collect()
    }

    /// Transfer ownership of the underlying Vec<u8>
    pub fn into_raw_bytes(self) -> Vec<u8> {
        self.raw_values
    }

    /// Get a reference to the raw bytes (header + data)
    pub fn as_raw_bytes(&self) -> &[u8] {
        &self.raw_values
    }

    /// Iterate over all columns
    pub fn iter_columns(&self) -> impl Iterator<Item = &Column> {
        self.header.columns.iter()
    }
}

/// Root cause errors
#[derive(Debug)]
pub enum Rc {
    // Configuration errors
    DuplicateColumnId(u16),
    ReservedColumnId,
    InvalidColumnType(u8),
    InvalidColumnSize,
    ColumnIdExceeded,
    InvalidDBufId,
    ColumnCountExceeded,
    RowSizeExceeded,
    MissingSizeForVariableType,
    
    // Capacity errors
    CapacityExceeded,
    RowCapacityZero,
    DataSizeMismatch,
    
    // Access errors
    IndexOutOfBounds,
    ColumnNotFound(u16),
    BufferTooSmall,
    
    // Data integrity errors
    HeaderCorruption,
    RowCountExceedsCapacity,
    RowSizeMismatch,
    
    // Version errors
    InvalidRollingVersion(u8),
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
        }
    }
}

/// Function where the error transits
#[derive(Debug)]
pub enum Fe {
    NewDBuf,
    NewColumnHeader,
    NewDBufHeader,
    FromTypeId,
    FromBytes,
    FromBytesDBuf,
    AddRows,
    GetRow,
    GetCol,
    GetColumnMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ Helper functions for tests ============
    
    fn create_test_columns() -> Vec<ColumnHeader> {
        vec![
            ColumnHeader::new(1, ColumnType::U8).unwrap(),
            ColumnHeader::new(2, ColumnType::U16).unwrap(),
            ColumnHeader::new(3, ColumnType::U32).unwrap(),
        ]
    }

    fn create_test_d_buf() -> DBuf {
        let columns = create_test_columns();
        DBuf::new(1, columns, 100).unwrap()
    }

    // ============ ColumnType Tests ============

    #[test]
    fn test_col_type_size_of() {
        assert_eq!(ColumnType::Boolean.size_of(), 1);
        assert_eq!(ColumnType::U8.size_of(), 1);
        assert_eq!(ColumnType::U16.size_of(), 2);
        assert_eq!(ColumnType::U32.size_of(), 4);
        assert_eq!(ColumnType::U64.size_of(), 8);
        assert_eq!(ColumnType::Char(10).size_of(), 10);
        assert_eq!(ColumnType::Ascii(20).size_of(), 20);
    }

    #[test]
    fn test_col_type_type_id() {
        assert_eq!(ColumnType::Boolean.type_id(), 0);
        assert_eq!(ColumnType::U8.type_id(), 1);
        assert_eq!(ColumnType::U16.type_id(), 2);
        assert_eq!(ColumnType::Char(10).type_id(), 12);
        assert_eq!(ColumnType::Ascii(20).type_id(), 13);
    }

    #[test]
    fn test_col_type_requires_size_field() {
        assert!(!ColumnType::U8.requires_size_field());
        assert!(!ColumnType::U64.requires_size_field());
        assert!(ColumnType::Char(10).requires_size_field());
        assert!(ColumnType::Ascii(20).requires_size_field());
    }

    #[test]
    fn test_col_type_from_type_id_valid() {
        assert_eq!(ColumnType::from_type_id(0, None).unwrap(), ColumnType::Boolean);
        assert_eq!(ColumnType::from_type_id(1, None).unwrap(), ColumnType::U8);
        assert_eq!(ColumnType::from_type_id(12, Some(10)).unwrap(), ColumnType::Char(10));
        assert_eq!(ColumnType::from_type_id(13, Some(20)).unwrap(), ColumnType::Ascii(20));
    }

    #[test]
    fn test_col_type_from_type_id_invalid_type() {
        let result = ColumnType::from_type_id(14, None);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("InvalidColumnType(14)"), true);
        }
    }

    #[test]
    fn test_col_type_from_type_id_missing_size() {
        let result = ColumnType::from_type_id(12, None);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("MissingSizeForVariableType"), true);
        }
    }

    #[test]
    fn test_col_type_from_type_id_zero_size() {
        let result = ColumnType::from_type_id(12, Some(0));
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("InvalidColumnSize"), true);
        }
    }

    // ============ ColumnHeader Tests ============

    #[test]
    fn test_col_header_new_valid() {
        let header = ColumnHeader::new(1, ColumnType::U8).unwrap();
        assert_eq!(header.col_id, 1);
        assert_eq!(header.col_type, ColumnType::U8);
    }

    #[test]
    fn test_col_header_new_reserved_id() {
        let result = ColumnHeader::new(0, ColumnType::U8);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("ReservedColumnId"), true);
        }
    }

    #[test]
    fn test_col_header_new_id_exceeded() {
        let result = ColumnHeader::new(2048, ColumnType::U8);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("ColumnIdExceeded"), true);
        }
    }

    // ============ DBufHeader Tests ============

    #[test]
    fn test_d_buf_header_new_valid() {
        let columns = create_test_columns();
        let cols: Vec<Column> = columns.into_iter().map(Column::new).collect();
        let header = DBufHeader::new(1, cols, 100).unwrap();
        
        assert_eq!(header.d_buf_id, 1);
        assert_eq!(header.columns.len(), 3);
        assert_eq!(header.row_capacity, 100);
        assert_eq!(header.row_count, 0);
        assert_eq!(header.row_size, 7); // 1 + 2 + 4 = 7
    }

    #[test]
    fn test_d_buf_header_new_invalid_d_buf_id() {
        let columns = create_test_columns();
        let cols: Vec<Column> = columns.into_iter().map(Column::new).collect();
        let result = DBufHeader::new(0, cols, 100);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("InvalidDBufId"), true);
        }
    }

    #[test]
    fn test_d_buf_header_new_duplicate_col_id() {
        let columns = vec![
            ColumnHeader::new(1, ColumnType::U8).unwrap(),
            ColumnHeader::new(1, ColumnType::U16).unwrap(),
        ];
        let cols: Vec<Column> = columns.into_iter().map(Column::new).collect();
        let result = DBufHeader::new(1, cols, 100);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("DuplicateColumnId"), true);
        }
    }

    #[test]
    fn test_d_buf_header_new_too_many_columns() {
        // Create 2048 columns with duplicate IDs (all ID 1)
        let columns: Vec<ColumnHeader> = (0..2048).map(|_| ColumnHeader::new(1, ColumnType::U8).unwrap()).collect();
        let cols: Vec<Column> = columns.into_iter().map(Column::new).collect();
        let result = DBufHeader::new(1, cols, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_d_buf_header_header_size() {
        let columns = create_test_columns();
        let cols: Vec<Column> = columns.into_iter().map(Column::new).collect();
        let header = DBufHeader::new(1, cols, 100).unwrap();
        
        // 10 (fixed) + 3 columns * 2 bytes (no size fields) = 16
        assert_eq!(header.header_size(), 16);
    }

    #[test]
    fn test_d_buf_header_header_size_with_char() {
        let columns = vec![
            ColumnHeader::new(1, ColumnType::U8).unwrap(),
            ColumnHeader::new(2, ColumnType::Char(100)).unwrap(),
        ];
        let cols: Vec<Column> = columns.into_iter().map(Column::new).collect();
        let header = DBufHeader::new(1, cols, 100).unwrap();
        
        // 10 (fixed) + 2 (U8) + 2 (Char id+type) + 2 (Char size) = 16
        assert_eq!(header.header_size(), 16);
    }

    #[test]
    fn test_d_buf_header_to_bytes() {
        let columns = create_test_columns();
        let cols: Vec<Column> = columns.into_iter().map(Column::new).collect();
        let header = DBufHeader::new(1, cols, 100).unwrap();
        
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), header.header_size());
        
        // Check first byte: version (0) + col_count_high (0) = 0
        assert_eq!(bytes[0], 0);
        
        // Check second byte: col_count_low (3)
        assert_eq!(bytes[1], 3);
        
        // Check DBuf ID (1 in big-endian)
        assert_eq!(bytes[2], 0);
        assert_eq!(bytes[3], 1);
    }

    #[test]
    fn test_d_buf_header_from_bytes() {
        let columns = create_test_columns();
        let cols: Vec<Column> = columns.into_iter().map(Column::new).collect();
        let original_header = DBufHeader::new(1, cols, 100).unwrap();
        
        let bytes = original_header.to_bytes();
        let parsed_header = DBufHeader::from_bytes(&bytes).unwrap();
        
        assert_eq!(parsed_header.d_buf_id, 1);
        assert_eq!(parsed_header.columns.len(), 3);
        assert_eq!(parsed_header.row_capacity, 100);
        assert_eq!(parsed_header.row_size, 7);
    }

    #[test]
    fn test_d_buf_header_from_bytes_invalid_version() {
        // Create bytes with invalid version (version 4 in first 2 bits = 0b00000100 = 0x04)
        let bytes = vec![0x04; 16]; // Version 4 (invalid, max is 3)
        
        let result = DBufHeader::from_bytes(&bytes);
        assert!(result.is_err());
    }

    // ============ DBuf Tests ============

    #[test]
    fn test_d_buf_new_valid() {
        let d_buf = create_test_d_buf();
        assert_eq!(d_buf.header.d_buf_id, 1);
        assert_eq!(d_buf.header.columns.len(), 3);
        assert_eq!(d_buf.header.row_capacity, 100);
        assert_eq!(d_buf.header.row_count, 0);
    }

    #[test]
    fn test_d_buf_add_rows() {
        let mut d_buf = create_test_d_buf();
        
        // Create row data: U8=1, U16=256, U32=65536
        let row_data = vec![1u8, 0, 1, 0, 0, 1, 0];
        
        d_buf.add_rows(&row_data).unwrap();
        
        assert_eq!(d_buf.header.row_count, 1);
        assert_eq!(d_buf.raw_values.len(), d_buf.header_size() + 7);
    }

    #[test]
    fn test_d_buf_add_multiple_rows() {
        let mut d_buf = create_test_d_buf();
        
        // Create 3 rows
        let row_data = vec![
            1u8, 0, 1, 0, 0, 1, 0,  // Row 1
            2u8, 0, 2, 0, 0, 2, 0,  // Row 2
            3u8, 0, 3, 0, 0, 3, 0,  // Row 3
        ];
        
        d_buf.add_rows(&row_data).unwrap();
        
        assert_eq!(d_buf.header.row_count, 3);
    }

    #[test]
    fn test_d_buf_add_rows_capacity_exceeded() {
        let mut d_buf = DBuf::new(1, create_test_columns(), 2).unwrap();
        
        // Try to add 3 rows
        let row_data = vec![
            1u8, 0, 1, 0, 0, 1, 0,
            2u8, 0, 2, 0, 0, 2, 0,
            3u8, 0, 3, 0, 0, 3, 0,
        ];
        
        let result = d_buf.add_rows(&row_data);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("CapacityExceeded"), true);
        }
    }

    #[test]
    fn test_d_buf_add_rows_wrong_size() {
        let mut d_buf = create_test_d_buf();
        
        // Row data with wrong size (6 bytes instead of 7)
        let row_data = vec![1u8, 0, 1, 0, 0, 1];
        
        let result = d_buf.add_rows(&row_data);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("DataSizeMismatch"), true);
        }
    }

    #[test]
    fn test_d_buf_get_row() {
        let mut d_buf = create_test_d_buf();
        
        let row_data = vec![1u8, 0, 1, 0, 0, 1, 0];
        d_buf.add_rows(&row_data).unwrap();
        
        let row = d_buf.get_row(0).unwrap();
        assert_eq!(row, row_data);
    }

    #[test]
    fn test_d_buf_get_row_out_of_bounds() {
        let d_buf = create_test_d_buf();
        
        let result = d_buf.get_row(0);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("IndexOutOfBounds"), true);
        }
    }

    #[test]
    fn test_d_buf_get_col() {
        let mut d_buf = create_test_d_buf();
        
        let row_data = vec![1u8, 0, 1, 0, 0, 1, 0];
        d_buf.add_rows(&row_data).unwrap();
        
        // Get U8 column (col_id=1)
        let col = d_buf.get_col(0, 1).unwrap();
        assert_eq!(col, vec![1u8]);
        
        // Get U16 column (col_id=2)
        let col = d_buf.get_col(0, 2).unwrap();
        assert_eq!(col, vec![0u8, 1]);
        
        // Get U32 column (col_id=3)
        let col = d_buf.get_col(0, 3).unwrap();
        assert_eq!(col, vec![0u8, 0, 1, 0]);
    }

    #[test]
    fn test_d_buf_get_col_not_found() {
        let mut d_buf = create_test_d_buf();
        let row_data = vec![1u8, 0, 1, 0, 0, 1, 0];
        d_buf.add_rows(&row_data).unwrap();
        
        let result = d_buf.get_col(0, 99);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("ColumnNotFound"), true);
        }
    }

    #[test]
    fn test_d_buf_get_d_buf_id() {
        let d_buf = create_test_d_buf();
        assert_eq!(d_buf.get_d_buf_id(), 1);
    }

    #[test]
    fn test_d_buf_get_column_metadata() {
        let d_buf = create_test_d_buf();
        
        let (header, size) = d_buf.get_column_metadata(1).unwrap();
        assert_eq!(header.col_id, 1);
        assert_eq!(header.col_type, ColumnType::U8);
        assert_eq!(size, 1);
    }

    #[test]
    fn test_d_buf_get_all_rows() {
        let mut d_buf = create_test_d_buf();
        
        let row1 = vec![1u8, 0, 1, 0, 0, 1, 0];
        let row2 = vec![2u8, 0, 2, 0, 0, 2, 0];
        
        let mut data = row1.clone();
        data.extend(row2.clone());
        
        d_buf.add_rows(&data).unwrap();
        
        let rows = d_buf.get_all_rows();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], row1);
        assert_eq!(rows[1], row2);
    }

    #[test]
    fn test_d_buf_into_raw_bytes() {
        let mut d_buf = create_test_d_buf();
        let row_data = vec![1u8, 0, 1, 0, 0, 1, 0];
        d_buf.add_rows(&row_data).unwrap();
        
        let bytes = d_buf.into_raw_bytes();
        assert!(bytes.len() > 0);
    }

    #[test]
    fn test_d_buf_from_bytes() {
        let mut d_buf = create_test_d_buf();
        let row_data = vec![1u8, 0, 1, 0, 0, 1, 0];
        d_buf.add_rows(&row_data).unwrap();
        
        // Update the header bytes in raw_values to reflect the new row_count
        let header_bytes = d_buf.header.to_bytes();
        d_buf.raw_values[..header_bytes.len()].copy_from_slice(&header_bytes);
        
        let bytes = d_buf.raw_values.clone();
        let reconstructed = DBuf::from_bytes(bytes).unwrap();
        
        assert_eq!(reconstructed.header.d_buf_id, 1);
        assert_eq!(reconstructed.header.row_count, 1);
        assert_eq!(reconstructed.raw_values.len(), d_buf.raw_values.len());
    }

    #[test]
    fn test_d_buf_from_bytes_too_small() {
        let bytes = vec![0u8; 5]; // Too small for header
        let result = DBuf::from_bytes(bytes);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("BufferTooSmall"));
        }
    }

    // ============ Edge Cases ============

    #[test]
    fn test_d_buf_zero_capacity() {
        let d_buf = DBuf::new(1, create_test_columns(), 0).unwrap();
        assert_eq!(d_buf.header.row_capacity, 0);
    }

    #[test]
    fn test_d_buf_single_column() {
        let columns = vec![ColumnHeader::new(1, ColumnType::U8).unwrap()];
        let d_buf = DBuf::new(1, columns, 100).unwrap();
        assert_eq!(d_buf.header.columns.len(), 1);
        assert_eq!(d_buf.header.row_size, 1);
    }

    #[test]
    fn test_d_buf_single_row() {
        let mut d_buf = DBuf::new(1, create_test_columns(), 1).unwrap();
        let row_data = vec![1u8, 0, 1, 0, 0, 1, 0];
        d_buf.add_rows(&row_data).unwrap();
        assert_eq!(d_buf.header.row_count, 1);
    }

    #[test]
    fn test_d_buf_all_column_types() {
        let columns: Vec<ColumnHeader> = (0..=13).map(|i| {
            let col_type = match i {
                0 => ColumnType::Boolean,
                1 => ColumnType::U8,
                2 => ColumnType::U16,
                3 => ColumnType::U32,
                4 => ColumnType::U64,
                5 => ColumnType::I8,
                6 => ColumnType::I16,
                7 => ColumnType::I32,
                8 => ColumnType::I64,
                9 => ColumnType::F32,
                10 => ColumnType::F64,
                11 => ColumnType::UtcTime,
                12 => ColumnType::Char(10),
                13 => ColumnType::Ascii(10),
                _ => ColumnType::Boolean,
            };
            ColumnHeader::new((i + 1) as u16, col_type).unwrap()
        }).collect();
        
        let d_buf = DBuf::new(1, columns, 100).unwrap();
        assert_eq!(d_buf.header.columns.len(), 14);
    }

    #[test]
    fn test_d_buf_large_char_column() {
        let columns = vec![ColumnHeader::new(1, ColumnType::Char(65535)).unwrap()];
        let d_buf = DBuf::new(1, columns, 10).unwrap();
        assert_eq!(d_buf.header.row_size, 65535);
    }

    // ============ Bit Packing Tests ============

    #[test]
    fn test_d_buf_many_columns() {
        // Test with 256 columns (spans byte boundary)
        let columns: Vec<ColumnHeader> = (1..=256).map(|i| ColumnHeader::new(i, ColumnType::U8).unwrap()).collect();
        let d_buf = DBuf::new(1, columns, 100).unwrap();
        assert_eq!(d_buf.header.columns.len(), 256);
    }

    #[test]
    fn test_d_buf_column_id_256() {
        let columns = vec![ColumnHeader::new(256, ColumnType::U8).unwrap()];
        let d_buf = DBuf::new(1, columns, 100).unwrap();
        assert_eq!(d_buf.header.columns[0].header.col_id, 256);
    }

    // ============ Error Cases ============

    #[test]
    fn test_d_buf_invalid_d_buf_id() {
        let columns = create_test_columns();
        let result = DBuf::new(0, columns, 100);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("InvalidDBufId"), true);
        }
    }

    #[test]
    fn test_d_buf_duplicate_column_id() {
        let columns = vec![
            ColumnHeader::new(1, ColumnType::U8).unwrap(),
            ColumnHeader::new(1, ColumnType::U16).unwrap(),
        ];
        let result = DBuf::new(1, columns, 100);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string().contains("DuplicateColumnId"), true);
        }
    }

    #[test]
    fn test_d_buf_reserved_column_id() {
        let column = ColumnHeader::new(0, ColumnType::U8);
        assert!(column.is_err());
        if let Err(e) = column {
            assert!(e.to_string().contains("ReservedColumnId"));
        }
    }

    #[test]
    fn test_d_buf_column_count_exceeded() {
        // Test with 2048 columns (exceeds MAX_COLUMN_COUNT of 2047)
        let mut columns: Vec<ColumnHeader> = (1..=2047).map(|i| ColumnHeader::new(i, ColumnType::U8).unwrap()).collect();
        columns.push(ColumnHeader::new(1, ColumnType::U8).unwrap()); // Duplicate
        let result = DBuf::new(1, columns, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_d_buf_column_id_exceeded() {
        let column = ColumnHeader::new(2048, ColumnType::U8).unwrap_err();
        assert!(column.to_string().contains("ColumnIdExceeded"));
    }



}
#[cfg(test)]
mod verbal_use_exemple {
    use std::borrow::Cow;
    use super::*;

    pub enum User{
        Nom,
        Prenom,
        BirthYear,
    }
    impl User {
        /// Get the type ID (0-13)
        pub fn col_id(&self) -> u16 {
            match self {
                User::Nom => 0,
                User::Prenom => 1,
                User::BirthYear => 2,
            }
        }
        pub fn col_type(&self) -> ColumnType {
            match self {
                User::Nom => ColumnType::Char(12),     // 12 bytes = 12/4 = 3 chars
                User::Prenom => ColumnType::Ascii(12), // 12 bytes = 12 ascii char
                User::BirthYear => ColumnType::U16,
            }
        }
        fn create_colunms_header() -> Vec<ColumnHeader> {
            vec![
                ColumnHeader::new(User::Nom.col_id(),            User::Nom.col_type()).unwrap(),
                ColumnHeader::new(User::Prenom.col_id(),         User::Prenom.col_type()).unwrap(),
                ColumnHeader::new(User::BirthYear.col_id(),      User::BirthYear.col_type()).unwrap(),
            ]
        }
        fn create_d_buf(capacity : u16) -> Re<DBuf> {
            DBuf::new(Tables::User.table_id(), User::create_colunms_header(), 10)
        }
        fn add_row(d_buff_ref: &mut DBuf, name : &str, surname : &str, birth_date :u32)  {
            // Convertir le u32 en bytes (big-endian par défaut)
            let birth_bytes = birth_date.to_be_bytes();

            // Créer un vecteur pour stocker les données concaténées
            let mut combined = Vec::with_capacity(name.len() + surname.len() + 4);

            // Concaténer les bytes
            combined.extend_from_slice(name.as_bytes());
            combined.extend_from_slice(surname.as_bytes());
            combined.extend_from_slice(&birth_bytes);
            d_buff_ref.add_rows(combined.as_slice()).unwrap();
        }

        fn get_row(user_dbuff: &DBuf, index : u16) -> Re<(Cow<'static, str>, Cow<'static, str>,u32)> {
            let r = user_dbuff.get_row(index).unwrap();

            // On convertit le Cow emprunté en Cow possédé (Owned) avec into_owned()
            let name = Cow::Owned(String::from_utf8_lossy(&r[0..12]).into_owned());
            let surname = Cow::Owned(String::from_utf8_lossy(&r[12..24]).into_owned());
            let birth_date = u32::from_be_bytes(r[24..28].try_into().unwrap()).clone();
            Ok((name, surname,birth_date))
        }
    }
    pub enum Tables{
        User,
    }
    impl Tables {
        /// Get the type ID (0-13)
        pub fn table_id(&self) -> u16 {
            match self {
                Tables::User => 1,
            }
        }
    }
    #[test]
    fn test_verbal_exemple() {
        let mut user_dbuff = User::create_d_buf(3).unwrap();
        User::add_row(& mut user_dbuff ,"jean","toto",1990u32);
        User::add_row(& mut user_dbuff ,"maurice","titi",1990u32);
        let(nom,prenom,date) = User::get_row(&user_dbuff, 0).unwrap();
        assert_eq!(nom, "jean");
    }
}
