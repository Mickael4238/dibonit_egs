pub struct RecurseDepth {
    depth: u8,
    max_depth: u8,
}
impl RecurseDepth{
    pub fn new(max_depth: u8) -> Self {
        Self {depth : 0, max_depth}
    }
    pub fn go_down_rec(&self) -> Option<RecurseDepth> {
        if self.depth < self.max_depth {
            Some(Self {depth : self.depth +1, max_depth: self.max_depth})
        }else{None}
    }
}