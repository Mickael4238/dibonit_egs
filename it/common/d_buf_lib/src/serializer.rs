use crate::{DColumn, Re};
use crate::column::Cid;
use crate::table::{Rcapa, Tid};

pub trait ToDBufStruct {
    // Static methods for structure definition (type-level)
    fn get_dbuff_info_static() -> Re<(Tid, Vec<DColumn>)> where Self: Sized;
    fn get_child_structure_static(cid: Cid) -> Re<Box<dyn ToDBufStruct>> where Self: Sized;

    // Instance methods for trait objects
    fn get_dbuff_info(&self) -> Re<(Tid, Vec<DColumn>)>;
    fn get_child_structure(&self, cid: Cid) -> Re<Box<dyn ToDBufStruct>>;

    // check if we can only implement the static methods of the trait and use those for the instance methods
    fn get_child_structureT(&self, cid: Cid) -> Re<Box<dyn ToDBufStruct>> where Self: Sized {
        Self::get_child_structure_static(cid)
    }
}
pub trait ToDBufChildren {
    fn get_child_structure(&self, cid: Cid) -> Re<Box<dyn ToDBufChildren>>;
    fn get_dbuff_info(&self) -> Re<(Tid, Vec<DColumn>)>;
}
