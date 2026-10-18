# DBuf Library - Binary Data Buffer with Recursive Support

## Overview

DBuf is a binary data buffer library designed for efficient, zero-copy data transfer between systems. It provides a structured way to organize tabular data in a contiguous byte vector, with all metadata (header and column information) encoded in a compact binary format using bit-packing for efficiency.

**New in this version**: Support for recursive/nested DBufs through the `Child` column type, enabling hierarchical data structures similar to JSON or XML but with binary efficiency.

## Core Structure

The DBuf consists of a **header** and **raw values**, both stored in a single `Vec<u8>`. The header contains all metadata needed to interpret the raw data, with calculated variables (header_size, row_size) stored separately for O(1) access.

### Binary Layout

```
[BYTE_0: rolling_ver(2b) + col_count_high(3b) + unused(3b)]
[BYTE_1: col_count_low(8b)]
[BYTE_2-3: d_buf_id(16b)]
[COLUMN_DEFINITIONS...]
  [col_type(5b) + col_id_high(3b)][col_id_low(8b)]
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
     - **14: Child (6 bytes in row data)** - Points to nested DBuf
     - 15-31: Reserved
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

## Recursive DBuf Support (Child Column Type)

### Overview

The `Child` column type enables hierarchical/nested data structures. Instead of duplicating data, a `Child` column contains references (offsets) to child DBufs stored within the same `Vec<u8>`.

### Child Column Data Format

For a column of type `Child`, each row contains **6 bytes** with the following structure:

```
[child_header_offset: u16 (2 bytes)] - Absolute offset from start of raw_values to child DBuf header
[first_child_row_index: u16 (2 bytes)] - Index of first corresponding row in child DBuf
[last_child_row_index: u16 (2 bytes)] - Index of last corresponding row in child DBuf
```

**Total: 6 bytes per Child column per row**

### Why Only 6 Bytes?

The child DBuf's ID is **NOT** stored in the parent's row data because:
- The child DBuf's header already contains its `d_buf_id`
- This saves 2 bytes per Child column
- The type can still be validated by reading the child's header
- All necessary information is available through the offset

### Example: User → Orders → Products

```
User DBuf (ID=1):
  Header: [version][col_count=3][d_buf_id=1][columns...][row_count=1][row_capacity=10][row_size=22]
  Columns:
    - col_id=1, type=U32 (user_id)
    - col_id=2, type=Char(50) (name)
    - col_id=3, type=Child (orders)
  Row Data:
    [user_id:4b][name:50b][child_offset:2b][first_idx:2b][last_idx:2b]
    
Order DBuf (ID=2) - stored at child_offset:
  Header: [version][col_count=3][d_buf_id=2][columns...][row_count=2][row_capacity=40][row_size=18]
  Columns:
    - col_id=1, type=U32 (order_id)
    - col_id=2, type=U64 (date)
    - col_id=3, type=Child (products)
  Row Data:
    [order_id:4b][date:8b][child_offset:2b][first_idx:2b][last_idx:2b]
    [order_id:4b][date:8b][child_offset:2b][first_idx:2b][last_idx:2b]
    
Product DBuf (ID=3) - stored at its child_offset:
  Header: [version][col_count=2][d_buf_id=3][columns...][row_count=3][row_capacity=800][row_size=12]
  Columns:
    - col_id=1, type=U32 (product_id)
    - col_id=2, type=Char(8) (name)
  Row Data:
    [product_id:4b][name:8b]
    [product_id:4b][name:8b]
    [product_id:4b][name:8b]
```

### Key Features of Recursive DBufs

1. **Zero Duplication**: Child DBuf headers are stored once, not per reference
2. **Efficient Access**: O(1) access to child DBufs via absolute offsets
3. **Flexible Hierarchy**: Supports any depth of nesting (with configurable limit)
4. **Memory Contiguity**: All data (parent + children) in a single `Vec<u8>`
5. **Type Safety**: Child DBuf type can be validated through its header

### Maximum Depth Limit

To prevent stack overflow during deserialization and ensure safety:
- **Default limit**: 16 levels of recursion
- **Configurable**: Can be set per DBuf or globally
- **Validation**: Enforced during both serialization and deserialization

## Key Features

### 1. Compact Binary Encoding
- Bit-packed header fields minimize overhead
- u16 IDs support up to 65,535 unique DBufs and columns
- Optional column size field only for variable-length types (Char, ASCII)
- Child columns use only 6 bytes (offset + row indices)
- Eliminates string overhead and UTF-8 conversion errors

### 2. Zero-Copy Ownership Transfer
- The entire DBuf (header + data + children) is stored in a single `Vec<u8>`
- Ownership can be transferred to external libraries (e.g., socket libraries) without copying
- Enables efficient serialization/deserialization for network transfer

### 3. Developer-Friendly Traits

Applications using DBuf can implement the `IntoDBuf` trait:

```rust
pub trait IntoDBuf: Downcast + Sync {
    /// Unique identifier for this DBuf type
    fn d_buf_id() -> u16;
    
    /// Column definitions
    fn columns() -> Vec<ColumnHeader>;
    
    /// Estimate capacity for a slice of data
    fn estimate_capacity(data: &[Self]) -> u16;
    
    /// Convert instance to row data (excluding children)
    fn to_row_data(&self) -> Vec<u8>;
    
    /// Get child structures
    fn children(&self) -> Vec<&dyn IntoDBuf>;
    
    /// Optional: ID of the child column (if this type has children)
    fn child_column_id() -> Option<u16> { None }
}
```

This trait provides human-readable types while enabling automatic DBuf construction.

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
| **Child** | **14** | **6** | **Offset to child DBuf + row indices** |
| Reserved | 15-31 | - | For future use |

## Functional Use Cases

### Nominal Cases

1. **DBuf Creation**
   - Create a new DBuf with specified ID, columns, and capacity
   - Validate that all column IDs are unique within the DBuf
   - Validate that DBuf ID is unique in the application context
   - Validate that column count <= 2047
   - Validate that all column types are valid (0-14)

2. **Adding Rows**
   - Add one or more rows to the DBuf
   - Validate that the total rows do not exceed capacity (max 65,535)
   - Validate that the input data size matches `ROW_SIZE * number_of_rows`

3. **Reading Data**
   - Get a complete row by index
   - Get a specific column value from a row by column ID
   - Get the DBuf ID
   - Get column metadata (type, size) by column ID

4. **Child DBuf Operations**
   - Add a row with child DBuf references
   - Get a child DBuf from a parent row
   - Navigate hierarchical data structures
   - Validate child DBuf types

5. **Ownership Transfer**
   - Transfer ownership of the underlying `Vec<u8>` to external systems
   - Reconstruct DBuf from raw bytes (including children)
   - Validate header integrity during reconstruction

6. **Iteration**
   - Iterate over all rows
   - Iterate over all columns
   - Recursively iterate through child DBufs

### Recursive Use Cases

1. **Single Level Hierarchy**
   - Parent DBuf with Child column pointing to one child DBuf type
   - Example: User → Orders

2. **Multi-Level Hierarchy**
   - Parent DBuf with Child column pointing to child DBuf with its own Child column
   - Example: User → Orders → Products
   - Supports arbitrary depth (up to configured limit)

3. **Multiple Child Columns**
   - Single DBuf with multiple Child columns
   - Example: User with both `current_orders` and `historical_orders` Child columns
   - Each points to a separate Orders DBuf

4. **Empty Children**
   - Parent rows with no children (Child column contains zero/invalid offset)
   - Graceful handling of optional child relationships

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

5. **Recursion**
   - Maximum depth recursion (16 levels by default)
   - DBuf with multiple child types
   - Shared child DBuf references

## Error Cases

### Configuration Errors

| Error | Description |
|-------|-------------|
| DuplicateColumnId | Two columns with the same ID in a DBuf |
| ReservedColumnId | Column ID 0 (reserved for future use) |
| InvalidColumnType | Column type ID > 14 |
| InvalidColumnSize | Char/ASCII column with size 0 |
| ColumnIdExceeded | Column ID > 2047 |
| InvalidDBufId | DBuf ID 0 (reserved) |
| ColumnCountExceeded | More than 2047 columns |
| RowSizeExceeded | Row size exceeds u16::MAX |
| MaxDepthExceeded | Recursion depth exceeds limit |

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
| ChildNotFound | Child DBuf offset is invalid |
| BufferTooSmall | Attempt to read beyond buffer bounds |

### Data Integrity Errors

| Error | Description |
|-------|-------------|
| HeaderCorruption | Header fields are inconsistent or corrupted |
| RowSizeMismatch | Declared row size != sum of column sizes |
| ChildDBufTypeMismatch | Child DBuf type doesn't match expected type |
| InvalidChildOffset | Child offset points outside valid range |

## Recursive DBuf Construction

### Two-Phase Construction Process

The library provides a two-phase approach for building recursive DBufs:

1. **Discovery Phase**: Identify all unique DBuf types needed
2. **Construction Phase**: Create and fill all DBufs with data

### Example: Building User→Order→Product Hierarchy

```rust
// Define your types
struct Product { /* ... */ }
struct Order { /* ... */ products: Vec<Product> }
struct User { /* ... */ orders: Vec<Order> }

// Implement IntoDBuf for each
impl IntoDBuf for Product { /* ... */ }
impl IntoDBuf for Order { /* ... */ }
impl IntoDBuf for User { /* ... */ }

// Build the complete hierarchy
let users: Vec<User> = load_users();
let d_buf = build_d_buf_from_structure(&users, 16)?; // max depth = 16
```

### Implementation Details

The `build_d_buf_from_structure` function:
1. Recursively discovers all DBuf types (User, Order, Product)
2. Calculates exact capacities for each DBuf type
3. Creates all DBufs with appropriate capacities
4. Fills each DBuf with its data
5. Assembles everything into a single `Vec<u8>` with correct offsets

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
| F-012 | Add row with child | DBuf with Child column, child DBuf | Success, row with child reference |
| F-013 | Get child DBuf | Parent DBuf, valid Child column | Success, returns child DBuf |

### Recursive Tests

| Test ID | Description | Input | Expected Result |
|---------|-------------|-------|-----------------|
| R-001 | Single level hierarchy | User → Orders | Success, 2 DBufs created |
| R-002 | Multi-level hierarchy | User → Orders → Products | Success, 3 DBufs created |
| R-003 | Multiple children per parent | User with 5 Orders, each with Products | Success, all data preserved |
| R-004 | Empty children | User with no Orders | Success, Child column has null offset |
| R-005 | Multiple child columns | User with current_orders and historical_orders | Success, both Child columns work |
| R-006 | Deep hierarchy | 5+ levels of nesting | Success (if <= max_depth) |
| R-007 | Max depth exceeded | 17 levels with max_depth=16 | Error: MaxDepthExceeded |
| R-008 | Child offset validation | Invalid child offset | Error: InvalidChildOffset |
| R-009 | Child type validation | Wrong child DBuf type | Error: ChildDBufTypeMismatch |

### Edge Case Tests

| Test ID | Description | Input | Expected Result |
|---------|-------------|-------|-----------------|
| E-001 | Zero capacity DBuf | capacity=0 | Success, empty DBuf |
| E-002 | Maximum columns | 2047 unique column IDs | Success |
| E-003 | Maximum row capacity | capacity=65535 | Success |
| E-004 | Single column | 1 column | Success |
| E-005 | Single row | capacity=1, add 1 row | Success |
| E-006 | All column types | 1 column of each type (0-14) | Success |
| E-007 | Large Char column | Char(65535) | Success |
| E-008 | Column count spanning bytes | 256 columns | Success |
| E-009 | Column ID spanning bytes | col_id=256 | Success |
| E-010 | Child with zero rows | Child DBuf with row_count=0 | Success |

### Error Case Tests

| Test ID | Description | Input | Expected Result |
|---------|-------------|-------|-----------------|
| ER-001 | Duplicate column ID | Two columns with same ID | Error: DuplicateColumnId |
| ER-002 | Column ID 0 | Column with ID=0 | Error: ReservedColumnId |
| ER-003 | Invalid column type | Column with type_id=15 | Error: InvalidColumnType |
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
| ER-015 | Max depth exceeded | depth=17, max=16 | Error: MaxDepthExceeded |
| ER-016 | Invalid child offset | offset >= raw_values.len() | Error: InvalidChildOffset |

## Implementation Guidelines

1. **Bit Manipulation**: Use bitwise operations for packing/unpacking header fields
2. **Type Safety**: Use newtypes or wrappers for IDs to prevent mixing DBuf IDs and Column IDs
3. **Validation**: Always validate header integrity before accessing data
4. **Bounds Checking**: Check all indices and offsets against buffer size
5. **Error Handling**: Provide clear, specific error types for all failure modes
6. **Performance**: Optimize for the common case (valid, well-formed DBufs)
7. **Recursion Limit**: Always enforce maximum depth limit to prevent stack overflow
8. **Documentation**: Document all binary layout assumptions explicitly

## Usage Pattern

### Basic Usage

1. Define DBuf and Column enums for your application
2. Create DBuf instances with appropriate IDs and column definitions
3. Add rows with binary data
4. Read data using column IDs
5. Transfer ownership when needed for external processing

### Recursive Usage

1. Implement `IntoDBuf` trait for your data types
2. Call `build_d_buf_from_structure()` with your root data
3. Navigate hierarchy using `get_child()` method
4. Process child DBufs recursively

## Design Rationale

- **Bit-packed header**: Minimizes overhead while maintaining flexibility
- **Rolling version**: Enables backward compatibility across 4 versions
- **Optional column size**: Saves space for fixed-size types (numeric, boolean, time)
- **u16 IDs**: Balances compactness with sufficient address space
- **Calculated variables**: header_size and row_size computed separately for O(1) access
- **Header in raw_values**: Enables zero-copy transfer of complete DBuf state
- **Child column with 6 bytes**: Optimal balance between information and compactness
- **Offset-only references**: Eliminates redundant ID storage while maintaining type safety

## Example: Complete Recursive DBuf Implementation

```rust
// 1. Define your data structures
struct Product {
    id: u32,
    name: String,
    price: f32,
}

struct Order {
    id: u32,
    date: u64,
    products: Vec<Product>,
}

struct User {
    id: u32,
    name: String,
    orders: Vec<Order>,
}

// 2. Implement IntoDBuf for each type
impl IntoDBuf for Product {
    fn d_buf_id() -> u16 { 3 }
    
    fn columns() -> Vec<ColumnHeader> {
        vec![
            ColumnHeader::new(1, ColumnType::U32).unwrap(),
            ColumnHeader::new(2, ColumnType::Char(50)).unwrap(),
            ColumnHeader::new(3, ColumnType::F32).unwrap(),
        ]
    }
    
    fn estimate_capacity(data: &[Product]) -> u16 {
        data.len() as u16
    }
    
    fn to_row_data(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.to_be_bytes());
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.extend_from_slice(&self.price.to_be_bytes());
        bytes
    }
    
    fn children(&self) -> Vec<&dyn IntoDBuf> {
        Vec::new()
    }
    
    fn child_column_id() -> Option<u16> {
        None
    }
}

// Implement for Order and User similarly...

// 3. Build the complete hierarchy
let users = vec![
    User {
        id: 1,
        name: "Alice".to_string(),
        orders: vec![
            Order {
                id: 101,
                date: 1234567890,
                products: vec![
                    Product { id: 1001, name: "Laptop".to_string(), price: 999.99 },
                    Product { id: 1002, name: "Mouse".to_string(), price: 25.50 },
                ],
            },
        ],
    },
];

let d_buf = build_d_buf_from_structure(&users, 16)?;

// 4. Access the data
let user_d_buf = d_buf; // The root DBuf
let first_user_row = user_d_buf.get_row(0)?;
let orders_child_offset = /* extract from row */;
let orders_d_buf = user_d_buf.get_child(0, 3)?; // column_id=3 is orders
let first_order_row = orders_d_buf.get_row(0)?;
let products_d_buf = orders_d_buf.get_child(0, 3)?; // column_id=3 is products
```
