use crate::*;
use crate::buffer::Vsize;
use crate::serializer::ToDBufStruct;
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

    pub fn build_tables<T: ToDBufStruct>(&mut self,  structure_vec: &Vec<T>) -> Re<()> {
        let (table_id, columns) = T::get_dbuff_info().d_err_map(Fe::BuildHeaders,1)?;
        let table: DBufTable = DBufTable::new(table_id, columns, 0,false)?;

        // Collect child column IDs before pushing the table
        let child_column_ids: Vec<_> = table.columns.iter()
            .filter(|c| c.is_child())
            .map(|c| c.id())
            .collect();

        self.push_table(table).d_err_map(Fe::BuildTable,1)?;

        // Process children recursively
        for cid in child_column_ids {
            let child = T::get_child_structure(cid).d_err_map(Fe::BuildHeaders,1)?.as_ref();
            self.build_tables(&vec![child]).d_err_map(Fe::BuildTable,1)?;
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
    pub fn build_rowvalues<T: ToDBufStruct>(&self,structure : &Vec<T> ) -> Re<()> {
        //todo
        Ok(())
    }
    pub fn consume(self) -> Re<DBufTOTO> {
        Ok(DBufTOTO{ d_buf_id: self.d_buf_id, rolling_version: self.rolling_version, has_app_token: self.has_app_token, has_user_token: self.has_user_token, app_token:self.app_token, user_token: self.user_token, tables : self.tables,raw_values : self.raw_values})
    }
}
#[cfg(test)]
mod tests {
    use crate::buffer::Bpos;
    use crate::serializer::{ToDBufStruct, ToDBufChildren};
    use crate::table::Tid;
    use super::*;

    enum Tables {
        User,City,Order,Product,
    }
    impl Tables {
        fn id(&self) ->Tid{
            match self{
                Tables::User     => {1}
                Tables::City     => {2}
                Tables::Order    => {3}
                Tables::Product  => {4}
            }
        }
    }
    /*
    impl ToDBufChildren for Tables {
        fn get_dbuff_info  (&self)-> Re<(Tid,Vec<DColumn>)>{
            match self {
                Tables::User => {
                    Ok((self.id(), vec![
                        DColumn::U32(1),
                        DColumn::UtcTime(2),
                        DColumn::Child(3, 0), // the buffer position of the child is not known yet
                        DColumn::Child(4, 0), // the buffer position of the child is not known yet
                    ]))
                },
                Tables::City => { City::get_dbuff_info() }
                Tables::Order => { Order::get_dbuff_info() }
                Tables::Product => { Product::get_dbuff_info() }
            }
        }
        fn get_child_structure(&self, cid: Cid) -> Re<Box<dyn ToDBufChildren>> {
            todo!()
        }
    }*/
    #[derive(Clone)]
    pub struct Product {
        id: u32,
        name: String,
        price: f32
    }
    impl ToDBufStruct for Product {
         fn get_dbuff_info() -> Re<(Tid, Vec<DColumn>)> {
            Ok((Tables::Product.id(), vec![
                DColumn::U32(1),
                DColumn::String(2, 50),
                DColumn::F32(3), // the buffer position of child is not known yet
            ]))
        }
        fn get_child_structure(cid: Cid) -> Re<Box<dyn ToDBufStruct>> {
            match cid {
                _ => { Err(DErr::RootError(Fe::ProductGetChildStructure, 1, Rc::ColumnNotFound(cid))) }
            }
        }
    }
    #[derive(Clone)]
    struct Order { id: u32, date: u64, products: Vec<Product> }
    impl ToDBufStruct for  Order {
        fn get_dbuff_info  ()-> Re<(Tid,Vec<DColumn>)>{
            Ok((  1,vec![
                DColumn::U32(1),
                DColumn::UtcTime(2),
                DColumn::Child(3,0), // the buffer position of child is not known yet
            ]))
        }
        fn get_child_structure(cid: Cid) ->  Re<Box<dyn ToDBufStruct>>{
            match cid{
                3 => {Ok(Box::new(Product{id: 0,name :"n".to_string(), price : 10.0,}))}
                _ => {Err(DErr::RootError(Fe::OrderGetChildStructure,1,Rc::ColumnNotFound(cid)))}
            }
        }
    }
    #[derive(Clone)]
    struct City { id: u32, name: String }
    impl ToDBufStruct for City {
        fn get_dbuff_info  ()-> Re<(Tid,Vec<DColumn>)>{
            Ok((  Tables::City.id(),vec![
                DColumn::U32(1),
                DColumn::String(2, 50),
            ]))
        }

        fn get_child_structure(cid: Cid) -> Re<Box<dyn ToDBufStruct>>{
            match cid {
                _ => { Err(DErr::RootError(Fe::CityGetChildStructure, 1, Rc::ColumnNotFound(cid))) }
            }
        }
    }
    #[derive(Clone)]
    struct User { id: u32, name: String, orders: Vec<Order> ,cities: Vec<City>}
    impl ToDBufStruct for User {
        fn get_dbuff_info  ()-> Re<(Tid,Vec<DColumn>)>{
            Ok((  Tables::User.id(),vec![
                DColumn::U32(1),
                DColumn::UtcTime(2),
                DColumn::Child(3,0), // the buffer position of child is not known yet
                DColumn::Child(4,0), // the buffer position of child is not known yet
            ]))
        }
   /* }
    impl  User {*/
        fn get_child_structure( cid: Cid) -> Re<Box<dyn ToDBufStruct>>{
            match cid{
                1 =>   todo!("c'est étrange de devoir passer des instances fictives"),
                3 => {Ok(Box::new(Order{id: 0,date: 0, products: vec![],}))}
                4 => {Ok(Box::new(City{id: 0, name: "Paris".to_string() }))}
                _ => {Err(DErr::RootError(Fe::OrderGetChildStructure,1,Rc::ColumnNotFound(cid)))}
            }
        }
    }


    #[test]
    fn build_users()-> Vec<User> {
        let alex_order1_products : Vec<Product>= vec![
            Product{id: 0,name: "shoes".to_string(), price: 100.0,},
            Product{id: 1,name: "laptop".to_string(),price: 999.0,},
        ];
        let alex_order2_products : Vec<Product>= vec![
            Product{id: 0,name: "shoes".to_string(), price: 100.0,},
            Product{id: 2,name: "bike".to_string(),price: 1230.0,},
        ];
        let alex_orders : Vec<Order>= vec![
            Order{id: 1,date: 2022,products:alex_order1_products,},
            Order{id: 2,date: 2023,products:alex_order2_products,},
        ];
        let alex_city : Vec<City>= vec![
            City{id: 1,name : "Grenoble".to_string()},
        ];
        let fany_order1_products : Vec<Product>= vec![
            Product{id: 3,name: "harness".to_string(), price: 53.0,},
            Product{id: 4,name: "rope".to_string(),price: 125.0,},
        ];
        let fany_order2_products : Vec<Product>= vec![
            Product{id: 5,name: "carabiner".to_string(), price: 15.0,},
            Product{id: 1,name: "laptop".to_string(),price: 999.0,},
        ];
        let fany_orders : Vec<Order>= vec![
            Order{id: 1,date: 2021,products:fany_order1_products,},
            Order{id: 1,date: 2024,products:fany_order2_products,},
        ];
        let fany_city : Vec<City>= vec![
            City{id: 1,name : "Praha".to_string()},
        ];
         vec![
            User{ id: 0,name: "alex".to_string(),orders: alex_orders,cities: alex_city,},
            User{ id: 0,name: "fany".to_string(),orders: fany_orders,cities: fany_city,},
        ]
    }
    fn test_build_users(){
        let users = build_users();
        let mut builder = DBuftBuilderTOTO::new(0,ROLLING_VERSION,false,false,vec![],vec![],4);
        builder.build_tables(&users).unwrap();
        builder.build_rowvalues(&users).unwrap();

    }
}