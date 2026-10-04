
//mod index;
pub mod condition;

pub fn  whoami() -> &'static str {
	return "d_db_lib";
}

#[cfg(test)] // Indique de ne compiler ce bloc que lors des tests
mod tests {
    use super::*; // Importe tout ce qui est au-dessus (le code à tester)

    #[test]
    fn test_whoami() {
        assert_eq!(whoami(), "d_db_lib");
    }
}