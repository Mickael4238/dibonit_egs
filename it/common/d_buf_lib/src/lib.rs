use std::fmt;
use std::string::FromUtf8Error;
use d_err_lib::{DErr, DErrMap};

/// A specialized `Result` type for operations in this module.
///
/// # Type Parameters
/// - `T`: The success type, wrapped in `Ok`.
/// - `DErr`: The full function path of an error and its root cause
type Re<T> = Result<T, DErr<Fe, Rc>>;
#[derive(Debug, Clone,PartialEq)]
pub enum ColumnType{
    Boolean, // TODO add some tests
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Char(usize),
    UtcTime,
}

impl ColumnType{
    pub fn size_of(&self) -> usize {
        match self {
            ColumnType::Boolean => {1} // TODO mix all boolean in a single byte
            ColumnType::U8 => {1}
            ColumnType::U16 => {2}
            ColumnType::U32 => {4}
            ColumnType::U64 => {8}
            ColumnType::I8 => {1}
            ColumnType::I16 => {2}
            ColumnType::I32 => {4}
            ColumnType::I64 => {8}
            ColumnType::F32 => {4}
            ColumnType::F64 => {8}
            ColumnType::Char(_count) => {*_count}
            ColumnType::UtcTime => {8}
        }
    }
}

#[derive(Clone)]
pub struct ColumnHeader{
    pub c_name : Name,
    pub c_type : ColumnType,
}
//#[allow(dead_code)]
#[derive(Clone)]
pub struct Column{
    pub c_header : ColumnHeader, //RUST BUG c_type is used but detected as unused
    pub c_start_offset : usize,
    pub c_size : usize,
   // c_end_offset : usize, // TODO see which to use     c_length / c_end_offset
}
#[derive(Clone)]
pub struct DBufHeader {
    pub name : Name,
    //  col_count : usize, //todo see not duplicates from columns.count
    pub columns : Vec<Column>,
    pub names_max_size : usize,
    //col_types : Vec<Box<dyn SizeOf>>,
    pub row_count : usize,
    pub row_capacity : usize,
    pub row_size  : usize,
}
/// Dibonit.BUFFER a structure representing a tabular data buffer.
///
/// # Internal Structure
///
/// The buffer consists of two main parts:
/// - **`d_buf_header`**: Header containing column metadata.
///   ```text
///   d_buf_header: {
///       col_count          : u16,              // Number of columns
///       col_names          : Vec<String>,      // Column names
///       col_names_max_size : u16,              // Maximum size of a column name
///       col_types          : Vec<Box<dyn SizeOf>>, // Column types
///       row_count          : u64               // Number of rows
///   }
///   ```
/// - **`d_buf_row_values`**: Vector containing row values (untyped in this example).
///
/// # public functions
/// - `new`: Creates a new DBuf instance.
/// - `add_row`: Adds a new row to the buffer.
/// - `get_row`: Retrieves a row by its index.
/// - `get_col`: Retrieves a column by a row index and its name.
///
/// # Usage Example
///
/// ```text
///     COL_A COL_B COL_C
/// ROW 1     A     10:12
/// ROW 2     B     10:13
/// ```
#[derive(Clone)]
pub struct DBuf {
    pub d_buf_header     : DBufHeader,
    pub d_buf_row_values : Vec<u8>,
}


impl DBuf { // col_types :Vec<Box<dyn SizeOf>>
    pub fn new(name : &Name, names_max_size : usize, cols_headers : &Vec<ColumnHeader>,row_capacity : usize) -> Re<DBuf> {
        DBuf::check_name (&name,names_max_size).d_err_map(Fe::NewDBuf,1)?;
        let mut columns : Vec<Column> = Vec::new();
        let mut row_size: usize = 0;
        if cols_headers.len() == 0 {
            return Err(DErr::RootError(Fe::NewDBuf,1,Rc::VectorLengthIrrelevant));
        }
        for col_header in cols_headers.iter() {
            columns.push(DBuf::build_column(col_header, names_max_size, &mut row_size).d_err_map(Fe::NewDBuf, 2)?);
        }

        //let row_size = DBuf::get_row_size(col_types);
        Ok(DBuf {
            d_buf_header : DBufHeader {
             //   col_count : cols_input_str.len(),
                name: name.clone(),
                columns,
                names_max_size,
                row_count : 0,
                row_capacity,
                row_size,
            },
            d_buf_row_values : {
                if row_capacity == 0 {Vec::new()}
                else {Vec::with_capacity(row_capacity * row_size)}
            },

        })
    }

    pub fn add_rows(&mut self, nb_rows:usize,rows_values : &Vec<u8>) -> Re<(&DBuf)> {
        if nb_rows + self.d_buf_header.row_count > self.d_buf_header.row_capacity {
            return Err(DErr::RootError(Fe::AddRows,1,Rc::VectorCapacityTooLow));
        }
        if nb_rows * self.d_buf_header.row_size != rows_values.len()  {
            return Err(DErr::RootError(Fe::AddRows,1,Rc::VectorLengthIrrelevant));
        }
        self.d_buf_row_values.extend(rows_values.clone());
        self.d_buf_header.row_count += nb_rows ;
        Ok((self))
    }
    pub fn get_row(&self, row_index : usize) -> Re<Vec<u8>> {
        let start_offset = self.d_buf_header.row_size * row_index;
        let end_offset = start_offset + self.d_buf_header.row_size;
        let tab_out = &self.d_buf_row_values[start_offset..end_offset];
        Ok(tab_out.to_vec())

      /*  // borrow
        let slice = &t[2..4]; // &[u8] - tu borrow les octets
        let s: &str = std::str::from_utf8(slice).unwrap(); // "II"

        // transfer ownership
        let s: String = String::from_utf8(t[2..4].to_vec()).unwrap(); // "II" // unsafe?

        if let Ok(s) = String::from_utf8(t[2..4]) { // safe
            println!("{}", s); // Affiche : HH
        } else {
            println!("error : les octets ne format pas une séquence UTF-8 valid");
        }**/
    }
    pub fn get_row_col_from_name(&self, row_index : usize, col_name : &Name) -> Re<Vec<u8>> {
        let column = self.find_column(col_name).d_err_map(Fe::GetRowColFromName,1)?;
        let start_offset = self.d_buf_header.row_size * row_index + column.c_start_offset;
        let end_offset = start_offset + column.c_size ;
        let tab_out = &self.d_buf_row_values[start_offset..end_offset];
        Ok(tab_out.to_vec())
    }

    fn find_column(&self,col_name: &Name) -> Re<&Column> {
        for column in  &self.d_buf_header.columns {
            if column.c_header.c_name == *col_name {
                return Ok(column);
            }
        }
        Err(DErr::RootError(Fe::FindColumn,1,Rc::ColumnNameNotFound(export_name_to_str(col_name)?)))
    }


    fn check_name(name : &Name, max_size_name : usize) -> Re<()> {
        let str_name = export_name_to_str(name).d_err_map(Fe::CheckNames,3)?;
        if name.len() > max_size_name {
            return Err(DErr::RootError(Fe::CheckNames,1, Rc::ColumnNameTooBig(str_name,max_size_name)));
        }
        for c in name {
            //if !c.is_ascii() { return Err(DErr::RootError(Fe::ExtractColumnName,2, Rc::ColumnNameNotAscii(c.)));}
            if DBuf::is_forbidden(*c) { return Err(DErr::RootError(Fe::CheckNames,2, Rc::ColumnNameForbiddenChar(*c as char)));}
        }
        Ok(())
    }
    fn build_column(c_header: &ColumnHeader, max_size_name : usize, tmp_row_size : &mut usize) -> Re<Column> {
        DBuf::check_name(&c_header.c_name,max_size_name).d_err_map(Fe::BuildColumn,1)?;
        let c_start_offset = *tmp_row_size;
        let c_size = c_header.c_type.size_of();
        *tmp_row_size = *tmp_row_size + c_size;
        Ok(Column{
            c_header :c_header.clone(),
            c_start_offset,
            c_size,
       //     c_end_offset: *tmp_row_size-1,
        })
    }
    fn is_forbidden(c: u8) -> bool {
        matches!(c as char, '"' | '\\' | '\n' | '\r' | ' ')
    }
}

type Name = Vec<u8>;
pub fn extract_name_from_str(c_name_str : &str) -> Name{
    c_name_str.as_bytes().into()
}
pub fn export_name_to_str(c_name_str : &Name) -> Re<String> {
    Ok(String::from_utf8(c_name_str.clone()).map_err(|e|  DErr::RootError(Fe::ExportNameToStr,2, Rc::Utf8ConversionFailed(e.to_string())))?)
}

pub trait SizeOf {
    fn size_of(&self) -> usize;
}
#[derive(Debug)]
pub enum Rc {
    VectorLengthIrrelevant,
    VectorCapacityTooLow,
    ColumnNameTooBig(String,usize),
    ColumnNameNotAscii(char),
    ColumnNameForbiddenChar(char),
    ColumnNameNotFound(String),
    Utf8ConversionFailed(String),
}

impl fmt::Display for Rc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rc::VectorLengthIrrelevant => write!(f, "VectorLengthIrrelevant"),
            Rc::VectorCapacityTooLow => write!(f, "VectorCapacityTooLow"),
            Rc::ColumnNameTooBig(size,name) => write!(f, "ColumnNameTooBig({}:{})",name,size),
            Rc::ColumnNameNotAscii(c) => write!(f, "ColumnNameNotAscii({})", c),
            Rc::ColumnNameForbiddenChar(c) => write!(f, "ColumnNameForbiddenChar({})", c),
            Rc::ColumnNameNotFound(name) => write!(f, "ColumnNameNotFound({})", name),
            Rc::Utf8ConversionFailed(err) => write!(f, "ExportNameToStr({})", err),
        }
    }
}

#[derive(Debug)]
/// Function where the error transits
pub enum Fe {
    NewDBuf,
    BuildColumn,
    CanBeLowerOrEqualThanKey,
    AddRows,
    FindColumn,
    GetRowColFromName,
    GetRowColFromIndex,
    ExtractColumnName,
    ExportNameToStr,
    CheckNames,
}

#[cfg(test)]
mod tests {
    use super::*;
        fn build_vec_input_column()-> Vec<ColumnHeader>{
            vec![
                ColumnHeader { c_name : Vec::from(b"a"), c_type: ColumnType::U8 },
                ColumnHeader { c_name : Vec::from(b"b"), c_type: ColumnType::U16 },
                ColumnHeader { c_name : Vec::from(b"c"), c_type: ColumnType::U32 },
            ]
        }
        // ============ Tests DBuf::find_column (private) ============
        #[test]
        fn test0001_extract_char_ascii() {
            let s = "Hello world!";
            let c_name = extract_name_from_str(s);
            assert_eq!(c_name, vec![72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33]);
            }
        #[test]
        fn test0002_extract_char_non_ascii() {
            let s = "Hello world!中";
            let c_name = extract_name_from_str(s);
            assert_eq!(c_name, vec![72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33, 228, 184, 173]);
        }

        #[test]
        fn test0003_export_char_ascii() {
            let octets = b"Hello"; // Un &[u8]
            let string = export_name_to_str(&Vec::from(octets)).unwrap();
            assert_eq!(string, "Hello");
        }
        #[test]
        fn test0004_export_char_non_ascii() {
            let input_str = "Hello world!中";
            let c_name = extract_name_from_str(input_str);
            let output_str = export_name_to_str(&c_name).unwrap();
            assert_eq!(input_str, output_str);
        }
        // ============ Tests DBuf::build_column ============
        #[test]
        fn test_001_build_column_forbidden_char_quote() {
            let cols = vec![
                ColumnHeader { c_name : Vec::from(b"name\"test"), c_type: ColumnType::U8 },
            ];
            match DBuf::new(&Vec::from(b""),20, &cols, 10){
                Err(e) => assert_eq!(e.to_string(),"NewDBuf[2]/BuildColumn[1]/CheckNames[2]=>ColumnNameForbiddenChar('\"')"),
                _ => assert!(false)
            }
        }

        #[test]
        fn test_002_build_column_forbidden_char_backslash() {
            let cols = vec![
                ColumnHeader { c_name : Vec::from(b"name\\test"), c_type: ColumnType::U8 },
            ];
            match DBuf::new(&Vec::from(b"a"),20, &cols, 10){
                Err(e) => assert_eq!(e.to_string(),"NewDBuf[2]/BuildColumn[1]/CheckNames[2]=>ColumnNameForbiddenChar('\\\\')"),
                _ => assert!(false)
            }
        }

        #[test]
        fn test_003_build_column_forbidden_char_newline() {
            let cols = vec![
                ColumnHeader { c_name : Vec::from(b"name\ntest"), c_type: ColumnType::U8 },
            ];
            match DBuf::new(&Vec::from(b"a"),20, &cols, 10){
                Err(e) => assert_eq!(e.to_string(),"NewDBuf[2]/BuildColumn[1]/CheckNames[2]=>ColumnNameForbiddenChar('\\n')"),
                _ => assert!(false)
            }
        }

        #[test]
        fn test_004_build_column_forbidden_char_space() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"name test"), c_type: ColumnType::U8 },
            ];
            match DBuf::new(&Vec::from(b"a"),20, &cols, 10){
                Err(e) => assert_eq!(e.to_string(),"NewDBuf[2]/BuildColumn[1]/CheckNames[2]=>ColumnNameForbiddenChar(' ')"),
                _ => assert!(false)
            }
        }

        // ============ Tests edge cases ============
        #[test]
        fn test_005_empty_column_list() {
            let cols: Vec<ColumnHeader> = vec![];
            match DBuf::new(&Vec::from(b"a"),10, &cols, 10){
                Err(e) => assert_eq!(e.to_string(),"NewDBuf[1]=>VectorLengthIrrelevant"),
                _ => assert!(false)
            }
        }

        #[test]
        fn test_006_large_char_column() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"text"), c_type: ColumnType::Char(1000) },
            ];
            match DBuf::new(&Vec::from(b"a"),10, &cols, 5){
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                _ => assert!(true)
            }
        }

        #[test]
        fn test_007_all_column_types_together() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"u8")  ,c_type: ColumnType::U8 },
                ColumnHeader {c_name : Vec::from(b"u16") ,c_type: ColumnType::U16 },
                ColumnHeader {c_name : Vec::from(b"u32") ,c_type: ColumnType::U32 },
                ColumnHeader {c_name : Vec::from(b"u64") ,c_type: ColumnType::U64 },
                ColumnHeader {c_name : Vec::from(b"i8")  ,c_type: ColumnType::I8 },
                ColumnHeader {c_name : Vec::from(b"i16") ,c_type: ColumnType::I16 },
                ColumnHeader {c_name : Vec::from(b"i32") ,c_type: ColumnType::I32 },
                ColumnHeader {c_name : Vec::from(b"i64") ,c_type: ColumnType::I64 },
                ColumnHeader {c_name : Vec::from(b"f32") ,c_type: ColumnType::F32 },
                ColumnHeader {c_name : Vec::from(b"f64") ,c_type: ColumnType::F64 },
                ColumnHeader {c_name : Vec::from(b"char"),c_type: ColumnType::Char(5) },
                ColumnHeader {c_name : Vec::from(b"time"),c_type: ColumnType::UtcTime },
            ];
            match DBuf::new(&Vec::from(b"a"),10, &cols, 10){
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(r) => assert_eq!(r.d_buf_header.columns.len(), cols.len())
            }
        }

        // ============ Tests DBuf::new ============
        #[test]
        fn test_501_new_d_buf_success() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"id")  ,c_type: ColumnType::U32 },
                ColumnHeader {c_name : Vec::from(b"name"),c_type: ColumnType::Char(10) },
            ];
            match DBuf::new(&Vec::from(b"a"),10, &cols, 100){
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(r) => assert_eq!(r.d_buf_header.row_capacity,100)
            }
        }

        #[test]
        fn test_502_new_d_buf_zero_capacity() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"id"), c_type: ColumnType::U32 },
            ];
            match DBuf::new(&Vec::from(b"a"),10, &cols, 0){
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(r) => assert_eq!(r.d_buf_header.row_capacity,0)
            }
        }

        #[test]
        fn test_503_new_d_buf_column_name_too_big() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"very_long_name"), c_type: ColumnType::U32 },
            ];
            match DBuf::new(&Vec::from(b"a"),5, &cols, 10){
                Err(e) => assert_eq!(e.to_string(),"NewDBuf[2]/BuildColumn[1]/CheckNames[1]=>ColumnNameTooBig(\"very_long_name\", 5)"),
                _ => assert!(false)
            }
        }

        #[test]
        fn test_504_new_d_buf_multiple_columns() {
            let cols = build_vec_input_column();
            match DBuf::new(&Vec::from(b"a"),10, &cols, 50){
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(r) => assert_eq!(r.d_buf_header.columns.len(),cols.len())
            }
        }

        // ============ Tests DBuf::add_rows ============
        #[test]
        fn test_601_add_rows_success() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"id"), c_type: ColumnType::U32 },
            ];
            let mut d_buf = DBuf::new(&Vec::from(b"a"),10, &cols, 10).unwrap();
            let data = vec![1u8, 0, 0, 0]; // U32 = 4 bytes
            match d_buf.add_rows(1, &data){
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(..) => assert!(true)
            }
        }

        #[test]
        fn test_602_add_rows_capacity_exceeded() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"id"), c_type: ColumnType::U32 },
            ];
            let mut d_buf = DBuf::new(&Vec::from(b"a"),10, &cols, 2).unwrap();
            let data = vec![1u8, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]; // 3 rows
            match d_buf.add_rows(3, &data){
                Err(e) => assert_eq!(e.to_string(),"AddRows[1]=>VectorCapacityTooLow"),
                _ => assert!(false)
            }
        }

        #[test]
        fn test_603_add_rows_wrong_length() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"id") , c_type: ColumnType::U32 },
            ];
            let mut d_buf = DBuf::new(&Vec::from(b"a"),10, &cols, 10).unwrap();
            let data = vec![1u8, 0, 0]; // 3 bytes au lieu de 4
            match d_buf.add_rows(1, &data){
                Err(e) => assert_eq!(e.to_string(),"AddRows[1]=>VectorLengthIrrelevant"),
                _ => assert!(false)
            }
        }

        #[test]
        fn test_604_add_multiple_rows() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"id") , c_type: ColumnType::U16 },
            ];
            let mut d_buf = DBuf::new(&Vec::from(b"a"),10, &cols, 10).unwrap();
            let data = vec![1u8, 0, 2, 0, 3, 0]; // 3 rows * 2 bytes
            match d_buf.add_rows(3, &data){
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(..) => assert!(true)
            }
        }

        // ============ Tests DBuf::get_row ============
        #[test]
        fn test_610_get_row_success() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"id") , c_type: ColumnType::U32 },
            ];
            let mut d_buf = DBuf::new(&Vec::from(b"a"),10, &cols, 10).unwrap();
            let data = vec![10u8, 20, 30, 40];
            d_buf.add_rows(1, &data).unwrap();
            match d_buf.get_row(0){
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(v) => assert_eq!(v.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(":")
                                        , data.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(":")),
            }
        }

        #[test]
        fn test_611_get_row_multiple_rows() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"id") , c_type: ColumnType::U16 },
            ];
            let mut d_buf = DBuf::new(&Vec::from(b"a"),10, &cols, 10).unwrap();
            let row1 = vec![1u8, 0,];
            let row2 = vec![2u8, 0,];
            let row3 = vec![3u8, 0,];

            let mut data : Vec<u8> = Vec::with_capacity(6);
            data.extend_from_slice(&row1);
            data.extend_from_slice(&row2);
            data.extend_from_slice(&row3);
            match d_buf.add_rows(3, &data){
                Err(e)=> assert_eq!(e.to_string(),"no_error"),
                Ok(..)=> {assert!(true);}
            }
            match d_buf.get_row(0) {
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(v) => assert_eq!(v.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(":")
                                    , row1.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(":")),
            }
            match d_buf.get_row(1) {
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(v) => assert_eq!(v.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(":")
                                    , row2.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(":")),
            }
            match d_buf.get_row(2) {
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(v) => assert_eq!(v.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(":")
                                    , row3.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(":")),
            }
        }

        // ============ Tests DBuf::get_row_col_from_name ============
        #[test]
        fn test_612_get_row_col_from_name_success() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"id") , c_type: ColumnType::U16 },
                ColumnHeader {c_name : Vec::from(b"age"), c_type: ColumnType::U16 },
            ];
            let mut d_buf = DBuf::new(&Vec::from(b"a"),10, &cols, 10).unwrap();
            let data = vec![1u8, 2, 25, 1]; // id=1, age=25
            d_buf.add_rows(1, &data).unwrap();

            match d_buf.get_row_col_from_name(0, &Vec::from(b"age")){
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(v) => assert_eq!(v[0],25)
            }
        }

        #[test]
        fn test_613_get_row_col_from_name_not_found() {
            let cols = vec![
                ColumnHeader {c_name : Vec::from(b"a") , c_type: ColumnType::U16 },
            ];
            let mut d_buf = DBuf::new(&Vec::from(b"a"),10, &cols, 10).unwrap();
            let data = vec![1u8, 0];
            d_buf.add_rows(1, &data).unwrap();

            match d_buf.get_row_col_from_name(0, &Vec::from(b"unknown")){
                Err(e) => assert_eq!(e.to_string(),"GetRowColFromName[1]/FindColumn[1]=>ColumnNameNotFound(\"unknown\")"),
                _ => assert!(false)
            }
        }

        #[test]
        fn test_614_get_row_col_from_name_multiple_columns() {
            let cols = build_vec_input_column();
            let mut d_buf = DBuf::new(&Vec::from(b"a"),10, &cols, 10).unwrap();
            let data = vec![1u8, 2, 0, 3, 0, 0, 0]; // a=1, b=2, c=3
            d_buf.add_rows(1, &data).unwrap();


            match d_buf.get_row_col_from_name(0, &Vec::from(b"a")){
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(r) => assert_eq!(r.len(), 1),
            }
            match d_buf.get_row_col_from_name(0,  &Vec::from(b"b")) {
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(r) => assert_eq!(r.len(), 2),
            }
            match d_buf.get_row_col_from_name(0,  &Vec::from(b"c")) {
                Err(e) => assert_eq!(e.to_string(),"no_error"),
                Ok(r) => assert_eq!( r.len(), 4),
            }
        }


    /*
    #[test]
    fn test_new_d_buf_struct() {
        // Utilisation
        let mut header = DBufHeader { vec_types: Vec::new() };
        header.add_data(Box::new(MyCustomType { field1: 42, field2: String::from("test") }));

    }
    #[test]
    fn test_new_d_buf() {
        let d_buf = DBuf::new(10, vec!["a".to_string(), "b".to_string()], vec![Box::new(10), Box::new(10)]);
        assert_eq!(d_buf.is_ok(), true);
    }
    #[test]
    fn test_new_d_buf_err() {
        let d_buf = DBuf::new(3, vec!["aaa".to_string(), "b".to_string()], vec![Box::new(10),]);
        assert_eq!(d_buf.is_err(), true);
        if let Err(e) = res {
            assert_eq!(e.to_string_chain(), "NewBuf[1]=>VectorLengthNotEquals");
        }
    }
    #[test]
    fn test_new_d_buf_col_err() {
        let d_buf = DBuf::new(3, vec!["aaaa".to_string(), "b".to_string()], vec![Box::new(10), Box::new(10)]);
        assert_eq!(d_buf.is_err(), true);
        if let Err(e) = res {
            assert_eq!(e.to_string_chain(), "NewBuf[1]/NewColName[3]=>ColumnNameTooBig");
        }
    }
    #[test]
    fn test_new_col_name() {
        let col_name = ColName::new("TEST_OK", 7);
        assert_eq!(col_name.is_ok(), true);
    }
    #[test]
    fn test_new_col_name_not_ascii() {
        let col_name = ColName::new("é", 10);
        if let Err(e) = res {
            assert_eq!(e.to_string_chain(), "NewColName[1]=>ColumnNameNotAscii");
        }
    }
    #[test]
    fn test_new_col_name_is_forbidden() {
        let col_name = ColName::new("a\n", 10);
        assert_eq!(col_name.is_err(), true);
        if let Err(e) = res {
            assert_eq!(e.to_string_chain(), "NewColName[2]=>ColumnNameForbiddenChar");
        }
    }
    #[test]
    fn test_new_col_name_too_big() {
        let col_name = ColName::new("ab", 1);
        assert_eq!(col_name.is_err(), true);
        if let Err(e) = res {
            assert_eq!(e.to_string_chain(), "NewColName[3]=>ColumnNameTooBig");
        }
    }*/
}
