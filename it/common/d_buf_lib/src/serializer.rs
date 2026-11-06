use d_err_lib::DErr;
use crate::{DColumn, Fe, Rc, Re};
use crate::column::Cid;
use crate::table::{Rcapa, Tid};
pub type DStructBox<T> = Box<T>;
pub type DTableBox = Box<dyn DBuffTablesTrait>;

pub trait DBuffTablesTrait{
    fn get_children_table(&self, cid: Cid, rid :Rcapa) -> Re<DTableBox>;
    fn get_table_info(&self) -> Re<(Tid, Vec<DColumn>)>;
    fn get_capa(&self) -> Rcapa;

}

