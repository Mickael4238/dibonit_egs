use crate::*;
use crate::buffer::Vsize;
use crate::serializer::{ DTableBox, ToDBufStruct};
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
    pub fn new(d_buf_id : u16, rolling_version : Vsize,has_app_token : bool, has_user_token : bool, app_token : Vec<u8>, user_token : Vec<u8>,max_depth: u8 ) -> Self {
        Self {        d_buf_id,       rolling_version,        has_app_token,        has_user_token,        app_token,           user_token,          max_depth,tables: vec![], raw_values: vec![], current_size: 0, current_depth: 0 } // the children will never have tokens
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
    pub fn build_rowvalues(&self,structure : DTableBox ) -> Re<()> {
        //todo
        Ok(())
    }
    pub fn consume(self) -> Re<DBufTOTO> {
        Ok(DBufTOTO{ d_buf_id: self.d_buf_id, rolling_version: self.rolling_version, has_app_token: self.has_app_token, has_user_token: self.has_user_token, app_token:self.app_token, user_token: self.user_token, tables : self.tables,raw_values : self.raw_values})
    }
}
#[cfg(test)]
mod tests {
    use crate::serializer::{ DTableBox,};

use crate::serializer::DStructBox;
use crate::buffer::Bpos;
    use crate::serializer::{ToDBufStruct, DBuffTablesTrait, DVec};
    use crate::table::Tid;
    use super::*;

    enum Tables {
        User(DVec<User>),City(DVec<City>),Order(DVec<Order>),Product(DVec<Product>),
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
            match self {
                Tables::User(_)    => { User::get_dbuff_info_static() }
                Tables::City(_)    => { City::get_dbuff_info_static() }
                Tables::Order(_)   => { Order::get_dbuff_info_static() }
                Tables::Product(_) => { Product::get_dbuff_info_static() }
            }
        }
        fn get_children_table(&self, cid: Cid,rid : Rcapa) -> Re<DTableBox> {
            Ok(Box::new(match self {
                Tables::User(dv)    => {
                    let user = dv.get(rid)?;
                    match cid {
                        3 => Tables::Order(user.orders.clone()),
                        4 => Tables::City(user.cities.clone()),
                        _ => return Err(DErr::RootError(Fe::BuildTable, 1, Rc::ColumnNotFound(cid)))
                    }
                }
                Tables::City(dv)    => {
                    let city = dv.get(rid)?;
                    return Err(DErr::RootError(Fe::CityGetChildStructure, 1, Rc::ColumnNotFound(cid)))
                }
                Tables::Order(dv)   => {
                    let order = dv.get(rid)?;
                    match cid {
                        3 => Tables::Product(order.products.clone()),
                        _ => return Err(DErr::RootError(Fe::OrderGetChildStructure, 1, Rc::ColumnNotFound(cid)))
                    }
                }
                Tables::Product(dv) => {
                    let product = dv.get(rid)?;
                    return Err(DErr::RootError(Fe::BuildTable, 1, Rc::ColumnNotFound(cid)))
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
    }
    #[derive(Clone)]
    pub struct Product {
        id: u32,
        name: String,
        price: f32
    }
    impl ToDBufStruct for Product {
        fn get_dbuff_info_static() -> Re<(Tid, Vec<DColumn>)> {
            Ok((Tables::Product(DVec::none()).id(), vec![
                DColumn::U32(1),
                DColumn::String(2, 50),
                DColumn::F32(3),
            ]))
        }

        fn get_child_table_static(cid: Cid) -> Re<DTableBox> where Self: Sized{
            todo!("no child table for Product")
        }
        impl_dbuf_struct_instance_methods!();
    }
    #[derive(Clone)]
    struct Order { id: u32, date: u64, products: DVec<Product> }
    impl ToDBufStruct for  Order {
        fn get_dbuff_info_static() -> Re<(Tid,Vec<DColumn>)>{
            Ok((Tables::Order(DVec::none()).id(),vec![
                DColumn::U32(1),
                DColumn::UtcTime(2),
                DColumn::Child(3,0),
            ]))
        }
        fn get_child_table_static(cid: Cid) -> Re<DTableBox> where Self: Sized{
            match cid{
                3 => {Ok(Box::new(Tables::Product(DVec::none())))}
                _ => {Err(DErr::RootError(Fe::OrderGetChildStructure,1,Rc::ColumnNotFound(cid)))}
            }
        }
        impl_dbuf_struct_instance_methods!();
    }
    #[derive(Clone)]
    struct City { id: u32, name: String }
    impl ToDBufStruct for City {
        fn get_dbuff_info_static() -> Re<(Tid,Vec<DColumn>)>{
            Ok((Tables::City(DVec::none()).id(),vec![
                DColumn::U32(1),
                DColumn::String(2, 50),
            ]))
        }

        fn get_child_table_static(cid: Cid) -> Re<DTableBox> where Self: Sized{
            match cid {
                _ => { Err(DErr::RootError(Fe::CityGetChildStructure, 1, Rc::ColumnNotFound(cid))) }
            }
        }
        impl_dbuf_struct_instance_methods!();
    }
    #[derive(Clone)]
    struct User { id: u32, name: String, orders: DVec<Order> ,cities: DVec<City>}
    impl ToDBufStruct for User {
        fn get_dbuff_info_static() -> Re<(Tid,Vec<DColumn>)>{
            Ok((Tables::User(DVec::none()).id(),vec![
                DColumn::U32(1),
                DColumn::UtcTime(2),
                DColumn::Child(3,0),
                DColumn::Child(4,0),
            ]))
        }

        fn get_child_table_static(cid: Cid) -> Re<DTableBox> where Self: Sized{
            match cid{
                3 => {Ok(Box::new(Tables::Order(DVec::none())))}
                4 => {Ok(Box::new(Tables::City(DVec::none())))}
                _ => {Err(DErr::RootError(Fe::OrderGetChildStructure,1,Rc::ColumnNotFound(cid)))}
            }
        }
        impl_dbuf_struct_instance_methods!();
    }

    fn build_users()-> DVec<User> {
        let mut alex_order1_products = DVec::new(2);
        alex_order1_products.push(Product{id: 0,name: "shoes".to_string(), price: 100.0,});
        alex_order1_products.push(Product{id: 1,name: "laptop".to_string(),price: 999.0,});
        alex_order1_products.push(Product{id: 2,name: "toto".to_string(),price: 999.0,});

        let mut alex_order2_products= DVec::new(2);
        alex_order2_products.push(Product{id: 0,name: "shoes".to_string(), price: 100.0,});
        alex_order2_products.push(Product{id: 2,name: "bike".to_string(),price: 1230.0,});

        let mut alex_orders= DVec::new(2);
        alex_orders.push(Order{id: 1,date: 2022,products:alex_order1_products,});
        alex_orders.push(Order{id: 2,date: 2023,products:alex_order2_products,});

        let mut alex_city = DVec::new(1);
        alex_city.push(City{id: 1,name : "Grenoble".to_string()});

        let mut fany_order1_products = DVec::new(2);
        fany_order1_products.push(Product{id: 3,name: "harness".to_string(), price: 53.0,});
        fany_order1_products.push(Product{id: 4,name: "rope".to_string(),price: 125.0,});

        let mut fany_order2_products = DVec::new(2);
        fany_order2_products.push(Product{id: 5,name: "carabiner".to_string(), price: 15.0,});
        fany_order2_products.push(Product{id: 1,name: "laptop".to_string(),price: 999.0,});

        let mut fany_orders = DVec::new(2);
        fany_orders.push(Order{id: 1,date: 2021,products:fany_order1_products,});
        fany_orders.push(Order{id: 1,date: 2024,products:fany_order2_products,});

        let mut fany_city = DVec::new(1);
        fany_city.push(City{id: 1,name : "Praha".to_string()});

        let mut ret = DVec::new(2);
        ret.push(User{ id: 0,name: "alex".to_string(),orders: alex_orders,cities: alex_city,});
        ret.push(User{ id: 0,name: "fany".to_string(),orders: fany_orders,cities: fany_city,});

        ret
    }
    #[test]
    fn test_build_users(){
        let users = build_users();
        let mut builder = DBuftBuilderTOTO::new(0,ROLLING_VERSION,false,false,vec![],vec![],4);
        // Build table structure based on User type
        let ub : DTableBox = Box::new(Tables::User(users));
        builder.build_tables(&ub).unwrap();
        builder.print_tables();
        builder.build_rowvalues(ub).unwrap();

    }
}
