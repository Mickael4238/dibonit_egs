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
}

// Macro to auto-generate the instance method implementations
#[macro_export]
macro_rules! impl_dbuf_struct_instance_methods {
    () => {
        fn get_dbuff_info(&self) -> Re<(Tid, Vec<DColumn>)> {
            Self::get_dbuff_info_static()
        }
        fn get_child_structure(&self, cid: Cid) -> Re<Box<dyn ToDBufStruct>> {
            Self::get_child_structure_static(cid)
        }
    };
}
pub trait ToDBufChildren {
    fn get_child_structure(&self, cid: Cid) -> Re<Box<dyn ToDBufChildren>>;
    fn get_dbuff_info(&self) -> Re<(Tid, Vec<DColumn>)>;
}
