use crate::*;
use crate::serializer::DTableBox;

pub type Bsize = usize;   // type of position in buffer, (must contain the maximum buffer length) Within the colum it could be replaced by the ID of the child, the index position of the child in the buffer...
pub type Bpos = Bsize;
pub type Rsize = usize;
pub type Vsize = u8;            // type of rolling versions
pub enum RollingVersion {    // when receive :
    PreviousDeprecated = 3,  //  -> Raise an Error, the other app is too old, but we manage the case and send buffers with the PreviousDeprecated buffer version
    PreviousToDeploy   = 0,  //  -> Raise a warning, our app is newly deployed (but not the other), we manage the case and send PreviousToDeploy buffer version
    Current            = 1,  //  -> normal case, both apps are up to date; we can send Current buffer version
    UnknownNext        = 2,  //  -> receive UnknownNext ==> an app is sending a buffer version that is not yet known (her current is 1): should ask to send our current (0 - for her it will be PreviousToDeploy)
}
impl RollingVersion {
    pub const MASK: Vsize = 0x03;
    pub const MAX: Vsize = 3;
    /// Any updates to the D_buf_lib when in production should see a roll in the versions
    pub fn from_u8(item: u8) -> RollingVersion {
        match item & RollingVersion::MASK {
            0 => RollingVersion::PreviousToDeploy,
            1 => RollingVersion::Current,
            2 => RollingVersion::UnknownNext,
            3 => RollingVersion::PreviousDeprecated,
            _ => unreachable!(), // MASK ensures only 0-3 possible
        }
    }
}


pub struct DBufTOTO {
    pub rolling_version: RollingVersion,
    pub d_buf_id: u16,
    pub has_app_token: bool,
    pub has_user_token: bool,
    pub app_token: Vec<u8>,
    pub user_token: Vec<u8>,
    pub tables: Vec<DBufTable>,
    pub raw_values: Vec<u8>,
}
impl DBufTOTO {
    pub fn new(structure :DTableBox,d_buf_id : u16, app_token : Vec<u8>, user_token : Vec<u8>, max_depth: u8 ) -> Re<Self> {
        let mut builder = DBuftBuilderTOTO::new(d_buf_id,RollingVersion::Current, app_token, user_token, max_depth).d_err_map(Fe::NewDBuf, 1)?;
        builder.build_tables(&structure).d_err_map(Fe::NewDBuf, 1)?;
        builder.build_rowvalues(structure).d_err_map(Fe::NewDBuf,2)?;
        builder.consume().d_err_map(Fe::NewDBuf,3)
    }
    pub fn new_basic(structure : DTableBox,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure, 1, vec![], vec![], max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_tokens(structure : DTableBox,app_token : Vec<u8>, user_token : Vec<u8>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1, app_token, user_token, max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_user_tokens(structure : DTableBox, user_token : Vec<u8>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1,vec![], user_token, max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn new_app_tokens(structure : DTableBox,app_token : Vec<u8>,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure,1, app_token, vec![], max_depth).d_err_map(Fe::NewDBuf,1)
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Re<DBufTOTO> {
        //todo read th child  headers, create a specific function in the builder? in the table?
        let tables = vec![DBufTable::from_bytes(&bytes)?,];
        let table_size = tables[0].header_size();
        if bytes.len() < table_size { return Err(DErr::RootError(Fe::FromBytesDBuf, 1, Rc::BufferTooSmall)); }
        let expected_data_size = tables[0].row_count as usize * tables[0].row_size as usize;
        let actual_data_size = bytes.len() - table_size;
        if actual_data_size < expected_data_size { return Err(DErr::RootError(Fe::FromBytesDBuf, 2, Rc::BufferTooSmall)); }
        let  rolling_version = RollingVersion::Current; //todo!()
        Ok(DBufTOTO { d_buf_id: 0, rolling_version , has_app_token: false, has_user_token: false, app_token: vec![], user_token: vec![], tables,  raw_values: bytes })
    }
}