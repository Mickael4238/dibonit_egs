#[derive(Debug)]

/// DibonitError:Can be either a
///  - RootError (fonctionSource, indexError,RootCause) ==> when the first error is raised
///  - PassError (fonctionTransit, indexError,Box<DErr>)  ==> to make the error transit to upper level
pub enum DErr<FnErr,RootCause>{
    RootError(FnErr, i8,RootCause),
    PassError(FnErr, i8,Box<DErr<FnErr,RootCause>>)
}

impl <FnErr: std::fmt::Debug, RootCause: std::fmt::Debug> DErr<FnErr,RootCause> {
    /// Make a full path error String
    ///
    /// # Returns
    ///
    /// 'String' fn1[id1]/fn2[id2]/[...]/RootCause
    /// TODO : resolving recursive calls
    pub fn to_string_chain(&self) -> String {
        match self {
            DErr::RootError(f, id, e) => format!("{:?}[{}]=>{:?}", f, id, e),
            DErr::PassError(f, id, cause) => {
                format!("{:?}[{}]/{}", f,id, cause.to_string_chain())
            }
        }
    }
    pub fn to_string(&self) -> String {
        self.to_string_chain()
    }
}

/// Trait to convert between different DErr systems
/// Implement this to allow conversion from one library's error system to another
pub trait ConvertDErr<TargetFnErr, TargetRootCause> {
    fn convert_d_err(self, target_fn: TargetFnErr, index: i8) -> DErr<TargetFnErr, TargetRootCause>;
}

pub trait DErrMap<T, FnErr, RootCause> {
    fn d_err_map(self, fn_err: FnErr, code: i8) ->  Result<T, DErr<FnErr, RootCause>>;
}

impl<T, E,FnErr, RootCause> DErrMap<T,FnErr, RootCause> for Result<T, E>
where
    E: Into<DErr<FnErr, RootCause>>,
{
    /// Map a DErr by adding "FnErr(index)/" to its error path
    fn d_err_map(self, fn_err: FnErr, index: i8) -> Result<T, DErr<FnErr, RootCause>>
    {
        self.map_err(|e| DErr::PassError(fn_err, index, Box::new(e.into())))
    }
}

/// Extended version of DErrMap for cross-library error conversion
pub trait DErrMapConvert<T, TargetFnErr, TargetRootCause> {
    fn d_err_map_convert(self, target_fn: TargetFnErr, index: i8) -> Result<T, DErr<TargetFnErr, TargetRootCause>>;
}

impl<T, SourceFnErr, SourceRootCause, TargetFnErr, TargetRootCause> DErrMapConvert<T, TargetFnErr, TargetRootCause>
    for Result<T, DErr<SourceFnErr, SourceRootCause>>
where
    DErr<SourceFnErr, SourceRootCause>: ConvertDErr<TargetFnErr, TargetRootCause>,
{
    fn d_err_map_convert(self, target_fn: TargetFnErr, index: i8) -> Result<T, DErr<TargetFnErr, TargetRootCause>> {
        self.map_err(|e| e.convert_d_err(target_fn, index))
    }
}

#[cfg(test)]
mod b_libA{

    use super::DErr;
    type Re<T> = Result<T,DErr<Fe, Rc>>;
    #[derive(Debug)]
    pub enum Rc {
        Error1,
    }
    #[derive(Debug)]
    pub enum Fe {
        FunctionA,
    }
    pub fn function_a() -> Re<bool>{
        Err(DErr::RootError(Fe::FunctionA, 1, Rc::Error1))
    }

}
#[cfg(test)]
mod b_libB{
    use super::DErr;
    use super::b_libA::*;
    type Re<T> = Result<T,DErr<Fe, Rc>>;
    #[derive(Debug)]
    pub enum Rc {
        ErrorLibA(String),
    }
    #[derive(Debug)]
    pub enum Fe {
        FunctionB,
    }
    pub fn function_b() -> Re<bool>{
        let b = function_a().map_err(|e|  DErr::RootError(Fe::FunctionB, 1, Rc::ErrorLibA(e.to_string_chain())))?;
        Ok(b)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::b_libB::function_b;

    type Re<T> = Result<T, DErr<Fe, Rc>>;

    #[derive(Debug)]
    /// RootCause of the error
    enum Rc {
        Error1,
        EqualNumbers,
    }
    #[derive(Debug)]
    /// Function where the Error transit
    enum Fe {
        Af,
        Bf,
        Cf,
        Df,
        Ef,
    }

    fn a() -> Re<String> {
        Err(DErr::RootError(Fe::Af, 1, Rc::Error1))
    }

    fn b(x:i8, z:i8) -> Result<String, DErr<Fe, Rc>>  {
        if x==z {
            return Err(DErr::RootError(Fe::Bf, 1, Rc::EqualNumbers));
        }
        a().map_err(|e| DErr::PassError(Fe::Bf, 2, Box::new(e)))
    }

    fn c(x:i8, y:i8, z:i8) ->  Result<String, DErr<Fe, Rc>> {
        if x==y && x == z {
            return Err(DErr::RootError(Fe::Cf, 1, Rc::EqualNumbers));
        }
        if x==y {
            return Err(DErr::RootError(Fe::Cf, 2, Rc::EqualNumbers));
        }
        // call b, exit on error, continue on success
        b(x, z).map_err(|e| DErr::PassError(Fe::Cf, 3, Box::new(e)))?;

        // call a and exit
        a().map_err(|e| DErr::PassError(Fe::Cf, 1, Box::new(e)))
    }
    fn d(x:i8, y:i8, z:i8) ->  Result<String, DErr<Fe, Rc>> {
        if x==y && x == z {
            return Err(DErr::RootError(Fe::Df, 1, Rc::EqualNumbers));
        }
        if x==y {
            return Err(DErr::RootError(Fe::Df, 2, Rc::EqualNumbers));
        }
        // call b, exit on error (with adaptation), continue on success
        b(x, z).d_err_map(Fe::Df,3)?;

        // call a and exit
        a().d_err_map(Fe::Df,1)
    }

    fn e(x:i8) -> Result<i8, DErr<Fe, Rc>>  {
        if x == 1 {
            Err(DErr::RootError(Fe::Ef, 1, Rc::EqualNumbers))
        }else{
            Ok(0)
        }
    }
    fn return_example(x:i8) -> Re<String>  {
        if x == 19 {
            return Err(DErr::RootError(Fe::Ef, 1, Rc::EqualNumbers))
        }
        if e(x).d_err_map(Fe::Df, 3)? == 0 {
            Ok("you can read a 0 result !".to_string())
        } else {
            Ok("you can read another result ".to_string())
        }
    }

    #[test]
    fn test_c_error_1() {
        let res = c(1, 1, 1);
        assert!(res.is_err());

        if let Err(e) = res {
            assert_eq!(e.to_string_chain(), "Cf[1]=>EqualNumbers");
        }
    }
    #[test]
    fn test_c_error_2() {
        let res = c(1, 1, 2);
        assert!(res.is_err());

        if let Err(e) = res {
            assert_eq!(e.to_string_chain(), "Cf[2]=>EqualNumbers");
        }
    }

    #[test]
    fn test_c_error_with_cause_b() {
        let res = c(2, 1, 2);
        assert!(res.is_err());
        // Vérification de la chaîne d'erreur complète

        let e = res.unwrap_err();
        assert_eq!(e.to_string_chain(), "Cf[3]/Bf[1]=>EqualNumbers");
    }

    #[test]
    fn test_c_error_with_cause_b_cause_a() {
        let res = c(1, 2, 3);
        assert!(res.is_err());
        // Vérification de la chaîne d'erreur complète
        if let Err(e) = res {
            assert_eq!(e.to_string_chain(), "Cf[3]/Bf[2]/Af[1]=>Error1");
        }
    }
    #[test]
    fn test_d_error_with_trait_map_err() {
        let res = d(1, 2, 3);
        assert!(res.is_err());
        // Vérification de la chaîne d'erreur complète
        if let Err(e) = res {
            assert_eq!(e.to_string_chain(), "Df[3]/Bf[2]/Af[1]=>Error1");
        }
    }
    #[test]
    fn test_f_success() {
        let res = return_example(3);
        assert!(res.is_ok());
        assert_eq!(res.unwrap() , "you can read a 0 result !");

    }

    #[test]
    fn test_sub_library() {
        match function_b(){
            Ok(b) => assert!(false),
            Err(e) => {assert_eq!(e.to_string_chain(), "FunctionB[1]=>ErrorLibA(\"FunctionA[1]=>Error1\")"); }
        }
    }
/*
    #[test]
    fn test_c_error_with_cause_a() {
        let res = c(0,0,0);
        assert!(res.is_err());

        if let Err(e) = res {
            // Erreur principale CError
            match e {
                MyError::CError(main_err, cause) => {
                    assert_eq!(main_err, myErrors::Error2);

                    // Cause doit être AError(Error3)
                    match *cause {
                        MyError::AError(ref cause_err) => {
                            assert_eq!(*cause_err, myErrors::Error3);
                        }
                        _ => panic!("Cause should be AError"),
                    }

                    // Vérification de la chaîne d'erreur complète
                    assert_eq!(e.to_string_chain(), "c[Error2]/a[Error3]");
                }
                _ => panic!("Expected CError"),
            }
        }
    }

    #[test]
    fn test_success_path() {
        // Exemple simple sans erreur, on modifie a pour réussir
        fn A_success() -> Result<String, MyError<myErrors>> {
            Ok("ok".to_string())
        }

        fn B_success() -> Result<String, MyError<myErrors>> {
            match A_success() {
                Ok(data) => Ok(data),
                Err(e) => Err(MyError::BError(myErrors::Error1, Box::new(e))),
            }
        }

        let res = B_success();
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "ok".to_string());
    }*/
}

