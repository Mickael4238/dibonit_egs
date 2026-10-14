use crate::{ColumnHeaderTOTO, Re};

pub trait ToDBufTOTO {
    fn get_dbuff_info  (&self)-> Re<(u16,Vec<ColumnHeaderTOTO>,u16)>;
}
