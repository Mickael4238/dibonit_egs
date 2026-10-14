use crate::*;//{ColumnType, Fe, Rc, Re, MAX_COLUMN_ID};

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnTypeTOTO {
    Boolean, U8, U16, U32, U64, I8, I16, I32, I64, F32, F64, UtcTime, Char(usize), Ascii(usize), Child,
}

impl ColumnTypeTOTO {
    pub fn type_id(&self) -> u8 {
        match self {
            ColumnTypeTOTO::Boolean => 0, ColumnTypeTOTO::U8 => 1, ColumnTypeTOTO::U16 => 2,
            ColumnTypeTOTO::U32 => 3, ColumnTypeTOTO::U64 => 4, ColumnTypeTOTO::I8 => 5,
            ColumnTypeTOTO::I16 => 6, ColumnTypeTOTO::I32 => 7, ColumnTypeTOTO::I64 => 8,
            ColumnTypeTOTO::F32 => 9, ColumnTypeTOTO::F64 => 10, ColumnTypeTOTO::UtcTime => 11,
            ColumnTypeTOTO::Char(_) => 12, ColumnTypeTOTO::Ascii(_) => 13, ColumnTypeTOTO::Child => 14,
        }
    }
    pub fn size_of(&self) -> usize {
        match self {
            ColumnTypeTOTO::Boolean | ColumnTypeTOTO::U8 | ColumnTypeTOTO::I8 => 1,
            ColumnTypeTOTO::U16 | ColumnTypeTOTO::I16 => 2,
            ColumnTypeTOTO::U32 | ColumnTypeTOTO::I32 | ColumnTypeTOTO::F32 => 4,
            ColumnTypeTOTO::U64 | ColumnTypeTOTO::I64 | ColumnTypeTOTO::F64 | ColumnTypeTOTO::UtcTime => 8,
            ColumnTypeTOTO::Char(s) | ColumnTypeTOTO::Ascii(s) => *s,
            ColumnTypeTOTO::Child => 6,
        }
    }
    pub fn requires_size_field(&self) -> bool {
        matches!(self, ColumnTypeTOTO::Char(_) | ColumnTypeTOTO::Ascii(_))
    }
    pub fn is_child(&self) -> bool { matches!(self, ColumnTypeTOTO::Child) }
    pub fn from_type_id(type_id: u8, size: Option<usize>) -> Re<ColumnTypeTOTO> {
        match type_id {
            0 => Ok(ColumnTypeTOTO::Boolean), 1 => Ok(ColumnTypeTOTO::U8), 2 => Ok(ColumnTypeTOTO::U16),
            3 => Ok(ColumnTypeTOTO::U32), 4 => Ok(ColumnTypeTOTO::U64), 5 => Ok(ColumnTypeTOTO::I8),
            6 => Ok(ColumnTypeTOTO::I16), 7 => Ok(ColumnTypeTOTO::I32), 8 => Ok(ColumnTypeTOTO::I64),
            9 => Ok(ColumnTypeTOTO::F32), 10 => Ok(ColumnTypeTOTO::F64), 11 => Ok(ColumnTypeTOTO::UtcTime),
            12 => match size { Some(0) => Err(DErr::RootError(Fe::FromTypeId,1,Rc::InvalidColumnSize)), Some(s) => Ok(ColumnTypeTOTO::Char(s)), None => Err(DErr::RootError(Fe::FromTypeId, 2, Rc::MissingSizeForVariableType)) },
            13 => match size { Some(0) => Err(DErr::RootError(Fe::FromTypeId,1,Rc::InvalidColumnSize)), Some(s) => Ok(ColumnTypeTOTO::Ascii(s)), None => Err(DErr::RootError(Fe::FromTypeId, 2, Rc::MissingSizeForVariableType)) },
            14 => Ok(ColumnTypeTOTO::Child),
            _ => Err(DErr::RootError(Fe::FromTypeId,3,Rc::InvalidColumnType(type_id))),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnHeaderTOTO { pub col_id: u16, pub size: usize, pub col_type: ColumnTypeTOTO }

impl ColumnHeaderTOTO {
    pub fn new(col_id: u16, col_type: ColumnTypeTOTO) -> Re<ColumnHeaderTOTO> {
        if col_id == 0 { return Err(DErr::RootError(Fe::NewColumnHeader,1,Rc::ReservedColumnId)); }
        if col_id > MAX_COLUMN_ID { return Err(DErr::RootError(Fe::NewColumnHeader,2,Rc::ColumnIdExceeded)); }
        Ok(ColumnHeaderTOTO { col_id, size:col_type.size_of(),col_type})
    }
}
