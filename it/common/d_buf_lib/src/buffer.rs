use crate::*;

pub(crate) type Bsize = usize;   // type of position in buffer, (must contain the maximum buffer length) Within the colum it could be replaced by the ID of the child, the index position of the child in the buffer...
pub type Rsize = usize;
pub type Vsize = u8;            // type of rolling versions
const ROLLING_VERSION: Vsize = 0;
pub struct DBufTOTO {
    pub d_buf_id: u16,
    pub rolling_version: u8,
    pub has_app_token: bool,
    pub has_user_token: bool,
    pub app_token: Vec<u8>,
    pub user_token: Vec<u8>,
    pub tables: Vec<DBufTable>,
    pub raw_values: Vec<u8>,
}
impl DBufTOTO {
    pub fn new<T: ToDBufTOTO>(structure : &T,d_buf_id : u16,has_app_token : bool, has_user_token : bool, app_token : Vec<u8>, user_token : Vec<u8>, max_depth: u8 ) -> Re<Self> {
        let mut builder = DBuftBuilderTOTO::new(d_buf_id,ROLLING_VERSION,has_app_token, has_user_token, app_token, user_token, max_depth);
        builder.build_tables(structure).d_err_map(Fe::NewDBuf,1)?;
        builder.build_rowvalues(structure).d_err_map(Fe::NewDBuf,2)?;
        builder.consume().d_err_map(Fe::NewDBuf,3)
    }
    pub fn new_basic<T: ToDBufTOTO>(structure : &T,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure, 1, false, false, vec![], vec![], max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_tokens<T: ToDBufTOTO>(structure : &T,app_token : Vec<u8>, user_token : Vec<u8>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1,true, true,  app_token, user_token, max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_user_tokens<T: ToDBufTOTO>(structure : &T, user_token : Vec<u8>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1,false, true, vec![], user_token, max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_app_tokens<T: ToDBufTOTO>(structure : &T,app_token : Vec<u8>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1,true, false, app_token, vec![], max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Re<DBufTOTO> {
        //todo read th child  headers, create a specific function in the builder?
        let header = DBufTable::from_bytes(&bytes)?;
        let header_size = header.header_size();
        if bytes.len() < header_size { return Err(DErr::RootError(Fe::FromBytesDBuf, 1, Rc::BufferTooSmall)); }
        let expected_data_size = header.row_count as usize * header.row_size as usize;
        let actual_data_size = bytes.len() - header_size;
        if actual_data_size < expected_data_size { return Err(DErr::RootError(Fe::FromBytesDBuf, 2, Rc::BufferTooSmall)); }
        Ok(DBufTOTO { tables: vec![header],  raw_values: bytes })
    }
}