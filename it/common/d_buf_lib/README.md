# DBuf Library - Binary Data Buffer

## Overview

DBuf is a binary data buffer library designed for efficient, zero-copy data transfer between systems. It provides a structured way to organize tabular data in a contiguous byte vector, with all metadata (header and column information) encoded in a compact binary format using bit-packing for efficiency.

## Core Structure

The DBuf consists of a **header** and **raw values**, both stored in a single `Vec<u8>`. The header contains all metadata needed to interpret the raw data, with calculated variables (header_size, row_size) stored separately for O(1) access.

### Binary Layout

```
[BYTE_0: rolling_ver(2b) + col_count_high(3b) + unused(3b)]
[BYTE_1: col_count_low(8b)]
[BYTE_2-3: d_buf_id(16b)]
[COLUMN_DEFINITIONS...]
  [col_id_high(3b) + col_type(5b)][col_id_low(8b)]
  [OPTIONAL: col_size(16b) if type is Char or ASCII]
[ROW_COUNT: u16]
[ROW_CAPACITY: u16]
[ROW_SIZE: u16]
[RAW_VALUES...]
```

#### Header Fields

1. **Rolling Version (2 bits in BYTE_0)**: Enables newer code to handle 3 previous versions of DBuf header and data structure. 3 bits unused.

2. **Column Count (11 bits total)**:
   - High 3 bits: stored in BYTE_0 (bits 2-4)
   - Low 8 bits: stored in BYTE_1
   - Maximum: 2047 columns (0x7FF)

3. **DBuf ID (2 bytes)**: Unique identifier for this DBuf type. Stored in BYTE_2-3 as u16.

4. **Column Definitions** (for each column):
   - Column ID (11 bits total):
     - High 3 bits: packed with column type in first byte
     - Low 8 bits: separate byte
   - Column Type (5 bits): Encoded type ID (0-31)
     - 0: Boolean (1 byte)
     - 1: U8 (1 byte)
     - 2: U16 (2 bytes)
     - 3: U32 (4 bytes)
     - 4: U64 (8 bytes)
     - 5: I8 (1 byte)
     - 6: I16 (2 bytes)
     - 7: I32 (4 bytes)
     - 8: I64 (8 bytes)
     - 9: F32 (4 bytes)
     - 10: F64 (8 bytes)
     - 11: UtcTime (8 bytes)
     - 12: Char (variable size)
     - 13: ASCII (variable size)
     - 14-31: Reserved
   - **Optional Column Size (2 bytes)**: Only present when type is Char or ASCII. Specifies the size in bytes for that column's data in each row.

5. **Row Count (2 bytes)**: Current number of rows stored (u16).

6. **Row Capacity (2 bytes)**: Maximum number of rows the buffer can hold (u16).

7. **Row Size (2 bytes)**: Size in bytes of a single row (u16). Calculated as sum of all column sizes.

#### Calculated Variables

These are computed from header fields and stored separately (not in the byte vector):

1. **header_size**: Total size of the header in bytes. Used as offset for accessing the first row value.
2. **row_size**: Size of a single row in bytes. Used as multiplier for navigating between rows.

#### Raw Values

All row data follows the header contiguously. Each row contains values for all columns in the order they were defined. Access to row data must account for the header offset.

## Key Features

### 1. Compact Binary Encoding
- Bit-packed header fields minimize overhead
- u16 IDs support up to 65,535 unique DBufs and columns
- Optional column size field only for variable-length types (Char, ASCII)
- Eliminates string overhead and UTF-8 conversion errors

### 2. Zero-Copy Ownership Transfer
- The entire DBuf (header + data) is stored in a single `Vec<u8>`
- Ownership can be transferred to external libraries (e.g., socket libraries) without copying
- Enables efficient serialization/deserialization for network transfer

### 3. Developer-Friendly Enums

Applications using DBuf should define:

```rust
// Example enum for DBuf IDs
pub enum DBufId {
    UserData = 1,
    ProductCatalog = 2,
    TransactionLog = 3,
}

// Example enum for column IDs within a specific DBuf
pub enum UserDataColumns {
    Id = 1,
    Name = 2,
    Age = 3,
    Email = 4,
}
```

These enums provide human-readable names while compiling to the underlying u16 IDs.

## Column Types

| Type | ID | Size (bytes) | Notes |
|------|----|--------------|-------|
| Boolean | 0 | 1 | 8 booleans should share 1 byte |
| U8 | 1 | 1 | |
| U16 | 2 | 2 | |
| U32 | 3 | 4 | |
| U64 | 4 | 8 | |
| I8 | 5 | 1 | |
| I16 | 6 | 2 | |
| I32 | 7 | 4 | |
| I64 | 8 | 8 | |
| F32 | 9 | 4 | |
| F64 | 10 | 8 | |
| UtcTime | 11 | 8 | |
| Char | 12 | variable | Size specified in column definition |
| ASCII | 13 | variable | Size specified in column definition |
| Reserved | 14-31 | - | For future use |

## Functional Use Cases

### Nominal Cases

1. **DBuf Creation**
   - Create a new DBuf with specified ID, columns, and capacity
   - Validate that all column IDs are unique within the DBuf
   - Validate that DBuf ID is unique in the application context
   - Validate that column count <= 2047
   - Validate that all column types are valid (0-13)

2. **Adding Rows**
   - Add one or more rows to the DBuf
   - Validate that the total rows do not exceed capacity (max 65,535)
   - Validate that the input data size matches `ROW_SIZE * number_of_rows`

3. **Reading Data**
   - Get a complete row by index
   - Get a specific column value from a row by column ID
   - Get the DBuf ID
   - Get column metadata (type, size) by column ID

4. **Ownership Transfer**
   - Transfer ownership of the underlying `Vec<u8>` to external systems
   - Reconstruct DBuf from raw bytes
   - Validate header integrity during reconstruction

5. **Iteration**
   - Iterate over all rows
   - Iterate over all columns

### Edge Cases

1. **Empty DBuf**
   - Create DBuf with zero capacity
   - Create DBuf with zero rows initially
   - Access operations on empty DBuf

2. **Maximum Sizes**
   - DBuf with maximum column count (2047)
   - DBuf with maximum row capacity (65,535)
   - Column with maximum size (65,535 bytes for Char/ASCII types)

3. **Single Element**
   - DBuf with one column
   - DBuf with one row
   - Column of size 1 byte

4. **Mixed Types**
   - DBuf with all supported column types
   - Rows with mixed numeric types

5. **Bit Packing**
   - Column count spanning both bytes (e.g., 256 columns)
   - Column ID spanning both bytes (e.g., 256)

## Error Cases

### Configuration Errors

| Error | Description |
|-------|-------------|
| DuplicateColumnId | Two columns with the same ID in a DBuf |
| ReservedColumnId | Column ID 0 (reserved for future use) |
| InvalidColumnType | Column type ID > 13 |
| InvalidColumnSize | Char/ASCII column with size 0 |
| ColumnCountExceeded | More than 2047 columns |
| InvalidDBufId | DBuf ID 0 (reserved) |

### Capacity Errors

| Error | Description |
|-------|-------------|
| CapacityExceeded | Adding rows would exceed row capacity |
| RowCapacityZero | Attempt to add rows to zero-capacity DBuf |
| DataSizeMismatch | Input data size != ROW_SIZE * number_of_rows |

### Access Errors

| Error | Description |
|-------|-------------|
| IndexOutOfBounds | Row index >= row_count |
| ColumnNotFound | Column ID not found in DBuf |
| BufferOverflow | Attempt to read beyond buffer bounds |

### Data Integrity Errors

| Error | Description |
|-------|-------------|
| HeaderCorruption | Header fields are inconsistent or corrupted |
| RowSizeMismatch | Declared row size != sum of column sizes |
| BufferTooSmall | Raw values vector smaller than declared size |
| InvalidRollingVersion | Rolling version not supported |

## Test Cases Matrix

### Functional Tests

| Test ID | Description | Input | Expected Result |
|---------|-------------|-------|-----------------|
| F-001 | Create empty DBuf | Valid DBuf ID, empty columns, capacity=0 | Success, empty DBuf |
| F-002 | Create DBuf with columns | Valid DBuf ID, 3 columns, capacity=100 | Success, DBuf with header |
| F-003 | Add single row | DBuf with capacity=10, 1 row data | Success, row_count=1 |
| F-004 | Add multiple rows | DBuf with capacity=10, 5 rows data | Success, row_count=5 |
| F-005 | Get row by index | DBuf with 3 rows, index=1 | Success, returns row 1 |
| F-006 | Get column from row | DBuf with columns, row=0, col_id=2 | Success, returns column value |
| F-007 | Get DBuf ID | Any DBuf | Success, returns DBuf ID (u16) |
| F-008 | Get column metadata | DBuf, valid col_id | Success, returns type and size |
| F-009 | Iterate all rows | DBuf with 5 rows | Success, returns all 5 rows |
| F-010 | Transfer ownership | DBuf | Success, external system owns bytes |
| F-011 | Reconstruct from bytes | Valid byte vector | Success, reconstructed DBuf |

### Edge Case Tests

| Test ID | Description | Input | Expected Result |
|---------|-------------|-------|-----------------|
| E-001 | Zero capacity DBuf | capacity=0 | Success, empty DBuf |
| E-002 | Maximum columns | 2047 unique column IDs | Success |
| E-003 | Maximum row capacity | capacity=65535 | Success |
| E-004 | Single column | 1 column | Success |
| E-005 | Single row | capacity=1, add 1 row | Success |
| E-006 | All column types | 1 column of each type (0-13) | Success |
| E-007 | Large Char column | Char(65535) | Success |
| E-008 | Column count spanning bytes | 256 columns | Success |
| E-009 | Column ID spanning bytes | col_id=256 | Success |
| E-010 | All boolean columns | 8 Boolean columns | Success |

### Error Case Tests

| Test ID | Description | Input | Expected Result |
|---------|-------------|-------|-----------------|
| ER-001 | Duplicate column ID | Two columns with same ID | Error: DuplicateColumnId |
| ER-002 | Column ID 0 | Column with ID=0 | Error: ReservedColumnId |
| ER-003 | Invalid column type | Column with type_id=14 | Error: InvalidColumnType |
| ER-004 | Exceed capacity | capacity=5, add 6 rows | Error: CapacityExceeded |
| ER-005 | Wrong data size | row_size=10, provide 5 bytes | Error: DataSizeMismatch |
| ER-006 | Row index out of bounds | row_count=3, get row 5 | Error: IndexOutOfBounds |
| ER-007 | Column not found | col_id=99 (not in DBuf) | Error: ColumnNotFound |
| ER-008 | Header corruption | Declared size != actual | Error: HeaderCorruption |
| ER-009 | Row size mismatch | Declared != sum of columns | Error: RowSizeMismatch |
| ER-010 | Buffer too small | row_count=10, not enough bytes | Error: BufferTooSmall |
| ER-011 | Invalid DBuf ID | d_buf_id=0 | Error: InvalidDBufId |
| ER-012 | Invalid rolling version | version=4 | Error: InvalidRollingVersion |
| ER-013 | Char size 0 | Char(0) column | Error: InvalidColumnSize |
| ER-014 | Too many columns | 2048 columns | Error: ColumnCountExceeded |

## Implementation Guidelines

1. **Bit Manipulation**: Use bitwise operations for packing/unpacking header fields
2. **Type Safety**: Use newtypes or wrappers for IDs to prevent mixing DBuf IDs and Column IDs
3. **Validation**: Always validate header integrity before accessing data
4. **Bounds Checking**: Check all indices and offsets against buffer size
5. **Error Handling**: Provide clear, specific error types for all failure modes
6. **Performance**: Optimize for the common case (valid, well-formed DBufs)
7. **Documentation**: Document all binary layout assumptions explicitly

## Usage Pattern

1. Define DBuf and Column enums for your application
2. Create DBuf instances with appropriate IDs and column definitions
3. Add rows with binary data
4. Read data using column IDs
5. Transfer ownership when needed for external processing

## Design Rationale

- **Bit-packed header**: Minimizes overhead while maintaining flexibility
- **Rolling version**: Enables backward compatibility across 4 versions
- **Optional column size**: Saves space for fixed-size types (numeric, boolean, time)
- **u16 IDs**: Balances compactness with sufficient address space
- **Calculated variables**: header_size and row_size computed separately for O(1) access
- **Header in raw_values**: Enables zero-copy transfer of complete DBuf state
