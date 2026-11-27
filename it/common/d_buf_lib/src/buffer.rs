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
    pub fn new(structure :DTableBox,d_buf_id : u16, rolling_version : RollingVersion, app_token :Token, user_token :Token, max_depth: u8 ) -> Re<Self> {
        let mut myself =  DBufTOTO{d_buf_id,rolling_version,app_token,user_token,tables: vec![], raw_values: vec![] };
        myself.build_tables(&structure,max_depth).d_err_map(Fe::NewDBuf,1)?;
        Ok(myself)
    }
    pub fn new_basic(structure : DTableBox,max_depth : u8) -> Re<Self> {
        DBufTOTO::new(structure, 1,RollingVersion::Current, Token::None, Token::None, max_depth).d_err_map(Fe::NewDBuf,1)
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
    pub fn build_tables(&mut self, table_in: &DTableBox,max_depth :u8) -> Re<()> {
        self.build_tables_rec(table_in,&mut RecurseDepth::new(max_depth)).unwrap();
        self.set_tables_positions();
        self.build_rowvalues(table_in,max_depth).unwrap();
        Ok(())
    }

    fn build_tables_rec(&mut self, table_in: &DTableBox, rec: &RecurseDepth) -> Re<()> {
        if let Some (next_rec) = rec.go_down_rec() {
            // Use instance method (delegates to static internally, no instance data used)
            let (table_id, columns) = table_in.get_table_info().d_err_map(Fe::BuildHeaders,1)?;
            let capa = table_in.get_capa();
            let table: DBufTable = DBufTable::new(table_id, columns,capa,false)?;
            // Collect child column IDs before pushing the table
            let child_column_ids: Vec<_> = table.columns.iter()
                .filter(|c| c.is_child())
                .map(|c| c.id())
                .collect();
            self.push_table(table).d_err_map(Fe::BuildTable,1)?;
            // Process children recursively
            for cid in child_column_ids {
                for rid in 0..capa {
                    let child_table = table_in.get_children_table(cid,rid).d_err_map(Fe::BuildHeaders,1)?;
                    self.build_tables_rec(&child_table,&next_rec).d_err_map(Fe::BuildTable,1)?;
                }
            }
            Ok(())
        }else{
            Ok(()) //End of recursivity
        }
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
    fn set_tables_positions(&mut self ) {
        let mut current_table_pos = 0;
        self.tables.iter().for_each(|table| {
            self.tables.iter().for_each(|table| {
                table.set_table_position_for_columns(table.table_id,current_table_pos)
            });
            current_table_pos = current_table_pos + table.calculate_size();
        })
    }

    fn build_rowvalues(&mut self, table_in : &DTableBox ,max_depth : u8) -> Re<()> {
        //let mut raw_values: Vec<u8> = Vec::with_capacity(self.calculate_total_size()); //todo the calculation could be done at build tables?
        self.raw_values = Vec::with_capacity(self.calculate_total_size());

        self.push_header_rowvalues();
        let tables_ids: Vec<_> = self.tables.iter().enumerate().map(|(i, t)| (t.table_id, i)).collect();
        let mut rec = &mut RecurseDepth::new(max_depth);
        for (tid,tidx) in tables_ids {
            println!("BUFFER PUSH TABLE {}.{}",tidx,tid);
            self.tables[tidx].push_header_rowvalues(&mut self.raw_values)?;
            table_in.push_box_value_rec(&mut self.raw_values,tid,tidx,&mut self.tables,rec.iter_on_same())?;
        }
        Ok(())
    }

    fn push_header_rowvalues(&mut self) -> (){
        println!("{}\t{}\t{}\t{}pushing header - Rv - hasToken ",self.raw_values.len(),1,self.raw_values.capacity(),self.rolling_version as u8 | self.app_token.has_token_as_row()| self.user_token.has_token_as_row());
        self.raw_values.push(self.rolling_version as u8 | self.app_token.has_token_as_row()| self.user_token.has_token_as_row());

        println!("{}\t{}\t{}\t{}pushing header - ID ",self.raw_values.len(),2,self.raw_values.capacity(),&self.d_buf_id);
        self.raw_values.extend_from_slice(&self.d_buf_id.to_le_bytes());
        if let Some(t) =  self.app_token.get() {self.raw_values.append(&mut t.clone())}  // todo check clone !!
        if let Some(t) =  self.user_token.get() {self.raw_values.append(&mut t.clone())}
        // for now there is no table info in header, we can only reach the first table (who links others)
    }
    fn calculate_total_size(&self) -> Bsize {
        self.tables.iter().map(|v| v.calculate_size()).sum::<Bsize>()
            + self.calculate_header_size()
    }
    pub(crate) fn calculate_header_size(&self) -> Bsize {
        let rol_vers_tocken_info_size: Bsize = 1 ;
        let d_buf_id_size: Bsize = 2;
        let tokens_size =( self.app_token.size_of()  +  self.user_token.size_of() )as Bsize;
        let nb_table_size: Bsize = size_of::<Tcapa>();
        rol_vers_tocken_info_size + d_buf_id_size + tokens_size + nb_table_size
    }
}
#[cfg(test)]
impl fmt::Display for DBufTOTO {
    #[cfg(test)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BUFFER (id = {},capa = {}, count = {}, rolling version = {:?}, user token = {:?} , app token = {:?} )", self.d_buf_id,self.raw_values.capacity(),self.raw_values.len(), self.rolling_version, self.user_token, self.app_token);
        let mut start_index = self.calculate_header_size();
        for t in &self.tables {
            write!(f, "\r\n+ {}", t)?;
            if f.alternate() { // pretty-print mode on {:#}
                start_index = start_index + t.calculate_header_size();
                write!(f, "- startIndex = {} ", start_index)?;
                for r in 0..t.row_count{
                    write!(f, "\r\n\t{}", r)?;
                    for c in t.columns.iter() {
                        let end_index = start_index + c.size_of()as usize;
                        write!(f, "\t{:?}",c.to_string(&self.raw_values[start_index..end_index]))?;
                        start_index = end_index;
                    }
                }
            }
        }
        Ok(())
    }
}
