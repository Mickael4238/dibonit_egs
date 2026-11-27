use std::fmt;

#[derive(Debug)]
pub struct RecurseDepth {
    depth: u8,
    max_depth: u8,
    iter_count_at_same_level: u8,
}
impl RecurseDepth{
    pub fn new(max_depth: u8) -> Self {
        Self {depth : 0, max_depth,iter_count_at_same_level: 0}
    }
    pub fn go_down_rec(&self) -> Option<RecurseDepth> {
        if self.depth < self.max_depth {
            Some(Self {depth : self.depth +1, max_depth: self.max_depth,iter_count_at_same_level: 0})
        }else{None}
    }
    pub fn iter_on_same(&mut self) ->&mut RecurseDepth{
        if self.iter_count_at_same_level < u8::MAX { self.iter_count_at_same_level = self.iter_count_at_same_level +1;}
        self // we don't really care about the iter count, only 0/1/more
    }
    pub fn is_first_call(&self) -> bool {self.depth == 0}
}
impl fmt::Display for RecurseDepth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rec{}-{}/{}", self.depth,self.iter_count_at_same_level,self.max_depth )
    }
}