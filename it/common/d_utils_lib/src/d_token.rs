#[derive(Debug)]
pub enum Token {
    App(Vec<u8>),
    User(Vec<u8>),
    None,
}
impl Token {
    pub const APP_MASK:u8=4;  // 3rd bit
    pub const USER_MASK:u8=8; // 4th bit
    pub fn from_bytes(b : u8,index:  &mut usize  , rowdata : &Vec<u8> )->Self {
        if Token::has_app_token(b) | Token::has_user_token(b) {
            // note: if we want not to copy from the row, we could use a reference to the rowdata
            let esize = rowdata[*index];
            let size = 2^esize as usize;
            let mut v = rowdata[*index..size].to_vec();
            *index += 1 + size;
            if Token::has_app_token(b) {
                Token::App(v)
            }else{
                Token::User(v)
            }
        }
        else{Token::None}
    }
    pub fn has_app_token(b : u8)->bool{ b & Token::APP_MASK == Token::APP_MASK}
    pub fn has_user_token(b : u8)->bool{ b & Token::USER_MASK == Token::USER_MASK}
    pub fn has_token_as_row(&self)->u8{
        match self{
            &Token::App(_)=>Token::APP_MASK,
            &Token::User(_)=>Token::USER_MASK,
            &Token::None=>0,
        }
    }

    pub fn get(&self)->Option<&Vec<u8>>{
        match &self{
            &Token::App(v)=>Some(v),
            &Token::User(v)=>Some(v),
            &Token::None=>None,
        }
    }
    /// return the x as 2^x size of the token, 0 if None,
    /// a 255-byte token returns 8
    pub fn esize_of(&self)->(u8){
        match &self{
            &Token::App(v)=>Token::esize_of_vec(v),
            &Token::User(v)=>Token::esize_of_vec(v),
            &Token::None=>{0u8}
        }
    }
    pub fn size_of(&self)->(u8){
        match &self{
            &Token::App(v)=>2^Token::esize_of_vec(v),
            &Token::User(v)=>2^Token::esize_of_vec(v),
            &Token::None=>{0u8}
        }
    }
    fn esize_of_vec(v: &Vec<u8>)->(u8){(0usize.leading_zeros() - v.len().leading_zeros() )as u8}
}
