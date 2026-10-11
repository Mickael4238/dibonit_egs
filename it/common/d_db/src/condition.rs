use std::cmp::PartialEq;
use std::fmt;
use std::fmt::Debug;
use d_err::{DErr, DErrMap, DErrMapConvert, ConvertDErr};
use d_buf::{ColumnHeader, ColumnType, DBuf};

mod new_condition {

    use crate::condition::new_condition::value_mod::RetValue;
    use std::fmt;
    use d_buf::{ColumnHeader, ColumnType, DBuf};
    use d_err::{DErr, DErrMap, };
    use strum::IntoEnumIterator; // Trait pour l'itération
    //use strum_macros::EnumIter;
    use operator_mod::OperationNode;
    use logical_mod::LogicalNode;
    use function_mod::FunctionNode;
    use value_mod::ValueNode;
    use crate::condition::new_condition::value_mod::ValueType;

    type Re<T> = Result<T, DErr<Fe, Rc>>;
    #[derive(Debug)]
    /// Function where the error transits
    enum Fe {
        MatchKey,
        GetValue,
    }
    #[derive(Debug)]
    /// RootCause of the error
    ///
    pub enum Rc {
        NotImplemented(String),
        DBufError(String),
        IncorrectTableSelected(String)
    }
    impl fmt::Display for Rc {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Rc::NotImplemented(what) => { write!(f, "Not implemented: {}", what) },
                Rc::DBufError(msg) => write!(f, "DBuf error: {}", msg),
                Rc::IncorrectTableSelected(msg) => write!(f, "Incorrect table selected: {}", msg),
            }
        }
    }
    #[derive(Debug, )]
    pub enum Condition {
        Logical(LogicalNode),
        Operation(OperationNode),
        Function(FunctionNode),
        Single(ValueNode),
    }
    #[cfg(test)]
    impl fmt::Display for Condition {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }
    pub struct Keys {
        pub buf: d_buf::DBuf,
        pub current_row: usize,
    }
    impl Condition {
        pub fn new(str_in: &[char]) -> Condition {
                 if let Some(ln)  = LogicalNode::new  (str_in) {ln}
            else if let Some(on)  = OperationNode::new(str_in) {on}
            else if let Some(fun) = FunctionNode::new (str_in) {fun}
            else if let Some(sn)  = ValueNode::new    (str_in) {sn}
            else {
                todo!()
                //ValueNode::new (str_in).unwrap()
            }
        }
        pub fn match_key(&self, keys: &Keys) -> bool {
            match &self{
                Condition::Logical(ln)=> ln.match_key(keys),
                Condition::Operation(on)=> on.match_key(keys),
                Condition::Function(fun)=> fun.match_key(keys),
                Condition::Single(sn)=>sn.match_key(keys),
            }
        }
        //TODO ERROR HANDLING
        //.d_err_map(Fe::GetValue, 1)
        //.d_err_map(Fe::GetValue, 2)
        fn get_value(&self, keys: &Keys) -> RetValue {
            match &self {
                Condition::Logical(ln)    =>  ln.get_value(keys),
                Condition::Operation(on)=>  on.get_value(keys),
                Condition::Function(fun) => fun.get_value(keys),
                Condition::Single(sn)      =>  sn.get_value(keys).unwrap(),
            }
        }
        //TODO FIX ALL OD THAT
        /*
        fn get_value_from_logics(&self, keys: &Keys) -> Re<Value> {
            let mut val = Value {
                buf: DBuf::new(
                    &Vec::from(b"and"), 3, &vec![
                        ColumnHeader { c_name: Vec::from(b"and"), c_type: ColumnType::Boolean },
                    ], 1
                ).map_err(|e| DErr::RootError(Fe::GetValueFromAndOrNot, 1, Rc::DBufError(e.to_string_chain())))?,
                v_type: ValueType::Unknown,
            };
            val.buf.add_rows(1, &vec![self.match_key(keys).d_err_map(Fe::GetValueFromAndOrNot, 1)? as u8]).map_err(
                |e| DErr::RootError(Fe::GetValueFromAndOrNot, 2, Rc::DBufError(e.to_string_chain()))
            )?;
            Ok(val)
        }
        fn can_be_lower_or_equal_than_key(&self, keys: &Keys) -> Re<bool> {
            match self {
                ConditionNode::Single(key_condition) => {
                    Ok(match key_condition.condition_operator {
                        Operator::Equal => *key >= key_condition.condition_key,
                        Operator::NotEqual => true,
                        Operator::GreaterThan => *key >= key_condition.condition_key,
                        Operator::GreaterThanOrEquals => *key >= key_condition.condition_key,
                        Operator::LowerThan => true,
                        Operator::LowerThanOrEquals => true,
                        Operator::StartWith => {
                            // TODO IMPLEMENT
                            *key == key_condition.condition_key
                        },
                        Operator::Like => true,
                        Operator::Regex => true,
                    })
                }
                ConditionNode::And(left, right) => {
                    Ok(left.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 1)?
                        && right.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 2)?)
                }
                ConditionNode::Or(left, right) => {
                    Ok(left.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 3)?
                        || right.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 4)?)
                }
            }
        }
        fn can_be_greater_or_equal_than_key(&self, keys: &Keys) -> Re<bool> {
            match self {
                ConditionNode::Single(key_condition) => {
                    Ok(match key_condition.condition_operator {
                        Operator::Equal => *key <= key_condition.condition_key,
                        Operator::NotEqual => true,
                        Operator::GreaterThan => true,
                        Operator::GreaterThanOrEquals => true,
                        Operator::LowerThan => *key <= key_condition.condition_key,
                        Operator::LowerThanOrEquals => *key <= key_condition.condition_key,
                        Operator::StartWith => {
                            // TODO IMPLEMENT
                            *key == key_condition.condition_key
                        },
                        Operator::Like => true,
                        Operator::Regex => true,
                    })
                }
                ConditionNode::And(left, right) => {
                    Ok(left.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 1)?
                        && right.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 2)?)
                }
                ConditionNode::Or(left, right) => {
                    Ok(left.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 3)?
                        || right.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 4)?)
                }
            }
        }
        fn can_be_lower_than_key(&self, keys: &Keys) -> Re<bool> {
            match self {
                ConditionNode::Single(key_condition) => {
                    Ok(match key_condition.condition_operator {
                        Operator::Equal => *key > key_condition.condition_key,
                        Operator::NotEqual => true,
                        Operator::GreaterThan => *key > key_condition.condition_key,
                        Operator::GreaterThanOrEquals => *key > key_condition.condition_key,
                        Operator::LowerThan => true,
                        Operator::LowerThanOrEquals => true,
                        Operator::StartWith => {
                            // TODO IMPLEMENT
                            *key == key_condition.condition_key
                        },
                        Operator::Like => true,
                        Operator::Regex => true,
                    })
                }
                ConditionNode::And(left, right) => {
                    Ok(left.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 1)?
                        && right.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 2)?)
                }
                ConditionNode::Or(left, right) => {
                    Ok(left.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 3)?
                        || right.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 4)?)
                }
            }
        }
        fn can_be_greater_than_key(&self, keys: &Keys) -> Re<bool> {
            // est ce qyu'il est possible de trouver des ID plus grand que key qui pourront remplire la condition?
            match self {
                ConditionNode::Single(key_condition) => {
                    Ok(match key_condition.condition_operator {
                        Operator::Equal => *key < key_condition.condition_key,
                        Operator::NotEqual => true,
                        Operator::GreaterThan => true,
                        Operator::GreaterThanOrEquals => true,
                        Operator::LowerThan => *key < key_condition.condition_key,
                        Operator::LowerThanOrEquals => *key < key_condition.condition_key,
                        Operator::StartWith => {
                            // TODO IMPLEMENT
                            *key == key_condition.condition_key
                        },
                        Operator::Like => true,
                        Operator::Regex => true,
                    })
                }
                ConditionNode::And(left, right) => {
                    Ok(left.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 1)?
                        && right.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 2)?)
                }
                ConditionNode::Or(left, right) => {
                    Ok(left.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 3)?
                        || right.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 4)?)
                }
            }
        }
    */
    }

    mod logical_mod {
        use std::fmt;
        use strum_macros::EnumIter;
        use d_buf::{ColumnHeader, ColumnType, DBuf};
        use super::value_mod::{RetValue, ValueType};
        use super::{ParseStrToNode, Condition, Keys};
        #[derive(Debug, )]
        pub(crate) struct LogicalNode {
            left: Box<Condition>,
            right: Box<Condition>,
            logic: Logic, // "and", "or"
        }
        impl LogicalNode {
            pub fn new(str_in: &[char]) -> Option<Condition> {
                if let Some((logic, left, right)) = Logic::split_str_on_first_self(str_in) {
                    Some(Condition::Logical(LogicalNode {
                        left: Box::new(Condition::new(left.as_slice())),
                        right: Box::new(Condition::new(right.as_slice())),
                        logic
                    }))
                } else { None }
            }
            pub fn match_key(&self, keys: &Keys) -> bool {
                let left_match = self.left.match_key(keys);
                let right_match = self.right.match_key(keys);
                self.logic.calculate_bool(left_match, right_match)
            }
            pub fn get_value(&self, keys: &Keys) -> RetValue {
                //todo make the into works to converts bool to retvalue
                self.match_key(keys).into()
            }
        }

        #[derive(Debug, EnumIter, )]
        enum Logic {
            And,
            Or,
        }
        impl fmt::Display for Logic {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{:?}", self)
            }
        }
        impl ParseStrToNode for Logic {}
        impl Logic {
            fn calculate_bool(&self, left_value: bool, right_value: bool) -> bool {
                match self {
                    Logic::And => left_value && right_value,
                    Logic::Or => left_value || right_value,
                }
            }
        }
        #[cfg(test)]
        mod test_logical {
            use std::ops::Deref;
            use super::*;

            fn test_build_logic(input: &str, expected_left_input: &str, expected_right_input: &str, lo: Logic) -> Condition {
                let expected_left: Vec<u8> = expected_left_input.as_bytes().to_vec();
                let expected_right: Vec<u8> = expected_right_input.as_bytes().to_vec();
                let input_v: Vec<char> = input.chars().collect::<Vec<char>>();
                let cd: Condition = Condition::new(input_v.as_slice());
                match cd {
                    Condition::Logical(ref m_logic) => {
                        assert_eq!(m_logic.logic.to_string(), lo.to_string());
                        match m_logic.left.deref() {
                            Condition::Logical(_) => { assert_eq!("Logical", "Operation"); }
                            Condition::Operation(_) => { assert_eq!("Operation", "Operation"); }
                            Condition::Function(_) => { assert_eq!("Function", "Operation"); }
                            Condition::Single(v) => { assert_eq!(v.buf.get_row(0).unwrap(), expected_left); }
                        };
                        match m_logic.right.deref() {
                            Condition::Logical(_) => { assert_eq!("Logical", "Operation"); }
                            Condition::Operation(_) => { assert_eq!("Operation", "Operation"); }
                            Condition::Function(_) => { assert_eq!("Function", "Operation"); }
                            Condition::Single(v) => { assert_eq!(v.buf.get_row(0).unwrap(), expected_right); }
                        };
                    }
                    Condition::Operation(_) => { assert_eq!("Operation", "Logical"); }
                    Condition::Function(_) => { assert_eq!("Function", "Operation"); }
                    Condition::Single(_) => { assert_eq!("Single", "Operation"); }
                };
                cd
            }

            fn test_match_values(cd: &Condition, expectedBool: bool) {
                let emptyKeys = Keys {
                    buf: DBuf::new(&Vec::from(b"buf"), 4, &vec![
                        ColumnHeader { c_name: Vec::from(b"val"), c_type: ColumnType::Char(4) },
                    ], 1).unwrap(),
                    current_row: 0,
                };
                let b = cd.match_key(&emptyKeys);
                assert_eq!(b, expectedBool);
                let ret_val = cd.get_value(&emptyKeys);
                assert_eq!(bool::from(&ret_val), expectedBool);
            }
            #[test]
            fn test_logic_and_false() {
                let cd_o = test_build_logic(" ( True And False )  ", "\u{1}", "\0", Logic::And);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_logic_and_true() {
                let cd_o = test_build_logic(" ( True And True )  ", "\u{1}", "\u{1}", Logic::And);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_logic_or_false() {
                let cd_o = test_build_logic(" ( False Or False )  ", "\0", "\0",  Logic::Or);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_logic_or_true() {
                let cd_o = test_build_logic(" ( True Or False )  ", "\u{1}", "\0",  Logic::Or);
                test_match_values(&cd_o, true);
            }
        }
    }
    mod operator_mod{
        use std::cmp::{Ordering, PartialEq, PartialOrd};
        use std::fmt;
        use strum_macros::EnumIter;
        use d_buf::{ColumnHeader, ColumnType, DBuf};
        use super::{ParseStrToNode, Condition, Keys, ValueNode, RetValue,};

        use super::value_mod::{ValueType};
        #[derive(Debug,)]
        pub struct OperationNode {
            left: Box<Condition>,
            right: Box<Condition>,
            operator: Operator, // "==", "!=", ">" etc.
        }

        impl OperationNode {
            pub fn new(str_in: &[char]) -> Option<Condition> {
                if let Some((operator, left, right)) = Operator::split_str_on_first_self(str_in) {
                    Some(Condition::Operation(OperationNode{
                        left: Box::new(Condition::new(left.as_slice())),
                        right: Box::new(Condition::new(right.as_slice())),
                        operator
                    }))
                }else{ None }
            }
            pub fn match_key(&self,keys : &Keys) -> bool{
                let a = self.left.get_value(keys);
                let b = self.right.get_value(keys);
                self.operator.calculate_bool(keys, &a, &b)
            }
            pub fn get_value(&self, keys: &Keys) -> RetValue {
                self.match_key(keys).into()
            }
        }

        #[derive(Debug,EnumIter,)]
        enum Operator{
            Equal,
            NotEqual,
            GreaterThan,
            GreaterThanOrEquals,
            LowerThan,
            LowerThanOrEquals,
            StartWith,
        }
        impl fmt::Display for Operator {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    Operator::Equal => { write!(f, "==")}
                    Operator::NotEqual => { write!(f, "!=")}
                    Operator::GreaterThan => { write!(f, ">")}
                    Operator::GreaterThanOrEquals => { write!(f, ">=")}
                    Operator::LowerThan => { write!(f, "<")}
                    Operator::LowerThanOrEquals => { write!(f, "<=")}
                    Operator::StartWith => { write!(f, "~=")}
                }
            }
        }
        impl ParseStrToNode for Operator {}



        impl Operator {
            fn calculate_bool(&self, keys: &Keys, left: &RetValue, right: &RetValue) -> bool {
                match self {
                    Operator::Equal => left == right,
                    Operator::GreaterThan => left > right,
                    Operator::NotEqual => left != right,
                    Operator::GreaterThanOrEquals => left >= right,
                    Operator::LowerThan => left < right,
                    Operator::LowerThanOrEquals => left <= right,
                    Operator::StartWith => Operator::start_with(keys,left, right),
                }
            }
            fn start_with( keys: &Keys, left: &RetValue, right: &RetValue) -> bool {
                //todo Operator.start_with
                /*for (i,c) in right.iter().enumerate() {
                    if i >= left.len() {
                        return false;
                    }
                    if c != &right[i] {
                        return false;
                    }
                }*/
                true
            }
        }
        #[cfg(test)]
        mod test_operator {
            use std::ops::Deref;
            use super::*;

            fn test_build_operation(input :&str/*, expected_left_input : &str, expected_right_input : &str*/ ,op : Operator) -> Condition{

                /*let expected_left: Vec<u8> = expected_left_input.as_bytes().to_vec();
                let expected_right: Vec<u8> = expected_right_input.as_bytes().to_vec();




                / *let elv  : Vec<char> = String::from(expected_left_input).chars().collect();
                let erv  : Vec<char> = String::from(expected_right_input).chars().collect();
                let sout = cd.to_string();
                let r =  Condition::Operation(OperationNode {
                    left: Box::new(Condition::new(elv.as_slice())),
                    right: Box::new(Condition::new(erv.as_slice())),
                    operator: op,
                })
                let t = r.to_string();
                assert_eq!(sout,t);*/
                let input_v: Vec<char> = input.chars().collect::<Vec<char>>();
                let cd: Condition = Condition::new(input_v.as_slice());

                //TODO !! UNWRAP DEBUF IN A SINGLE PLACE WHO RAISE NEW ERROR
                match cd {
                    Condition::Logical(_) => { assert_eq!("Logical", "Operation"); }
                    Condition::Operation(ref mOper) => {
                        assert_eq!(mOper.operator.to_string(), op.to_string());

                        match mOper.left.deref() {
                            Condition::Logical(_) => { assert_eq!("Logical", "Operation") }
                            Condition::Operation(_) => { assert_eq!("Operation", "Operation") }
                            Condition::Function(_) => { assert_eq!("Function", "Operation") }
                            Condition::Single(v) => {
                                assert_eq!(v.buf.d_buf_header.row_count ,1)
                            }
                        };
                        match mOper.right.deref() {
                            Condition::Logical(_) => { assert_eq!("Logical", "Operation")}
                            Condition::Operation(_) => { assert_eq!("Operation", "Operation")}
                            Condition::Function(_) => { assert_eq!("Function", "Operation")}
                            Condition::Single(v) => {
                                assert_eq!(v.buf.d_buf_header.row_count ,1)
                            }
                        };
                    }
                    Condition::Function(_) => { assert_eq!("Function", "Operation")}
                    Condition::Single(_) => { assert_eq!("Single", "Operation")}
                };
                cd
            }

            fn test_match_values(cd : &Condition, expected_bool : bool) {
                let empty_keys = Keys{
                    buf: DBuf::new(&Vec::from(b"buf"),4, &vec![
                        ColumnHeader {c_name : Vec::from(b"val") , c_type: ColumnType::Char(4) },
                    ],1).unwrap(),
                    current_row: 0,
                };
                let b = cd.match_key(&empty_keys);
                assert_eq!(b, expected_bool);
                let ret_val = cd.get_value(&empty_keys);
                assert_eq!(bool::from(&ret_val), expected_bool);
            }
            #[test]
            fn test_operator_equal_char_false() {
                let cd_o = test_build_operation(" ( \"A\" == \"B\" )  ", Operator::Equal);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_operator_equal_char_true() {
                let cd_o = test_build_operation(" ( \"TOTO\" == \"TOTO\" )  ", Operator::Equal);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_operator_equal_unsigned_true() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 854123 == (854123 ) )  ",  Operator::Equal);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_operator_equal_unsigned_false() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( (854123) == 854122 )  ",  Operator::Equal);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_operator_lower_unsigned_true() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 854122 < 854123 )  ",  Operator::LowerThan);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_operator_lower_unsigned_false() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 854123 < 854122 )  ",  Operator::LowerThan);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_operator_lower_eq_unsigned_false() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 854123 <= 854122 )  ",  Operator::LowerThanOrEquals);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_operator_lower_eq_unsigned_true() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 854121 <= 854121 )  ",  Operator::LowerThanOrEquals);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_operator_greater_unsigned_false() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 854121 > 854121 )  ",  Operator::GreaterThan);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_operator_greater_unsigned_true() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 854121 > 854120 )  ",  Operator::GreaterThan);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_operator_greater_eq_unsigned_false() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 854121 >= 854122 )  ",  Operator::GreaterThanOrEquals);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_operator_greater_eq_unsigned_true() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 854121 >= 854121 )  ",  Operator::GreaterThanOrEquals);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_operator_equal_float_true() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 0.510 == (0.51 ) )  ",  Operator::Equal);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_operator_equal_float_false() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( (0.512) == 0.51 )  ",  Operator::Equal);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_operator_lower_float_true() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 185787.7 < 185787.71 )  ",  Operator::LowerThan);
                test_match_values(&cd_o, true);
                //185787.703
            }
            #[test]
            fn test_operator_lower_float2_true() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 185787.7 < 185787.712 )  ",  Operator::LowerThan);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_operator_lower_float_false() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 185787.75 < 185787.74 )  ",  Operator::LowerThan);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_operator_lower_eq_float_false() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 185787.79 <= 185787.78 )  ",  Operator::LowerThanOrEquals);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_operator_lower_eq_float_true() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 185787.72 <= 185787.72 )  ",  Operator::LowerThanOrEquals);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_operator_greater_float_false() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( -185787.7 > 1.0 )  ",  Operator::GreaterThan);
                test_match_values(&cd_o, false);
            }
            #[test]
            fn test_operator_greater_float_true() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 1.0 > -185787.71 )  ",  Operator::GreaterThan);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_operator_greater_eq_float_false() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 854121.0 >= -1.0 )  ",  Operator::GreaterThanOrEquals);
                test_match_values(&cd_o, true);
            }
            #[test]
            fn test_operator_greater_eq_float_true() {
                // for test conveniance i used number which represent assci chars ...
                let cd_o = test_build_operation(" ( 854121.1 >= 854121.1 )  ",  Operator::GreaterThanOrEquals);
                test_match_values(&cd_o, true);
            }
            //TODO REPLACE TOTOS by number and check real numbers ... :)
        }
    }

    mod function_mod{
        use std::fmt;
        use strum_macros::EnumIter;
        use d_buf::{ColumnHeader, ColumnType, DBuf};
        use super::{ParseStrToNode, Condition, Keys,};
        use super::value_mod::{ValueNode, RetValue,ValueType};

        #[derive(Debug,)]
        pub(crate) struct FunctionNode {
            name: String,
            args: Vec<Condition>,
            function: SqlFunction,
        }

        impl FunctionNode {
            pub fn new(str_in: &[char]) -> Option<Condition> {
                //TODO raise error on wrong numbers of args (maybe use a trait getNumberofArgs?)
                if let Some((function, left, right)) = SqlFunction::split_str_on_first_self(str_in) {
                    let name = "PARSE NAME TO DO".to_string();
                    let args : Vec<Condition>  = vec![];
                    //todo : parsing will be a bit different
                    Some(Condition::Function(FunctionNode{
                        // TODO : parsing will be a bit different
                        name,
                        args,
                        function
                    }))
                }else{ None }
            }
            pub fn match_key(&self,keys : &Keys) -> bool{
                (&self.get_value(keys)).into()
            }
            pub fn get_value(&self, keys: &Keys) -> RetValue {
                let mut vecRes : Vec<RetValue> = Vec::new();
                self.args.iter().for_each(|arg| {arg.get_value(keys); vecRes.push(arg.get_value(keys));});
                self.function.calculate_val(&vecRes)
            }
        }



        /// Represents SQL functions that can be used in queries.
        /// ```
        /// substr("Hello world", 0, 5) // Returns "Hello"
        /// like("Hello world", "Hello%") // Returns true
        /// regex("Hello world", "[0-9]+") // Returns false
        /// ```
        #[derive(Debug,EnumIter,)]
        pub enum SqlFunction {
            /// invert a value.
            ///
            /// # Example
            /// ```
            /// Not(true or false)   // Returns false
            /// ```
            Not,
            /// Extracts a substring from a string.
            ///
            /// # Example
            /// ```
            /// substr("Hello world", 0, 5)   // Returns "Hello"
            /// ```
            Substr,
            /// Checks if a string matches a pattern using SQL `LIKE` syntax.
            ///
            /// # Example
            /// ```
            /// like("Hello world", "Hello%") // Returns true
            /// ```
            Like,
            /// Checks if a string matches a regular expression.
            ///
            /// # Example
            /// ```
            /// regex("Hello world", "[0-9]+") // Returns false
            /// ```
            Regex,
            /// Represents a user-defined SQL function.
            ///
            /// # Note
            /// This variant is used for any custom SQL function implemented by the user.
            UserDefined,
        }

        impl fmt::Display for SqlFunction {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f,"{:?}",self)
            }
        }
        impl ParseStrToNode for SqlFunction{}
        impl SqlFunction{
            fn calculate_val(&self,args : &Vec<RetValue>) -> RetValue {
                match self {
                    SqlFunction::Not => {
                        return (!bool::from(&args[0]) ).into();
                    }
                    SqlFunction::Substr => {}
                    SqlFunction::Like => {}
                    SqlFunction::Regex => {}
                    SqlFunction::UserDefined => {}
                }

                //todo
                // todo err raise on incorrect parameters.
                RetValue{ // initiated temporary with an empty buffer
                    buf: DBuf::new(&Vec::from(b"bool"),4, &vec![
                        ColumnHeader {c_name : Vec::from(b"val") , c_type: ColumnType::Boolean },
                    ],1).unwrap(),
                    v_type: ValueType::Constant,
                }
            }
        }
    }
    mod value_mod{
        use crate::condition::new_condition::polish_vec_char;
        use crate::condition::fmt::Formatter;
        use std::cmp::Ordering;
        use std::fmt::Debug;
        use d_buf::{ColumnHeader, ColumnType, DBuf};
        use d_err::{DErr, DErrMap};
        use crate::condition::new_condition::function_mod::SqlFunction;
        use crate::condition::new_condition::logical_mod::LogicalNode;
        use crate::condition::new_condition::operator_mod::OperationNode;
        use super::{ParseStrToNode, Condition, Keys, Fe, Rc, Re};


        fn c_to_f64(bytes: &Vec<char>) -> Option<f64> {
            bytes.iter().collect::<String>().parse::<f64>().ok()
        }
        fn c_from_f64(f: f64) -> Vec<char> {
            todo!("not implemented but no need")
        }
        fn b_to_f64(bytes:&[u8]) -> Option<f64> {
            let array: [u8; 8] = bytes.try_into().ok()?;
            Some(f64::from_ne_bytes(array))
        }
        fn b_from_f64(f: f64) -> Vec<u8> {
            f.to_ne_bytes().to_vec()
        }
        fn c_to_f32(bytes: &Vec<char>) -> Option<f32> {
            let f32 : f32 = bytes.iter().collect::<String>().parse::<f32>().ok()?;
            // we check if the float fit into a f32
            let f32_str = format!("{}", f32);
            let f32_f64 : f64 = f32_str.parse::<f64>().ok()?;
            let f64 : f64 = bytes.iter().collect::<String>().parse::<f64>().ok()?;
            if f32_f64 == f64 {
                Some(f32)
            }else{ None }
        }
        fn b_to_f32(bytes: &[u8]) -> Option<f32> {
            let array: [u8; 4] = bytes.try_into().ok()?;
            Some(f32::from_ne_bytes(array))
        }
        fn b_from_f32(f: f32) -> Vec<u8> {
            f.to_ne_bytes().to_vec()
        }



       // #[derive(Debug,)]
        #[derive(Clone)]
        pub struct ValueNode{
            pub buf : d_buf::DBuf,
            pub v_type : ValueType,
        }
        impl Debug for ValueNode {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
                //write!(f,"{:?}",self)
                let b_name = String::from_utf8(self.buf.d_buf_header.name.clone()).unwrap();
                let c_name = String::from_utf8(self.buf.d_buf_header.columns[0].c_header.c_name.clone()).unwrap();
                let val_str = String::from_utf8(self.buf.get_row(0).unwrap().clone()).unwrap();

                write!(f,"{}/{}:{}",b_name,c_name,val_str)
            }
        }

        impl ValueNode{
            pub fn new(str_in: &[char])->Option<Condition> {
                let mut in_vec = polish_vec_char(str_in);
                if      let Some(c_char)  = ValueNode::new_chars (&mut in_vec) {Some(c_char)}
                else if let Some(c_bool)  = ValueNode::new_bool  (&in_vec)     {Some(c_bool)}
                else if let Some(c_date)  = ValueNode::new_date  (&in_vec)     {Some(c_date)}
                else if let Some(c_num)   = ValueNode::new_number(&in_vec)     {Some(c_num)}
                else {todo!("Error : value cant be parsed")}
            }
            fn new_number(in_vec:&Vec<char>)->Option<Condition> {
                if      let Some(n_u)  = ValueNode::new_unsigned(&in_vec) {Some(n_u)}
                else if let Some(n_f)  = ValueNode::new_float   (&in_vec) {Some(n_f)}
                else {None}//todo add some other numbers?
            }
            fn new_unsigned(in_vec:&Vec<char>)->Option<Condition> {
                if let Ok(n_u)  = in_vec.iter().collect::<String>().parse::<u64>() {
                    let c_type = ColumnType::U64;//todo add some other size for unsigned ?
                    let row_val =  n_u.to_ne_bytes().to_vec();
                    Some( Condition::Single(ValueNode::build_new(c_type,row_val)))
                }
                else {None}
            }
            fn new_float(in_vec:&Vec<char>)->Option<Condition> {
                if let Some(f)  = c_to_f32(in_vec) {
                    Some( Condition::Single(ValueNode::build_new(ColumnType::F32,b_from_f32(f))))
                }else if let Some(f)  = c_to_f64(in_vec) {
                    Some( Condition::Single(ValueNode::build_new(ColumnType::F64,b_from_f64(f))))
                }else {None}
            }
            fn new_date(vec_char_in:&Vec<char>)->Option<Condition> {
                None
                //TODO TO IMPLEMENT ? or to use a function to parse date?
            }
            fn new_bool(vec_char_in:&Vec<char>)->Option<Condition> {
                if "False" == vec_char_in.iter().collect::<String>() {
                    Some( Condition::Single(ValueNode::from(false)))
                }else if "True" == vec_char_in.iter().collect::<String>() {
                    Some( Condition::Single(ValueNode::from(true)))
                }else{None}
            }
            fn new_chars(vec_char_in:&mut  Vec<char>)->Option<Condition> {
                if vec_char_in.first() ==  Some(&'\"')  && vec_char_in.last() == Some(&'\"') {
                    vec_char_in.remove(0);
                    vec_char_in.pop();
                    let row_val :Vec<u8> = vec_char_in.iter().collect::<String>().into_bytes();
                    let c_type = ColumnType::Char(row_val.len());
                    Some( Condition::Single(ValueNode::build_new(c_type,row_val)))
                }else {
                    None
                }
            }
            fn build_new(c_type : ColumnType, row_val : Vec<u8>) -> ValueNode{
                let c_name = Vec::from(b"v");
                let mut buf = DBuf::new(&c_name.clone(),c_name.len(), &vec![
                    ColumnHeader {c_name , c_type },
                ],1).unwrap();
                buf.add_rows(1,&row_val).unwrap();
                ValueNode{
                    buf,
                    v_type: ValueType::Constant,
                }
            }
            pub fn match_key(&self,keys : &Keys) -> bool{
                //todo ERR
                (&self.get_value(keys).unwrap()).into()
            }
            pub fn get_value(&self,keys :&Keys)->  Re<RetValue> {
                match self.v_type {
                    ValueType::Constant => Ok(self.clone()),
                    ValueType::FromKey => { self.get_value_from_key(keys) },
                }
            }
            fn get_value_from_key(&self, keys :&Keys) ->  Re<RetValue> {
                //TODO ERROR HANDLING
                if self.buf.d_buf_header.name == keys.buf.d_buf_header.name {
                    let data_res = &keys.buf.get_row_col_from_name(keys.current_row,&self.buf.d_buf_header.columns[0].c_header.c_name).map_err(|e| DErr::RootError(Fe::GetValue, 1, Rc::DBufError(e.to_string_chain())))?;
                    let mut buf = DBuf::new(&self.buf.d_buf_header.name,4, &vec![
                        ColumnHeader {c_name : self.buf.d_buf_header.columns[0].c_header.c_name.clone(), c_type: ColumnType::Boolean },
                    ],1).unwrap();
                    buf.add_rows(1,data_res);
                    Ok(RetValue{
                        buf,
                        v_type: ValueType::FromKey,
                    })
                }else{
                    Err(DErr::RootError(Fe::GetValue, 1, Rc::IncorrectTableSelected("self.buf.d_buf_header.name".to_string())))
                }
            }
        }
        pub type RetValue = ValueNode;

        // Implémenter From<Foo> pour bool
        impl From<&RetValue> for bool {
            fn from(rv: &RetValue) -> Self {
                rv.buf.d_buf_row_values.as_slice() >= &[1]
            }
        }
        impl From<bool> for RetValue {
            fn from(b: bool) -> Self {
                let mut buf = DBuf::new(&Vec::from(b"ValueNode"),9, &vec![
                    ColumnHeader {c_name : Vec::from(b"val") , c_type: ColumnType::Boolean},
                ],1).unwrap();
                buf.add_rows(1,&vec![b.into()]);
                ValueNode{
                    buf,
                    v_type: ValueType::Constant,
                }
            }
        }

        impl PartialEq for &RetValue {
            fn eq(&self, other: &Self) -> bool {
                //TODO check types? check names ? check other rows?
                self.buf.d_buf_row_values == other.buf.d_buf_row_values
            }
        }
        impl PartialOrd for &RetValue {
            //TODO move to Dbuff
            //TODO Compare diffent DATA type F32 F64 ...
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {

                //TODO check types? for both items... check names ? check other rows? maybe add a fn in buf to get all data?
                // Get the first row from both buffers for comparison
                let self_row = &self.buf.d_buf_row_values;
                let other_row = &other.buf.d_buf_row_values;
                //self_row.partial_cmp(other_row)

                // Compare based on column type
                let col_type = &self.buf.d_buf_header.columns[0].c_header.c_type;

                match col_type {
                    ColumnType::F64 => {
                        let self_val :f64 = b_to_f64(self_row).unwrap(); // assume self_row is a valid f64, as we build it from a f64
                        let other_val:f64 = b_to_f64(other_row).unwrap();// if self is F64, other can be different TODO : to convert when possible

                        self_val.partial_cmp(&other_val)
                    }
                    ColumnType::F32 => {
                        let self_val :f32 = b_to_f32(self_row).unwrap(); // assume self_row is a valid f64, as we build it from a f64
                        let other_val:f32 = b_to_f32(other_row).unwrap();

                        self_val.partial_cmp(&other_val)
                    }
                    /*ColumnType::Boolean => {
                        let self_val = self_row[0] != 0;
                        let other_val = other_row[0] != 0;
                        self_val.partial_cmp(&other_val)
                    }*/
                    /*ColumnType::U8 => {
                        let self_val = self_row[0];
                        let other_val = other_row[0];
                        self_val.partial_cmp(&other_val)
                    }*/
                    /*ColumnType::I8 => {
                        let self_val = self_row[0] as i8;
                        let other_val = other_row[0] as i8;
                        self_val.partial_cmp(&other_val)
                    }*/
                    /*ColumnType::Char(_) => {
                        // For strings/char arrays, compare as byte slices
                        self_row.partial_cmp(other_row)
                    }*/
                    // Add other type comparisons as needed
                    _ => {
                        // For unimplemented types, fall back to byte comparison
                        self_row.partial_cmp(other_row)
                    }
                }
            }
        }


        type ValueSliceU8 = [u8];

        #[derive(Debug,)]
        #[derive(Clone)]
        pub enum ValueType{
            Constant,
            FromKey,
        }
        #[cfg(test)]
        mod value_tests{
            use std::ops::Deref;
            use d_buf::{ColumnHeader, ColumnType, DBuf};
            use crate::condition::new_condition::{Condition, Keys};
            use crate::condition::new_condition::value_mod::{b_from_f32, b_to_f32, RetValue};

            #[test]
            fn test_conversion_to_bool_false() {
                let u :u8 = false.into();
                assert_eq!(u,0);

                let bf = RetValue::from(false);
                assert_eq!(bf.buf.get_row(0).unwrap()[0], 0);
                assert_eq!(bool::from(&bf), false);
            }
            #[test]
            fn test_conversion_to_bool_true() {
                let u :u8 = true.into();
                assert_eq!(u,1);
                let bt = RetValue::from(true);
                assert_eq!(bt.buf.get_row(0).unwrap()[0],1);
                assert_eq!(bool::from(&bt),true);
            }
            fn test_build_value(input :&str, c_type : ColumnType) -> Condition{
                let input_v: Vec<char> = input.chars().collect::<Vec<char>>();
                let cd: Condition = Condition::new(input_v.as_slice());

                //TODO !! UNWRAP DEBUF IN A SINGLE PLACE WHO RAISE NEW ERROR
                match cd {
                    Condition::Logical(_) => { assert_eq!("Logical", "Value"); }
                    Condition::Operation(_) => { assert_eq!("Operation", "Value"); }
                    Condition::Function(_) => { assert_eq!("Function", "Value"); }
                    Condition::Single(ref ValueNode) => {
                        assert_eq!(ValueNode.buf.d_buf_header.columns[0].c_header.c_type ,c_type);
                    }
                };
                cd
            }

            fn test_get_values(cd : &Condition,expected_row_val : &[u8]) {
                let emptyKeys = Keys{
                    buf: DBuf::new(&Vec::from(b"buf"),4, &vec![
                        ColumnHeader {c_name : Vec::from(b"val") , c_type: ColumnType::Char(4) },
                    ],1).unwrap(),
                    current_row: 0,
                };
                let b = cd.get_value(&emptyKeys);
                assert_eq!(b.buf.d_buf_row_values, expected_row_val);
            }
            #[test]
            fn test_new_number_unsigned(){
                let cd = test_build_value("257",ColumnType::U64);
                test_get_values(&cd,&[1, 1, 0, 0, 0, 0, 0, 0]);
            }

            #[test]
            fn test_conversion_number_float(){
                let f : f32 = 12384.21;
                let b = b_from_f32(f);
                assert_eq!(b_to_f32(&b).unwrap(),f);
            }
            #[test]
            fn test_new_number_float(){
                let cd = test_build_value("-1.0",ColumnType::F32);
                test_get_values(&cd,&b_from_f32(-1.0));

                let cd = test_build_value("0.0",ColumnType::F32);
                test_get_values(&cd,&b_from_f32(0.0));

                let cd = test_build_value("2586.325",ColumnType::F32);
                test_get_values(&cd,&b_from_f32(2586.325));

                let cd = test_build_value("-2586.325",ColumnType::F32);
                test_get_values(&cd,&b_from_f32(-2586.325));

                let cd = test_build_value("185787.7 ",ColumnType::F32);
                test_get_values(&cd,&b_from_f32(185787.7));

            }
        }
    }
    pub trait ParseStrToNode: IntoEnumIterator + fmt::Display + Sized {
        // example: Self = (enum Foo {And,Or,Not})
        // right = "TOTO.B = "A" or TOTO.B = "B" and TOTO.C = "C"
        //           left  = ""
        // returns (Foo::Or)
        // modify :  right =  TOTO.B = "B" and TOTO.C = "C"
        //           left  =  TOTO.B = "A"
        fn split_str_on_first_self(str_in : &[char]) -> Option<(Self,Vec<char>,Vec<char>)>{
            let mut left : Vec<char> = Vec::new();
            let mut right = polish_vec_char(str_in);
            let mut current_token:Vec<char> = Vec::new();
            let mut bracket_count = 0;
            let mut double_quote_open :  bool = false;
            let mut single_quote_open :  bool = false;
            let mut any_quote_open :  bool = false;


            //TODO EXCLUDE "les string" et 'les string'
            while right.len() > 0 {
                //consuming the right to the current token
                let c = right.get (0).unwrap().clone(); // safe unwrap as we check existence just before

                // adding "text" or 'text' to current token
                let mut any_quote_changed : bool = false;
                if c == '"' && !single_quote_open{
                    double_quote_open = !double_quote_open;
                    any_quote_open = double_quote_open;
                    any_quote_changed = true
                }else if c == '\'' && !double_quote_open{
                    single_quote_open = !single_quote_open;
                    any_quote_open = single_quote_open;
                    any_quote_changed = true
                }
                if any_quote_open || any_quote_changed {
                    right.remove(0);
                    current_token.push(c);
                    continue;
                }
                /*
                                // removing unused spaces at the beginning of token
                                if c == ' ' {
                                    if current_token.is_empty(){
                                        right.remove(0);
                                        current_token.push(c);
                                        continue;
                                    }else if bracket_count == 0 {
                                        if let Some(myself) =  Self::deal_with_and_reset_token(&mut current_token, &mut left) {
                                            right.remove(0);
                                            return Some((myself,left,right));
                                        }else{
                                            right.remove(0);
                                            continue
                                        }
                                    }else{
                                        right.remove(0);
                                        current_token.push(c);
                                        continue
                                    }
                                }*/
                //only dealing closed brackets
                let add_to_token_before_deal : bool;
                let add_to_token_after_deal : bool;
                let mut token_to_deal : bool;
                match c{
                    ' '=>{token_to_deal=true; add_to_token_before_deal = true;},
                    '('=>{token_to_deal=true; add_to_token_before_deal = false;bracket_count += 1;}
                    ')'=>{token_to_deal=true; add_to_token_before_deal = true; bracket_count -= 1;}
                    _ => {token_to_deal=false;add_to_token_before_deal = true;},
                }
                if bracket_count < 0 {
                    //TODO RAISE ERROR
                    return None;
                }else if bracket_count > 0 && c != '(' {
                    token_to_deal = false;
                }
                if add_to_token_before_deal {
                    right.remove(0);
                    current_token.push(c);
                }else if current_token.is_empty() {
                    token_to_deal = false;
                }
                if token_to_deal {
                    if let Some(myself) =  Self::deal_with_and_reset_token(&mut current_token, &mut left) {
                        return Some((myself,left,right));
                    }
                }
                if !add_to_token_before_deal { //meaning wa add it after...
                    right.remove(0);
                    current_token.push(c);
                }
                //TODO here we might have a token : "(a==" ; right : "b)" we want to split the token into left.push("(a") and deal the tocken "=="

            }
            if bracket_count > 0 {
                //TODO RAISE ERROR
                None
            }else if let Some(myself) =  Self::deal_with_and_reset_token(&mut current_token, &mut left) {
                Some((myself,left,right))
            }else{
                None
            }
        }
        #[doc(hidden)]
        fn deal_with_and_reset_token(current_token: &mut Vec<char>, left : &mut Vec<char>) -> Option<Self>{
            let token_str: String = current_token.iter().filter(|&c| !c.is_whitespace()).collect();
            for enum_item in Self::iter() {
                if token_str == enum_item.to_string() {
                    return Some(enum_item);
                }
            }
            left.append(current_token);
            current_token.clear();
            None
        }

    }
    /// clear right string extremity by removing ' ' and '()'
    /// " (   hello world )   " => "hello world"
    fn polish_vec_char(str_in: &[char]) -> Vec<char> {
        let mut double_quote_open :  bool = false;
        let mut single_quote_open :  bool = false;
        let mut any_quote_open :  bool = false;
        let mut prev_c : char = ' ';
        let mut vec_char : Vec<char> = str_in.to_vec();
        /*
        for c in str_in.chars() {
            if c == '"' && !single_quote_open{
                double_quote_open = !double_quote_open;
            }else if c == '\'' && !double_quote_open{
                single_quote_open = !single_quote_open;
            }
            any_quote_open = single_quote_open || double_quote_open;
            //adding a space to parse the left side of parentesis token
            if matches!(c, '(' | ')') && prev_c != ' ' &&  !any_quote_open {
                vec_char.push(' ');
            }
            if any_quote_open || !c.is_whitespace() {
                vec_char.push(c);
                prev_c = c;
            }else if c.is_whitespace(){
                if  prev_c != ' '{
                    prev_c = ' ';
                    vec_char.push(prev_c);
                }
            }
        }*/
        let mut polished : bool = false;
        while (!polished){
            polished = true;
            if vec_char.first() == Some(&' ') {
                vec_char.remove(0);
                polished = false;
            }
            if vec_char.last() == Some(&' ') {
                vec_char.pop();
                polished = false;
            }
            if vec_char.first() == Some(&'(') && vec_char.last() == Some(&')') {
                vec_char.pop();
                vec_char.remove(0);
                polished = false;
            }
        }
        vec_char
    }
    #[cfg(test)]
    mod test_condition{

    }
}



/*
pub enum Rc {
    NotImplemented(String),
    KeyNotFound,
    DuplicateKey,
    IncorrectMemoryAddress,
    MaxNumberOfKeyInvalid,
    MaxNumberOfChildInInvalid,
    SearchPatternInvalid,
    NoInsertedKeyPtrProvided,
    NoDataPtr,
    MismatchedParentheses,
    InvalidToken(String),
    UnexpectedEndOfInput,
    InvalidOperator(String),
    InvalidValue(String),
    FunctionAsExpression(String),
    SingleAsExpression(String),
    OperationAsValue(String),
    UnexpectedSpaceOnUnknownToken,
    UnexpectedCharAfterBracketToken(char),
    UnexpectedBracketOnBracketToken,
    // Errors from d_buf library
    DBufError(String),
}

impl fmt::Display for Rc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rc::NotImplemented(what) => {write!(f, "Not implemented: {}", what)},
            Rc::KeyNotFound 				=> write!(f, "Key not found in the B+ Tree"),
            Rc::DuplicateKey 				=> write!(f, "Duplicate key in the B+ Tree"),
            Rc::IncorrectMemoryAddress  	=> write!(f, "Physical memory address incorrect"),
            Rc::MaxNumberOfKeyInvalid 		=> write!(f, "Maximum number of keys is invalid"),
            Rc::MaxNumberOfChildInInvalid 	=> write!(f, "Maximum number of child is invalid"),
            Rc::SearchPatternInvalid 		=> write!(f, "Incorrect search pattern provided"),
            Rc::NoInsertedKeyPtrProvided 	=> write!(f, "No inserted Key provided"),
            Rc::NoDataPtr 					=> write!(f, "No DataPtr provided"),
            Rc::MismatchedParentheses       => write!(f, "Mismatched parentheses"),
            Rc::InvalidToken(token) => write!(f, "Invalid token: {}", token),
            Rc::UnexpectedEndOfInput        => write!(f, "Unexpected end of input"),
            Rc::InvalidOperator(op) => write!(f, "Invalid operator: {}", op),
            Rc::InvalidValue(val)   => write!(f, "Invalid value: {}", val),
            Rc::FunctionAsExpression(func) => write!(f, "Function cannot be used as expression: {}", func),
            Rc::SingleAsExpression(single) => write!(f, "Single value cannot be used as expression: {}", single),
            Rc::OperationAsValue(op) => write!(f, "Operation cannot be used as value: {}", op),
            Rc::UnexpectedSpaceOnUnknownToken => write!(f, "Unexpected space on unknown token"),
            Rc::UnexpectedCharAfterBracketToken(c) => write!(f, "Unexpected character after bracket token: {}", c),
            Rc::UnexpectedBracketOnBracketToken => write!(f, "Unexpected bracket on bracket token"),
            Rc::DBufError(msg) => write!(f, "DBuf error: {}", msg),
        }
    }
}

#[derive(Debug)]
/// Function where the error transits
enum Fe {
    MatchKey,
    GetRowValue,
    CanBeLowerOrEqualThanKey,
    CanBeGreaterOrEqualThanKey,
    CanBeLowerThanKey,
    CanBeGreaterThanKey,
    ParseExpression,
    ParseAtom,
    ParseAnd,
    ParseCondition,
    Lex,
    Equal,
    GetValue,
    FillOrCreateNext,
    ParseConditionString,
    GetValueFromAndOrNot,
}
*/
/*
struct Keys{
    pub buf : d_buf::DBuf,
    pub current_row : usize,
}
#[derive(Clone,PartialEq)]
pub enum ValueType{
    Constant,
    FromFunction,
    FromKey,
    Unknown,
}
#[derive(Clone,)]
struct Value{
    pub buf : d_buf::DBuf,
    pub v_type : ValueType,
}

impl Value{


}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f,"{:?}",self.buf.d_buf_header.columns[0].c_header.c_name)
    }
}
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:?}",self.buf.d_buf_header.name)
    }
}

#[derive(Debug,Clone)]
enum Operator{
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEquals,
    LowerThan,
    LowerThanOrEquals,
    StartWith,
}
/// an sql operator allow an index to be not parsed (can break before the end of the index)
/// a function on the contry cannot (full index parse)
/// operator in a condition get the values of childs to convert into a boolean
impl Operator{
    fn match_key(&mut self,keys : &Keys,left : &mut Box<ConditionNode>,right : &mut Box<ConditionNode>) ->  Re<bool> {
        let left_val = left.get_value(keys).d_err_map(Fe::GetValue, 1)?;
        let right_val = right.get_value(keys).d_err_map(Fe::GetValue, 2)?;
        let left_row_val = left_val.get_row_value().d_err_map(Fe::GetValue, 3)?;
        let right_row_val = right_val.get_row_value().d_err_map(Fe::GetValue, 3)?;
        Ok(match self {
            Operator::Equal				    => left_row_val == right_row_val,
            Operator::GreaterThan			=> left_row_val > right_row_val,
            Operator::NotEqual				=> left_row_val != right_row_val,
            Operator::GreaterThanOrEquals	=> left_row_val >= right_row_val,
            Operator::LowerThan			    => left_row_val < right_row_val,
            Operator::LowerThanOrEquals 	=> left_row_val >= right_row_val,
            Operator::StartWith			    => Operator::start_with( left_row_val, right_row_val)
        })
    }
    fn start_with(left : Vec<u8>, right : Vec<u8>) -> bool {
        for (i,c) in right.iter().enumerate() {
            if i >= left.len() {
                return false;
            }
            if c != &right[i] {
                return false;
            }
        }
        true
    }
}
impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self{
            Operator::Equal				    =>  write!(f,"=="),
            Operator::GreaterThan			=>  write!(f,">"),
            Operator::NotEqual				=>  write!(f,"!="),
            Operator::GreaterThanOrEquals	=>  write!(f,">="),
            Operator::LowerThan			    =>  write!(f,"<"),
            Operator::LowerThanOrEquals 	=>  write!(f,"<="),
            Operator::StartWith			    =>  write!(f,"~="),
        }
    }
}

/// Represents SQL functions that can be used in queries.
/// ```
/// substr("Hello world", 0, 5) // Returns "Hello"
/// like("Hello world", "Hello%") // Returns true
/// regex("Hello world", "[0-9]+") // Returns false
/// ```
#[derive(Debug)]
pub enum SqlFunction {
    /// Extracts a substring from a string.
    ///
    /// # Example
    /// ```
    /// substr("Hello world", 0, 5)   // Returns "Hello"
    /// ```
    Substr,
    /// Checks if a string matches a pattern using SQL `LIKE` syntax.
    ///
    /// # Example
    /// ```
    /// like("Hello world", "Hello%") // Returns true
    /// ```
    Like,
    /// Checks if a string matches a regular expression.
    ///
    /// # Example
    /// ```
    /// regex("Hello world", "[0-9]+") // Returns false
    /// ```
    Regex,
    /// Represents a user-defined SQL function.
    ///
    /// # Note
    /// This variant is used for any custom SQL function implemented by the user.
    UserDefined,
}
impl SqlFunction{
    fn get_value(&self,keys : &Keys,params : Vec<ConditionNode>) ->  Re<Value> {
        match self {
            //TODO IMPLEMENT
            SqlFunction::Substr => {
                Err(DErr::RootError(Fe::GetValue, 1, Rc::NotImplemented(self.to_string())))
            }
            SqlFunction::Like => {
                Err(DErr::RootError(Fe::GetValue, 1, Rc::NotImplemented(self.to_string())))
            }
            SqlFunction::Regex => {
                Err(DErr::RootError(Fe::GetValue, 1, Rc::NotImplemented(self.to_string())))
            }
            SqlFunction::UserDefined => {
                Err(DErr::RootError(Fe::GetValue, 1, Rc::NotImplemented(self.to_string())))
            }
        }
    }
}
impl fmt::Display for SqlFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
/*
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Single {
    Constant,
    Key,
}
impl Single {
    fn get_value(&self,keys : &Keys)->  Re<Value> {
        match (self){
            Single::Constant => {}
            Single::Key => {}
        }
    }
}*/

/// ```rust
/// Enum representing a condition node used for constructing search queries.
///
/// This enum defines the structure of a condition node, which can represent
/// a basic key search, or a combination of key searches with logical operators.
/// It provides a flexible way to model search conditions in a hierarchical manner.
///
/// # Variants
///
/// * `Single(KeySearch)`
///     - Represents a single key search condition. The `KeySearch` type
///       encapsulates the specific search query details.
///
/// * `And(Box<ConditionNode>, Box<ConditionNode>)`
///     - Represents a logical AND operation between two condition nodes.
///       Both child nodes must evaluate to `true` for the entire condition to be met.
///       Stored as boxed variants to allow recursive definitions with heap allocation.
///
/// * `Or(Box<ConditionNode>, Box<ConditionNode>)`
///     - Represents a logical OR operation between two condition nodes.
///       At least one of the child nodes must evaluate to `true` for the condition to pass.
///       Stored as boxed variants to handle recursive definitions.
///
/// # Example
///
/// ```rust
/// use crate::ConditionNode;
/// use crate::KeySearch;
///
/// let single_condition = ConditionNode::Single(KeySearch::new("example_key"));
/// let and_condition = ConditionNode::And(
///     Box::new(single_condition),
///     Box::new(ConditionNode::Single(KeySearch::new("another_key"))),
/// );
/// let or_condition = ConditionNode::Or(
///     Box::new(and_condition),
///     Box::new(ConditionNode::Single(KeySearch::new("third_key"))),
/// );
///
/// println!("{:?}", or_condition);
/// ```
///
/// This would represent a condition where:
/// - The query matches if "example_key" AND "another_key" are true,
///   OR "third_key" is true.
///
/// # Notes
///
/// The usage of `Box` for nested `ConditionNode` types allows for recursive
/// data structures while avoiding stack overflow caused by excessive nesting.
/// ```
#[derive(Debug,Clone)]
enum ConditionType2 {
    Operation(Operator), // "transform" two values into a boolean
    Function,                  // provide a value
    Single,                                             // provide a value
    And,                // "AND" two boolean
    Or,                 //  "OR" two boolean
    Not,                                  // inverse a boolean
}
#[derive(Debug)]
enum ConditionType {
    Operation(Operator,Box<ConditionNode>, Box<ConditionNode>), // "transform" two values into a boolean
    Function(SqlFunction, Vec<ConditionNode>),                  // provide a value
    Single(),                                             // provide a value
    And(Box<ConditionNode>, Box<ConditionNode>),                // "AND" two boolean
    Or(Box<ConditionNode>, Box<ConditionNode>),                 //  "OR" two boolean
    Not(Box<ConditionNode>),                                  // inverse a boolean
}
#[derive(Debug)]
pub struct ConditionNode{
    value : Value,
    cdt_type : ConditionType
}

impl ConditionNode {
    fn match_key(&mut self,keys : &Keys) ->  Re<bool> {
        match &mut self.cdt_type {
            ConditionType::Operation(operator, left, right) => {
                operator.match_key(keys,left,right).d_err_map(Fe::MatchKey, 3)
            }
            ConditionType::And( left, right) => {
                Ok(left.match_key(keys).d_err_map(Fe::MatchKey, 1)?
                    && right.match_key(keys).d_err_map(Fe::MatchKey, 2)?)
            }
            ConditionType::Or( left, right) => {
                Ok(left.match_key(keys).d_err_map(Fe::MatchKey, 3)?
                    || right.match_key(keys).d_err_map(Fe::MatchKey, 4)?)
            }
            ConditionType::Function(sql_function,_) => {
                Err(DErr::RootError(Fe::MatchKey, 1, Rc::FunctionAsExpression(sql_function.to_string())))
            }
            ConditionType::Single() => {
                // TODO could return true when not null
                Err(DErr::RootError(Fe::MatchKey, 1, Rc::SingleAsExpression(self.value.to_string())))
            }
            ConditionType::Not(condition) => {
                Ok( ! condition.match_key(keys).d_err_map(Fe::MatchKey, 3)?)
            }
        }
    }
    //TODO  Re<Value>  vs  Re<&Value>  ????
    fn get_value(&self,keys : &Keys) -> Re<Value> {
        match self {
            ConditionNode::Operation(operator, _, _) => {
                Err(DErr::RootError(Fe::GetValue, 1, Rc::OperationAsValue(operator.to_string())))
            }
            ConditionNode::Function(function, params) => {
                function.get_value(keys, *params).d_err_map(Fe::GetValue, 2)
            }
            ConditionNode::Single(value) => {
                value.get_value(keys).d_err_map(Fe::GetValue, 2)
            }
            _ => {
                self.get_value_from_logics(keys).d_err_map(Fe::GetValue, 2)
            }
        }
    }
    fn get_value_from_logics(&self, keys : &Keys) -> Re<Value> {
        let mut val = Value {
            buf: DBuf::new(
                &Vec::from(b"and"), 3, &vec![
                    ColumnHeader { c_name: Vec::from(b"and"), c_type: ColumnType::Boolean },
                ], 1
            ).map_err(|e| DErr::RootError(Fe::GetValueFromAndOrNot, 1, Rc::DBufError(e.to_string_chain())))?,
            v_type:  ValueType::Unknown,
        };
        val.buf.add_rows(1,&vec![self.match_key(keys).d_err_map(Fe::GetValueFromAndOrNot, 1)? as u8]).map_err(
            |e| DErr::RootError(Fe::GetValueFromAndOrNot, 2, Rc::DBufError(e.to_string_chain()))
        )?;
        Ok(val)
    }
    fn can_be_lower_or_equal_than_key(&self,keys : &Keys) ->  Re<bool> {
        match self {
            ConditionNode::Single(key_condition) => {
                Ok(match key_condition.condition_operator {
                    Operator::Equal				    => *key >= key_condition.condition_key,
                    Operator::NotEqual				=> true,
                    Operator::GreaterThan			=> *key >= key_condition.condition_key,
                    Operator::GreaterThanOrEquals	=> *key >= key_condition.condition_key,
                    Operator::LowerThan			    => true,
                    Operator::LowerThanOrEquals	    => true,
                    Operator::StartWith			    => {
                        // TODO IMPLEMENT
                        *key == key_condition.condition_key
                    },
                    Operator::Like				    => true,
                    Operator::Regex				    => true,
                })
            }
            ConditionNode::And(left, right) => {
                Ok(     left.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 1)?
                    && right.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 2)?)
            }
            ConditionNode::Or(left, right) => {
                Ok(     left.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 3)?
                    || right.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 4)?)
            }
        }
    }
    fn can_be_greater_or_equal_than_key(&self,keys : &Keys) ->  Re<bool> {
        match self {
            ConditionNode::Single(key_condition) => {
                Ok(match key_condition.condition_operator {
                    Operator::Equal				    => *key <=  key_condition.condition_key,
                    Operator::NotEqual				=> true,
                    Operator::GreaterThan			=> true,
                    Operator::GreaterThanOrEquals	=> true,
                    Operator::LowerThan			    => *key <=  key_condition.condition_key,
                    Operator::LowerThanOrEquals	    => *key <= key_condition.condition_key,
                    Operator::StartWith			    => {
                        // TODO IMPLEMENT
                        *key == key_condition.condition_key
                    },
                    Operator::Like				    => true,
                    Operator::Regex				    => true,
                })
            }
            ConditionNode::And(left, right) => {
                Ok(     left.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 1)?
                    && right.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 2)?)
            }
            ConditionNode::Or(left, right) => {
                Ok(     left.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 3)?
                    || right.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 4)?)
            }
        }
    }
    fn can_be_lower_than_key(&self,keys : &Keys) ->  Re<bool> {
        match self {
            ConditionNode::Single(key_condition) => {
                Ok(match key_condition.condition_operator {
                    Operator::Equal				    => *key > key_condition.condition_key,
                    Operator::NotEqual				=> true,
                    Operator::GreaterThan			=> *key >  key_condition.condition_key,
                    Operator::GreaterThanOrEquals	=> *key > key_condition.condition_key,
                    Operator::LowerThan			    => true,
                    Operator::LowerThanOrEquals	    => true,
                    Operator::StartWith			    => {
                        // TODO IMPLEMENT
                        *key == key_condition.condition_key
                    },
                    Operator::Like				    => true,
                    Operator::Regex				    => true,
                })
            }
            ConditionNode::And(left, right) => {
                Ok(     left.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 1)?
                    && right.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 2)?)
            }
            ConditionNode::Or(left, right) => {
                Ok(     left.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 3)?
                    || right.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 4)?)
            }
        }
    }
    fn can_be_greater_than_key(&self,keys : &Keys) ->  Re<bool> {
        // est ce qyu'il est possible de trouver des ID plus grand que key qui pourront remplire la condition?
        match self {
            ConditionNode::Single(key_condition) => {
                Ok(match key_condition.condition_operator {
                    Operator::Equal				    => *key <  key_condition.condition_key,
                    Operator::NotEqual				=> true,
                    Operator::GreaterThan			=> true,
                    Operator::GreaterThanOrEquals	=> true,
                    Operator::LowerThan			    => *key <  key_condition.condition_key,
                    Operator::LowerThanOrEquals	    => *key < key_condition.condition_key,
                    Operator::StartWith			    => {
                        // TODO IMPLEMENT
                        *key == key_condition.condition_key
                    },
                    Operator::Like				    => true,
                    Operator::Regex				    => true,
                })
            }
            ConditionNode::And(left, right) => {
                Ok(     left.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 1)?
                    && right.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 2)?)
            }
            ConditionNode::Or(left, right) => {
                Ok(     left.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 3)?
                    || right.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 4)?)
            }
        }
    }
}

enum TokenType {
    Unknown,
    Column,         //TableAlias.ColumnName
    ChildOr,        //
    ChildAnd,       //
    ChildNot,       // !
    Value,          // "TOTO", 6, 6.3, 6+9, true, false, null ect...
    Operator,       // =, >, <, !=, >=, <=, LIKE, START_WITH, REGEX
    OpenBracket,    // (
    CloseBracket,   // )
}
struct Token {
    t_type : TokenType,
    t_type_can_be : Vec<TokenType>,
    t_type_must_be : Vec<TokenType>,
    t_str : String,
}
impl Token{
    fn new()->Re<Token>{
        Ok(Token{
            t_type: TokenType::Unknown,
            t_type_can_be: vec![TokenType::Column, TokenType::Value, TokenType::ChildOr, TokenType::ChildAnd, TokenType::ChildNot, TokenType::Operator, TokenType::OpenBracket, TokenType::CloseBracket],
            t_type_must_be: vec![],
            t_str: "".to_string(),
        })
    }
    fn add_must_be_type(&self, must_be_types : Vec<TokenType>)-> Re<()>{
        self.t_type_must_be.push(&mut must_be_types);
        Ok(())
    }
    fn remove_can_be_type(&self, cannot_be_types : Vec<TokenType>)-> Re<()>{
        self.t_type_can_be.remove(&mut cannot_be_types);
        Ok(())
    }
    fn fill_or_create_next(&self,c : char)->Re<Token>{



        Ok(Token::new(c).d_err_map(Fe::FillOrCreateNext,1)?)

    }
    fn charCanBeValue(c : char) -> bool {
        c.is_digit(10) || c == '-' || c == '+'
    }
    fn charCanBeColumn(c : char) -> bool {
        c.is_alphabetic() || c == '_'
    }
    fn isColumn(token_str : &str,c : char) -> bool {
        c.is_alphabetic() || c == '_'
    }
    fn isValue(c : char) -> bool {
        c.is_digit(10) || c == '-' || c == '+'
    }
    fn isOperator(c : char) -> bool {
        c == '=' || c == '>' || c == '<' || c == '!'
    }
    fn isBracket(c : char) -> bool {}
}

fn parse_condition_string(condition_string: &str) -> Re<ConditionNode> {
    let mut chars = condition_string.chars().peekable();
    let mut token_str : String;
    let mut token : Token = Token::Unknown;
    let mut lastToken : Token;

    //TODO ..... some work :)
    while let Some(&c) = chars.peek() {
        match token{
            Token::Unknown => {
                if c.is_whitespace() {
                    if !token_str.is_empty() {
                        return Err(DErr::RootError(Fe::ParseConditionString, 1, Rc::UnexpectedSpaceOnUnknownToken));
                    }
                    //chars.next(); // going to next char
                } else if c == '(' {
                    token = Token::OpenBracket;
                    chars.next();
                } else if c == ')' {
                    token = Token::CloseBracket;
                    chars.next();
                } else if c == '&' {}
            },
            Token::OpenBracket => {
                lastToken = token;
                token = Token::Unknown;
                if !(canBeValue(c) || canBeColumn(c)){
                    return Err(DErr::RootError(Fe::ParseConditionString, 1, Rc::UnexpectedCharAfterBracketToken(c)));
                }
                token_str = cd;
                if c == '(' || c == ')' {
                    return Err(DErr::RootError(Fe::ParseConditionString, 1, Rc::UnexpectedBracketOnBracketToken));
                }
            }

        };
        Err(DErr::RootError(Fe::ParseConditionString, 1, Rc::UnexpectedCharAfterBracketToken(c)))
    }
}
fn lex(input: &str) -> Re<Vec<String>> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' => {
                chars.next();
            }
            '(' | ')' | '&' | '|' => {
                let token = chars.next().unwrap().to_string();
                if token == "&" {
                    let next_char = chars.next();
                    if next_char != Some('&') {
                        return Err(DErr::RootError(Fe::Lex, 1, Rc::InvalidToken(token)));
                    }
                    tokens.push("and".to_string());
                } else if token == "|" {
                    let next_char = chars.next();
                    if next_char != Some('|') {
                        return Err(DErr::RootError(Fe::Lex, 2, Rc::InvalidToken(token)));
                    }
                    tokens.push("or".to_string());
                } else {
                    tokens.push(token);
                }
            }
            _ => {
                let mut token = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || c == '(' || c == ')' || c == '&' || c == '|' {
                        break;
                    }
                    token.push(chars.next().unwrap());
                }
                if !token.is_empty() {
                    tokens.push(token);
                }
            }
        }
    }

    Ok(tokens)
}

fn parse_expression(tokens: &mut Vec<String>) -> Re<ConditionNode> {
    parse_or(tokens).d_err_map(Fe::ParseExpression, 1)
}

fn parse_or(tokens: &mut Vec<String>) -> Re<ConditionNode> {
    let mut left = parse_and(tokens)?;

    while let Some(token) = tokens.get(0) {
        if token == "or" {
            tokens.remove(0);
            let right = parse_and(tokens)?;
            left = ConditionNode::Or(Box::new(left), Box::new(right));
        } else {
            break;
        }
    }

    Ok(left)
}

fn parse_and(tokens: &mut Vec<String>) -> Re<ConditionNode> {
    let mut left = parse_atom(tokens)?;

    while let Some(token) = tokens.get(0) {
        if token == "and" {
            tokens.remove(0);
            let right = parse_atom(tokens).d_err_map(Fe::ParseAnd, 1)?;
            left = ConditionNode::And(Box::new(left), Box::new(right));
        } else {
            break;
        }
    }

    Ok(left)
}

fn parse_atom(tokens: &mut Vec<String>) -> Re<ConditionNode> {
    if tokens.is_empty() {
        return Err(DErr::RootError(Fe::ParseAtom, 1, Rc::UnexpectedEndOfInput));
    }

    let token = tokens.remove(0);

    if token == "(" {
        let expr = parse_or(tokens).d_err_map(Fe::ParseAtom, 1)?;
        if tokens.is_empty() || tokens.remove(0) != ")" {
            return Err(DErr::RootError(Fe::ParseAtom, 1, Rc::MismatchedParentheses));
        }
        Ok(expr)
    } else {
        parse_condition(token, tokens)
    }
}

fn parse_condition(id_search: String, tokens: &mut Vec<String>) -> Re<ConditionNode> {
    if tokens.is_empty() {
        return Err(DErr::RootError(Fe::ParseCondition, 1, Rc::UnexpectedEndOfInput));
    }

    let op_token = tokens.remove(0);
    let operator = match op_token.as_str() {
        "="  => Operator::Equal,
        "!=" => Operator::NotEqual,
        ">"  => Operator::GreaterThan,
        ">=" => Operator::GreaterThanOrEquals,
        "<"  => Operator::LowerThan,
        "<=" => Operator::LowerThanOrEquals,
        "~"  => Operator::Like,
        ":"  => Operator::Regex,
        _ => return Err(DErr::RootError(Fe::ParseCondition, 1, Rc::UnexpectedEndOfInput)),
    };
    if tokens.is_empty() {
        return Err(DErr::RootError(Fe::ParseCondition, 2, Rc::UnexpectedEndOfInput));
    }

    let value_token = tokens.remove(0);
    //TODO check result
    let value = value_token.parse::<i32>().map_err(|_|DErr::RootError(Fe::ParseCondition, 2, Rc::InvalidValue(value_token)))?;

    Ok(ConditionNode::Single(KeyCondition { //TODO TOCHECK
        condition_key: Key { col2: value },
        condition_operator: operator,
    }))
}

pub fn build_expression_tree(input: &str) -> Re<ConditionNode> {
    let mut tokens = lex(input)?;
    parse_expression(&mut tokens)
}

// Implementation of conversion from d_buf errors to condition errors
impl ConvertDErr<Fe, Rc> for DErr<d_buf::Fe, d_buf::Rc> {
    fn convert_d_err(self, target_fn: Fe, index: i8) -> DErr<Fe, Rc> {
        // Convert the entire error chain from d_buf to condition
        let error_msg = self.to_string_chain();
        DErr::RootError(target_fn, index, Rc::DBufError(error_msg))
    }
}

#[cfg(test)]
mod tests_new_condition {
    use crate::draft::new_condition::*;

    #[test]
    fn test_logic_to_string() {
        assert_eq!(Logic::And.to_string(),"And");
    }

    #[test]
    fn test_logic_parse_str_basic_and() {
        let s: Vec<char> = "toto And titi".chars().collect::<Vec<char>>();
        let expected_left :  Vec<char> = "toto".chars().collect();
        let expected_right :  Vec<char> =  " titi".chars().collect();

        match Logic::split_str_on_first_self(s.as_slice()) {
            Some((logic,left,right)) => {
                assert_eq!(left,expected_left);
                assert_eq!(right,expected_right);
                assert_eq!(logic.to_string(),Logic::And.to_string());
            }
            _ => assert!(false)
        }
    }
    #[test]
    fn test_logic_parse_str_complexe_or() {
        let s: Vec<char> = "(toto = 1 And titi != 2) Or(tata = \"test\" And a.b ~= \"hihi\") And c".chars().collect::<Vec<char>>();
        let expected_left :  Vec<char> = "(toto = 1 And titi != 2)".chars().collect();
        let expected_right :  Vec<char> =  "(tata = \"test\" And a.b ~= \"hihi\") And c".chars().collect();

        match Logic::split_str_on_first_self(s.as_slice()) {
            Some((logic,left,right)) => {
                assert_eq!(left,expected_left);
                assert_eq!(right,expected_right);
                assert_eq!(logic.to_string(),Logic::Or.to_string());
            }
            _ => assert!(false)
        }
    }
    #[test]
    fn test_logic_parse_str_complexe_not_or() {
        let s: Vec<char> = "Not(a.b)Or tata = \"test\" And Not (a.b ~= \"hihi\")".chars().collect::<Vec<char>>();
        let expected_left :  Vec<char> = "Not(a.b)".chars().collect();
        let expected_right :  Vec<char> =  " tata = \"test\" And Not (a.b ~= \"hihi\")".chars().collect();

        match Logic::split_str_on_first_self(s.as_slice()) {
            Some((logic,left,right)) => {
                assert_eq!(left,expected_left);
                assert_eq!(right,expected_right);
                assert_eq!(logic.to_string(),Logic::Or.to_string());
            }
            _ => assert!(false)
        }
    }


    #[test]
    fn test_logic_parse_str_no_logic() {
        let s: Vec<char> = "Not (a.b)".chars().collect::<Vec<char>>();
        let expected_left :  Vec<char> = vec![];
        let expected_right :  Vec<char> =  " (a.b)".chars().collect();

        match Logic::split_str_on_first_self(s.as_slice()) {
            Some((logic,left,right)) => {
                assert_eq!(logic.to_string(),"should be None");
            }
            _ => assert!(true)
        }
    }

    #[test]
    fn test_operator_to_string() {
        assert_eq!(Operator::Equal.to_string(),"==");
    }
    #[test]
    fn test_operator_parse_str_equal_basic() {
        let s: Vec<char> = " (a == b )  ".chars().collect::<Vec<char>>();
        let expected_left :  Vec<char> = "a".chars().collect();
        let expected_right :  Vec<char> =  " b".chars().collect();

        match Operator::split_str_on_first_self(s.as_slice()) {
            Some((logic,left,right)) => {
                assert_eq!(left,expected_left);
                assert_eq!(right,expected_right);
                assert_eq!(logic.to_string(),Operator::Equal.to_string());
            }
            _ => assert!(false)
        }
    }
    #[test]
    fn test_operator_parse_str_equal_no_space() {
        let s: Vec<char> = " (a ==b )  ".chars().collect::<Vec<char>>();
        let expected_left :  Vec<char> = "a".chars().collect();
        let expected_right :  Vec<char> =  "b".chars().collect();

        match Operator::split_str_on_first_self(s.as_slice()) {
            Some((logic,left,right)) => {
                assert_eq!(left,expected_left);
                assert_eq!(right,expected_right);
                assert_eq!(logic.to_string(),Operator::Equal.to_string());
            }
            _ => assert!(false)
        }
    }
}

//TODO when check and delete
// : if a AND cdt is false : delete the AND cdt
//   if a Or cdt is false : replace the OR cdt by the cdt that is not false
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_0001_compare_rows_same_length() {
        let row1 = vec![2u8, 2u8, ];
        let row2 = vec![2u8, 2u8, ];
        let row3 = vec![2u8, 3u8, ];
        assert_eq!(row1, row2);
        assert_ne!(row1, row3);
    }
    #[test]
    fn test_0002_compare_gl_rows_same_length() {
        let row1 = vec![2u8, 2u8,];
        let row2 = vec![2u8, 2u8,];
        let row3 = vec![1u8, 3u8,];
        assert_eq!(row1>row2,false);
        assert_eq!(row1<row2,false);
        assert_eq!(row1>row3,true);
    }

    #[test]
    fn test_0003_compare_eq_rows_diff_length() {
        let row1 = vec![4u8, 3u8,];
        let row2 = vec![4u8, 3u8, 0u8,];
        let row3 = vec![0u8, 4u8, 3u8,];

        assert_ne!(row1, row2);
        assert_ne!(row1, row3);
    }
    #[test]
    fn test_0004_compare_gl_rows_diff_length() {
        let row1 = vec![4u8, 3u8,];
        let row2 = vec![4u8, 3u8, 0u8,];
        let row3 = vec![0u8, 4u8, 3u8,];
        let row4 = vec![4u8, 3u8, 3u8,];

        assert_eq!(row1>row2,false);
        assert_eq!(row1<row2,true);
        assert_eq!(row1>=row2,false);
        assert_eq!(row1<=row2,true);

        assert_eq!(row1>row3,true);
        assert_eq!(row1<row4,true);

    }
    #[test]
    fn test_success_build() {
        let input = "(id1 = 10 and id2 < 20) or ((id3 <= 30 and id4 != 40) or id5 > 50)";
        match build_expression_tree(input) {

            Ok(tree) => {
                println!("Expression tree built successfully: {:?}", tree);
               /* let result = tree.evaluate("id1", 10);
                println!("Evaluation result: {}", result);*/
            }
            Err(e) => println!("Error: {}", e.to_string_chain()),
        }

        assert!(false);
    }

    #[test]
    fn test_doc_condition_node() {
        /* let key = Key{col2:5};
         let key_condition = KeyCondition{key, condition_operator:Operator::Equal};
         let condition1 = ConditionNode::Single(key_condition);
          */
        let condition = ConditionNode::And(
            Box::new(ConditionNode::Single(KeyCondition { condition_key: Key { col2: 5 }, condition_operator: Operator::Equal })),
            Box::new(ConditionNode::Or(
                Box::new(ConditionNode::Single(KeyCondition { condition_key: Key { col2: 5 }, condition_operator: Operator::Equal })),
                Box::new(ConditionNode::Single(KeyCondition { condition_key: Key { col2: 5 }, condition_operator: Operator::Equal }))
            )),
        );
        println!("{:?}", condition);
    }
}
*/
