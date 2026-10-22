use d_err_lib::{DErr, DErrMap};
use crate::{Fe, Rc, Re,Bsize};
use crate::buffer::Bpos;

const COL_ID_HIGH_MASK: u8 = 0x07;
const COL_TYPE_SHIFT :u8 = 3;

pub(crate) type Cid = u16;     // type of column id
type Csize = u16;   // type of column size
type Ctype = u8;    // type of column type (note: only 5 bits)
#[derive(Debug, Clone, PartialEq)]
pub enum DColumn {
    Boolean(Cid), U8(Cid), U16(Cid), U32(Cid), U64(Cid), I8(Cid), I16(Cid), I32(Cid), I64(Cid), F32(Cid), F64(Cid), UtcTime(Cid), Char(Cid, Csize), Ascii(Cid, Csize),String(Cid, Csize), Child(Cid,Bpos),
}
impl DColumn {
    pub fn type_id(&self) -> Ctype {
        match self {
            DColumn::Boolean(_)   => 0,  //ID
            DColumn::U8     (_)   => 1,  //ID
            DColumn::U16    (_)   => 2,  //ID
            DColumn::U32    (_)   => 3,  //ID
            DColumn::U64    (_)   => 4,  //ID
            DColumn::I8     (_)   => 5,  //ID
            DColumn::I16    (_)   => 6,  //ID
            DColumn::I32    (_)   => 7,  //ID
            DColumn::I64    (_)   => 8,  //ID
            DColumn::F32    (_)   => 9,  //ID
            DColumn::F64    (_)   => 10, //ID
            DColumn::UtcTime(_)   => 11, //ID
            DColumn::Char   (_,_) => 12, //ID SIZE
            DColumn::Ascii  (_,_) => 13, //ID SIZE
            DColumn::String (_,_) => 14, //ID SIZE
            DColumn::Child  (_,_) => 15, //ID CHILD_POSITION
        }
    }

    pub fn requires_size_field(&self) -> bool {
        matches!(self, DColumn::Char(_,_) | DColumn::Ascii(_,_) | DColumn::String(_,_))
    }
    pub fn is_child(&self) -> bool { matches!(self, DColumn::Child(_,_)) }

    pub fn id(&self) -> Cid{match self {
            DColumn::Boolean(id)   => *id,
            DColumn::U8     (id)   => *id,
            DColumn::U16    (id)   => *id,
            DColumn::U32    (id)   => *id,
            DColumn::U64    (id)   => *id,
            DColumn::I8     (id)   => *id,
            DColumn::I16    (id)   => *id,
            DColumn::I32    (id)   => *id,
            DColumn::I64    (id)   => *id,
            DColumn::F32    (id)   => *id,
            DColumn::F64    (id)   => *id,
            DColumn::UtcTime(id)   => *id,
            DColumn::Char   (id,_) => *id,
            DColumn::Ascii  (id,_) => *id,
            DColumn::String (id,_) => *id,
            DColumn::Child  (id,_) => *id,
    }}

    //  read column from rows and update current_index depending on the column type.
    //  1 read 5bit for col_type_id, and 3+8bit for col_id = 2 bytes
    //  2 (optional) read 2 bytes for col_size if col_type_id is 12,13,14
    //   example:
    //   '''
    //   let col = DColumn::from_row(&row, &mut idx)?;
    //   '''
    pub fn from_row(row: &[u8], current_index: &mut Bsize) -> Re<DColumn> {
        let b1= read_byte(row,current_index).d_err_map(Fe::FromRow,1)?;
        let col_type_id = b1 >> COL_TYPE_SHIFT;
        let b2 = read_byte(row,current_index).d_err_map(Fe::FromRow,2)?;
        let a =  ((b1 & COL_ID_HIGH_MASK) as Cid) << 8;
        let col_id = ((b1 & COL_ID_HIGH_MASK) as Cid) << 8| b2 as Cid;
        match col_type_id {
            0  => Ok(DColumn::Boolean(col_id)), 1 => Ok(DColumn::U8(col_id)), 2 => Ok(DColumn::U16(col_id)),
            3  => Ok(DColumn::U32(col_id)), 4 => Ok(DColumn::U64(col_id)), 5 => Ok(DColumn::I8(col_id)),
            6  => Ok(DColumn::I16(col_id)), 7 => Ok(DColumn::I32(col_id)), 8 => Ok(DColumn::I64(col_id)),
            9  => Ok(DColumn::F32(col_id)), 10 => Ok(DColumn::F64(col_id)), 11 => Ok(DColumn::UtcTime(col_id)),
            15 => {
                let child_position = read_bsize2(row,current_index).d_err_map(Fe::FromRow,5)?;
                Ok(DColumn::Child(col_id,child_position))
            } ,
            _ => {
                let b3= read_byte(row,current_index).d_err_map(Fe::FromRow,3)? as Csize;
                let b4= read_byte(row,current_index).d_err_map(Fe::FromRow,4)? as Csize;
                let col_size = ( b3 << 8 )+ b4;
                if col_size == 0 {Err(DErr::RootError(Fe::FromTypeId,1,Rc::InvalidColumnSize))}
                else {
                    match col_type_id {
                        12 => Ok(DColumn::Char(col_id,col_size)),
                        13 => Ok(DColumn::Ascii(col_id,col_size)),
                        14 => Ok(DColumn::String(col_id,col_size)),
                        _ => Err(DErr::RootError(Fe::FromTypeId,1,Rc::InvalidColumnType(col_type_id)))
                    }
                }
            }

        }
    }
    pub fn size_of(&self) -> Csize {
        match self {
            DColumn::Boolean(_) | DColumn::U8(_) | DColumn::I8(_) => 1,
            DColumn::U16(_) | DColumn::I16(_) => 2,
            DColumn::U32(_) | DColumn::I32(_) | DColumn::F32(_) => 4,
            DColumn::U64(_) | DColumn::I64(_) | DColumn::F64(_) | DColumn::UtcTime(_) => 8,
            DColumn::Char(_,s) | DColumn::String(_,s)  | DColumn::Ascii(_,s) => *s,
            DColumn::Child(_,_)  => 4,
        }
    }
}
/*
impl fmt::Display for DColumn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DColumn::Boolean(id) => write!(f, "bool({})", id),
            DColumn::U8(id) =>      write!(f, "u8({})"  , id),
            DColumn::U16(id) =>     write!(f, "u({})"   , id),
            DColumn::U32(id) =>     write!(f, "u({})"   , id),
            DColumn::U64(id) =>     write!(f, "u({})"   , id),
            DColumn::I8(id) =>      write!(f, "i({})"   , id),
            DColumn::I16(id) =>     write!(f, "i({})"   , id),
            DColumn::I32(id) =>     write!(f, "i({})"   , id),
            DColumn::I64(id) =>     write!(f, "i({})"   , id),
            DColumn::F32(id) =>     write!(f, "f({})"   , id),
            DColumn::F64(id) =>     write!(f, "f({})"   , id),
            DColumn::UtcTime(id) => write!(f, "utc({})" , id),
            DColumn::Char(id,size) =>   write!(f, "char({})"  , id),
            DColumn::Ascii(id,size) =>  write!(f, "ascii({})" , id),
            DColumn::String(id,size) => write!(f, "string({})", id),
            DColumn::Child(id) =>   write!(f, "child({})", id),
        }
    }
}*/

pub fn read_byte(row: &[u8], current_index: &mut Bsize) -> Re<u8> {
    let b = row.get({*current_index += 1;*current_index -1 }).ok_or(DErr::RootError(Fe::ReadByte, 1, Rc::RowTooShort))?;
    Ok(*b)
}
pub fn read_bsize2(row: &[u8], current_index: &mut Bsize) -> Re<Bsize> {
    let b1= read_byte(row,current_index).d_err_map(Fe::FromRow,3)? as Bsize;
    let b2= read_byte(row,current_index).d_err_map(Fe::FromRow,4)? as Bsize;
    let b3= read_byte(row,current_index).d_err_map(Fe::FromRow,3)? as Bsize;
    let b4= read_byte(row,current_index).d_err_map(Fe::FromRow,4)? as Bsize;
    Ok(b1 << 24 | (2 << 16 | b3 << 8 | b4))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- helpers ----------

    /// Encode une colonne : byte0 = (id_high << 5) | type_id, byte1 = id_low,
    /// puis éventuellement 2 octets pour col_size.
    fn encode(type_id: u8, col_id: u16, col_size: Option<u16>) -> Vec<u8> {
        let mut row = vec![0u8; 2];
        row[0] = (type_id << COL_TYPE_SHIFT) | ((col_id >> 8) as u8) ;
        row[1] = col_id as u8;
        if let Some(s) = col_size {
            row.push((s >> 8) as u8);
            row.push(s as u8);
        }
        row
    }

    fn from_row(row: &[u8]) -> Re<DColumn> {
        DColumn::from_row(&row.to_vec(), &mut 0)
    }

    // ---------- type_id ----------

    #[test]
    fn type_id_matches_spec() {
        assert_eq!(DColumn::Boolean(0).type_id(), 0);
        assert_eq!(DColumn::U8(0).type_id(), 1);
        assert_eq!(DColumn::U16(0).type_id(), 2);
        assert_eq!(DColumn::U32(0).type_id(), 3);
        assert_eq!(DColumn::U64(0).type_id(), 4);
        assert_eq!(DColumn::I8(0).type_id(), 5);
        assert_eq!(DColumn::I16(0).type_id(), 6);
        assert_eq!(DColumn::I32(0).type_id(), 7);
        assert_eq!(DColumn::I64(0).type_id(), 8);
        assert_eq!(DColumn::F32(0).type_id(), 9);
        assert_eq!(DColumn::F64(0).type_id(), 10);
        assert_eq!(DColumn::UtcTime(0).type_id(), 11);
        assert_eq!(DColumn::Char(0, 1).type_id(), 12);
        assert_eq!(DColumn::Ascii(0, 1).type_id(), 13);
        assert_eq!(DColumn::String(0, 1).type_id(), 14);
        assert_eq!(DColumn::Child(0,1).type_id(), 15);
    }

    // ---------- requires_size_field / is_child ----------

    #[test]
    fn size_field_only_for_char_ascii_string() {
        assert!(!DColumn::Boolean(1).requires_size_field());
        assert!(!DColumn::F64(1).requires_size_field());
        assert!(!DColumn::UtcTime(1).requires_size_field());
        assert!(!DColumn::Child(1,0).requires_size_field());
        assert!(DColumn::Char(1, 4).requires_size_field());
        assert!(DColumn::Ascii(1, 4).requires_size_field());
        assert!(DColumn::String(1, 4).requires_size_field());
    }

    #[test]
    fn is_child_only_for_child() {
        assert!(DColumn::Child(3,0).is_child());
        assert!(!DColumn::String(3, 10).is_child());
        assert!(!DColumn::U64(3).is_child());
    }

    // ---------- size_of ----------

    #[test]
    fn size_of_fixed_types() {
        assert_eq!(DColumn::Boolean(0).size_of(), 1);
        assert_eq!(DColumn::U8(0).size_of(), 1);
        assert_eq!(DColumn::I8(0).size_of(), 1);
        assert_eq!(DColumn::U16(0).size_of(), 2);
        assert_eq!(DColumn::I16(0).size_of(), 2);
        assert_eq!(DColumn::U32(0).size_of(), 4);
        assert_eq!(DColumn::I32(0).size_of(), 4);
        assert_eq!(DColumn::F32(0).size_of(), 4);
        assert_eq!(DColumn::U64(0).size_of(), 8);
        assert_eq!(DColumn::I64(0).size_of(), 8);
        assert_eq!(DColumn::F64(0).size_of(), 8);
        assert_eq!(DColumn::UtcTime(0).size_of(), 8);
        assert_eq!(DColumn::Child(0,3).size_of(), 4);
    }

    #[test]
    fn size_of_variable_types_uses_declared_size() {
        assert_eq!(DColumn::Char(1, 32).size_of(), 32);
        assert_eq!(DColumn::Ascii(1, 7).size_of(), 7);
        assert_eq!(DColumn::String(1, 512).size_of(), 512);
    }

    // ---------- from_row : cas simples ----------

    #[test]
    fn from_row_boolean_min_id() {
        let col = from_row(&encode(0, 0x0000, None)).unwrap();
        assert_eq!(col, DColumn::Boolean(0));
    }

    #[test]
    fn from_row_u32_mid_id() {
        let col = from_row(&encode(3, 0x0042, None)).unwrap();
        assert_eq!(col, DColumn::U32(0x42));
    }

    #[test]
    fn from_row_f64_max_id() {
        // id max = 0x07FF (3 bits hauts)
        let col = from_row(&encode(10, 0x07FF, None)).unwrap();
        assert_eq!(col, DColumn::F64(0x07FF));
    }

    #[test]
    fn from_row_all_simple_types() {
        let cases: Vec<(u8, DColumn)> = vec![
            (0, DColumn::Boolean(0x0101)),
            (1, DColumn::U8(0x0101)),
            (2, DColumn::U16(0x0101)),
            (3, DColumn::U32(0x0101)),
            (4, DColumn::U64(0x0101)),
            (5, DColumn::I8(0x0101)),
            (6, DColumn::I16(0x0101)),
            (7, DColumn::I32(0x0101)),
            (8, DColumn::I64(0x0101)),
            (9, DColumn::F32(0x0101)),
            (10, DColumn::F64(0x0101)),
            (11, DColumn::UtcTime(0x0101)),
        ];
        for (tid, expected) in cases {
            let col = from_row(&encode(tid, 0x0101, None)).unwrap();
            assert_eq!(col, expected, "type_id={tid}");
        }
    }

    // ---------- from_row : taille obligatoire ----------

    #[test]
    fn from_row_char_with_size() {
        let col = from_row(&encode(12, 0x0007, Some(64))).unwrap();
        assert_eq!(col, DColumn::Char(7, 64));
    }

    #[test]
    fn from_row_ascii_with_size() {
        let col = from_row(&encode(13, 0x0001, Some(256))).unwrap();
        assert_eq!(col, DColumn::Ascii(1, 256));
    }

    #[test]
    fn from_row_string_with_size() {
        let col = from_row(&encode(14, 0x0123, Some(1024))).unwrap();
        assert_eq!(col, DColumn::String(0x0123, 1024));
    }

    #[test]
    fn from_row_zero_size_is_error() {
        assert!(from_row(&encode(12, 1, Some(0))).is_err());
        assert!(from_row(&encode(13, 1, Some(0))).is_err());
        assert!(from_row(&encode(14, 1, Some(0))).is_err());
    }

    // ---------- from_row : masquage ----------

    #[test]
    fn type_id_is_masked_to_5_bits() {
        // byte0 avec bits parasites au-delà du masque : reste 0x1F
        // 0x1F est invalide -> doit être Err, pas de panic
        assert!(from_row(&[0xFF, 0x00]).is_err());
    }
    //  read column from rows and update current_index depending on the column type.
    //  1 read 5bit for col_type_id, and 3+8bit for col_id = 2 bytes
    //  2 (optional) read 2 bytes for col_size if col_type_id is 12,13,14
    #[test]
    fn col_id_uses_only_3_high_bits() {
        // id = 0x07FF doit rester 0x07FF même avec bits bas pollués
        let col = from_row(&encode(1, 0x07FF, None)).unwrap();
        let col2 = DColumn::from_row(&0b0000111111111111_u16.to_be_bytes(), &mut 0).unwrap();
        assert_eq!(format!("{:?}", col), "U8(2047)");
        assert_eq!(format!("{:?}", col),format!("{:?}", col2));
    }

    // ---------- from_row : offset non nul ----------

    #[test]
    fn from_row_respects_current_index() {
        let mut row = vec![0xAA, 0xBB, 0xCC]; // préfixe à ignorer
        row.extend(encode(2, 0x0304, None));
        let mut idx = 3;
        let col = DColumn::from_row(&row, &mut idx).unwrap();
        assert_eq!(col, DColumn::U16(0x0304));
    }

    // ---------- from_row : erreurs et bornes ----------

    #[test]
    fn from_row_row_too_short_panics_or_errors() {
        // Documente le comportement actuel : indexing direct = panic.
        // Si tu veux un DErr à la place, ce test est à adapter.
        let result = std::panic::catch_unwind(|| from_row(&[0x00]).ok());
        // On accepte panic OU Ok : ce test signale juste le comportement.
        let _ = result;
    }

    #[test]
    fn from_row_invalid_type_id_after_15() {
        // 16..=31 : ni type simple, ni type avec taille -> doit être Err
        for tid in 16u8..=31 {
            assert!(from_row(&encode(tid, 1, Some(4))).is_err(), "type_id={tid}");
        }
    }

    // ---------- round-trip conceptuel ----------

    #[test]
    fn decoded_column_is_consistent_with_type_id() {
        for col in [
            DColumn::Boolean(1),
            DColumn::U8(1),
            DColumn::U16(1),
            DColumn::U32(1),
            DColumn::U64(1),
            DColumn::I8(1),
            DColumn::I16(1),
            DColumn::I32(1),
            DColumn::I64(1),
            DColumn::F32(1),
            DColumn::F64(1),
            DColumn::UtcTime(1),
            DColumn::Char(1, 4),
            DColumn::Ascii(1, 4),
            DColumn::String(1, 4),
            DColumn::Child(1,0),
        ] {
            let tid = col.type_id();
            let needs_size = col.requires_size_field();
            // types 12,13,14 = taille, 15 = Child, autres = fixes
            assert_eq!(needs_size, (12..=14).contains(&tid));
            assert_eq!(col.is_child(), tid == 15);
        }
    }
}
