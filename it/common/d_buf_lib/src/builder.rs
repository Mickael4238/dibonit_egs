use crate::*;
pub struct DBuftBuilderTOTO {
    current_size: usize, // current size of the DBuf being built, at the end of the recursion, this will be the size of the DBuf being built
    current_depth: u8,   // current depth of the recursion, at the end of the recursion, this will be the depth of the DBuf being built
    max_depth: u8,       // maximum depth of the recursion
    version:u8,
    has_app_token:bool,
    has_user_token:bool,
    headers : Vec<DBufHeaderTOTO>,
    raw_values : Vec<u8>,
}
impl DBuftBuilderTOTO{
    pub(crate) fn new(max_depth: u8, version:u8) -> Self {
        Self { current_size: 0, current_depth: 0, max_depth, version, has_app_token: false, has_user_token: false, headers : vec![], raw_values: vec![] } // the children will never have tokens
    }
    pub fn build_headers<T: ToDBufTOTO>(&mut self, structure : &T , has_app_token : bool, has_user_token : bool, has_index : bool, app_token : Vec<u8>, user_token : Vec<u8>) -> Re<()> {
        let (d_buf_id, columns, row_capacity) = structure.get_dbuff_info().d_err_map(Fe::BuildHeaders,1)?;
        let root_header : DBufHeaderTOTO = DBufHeaderTOTO::new(d_buf_id, columns, row_capacity, has_app_token, has_user_token, has_index, app_token, user_token)?;
        self.push_header(root_header);
        self.headers[1].build_child_headers(&self,&structure);

        Ok(())
    }
    fn push_header(&mut self, header: DBufHeaderTOTO) {
        self.headers.push(header);
        //self.current_size += header.header_size();//todo now? later? need to clarify algorithm !
        self.current_depth += 1;
    }

    //note, it might not be the best to push value, before having the entire size to malloc only once.
    fn push_row_value(&mut self, mut values: Vec<u8>) {
        self.current_size += values.len();
        self.raw_values.append(&mut values);
        self.current_depth += 1;
    }
    pub fn build_rowvalues<T: ToDBufTOTO>(&self,structure : &T ) -> Re<()> {
        //todo
        Ok(())
    }
    pub fn consume(self) -> Re<DBufTOTO> {
        Ok(DBufTOTO{headers : self.headers,raw_values : self.raw_values})
    }
}