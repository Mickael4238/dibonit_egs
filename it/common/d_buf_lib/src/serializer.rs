use std::any::Any;
use d_err_lib::{DErr, DErrMap};
use crate::{DColumn, Fe, Rc, Re};
use crate::buffer::Bsize;
use crate::builder::DBuftBuilderTOTO;
use crate::column::{Cid,Csize};
use crate::table::{DBufTable, Rcapa, Tid};
pub type DTableBox = Box<dyn DBuffTablesTrait>;

pub struct RecusrsityDepth{
    depth: u8,
    max_depth: u8,
}
impl RecusrsityDepth{
    pub fn new(max_depth: u8) -> Self {
        Self {depth : 0, max_depth}
    }
    pub fn go_down_rec(&self) -> Option<RecusrsityDepth> {
        if self.depth < self.max_depth {
            Some(Self {depth : self.depth +1, max_depth: self.max_depth})
        }else{None}
    }
}
pub trait DBuffTablesTrait{
    fn get_children_table(&self, cid: Cid, rid :Rcapa) -> Re<DTableBox>;
    fn get_table_info(&self) -> Re<(Tid, Vec<DColumn>)>;
    fn get_capa(&self) -> Rcapa;
    fn get_val_as_byte_slice(&self, col: &DColumn, rid :Rcapa)-> Re<Vec<u8>>;
    fn push_box_value_rec(&self, rv : &mut Vec<u8>, cur_tid: Tid, d_table : &mut Vec<DBufTable>, rec: &RecusrsityDepth) -> Re<()> {
        if let Some(next_rec) = rec.go_down_rec() {
            let (table_id, cols) = self.get_table_info().d_err_map(Fe::BuildHeaders,1)?;
            let tcapa = self.get_capa();
            if (table_id == cur_tid){ self.push_rowvalues(rv,d_table,tcapa,&cols).d_err_map(Fe::BuildHeaders,1)?;}
            // Process children recursively
            self.push_child_rec(rv,cur_tid,d_table,tcapa,&cols,&next_rec).d_err_map(Fe::BuildHeaders,1)
        }else{Ok(())}//End of recursivity
    }
    fn push_rowvalues(&self, rv: &mut Vec<u8>, d_table: &mut Vec<DBufTable>, tcapa : Rcapa, cols :&Vec<DColumn> ) ->Re<()>{
        for rid in 0..tcapa {
            for column in cols {
                if let Some(col_tid) = column.is_child_get_table_id() {
                    if let Some(col_table) = d_table.iter_mut().filter(|t|t.table_id == col_tid).last(){
                        let child_offset_in_table = col_table.row_count;
                        //todo:there should be a faster way to do this
                        let number_of_row_for_child = self.get_children_table(column.id(), rid).unwrap().get_capa();
                        col_table.increment_row_count(number_of_row_for_child);
                        rv.extend_from_slice(&child_offset_in_table.to_bytes_v());
                        rv.extend_from_slice(&number_of_row_for_child.to_bytes_v());

                    }else{
                        return Err(DErr::RootError(Fe::MaxDepthExceeded,1,Rc::CapacityExceeded));
                    }

                } else {
                    rv.append(&mut self.get_val_as_byte_slice(column, rid)?);
                }
            }
        }
        Ok(())
    }
    fn push_child_rec(&self, rv: &mut Vec<u8>, cur_tid: Tid, d_table : &mut Vec<DBufTable>,tcapa : Rcapa, cols :&Vec<DColumn> , rec: &RecusrsityDepth)->Re<()>{
        for rid in 0..tcapa {
            for column in cols {
                if column.is_child() {
                    let child_table = self.get_children_table(column.id(), rid).d_err_map(Fe::BuildHeaders, 1)?;
                    child_table.push_box_value_rec(rv,cur_tid, d_table,rec)?
                }
            }
        }
        Ok(())
    }
}
pub trait serialize_num{fn to_bytes_v(&self) -> Vec<u8>;}
impl serialize_num for u8{fn to_bytes_v(&self) -> Vec<u8>{self.to_le_bytes().into()}}
impl serialize_num for u16{fn to_bytes_v(&self) -> Vec<u8>{self.to_le_bytes().into()}}
impl serialize_num for u32{fn to_bytes_v(&self) -> Vec<u8>{self.to_le_bytes().into()}}
impl serialize_num for f32{fn to_bytes_v(&self) -> Vec<u8>{self.to_le_bytes().into()}}
impl serialize_num for u64{fn to_bytes_v(&self) -> Vec<u8>{self.to_le_bytes().into()}}
impl serialize_num for usize{fn to_bytes_v(&self) -> Vec<u8>{self.to_le_bytes().into()}}

pub trait serialize_str {
    fn to_ascii_bytes(&self, size: usize) ->  Re<Vec<u8>>;
    fn to_char_bytes(&self, size: usize) ->   Re<Vec<u8>>;
    fn to_bytes_v(&self, col: &DColumn) ->  Re<Vec<u8>>{
        match col {
            DColumn::Char(_, size) => {
                self.to_char_bytes(*size as usize)
            }
            DColumn::Ascii(_, size) => {
                self.to_ascii_bytes(*size as usize)
            }
            _ => {  Err(DErr::RootError(Fe::GetChildrenTable, 1, Rc::ColumnNotRow(col.id())))}
        }
    }
}
impl serialize_str for String{
    fn to_ascii_bytes(&self, size: usize) ->  Re<Vec<u8>> {
        if !self.is_ascii() { Err(DErr::RootError(Fe::ToAsciiBytes, 1, Rc::NonAsciiString(self.clone())))}
        else {self.to_char_bytes(size).d_err_map(Fe::ToAsciiBytes, 1)}
    }
    fn to_char_bytes(&self, size: usize) ->   Re<Vec<u8>>{
        if self.len() > size {  Err(DErr::RootError(Fe::ToCharBytes, 1, Rc::StringTooBig(self.clone(), size)))}
        else{
            let mut bytes = self.as_bytes().iter().copied().collect::<Vec<u8>>();
            bytes.resize(size, 0);
            Ok(bytes)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serialize_num_u8() {
        assert_eq!(u8::to_bytes_v(&1), vec![1]);
        assert_eq!(u8::to_bytes_v(&255), vec![255]);
    }
    #[test]
    fn test_serialize_num_u32() {
        assert_eq!(u32::to_bytes_v(&1), vec![1, 0, 0, 0]);
        assert_eq!(u32::to_bytes_v(&u32::MAX), vec![255,255, 255,255]);
    }
    #[test]
    fn test_serialize_num_u64() {
        assert_eq!(u64::to_bytes_v(&1), vec![1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(u64::to_bytes_v(&u64::MAX), vec![255,255, 255,255,255,255, 255,255]);
    }
    #[test]
    fn test_serialize_str_ascii() {
        assert_eq!("Hi Toto 9 ! :-)".to_string().to_ascii_bytes(17).unwrap(), vec![72, 105, 32, 84, 111, 116, 111, 32, 57, 32, 33, 32, 58, 45, 41, 0, 0]);

        match "Hi Toto 9 ! :-)".to_string().to_ascii_bytes(14) {
            Ok(_) => { assert_eq!("","Error: expected to fail, but it didn't")}
            Err(e) => {assert_eq!(e.to_string_chain(),"ToAsciiBytes[1]/ToCharBytes[1]=>StringTooBig(\"Hi Toto 9 ! :-)\", 14)".to_string())}
        }
        match "Hi Toto 9 ! :-)Ƣ".to_string().to_ascii_bytes(30) {
            Ok(_) => { assert_eq!("","Error: expected to fail, but it didn't")}
            Err(e) => {assert_eq!(e.to_string_chain(),"ToAsciiBytes[1]=>NonAsciiString(\"Hi Toto 9 ! :-)Ƣ\")".to_string())}
        }
    }
    #[test]
    fn test_serialize_str_char() {
        assert_eq!("Hi Toto 9 ! :-)Ƣ".to_string().to_char_bytes(17).unwrap(), vec![72, 105, 32, 84, 111, 116, 111, 32, 57, 32, 33, 32, 58, 45, 41, 198, 162]);
        match "Ƣ".to_string().to_char_bytes(1) {
            Ok(_) => { assert_eq!("","Error: expected to fail, but it didn't")}
            Err(e) => {assert_eq!(e.to_string_chain(),"ToCharBytes[1]=>StringTooBig(\"Ƣ\", 1)".to_string())}
        }
    }
}
