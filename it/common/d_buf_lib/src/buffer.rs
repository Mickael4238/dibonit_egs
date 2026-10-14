use crate::*;


pub struct DBufTOTO { pub headers: Vec<DBufHeaderTOTO>, pub raw_values: Vec<u8> }
impl DBufTOTO {
    pub fn new<T: ToDBufTOTO>(structure : &T,has_app_token : bool, has_user_token : bool, has_index : bool, app_token : Vec<u8>, user_token : Vec<u8> ) -> Re<Self> {
        let mut builder = DBuftBuilderTOTO::new(4, 0);
        builder.build_headers(structure,has_app_token, has_user_token, has_index, app_token, user_token).d_err_map(Fe::NewDBuf,1)?;
        builder.build_rowvalues(structure).d_err_map(Fe::NewDBuf,2)?;
        builder.consume().d_err_map(Fe::NewDBuf,3)
    }
    pub fn new_basic<T: ToDBufTOTO>(structure : &T) -> Re<Self> {
        DBufTOTO::new(structure,false, false, false, vec![], vec![]).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_tokens<T: ToDBufTOTO>(structure : &T,app_token : Vec<u8>, user_token : Vec<u8>) -> Re<Self> {
        DBufTOTO::new(structure,true, true, false, app_token, user_token).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_user_tokens<T: ToDBufTOTO>(structure : &T, user_token : Vec<u8>) -> Re<Self> {
        DBufTOTO::new(structure,false, true, false, vec![], user_token).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_app_tokens<T: ToDBufTOTO>(structure : &T,app_token : Vec<u8>) -> Re<Self> {
        DBufTOTO::new(structure,true, false, false, app_token, vec![]).d_err_map(Fe::NewDBuf,1)
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Re<DBufTOTO> {
        //todo read th child  headers, create a specific function in the builder?
        let header = DBufHeaderTOTO::from_bytes(&bytes)?;
        let header_size = header.header_size();
        if bytes.len() < header_size { return Err(DErr::RootError(Fe::FromBytesDBuf, 1, Rc::BufferTooSmall)); }
        let expected_data_size = header.row_count as usize * header.row_size as usize;
        let actual_data_size = bytes.len() - header_size;
        if actual_data_size < expected_data_size { return Err(DErr::RootError(Fe::FromBytesDBuf, 2, Rc::BufferTooSmall)); }
        Ok(DBufTOTO { headers: vec![header],  raw_values: bytes })
    }
}