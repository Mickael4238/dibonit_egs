use crate::{DColumn, Re};
use crate::column::Cid;

pub trait ToDBufTOTO {
    fn get_dbuff_info  (&self)-> Re<(u16,Vec<DColumn>,u16)>;
    fn get_child_structure <T: ToDBufTOTO>(&self, cid: Cid) ->Re<&T>;
}
