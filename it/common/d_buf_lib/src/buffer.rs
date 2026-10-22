use crate::*;

pub type Bsize = usize;   // type of position in buffer, (must contain the maximum buffer length) Within the colum it could be replaced by the ID of the child, the index position of the child in the buffer...
pub type Bpos = Bsize;
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
    pub fn new<T: ToDBufStruct>(structure : &Vec<T>,d_buf_id : u16,has_app_token : bool, has_user_token : bool, app_token : Vec<u8>, user_token : Vec<u8>, max_depth: u8 ) -> Re<Self> {
        let mut builder = DBuftBuilderTOTO::new(d_buf_id,ROLLING_VERSION,has_app_token, has_user_token, app_token, user_token, max_depth);
        builder.build_tables(structure).d_err_map(Fe::NewDBuf,1)?;
        builder.build_rowvalues(structure).d_err_map(Fe::NewDBuf,2)?;
        builder.consume().d_err_map(Fe::NewDBuf,3)
    }
    pub fn new_basic<T: ToDBufStruct>(structure : &Vec<T>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure, 1, false, false, vec![], vec![], max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_tokens<T: ToDBufStruct>(structure : &Vec<T>,app_token : Vec<u8>, user_token : Vec<u8>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1,true, true,  app_token, user_token, max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_user_tokens<T: ToDBufStruct>(structure : &Vec<T>, user_token : Vec<u8>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1,false, true, vec![], user_token, max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_app_tokens<T: ToDBufStruct>(structure : &Vec<T>,app_token : Vec<u8>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1,true, false, app_token, vec![], max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Re<DBufTOTO> {
        //todo read th child  headers, create a specific function in the builder? in the table?
        let tables = vec![DBufTable::from_bytes(&bytes)?,];
        let table_size = tables[0].header_size();
        if bytes.len() < table_size { return Err(DErr::RootError(Fe::FromBytesDBuf, 1, Rc::BufferTooSmall)); }
        let expected_data_size = tables[0].row_count as usize * tables[0].row_size as usize;
        let actual_data_size = bytes.len() - table_size;
        if actual_data_size < expected_data_size { return Err(DErr::RootError(Fe::FromBytesDBuf, 2, Rc::BufferTooSmall)); }

        Ok(DBufTOTO { d_buf_id: 0, rolling_version: 0, has_app_token: false, has_user_token: false, app_token: vec![], user_token: vec![], tables,  raw_values: bytes })
    }
}