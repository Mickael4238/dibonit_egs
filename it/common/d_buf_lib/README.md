# DBuf Library - Binary Data Buffer

## Overview

DBuf is a binary data buffer library designed for efficient, zero-copy data transfer between systems. It provides a structured way to organize tabular data in a contiguous byte vector, with all metadata (header and column information) encoded as `u8` bytes.

## Core Structure

The DBuf consists of a **header** and **raw values**, both stored in a single `Vec<u8>`. The header contains all metadata needed to interpret the raw data.

### Binary Layout

```
[ROLLING_VERSION + COLUMN_COUNT: u8*2][BUFFER_ID: u8*2][COLUMN_1_ID + COLUMN_1_TYPE : u8*2][OPTIONAL COLUMN_1_SIZE: u8*2]...[ROW_COUNT: u8*2][ROW_CAPACITY: u8*2][ROW_SIZE: u8*2][RAW_VALUES...]
```

#### Header Fields
1. **Dbuf rolling version (2 bit)**: so a newer code can handle 3 previous version of the Dbuff header and data structure. 3 bit are not used.
2. **Column Count (3 bit from 1st byte + 1 byte)**: Number of columns in this DBuf.(max 2048)
3. **DBuf ID (variable)**: Unique identifier for this DBuf type. (2 byte)
4. **Column Definitions**: For each column:
   - Column ID (1 byte + 3 bit): Unique identifier for the column (replaces string names)
   - Column Type (5 bit): Encoded type (U8, U16, U32, U64, I8, I16, I32, I64, F32, F64, Boolean, UtcTime, Char(Column Size),ASCII(Column Size)) (max 32 types)
   - optional Column size (2 byte): Size of this column's data in each row. this field is only present when the Type is char or ASCII.
5. **Row Count (2 byte)**: Current number of rows stored.
6. **Row Capacity (2 byte)**: Maximum number of rows the buffer can hold.

#### calculated variables 
some useful information calculated from the above fields can be stored in variables outside the buffer for example: header size, row size.
1. ** header size ** which can be used as an offset for getting the 1st row_value.
2. ** row size ** which can be used as a multuplicator for going to next rows.
developer may add others.

#### Raw Values

All row data follows the header contiguously. Each row contains values for all columns in the order they were defined. Access to row data must account for the header offset.

## Key Features

### 1. Binary-Only IDs
- Column names are replaced with `u16` IDs (0-65235)
- DBuf names are replaced with `u16` IDs
- Eliminates string overhead and UTF-8 conversion errors
- Enables direct binary comparison and faster lookups

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

These enums provide human-readable names while compiling to the underlying `u8` IDs.

## Column Types

| Type | ID | Size (bytes) |
|------|----|--------------|
| Boolean | 0 | 1 | note : 8 booleans should take 1 byte as well
| U8 | 1 | 1 |
| U16 | 2 | 2 |
| U32 | 3 | 4 |
| U64 | 4 | 8 |
| I8 | 5 | 1 |
| I16 | 6 | 2 |
| I32 | 7 | 4 |
| I64 | 8 | 8 |
| F32 | 9 | 4 |
| F64 | 10 | 8 |
| UtcTime | 11 | 8 |
| Char | 12 | specified in Column Size
| ASCII | 13 | specified in Column Size
## Functional Use Cases

### Nominal Cases

1. **DBuf Creation**
   - Create a new DBuf with specified ID, columns, and capacity
   - Validate that all column IDs are unique within the DBuf
   - Validate that DBuf ID is unique in the application context

2. **Adding Rows**
   - Add one or more rows to the DBuf
   - Validate that the total rows do not exceed capacity
   - Validate that the input data size matches `ROW_SIZE * number_of_rows`

3. **Reading Data**
   - Get a complete row by index
   - Get a specific column value from a row by column ID
   - Get the DBuf ID
   - Get column metadata (type, size) by column ID

4. **Ownership Transfer**
   - Transfer ownership of the underlying `Vec<u8>` to external systems
   - Reconstruct DBuf from raw bytes

5. **Iteration**
   - Iterate over all rows
   - Iterate over all columns

### Edge Cases

1. **Empty DBuf**
   - Create DBuf with zero capacity
   - Create DBuf with zero rows initially
   - Access operations on empty DBuf

2. **Maximum Sizes**
   - DBuf with maximum column count (2048)
   - DBuf with maximum row capacity (65,536)
   - Column with maximum size (65,536) bytes for Char type)

3. **Single Element**
   - DBuf with one column
   - DBuf with one row
   - Column of size 1 byte

4. **Mixed Types**
   - DBuf with all supported column types
   - Rows with mixed numeric types

## Error Cases

### Configuration Errors

2. **Invalid Column Definition**
   - Duplicate column ID within a DBuf
   - Column ID 0 (reserved)
   - Invalid column type ID
   - Column size 0

3. **Capacity Errors**
   - Row capacity is 0 but rows are added
   - Adding rows exceeds capacity
   - Row data size doesn't match expected row size

4. **Access Errors**
   - Row index out of bounds
   - Column ID not found in DBuf
   - Attempt to read beyond buffer bounds

### Data Integrity Errors

1. **Header Corruption**
   - Header size doesn't match actual header bytes
   - Declared row count exceeds capacity
   

2. **Type Mismatch**
   - Attempt to read column as wrong type
   - Char/ascii type with size 0

3. **Buffer Overflow**
   - Raw values vector smaller than declared row count * row size
   - Header extends beyond vector bounds

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
| F-007 | Get DBuf ID | Any DBuf | Success, returns DBuf ID bytes |
| F-008 | Get column metadata | DBuf, valid col_id | Success, returns type and size |
| F-009 | Iterate all rows | DBuf with 5 rows | Success, returns all 5 rows |
| F-010 | Transfer ownership | DBuf | Success, external system owns bytes |

### Edge Case Tests

| Test ID | Description | Input | Expected Result |
|---------|-------------|-------|-----------------|
| E-001 | Zero capacity DBuf | capacity=0 | Success, empty DBuf |
| E-002 | Maximum columns | 2056 unique column IDs | Success |
| E-003 | Maximum row capacity | capacity= 65,536| Success |
| E-004 | Single column | 1 column | Success |
| E-005 | Single row | capacity=1, add 1 row | Success |
| E-006 | All column types | 1 column of each type | Success |
| E-007 | Large Char column | Char(65,536/4) | Success |
| E-008 | Empty DBuf ID | DBuf ID size=0 | Success (if allowed) or Error |

### Error Case Tests

| Test ID | Description | Input | Expected Result |
|---------|-------------|-------|-----------------|
| ER-001 | Duplicate column ID | Two columns with same ID | Error: DuplicateColumnId |
| ER-002 | Column ID 0 | Column with ID=0 | Error: ReservedColumnId |
| ER-003 | Invalid column type | Column with type_id=255 | Error: InvalidColumnType |
| ER-004 | Exceed capacity | capacity=5, add 6 rows | Error: CapacityExceeded |
| ER-005 | Wrong data size | row_size=10, provide 5 bytes | Error: DataSizeMismatch |
| ER-006 | Row index out of bounds | row_count=3, get row 5 | Error: IndexOutOfBounds |
| ER-007 | Column not found | col_id=99 (not in DBuf) | Error: ColumnNotFound |
| ER-008 | Header size mismatch | Declared size != actual | Error: HeaderCorruption |
| ER-009 | Row size mismatch | Declared != sum of columns | Error: RowSizeMismatch |
| ER-010 | Buffer too small | row_count=10, but not enough bytes | Error: BufferOverflow |
| ER-011 | Invalid DBuf ID size | size=. 65,537 (too large) | Error: InvalidDBufIdSize |

## Implementation Guidelines

1. **Type Safety**: Use newtypes or wrappers for IDs to prevent mixing DBuf IDs and Column IDs
2. **Validation**: Always validate header integrity before accessing data
3. **Bounds Checking**: Check all indices and offsets against buffer size
4. **Error Handling**: Provide clear, specific error types for all failure modes
5. **Performance**: Optimize for the common case (valid, well-formed DBufs)
6. **Documentation**: Document all binary layout assumptions explicitly

## Usage Pattern

1. Define DBuf and Column enums for your application
2. Create DBuf instances with appropriate IDs and column definitions
3. Add rows with binary data
4. Read data using column IDs
5. Transfer ownership when needed for external processing

## Design Rationale

- **Header in raw_values**: Enables zero-copy transfer of complete DBuf state
- **calculated header size field**: Allows O(1) access to data section
- **optional colomn size** remove unused bytes for numeric colons where the size is known by the Type.
- **4 bit rolling version** allows code to work on older version of Dbuff structure when not all applications using the Dbuff had been migrated.