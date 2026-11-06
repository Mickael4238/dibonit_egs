use d_err_lib::DErr;
use crate::{DColumn, Fe, Rc, Re};
use crate::column::Cid;
use crate::table::{Rcapa, Tid};
pub type DStructBox<T> = Box<T>;
#[derive(Clone)]
pub struct DVec<T> {
    dv: Option<Vec<DStructBox<T>>>
}

pub type DTableBox = Box<dyn DBuffTablesTrait>;

pub trait DBuffTablesTrait{
    fn get_children_table(&self, cid: Cid, rid :Rcapa) -> Re<DTableBox>;
    fn get_table_info(&self) -> Re<(Tid, Vec<DColumn>)>;
    fn get_capa(&self) -> Rcapa;

}
impl<T> DVec<T> {
    pub fn new(capa : Rcapa) -> Self {
        Self {dv: Some(Vec::with_capacity(capa as usize))}
    }
    pub fn new_vec(v: Vec<DStructBox<T>>) -> Self {
        Self {dv:Some(v)}
    }
    pub fn none() -> Self {
        Self {dv:None}
    }
    pub fn push(&mut self, value :T){
        match &mut self.dv {
            None => {
                self.dv =  Some( vec![Box::new(value)]);
            } ,
            Some( a) => a.push(Box::new(value))
        }
    }
    pub fn len(&self) -> usize {
        match &self.dv {
            None => 0,
            Some(a) => a.len()
        }
    }
    pub fn is_empty(&self) -> bool {
        match &self.dv {
            None => true,
            Some(a) => a.is_empty()
        }
    }
    pub fn get(&self, index : Rcapa) -> Re<&DStructBox<T>>{
        self.dv.as_ref()
            .and_then(|v| v.get(index as usize))
            .ok_or(DErr::RootError(Fe::DBufNotFound,1,Rc::InvalidRowIndex(index)))
    }
}

