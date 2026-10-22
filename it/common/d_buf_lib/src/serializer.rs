use crate::{DColumn, Re};
use crate::column::Cid;
use crate::table::{Rcapa, Tid};

pub trait ToDBufStruct {
    fn get_dbuff_info() -> Re<(Tid, Vec<DColumn>)> where Self: Sized ;//where Self: Sized;
    fn get_child_structure( cid: Cid) -> Re<Box<dyn ToDBufStruct>> where Self: Sized;
}
pub trait ToDBufChildren {
    fn get_child_structure(&self, cid: Cid) -> Re<Box<dyn ToDBufChildren>>;
    fn get_dbuff_info(&self) -> Re<(Tid, Vec<DColumn>)>;
}
