use std::cmp;
use crate::*;
use crate::buffer::{RollingVersion, Token, Vsize};
use crate::Rc::TokenTooLarge;
use crate::serializer::{ DTableBox};
use crate::table::{Rcapa, Tid};

pub struct DBuftBuilderTOTO {
    pub rolling_version: RollingVersion, // byte 1 / Bit: 1->2
    pub d_buf_id: Tid,                   // byte 2-3
    pub app_token: Token,                // bytes 1 / bit: 3; (byte 3, byte 4->4+ 2^token_esize) or 0 if none
    pub user_token: Token,               // bytes 1 / bit: 4; (byte x, byte x+1->x+1+ 2^token_esize) or 0 if none
    pub tables: Vec<DBufTable>,          // bytes y-y + tables size
    pub raw_values: Vec<u8>,
    max_depth: u8,       // maximum depth of the recursion
    current_size: usize, // current size of the DBuf being built, at the end of the recursion, this will be the size of the DBuf being built
    current_depth: u8,   // current depth of the recursion, at the end of the recursion, this will be the depth of the DBuf being built
}
impl DBuftBuilderTOTO{
    pub fn new(d_buf_id : Tid, rolling_version : RollingVersion, app_token_v : Option<Vec<u8>>, user_token_v : Option<Vec<u8>>, max_depth: u8 ) -> Re<Self> {
        let app_token = match app_token_v{
            None => {Token::None}
            Some(v) => {Token::App(v)}
        };
        let user_token = match user_token_v{
            None => {Token::None}
            Some(v) => {Token::App(v)}
        };// maybe we can release Token public so only get a token insted of a vector of bytes
        Ok(Self {d_buf_id,rolling_version,app_token,user_token,max_depth,tables: vec![], raw_values: vec![], current_size: 0, current_depth: 0 } )
    }
    pub fn build_tables(&mut self, table_in: &DTableBox) -> Re<()> {
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
                self.build_tables(&child_table).d_err_map(Fe::BuildTable,1)?;
            }
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
    #[cfg(test)]
    fn print_tables(&self){
        for table in &self.tables{
            println!("table_id: {}, row_capacity: {}, columns: {:?}",table.table_id,table.row_capacity,table.columns);
        }
    }


    //note, it might not be the best to push value, before having the entire size to malloc only once.
    fn push_row_value(&mut self, mut values: Vec<u8>) {
        self.current_size += values.len();
        self.raw_values.append(&mut values);
        self.current_depth += 1;
    }
    pub fn build_rowvalues(&mut self, t_box : &DTableBox ) -> Re<()> {
        let mut raw_values: Vec<u8> = Vec::with_capacity(self.calculate_total_size()); //todo the calculation could be done at build tables?
        self.push_header_rowvalues( &mut raw_values);
        self.tables.iter().try_for_each(|table| {
            let tid = table.table_id;
            table.push_header_rowvalues(&mut raw_values)?;
            DBuftBuilderTOTO::push_box_value(&mut raw_values,t_box,tid) // TODO place somewhere else
        })?;
        Ok(())
    }
    fn push_box_value(rv : &mut Vec<u8>, table_in: &DTableBox,tid : Tid)  -> Re<()> {
        // TODO place somewhere else //todo replace by only get id
        let (table_id, columns) = table_in.get_table_info().d_err_map(Fe::BuildHeaders,1)?;
        if (table_id == tid){
            table_in.push_rowvalues(rv).d_err_map(Fe::BuildHeaders,1)?;
        }
        let table_capa = table_in.get_capa();
        // Process children recursively
        for column in columns {
            for rid in 0..table_capa {
                let child_table = table_in.get_children_table(column.id(), rid).d_err_map(Fe::BuildHeaders, 1)?;
                DBuftBuilderTOTO::push_box_value(rv, &child_table,tid)?
            }
        }
        Ok(())
    }
    pub fn push_header_rowvalues(&self,rv : &mut Vec<u8> ) -> (){
        rv.push(self.rolling_version as u8 | self.app_token.has_token_as_row()| self.user_token.has_token_as_row());
        rv.extend_from_slice(&self.d_buf_id.to_le_bytes());
        if let Some(t) =  self.app_token.get() {rv.append(&mut t.clone())}  // todo check clone !!
        if let Some(t) =  self.user_token.get() {rv.append(&mut t.clone())}
        // for now there is no table info in header, we can only reach the first table (who links others)
    }
    fn calculate_total_size(&self) -> Bsize {
       self.tables.iter().map(|v| v.calculate_size()).sum::<Bsize>()
           + self.calculate_header_size()
    }
    fn calculate_header_size(&self) -> Bsize {
        let rol_vers_tocken_info_size: Bsize = 1 ;
        let d_buf_id_size: Bsize = 2;
        let tokens_size =( self.app_token.size_of()  +  self.user_token.size_of() )as Bsize;
        let nb_table_size: Bsize = size_of::<Tcapa>();
        rol_vers_tocken_info_size + d_buf_id_size + tokens_size + nb_table_size
    }
    pub fn consume(self) -> Re<DBufTOTO> {
        Ok(DBufTOTO{ d_buf_id: self.d_buf_id, rolling_version: self.rolling_version, app_token:self.app_token, user_token: self.user_token, tables : self.tables,raw_values : self.raw_values})
    }
}
#[cfg(test)]
mod tests {
    use crate::buffer::RollingVersion;
    use crate::serializer::{DTableBox,};
    use crate::serializer::{ DBuffTablesTrait};
    use crate::table::Tid;
    use super::*;

    enum Tables {
        User(Vec<User>),City(Vec<City>),Order(Vec<Order>),Product(Vec<Product>),
    }
    impl Tables {
        fn id(&self) ->Tid{
            match self{
                Tables::User(_)      => {1}
                Tables::City(_)      => {2}
                Tables::Order(_)     => {3}
                Tables::Product(_)   => {4}
            }
        }
    }
    impl DBuffTablesTrait for Tables {
        fn get_table_info  (&self)-> Re<(Tid,Vec<DColumn>)>{
            Ok((self.id(),
                match self {
                    Tables::User(_)    => {vec![
                        DColumn::U32(1),
                        DColumn::UtcTime(2),
                        DColumn::Child(3,0),
                        DColumn::Child(4,0),
                    ]}
                    Tables::City(_)    => {vec![
                        DColumn::U32(1),
                        DColumn::String(2, 50),
                    ]}
                    Tables::Order(_)   => {vec![
                        DColumn::U32(1),
                        DColumn::UtcTime(2),
                        DColumn::Child(3,0),
                    ]}
                    Tables::Product(_) => {vec![
                        DColumn::U32(1),
                        DColumn::String(2, 50),
                        DColumn::F32(3),
                    ]}
            }))
        }
        fn get_children_table(&self, cid: Cid,rid : Rcapa) -> Re<DTableBox> {
            Ok(Box::new(match self {
                Tables::User(dv)    => {
                    let user = dv.get(rid as usize).ok_or(DErr::RootError(Fe::GetChildrenTable, 1, Rc::InvalidRowIndex(cid)))?;
                    match cid {
                        3 => Tables::Order(user.orders.clone()),
                        4 => Tables::City(user.cities.clone()),
                        _ => return Err(DErr::RootError(Fe::GetChildrenTable, 1, Rc::ColumnNotFound(cid)))
                    }
                }
                Tables::City(_)    => {
                    return Err(DErr::RootError(Fe::GetChildrenTable,2, Rc::ColumnNotFound(cid)))
                }
                Tables::Order(dv)   => {
                    let order = dv.get(rid as usize).ok_or(DErr::RootError(Fe::GetChildrenTable, 3, Rc::InvalidRowIndex(cid)))?;
                    match cid {
                        3 => Tables::Product(order.products.clone()),
                        _ => return Err(DErr::RootError(Fe::GetChildrenTable, 3, Rc::ColumnNotFound(cid)))
                    }
                }
                Tables::Product(_) => {
                    return Err(DErr::RootError(Fe::GetChildrenTable, 4, Rc::ColumnNotFound(cid)))
                }
            }) as DTableBox)
        }
        fn get_capa(&self) -> Rcapa{
            match self{
                Tables::User(dv )      => {dv.len() as Rcapa}
                Tables::City(dv)       => {dv.len() as Rcapa}
                Tables::Order(dv)     => {dv.len() as Rcapa}
                Tables::Product(dv) => {dv.len() as Rcapa}
            }
        }

        fn push_rowvalues(&self, rv: &mut Vec<u8>) -> Re<()> {
            todo!()
        }
    }
    #[derive(Clone)]
    pub struct Product {
        id: u32,
        name: String,
        price: f32
    }

    #[derive(Clone)]
    struct Order { id: u32, date: u64, products: Vec<Product> }
    #[derive(Clone)]
    struct City { id: u32, name: String }
    #[derive(Clone)]
    struct User { id: u32, name: String, orders: Vec<Order> ,cities: Vec<City>}
    fn build_users()-> Vec<User> {
        let mut alex_order1_products = Vec::with_capacity(2);
        alex_order1_products.push(Product{id: 0,name: "shoes".to_string(), price: 100.0,});
        alex_order1_products.push(Product{id: 1,name: "laptop".to_string(),price: 999.0,});
        alex_order1_products.push(Product{id: 2,name: "toto".to_string(),price: 999.0,});

        let mut alex_order2_products= Vec::with_capacity(2);
        alex_order2_products.push(Product{id: 0,name: "shoes".to_string(), price: 100.0,});
        alex_order2_products.push(Product{id: 2,name: "bike".to_string(),price: 1230.0,});

        let mut alex_orders= Vec::with_capacity(2);
        alex_orders.push(Order{id: 1,date: 2022,products:alex_order1_products,});
        alex_orders.push(Order{id: 2,date: 2023,products:alex_order2_products,});

        let mut alex_city = Vec::with_capacity(1);
        alex_city.push(City{id: 1,name : "Grenoble".to_string()});

        let mut fany_order1_products = Vec::with_capacity(2);
        fany_order1_products.push(Product{id: 3,name: "harness".to_string(), price: 53.0,});
        fany_order1_products.push(Product{id: 4,name: "rope".to_string(),price: 125.0,});

        let mut fany_order2_products = Vec::with_capacity(2);
        fany_order2_products.push(Product{id: 5,name: "carabiner".to_string(), price: 15.0,});
        fany_order2_products.push(Product{id: 1,name: "laptop".to_string(),price: 999.0,});

        let mut fany_orders = Vec::with_capacity(2);
        fany_orders.push(Order{id: 1,date: 2021,products:fany_order1_products,});
        fany_orders.push(Order{id: 1,date: 2024,products:fany_order2_products,});

        let mut fany_city = Vec::with_capacity(1);
        fany_city.push(City{id: 1,name : "Praha".to_string()});

        let mut ret = Vec::with_capacity(2);
        ret.push(User{ id: 0,name: "alex".to_string(),orders: alex_orders,cities: alex_city,});
        ret.push(User{ id: 0,name: "fany".to_string(),orders: fany_orders,cities: fany_city,});

        ret
    }
    #[test]
    fn test_build_users(){
        let users = build_users();
        let mut builder = DBuftBuilderTOTO::new(0,RollingVersion::Current,None,None,4).unwrap();
        // Build table structure based on User type
        let ub : DTableBox = Box::new(Tables::User(users));
        builder.build_tables(&ub).unwrap();
        builder.print_tables();
        builder.build_rowvalues(&ub).unwrap();

    }
}
