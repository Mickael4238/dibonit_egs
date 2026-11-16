use d_err_lib::DErrMap;
use crate::{DColumn, Fe, Re};
use crate::builder::DBuftBuilderTOTO;
use crate::column::Cid;
use crate::table::{DBufTable, Rcapa, Tid};
pub type DTableBox = Box<dyn DBuffTablesTrait>;

pub trait DBuffTablesTrait{
    fn get_children_table(&self, cid: Cid, rid :Rcapa) -> Re<DTableBox>;
    fn get_table_info(&self) -> Re<(Tid, Vec<DColumn>)>;
    fn get_capa(&self) -> Rcapa;
    fn get_val_as_byte_slice(&self, col: &DColumn, rid :Rcapa)-> Re<Vec<u8>>;
    fn push_box_value_rec(&self, rv : &mut Vec<u8>, cur_tid: Tid,d_table : &Vec<DBufTable>) -> Re<()> {
        let (table_id, cols) = self.get_table_info().d_err_map(Fe::BuildHeaders,1)?;
        let tcapa = self.get_capa();
        if (table_id == cur_tid){ self.push_rowvalues(rv,d_table,tcapa,&cols).d_err_map(Fe::BuildHeaders,1)?;}
        // Process children recursively
        self.push_child_rec(rv,cur_tid,d_table,tcapa,&cols).d_err_map(Fe::BuildHeaders,1)
    }
    fn push_rowvalues(&self, rv: &mut Vec<u8>,d_table : &Vec<DBufTable>,tcapa : Rcapa, cols :&Vec<DColumn> )->Re<()>{
            for rid in 0..tcapa {
                for column in cols {
                    if !column.is_child() {
                        rv.append(&mut self.get_val_as_byte_slice(column, rid)?);
                    }else {
                        todo!("calculate child offsets with d_table")

                    }
                }
            }
        Ok(())
    }
    fn push_child_rec(&self, rv: &mut Vec<u8>, cur_tid: Tid, d_table : &Vec<DBufTable>,tcapa : Rcapa, cols :&Vec<DColumn> )->Re<()>{
        for rid in 0..tcapa {
            for column in cols {
                if column.is_child() {
                    let child_table = self.get_children_table(column.id(), rid).d_err_map(Fe::BuildHeaders, 1)?;
                    child_table.push_box_value_rec(rv,cur_tid, d_table)?
                }
            }
        }
        Ok(())
    }
}
pub trait serialize{
    fn to_bytes(&self) -> Vec<u8>;
}

impl serialize for u8{
    fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}
/* 	Static (Monomorphic): The compiler generates a separate copy of the function for every type S used.
 	Faster: No runtime overhead. The call is inlined and optimized.
 	Strongly Typed: The caller knows the exact type S.
 	Stack Size	Zero: No heap allocation required for the function call itself.
 */
pub fn convert_to_bytes<S: serialize>( param: S) -> Re<Vec<u8>> {
    Ok(param.to_bytes())
}

/*Dynamic (Polymorphic): The function uses a single copy and looks up the method at runtime via a vtable.
Slower: Requires a vtable lookup and a pointer dereference.
Fixed Size: The binary size is constant regardless of how many types implement serialize.
Flexibility High: You can store heterogeneous types (e.g., Vec<Box<dyn serialize>>).
 */
pub fn convert_to_bytes_dynamic(param: Box<dyn serialize>) -> Re<Vec<u8>> {
    Ok(param.to_bytes())
}
