use crate::*;
use crate::serializer::DTableBox;

pub type Bsize = usize;   // type of position in buffer, (must contain the maximum buffer length) Within the colum it could be replaced by the ID of the child, the index position of the child in the buffer...
pub type Bpos = Bsize;
pub type Rsize = usize;
pub type Vsize = u8;            // type of rolling versions



pub struct DBufTOTO {
    pub rolling_version: RollingVersion,
    pub d_buf_id: u16,
    pub app_token: Token,
    pub user_token: Token,
    pub tables: Vec<DBufTable>,
    pub raw_values: Vec<u8>,
}
impl DBufTOTO {
    pub fn new(structure :DTableBox,d_buf_id : u16, app_token : Option<Vec<u8>>, user_token :Option<Vec<u8>>, max_depth: u8 ) -> Re<Self> {
        let mut builder = DBuftBuilderTOTO::new(d_buf_id,RollingVersion::Current, app_token, user_token, max_depth).d_err_map(Fe::NewDBuf, 1)?;
        builder.build_tables(&structure).d_err_map(Fe::NewDBuf, 1)?;
        builder.consume().d_err_map(Fe::NewDBuf,3)
    }
    pub fn new_basic(structure : DTableBox,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure, 1, None, None, max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_tokens(structure : DTableBox,app_token : Option<Vec<u8>>, user_token :Option<Vec<u8>>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1, app_token, user_token, max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_user_tokens(structure : DTableBox, user_token : Option<Vec<u8>>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1,None, user_token, max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_app_tokens(structure : DTableBox,app_token : Option<Vec<u8>>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1, app_token, None, max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Re<DBufTOTO> {
        //todo read th child  headers, create a specific function in the builder? in the table?
        let tables = vec![DBufTable::from_bytes(&bytes)?,];
        let table_size = tables[0].calculate_header_size();
        if bytes.len() < table_size { return Err(DErr::RootError(Fe::FromBytesDBuf, 1, Rc::BufferTooSmall)); }
        let expected_data_size = tables[0].row_count as usize * tables[0].row_size as usize;
        let actual_data_size = bytes.len() - table_size;
        if actual_data_size < expected_data_size { return Err(DErr::RootError(Fe::FromBytesDBuf, 2, Rc::BufferTooSmall)); }
        let  rolling_version = RollingVersion::Current; //todo!()
        Ok(DBufTOTO { d_buf_id: 0, rolling_version ,app_token:Token::None, user_token: Token::None, tables,  raw_values: bytes })
    }
}