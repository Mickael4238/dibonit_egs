use crate::*;
use crate::buffer::Vsize;
use crate::table::Rcapa;

pub struct DBuftBuilderTOTO {
    pub d_buf_id: u16,
    pub rolling_version: u8,
    pub has_app_token: bool,
    pub has_user_token: bool,
    pub app_token: Vec<u8>,
    pub user_token: Vec<u8>,
    pub tables: Vec<DBufTable>,
    pub raw_values: Vec<u8>,
    max_depth: u8,       // maximum depth of the recursion
    current_size: usize, // current size of the DBuf being built, at the end of the recursion, this will be the size of the DBuf being built
    current_depth: u8,   // current depth of the recursion, at the end of the recursion, this will be the depth of the DBuf being built
}
impl DBuftBuilderTOTO{
    pub(crate) fn new(d_buf_id : u16, rolling_version : Vsize,has_app_token : bool, has_user_token : bool, app_token : Vec<u8>, user_token : Vec<u8>,max_depth: u8 ) -> Self {
        Self {        d_buf_id,       rolling_version,        has_app_token,        has_user_token,        app_token,           user_token,          max_depth,tables: vec![], raw_values: vec![], current_size: 0, current_depth: 0 } // the children will never have tokens
    }

    pub fn build_tables<T: ToDBufTOTO>(&mut self, structure : &T) -> Re<()> {
        let (d_buf_id, columns, row_capacity) = structure.get_dbuff_info().d_err_map(Fe::BuildHeaders,1)?;
        let table: DBufTable = DBufTable::new(d_buf_id, columns, row_capacity, false)?;

        // Collect child column IDs before pushing the table
        let child_column_ids: Vec<_> = table.columns.iter()
            .filter(|c| c.is_child())
            .map(|c| c.id())
            .collect();

        self.push_table(table).d_err_map(Fe::BuildHeaders,1)?;

        // Process children recursively
        for cid in child_column_ids {
            let child: &T = structure.get_child_structure(cid).d_err_map(Fe::BuildHeaders,1)?;
            self.build_tables(child)?;
        }
        Ok(())
    }
    fn push_table(&mut self, table: DBufTable)-> Re<()>  {
        match self.tables.iter_mut().find(|t| t.table_id == table.table_id) {
            Some(t) => {
                if t.row_capacity <=  Rcapa::MAX - table.row_capacity  {
                    t.row_capacity += table.row_capacity;
                    Ok(())
                }else{Err(DErr::RootError(Fe::PushTable,1,Rc::CapacityExceeded))}
            },
            None => {
                self.tables.push(table);
                Ok(())
            }
        } // note to the future: if a table is full (a page from the table), we could push a new table to the buffer here...
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
        Ok(DBufTOTO{ d_buf_id: self.d_buf_id, rolling_version: self.rolling_version, has_app_token: self.has_app_token, has_user_token: self.has_user_token, app_token:self.app_token, user_token: self.user_token, tables : self.tables,raw_values : self.raw_values})
    }
}