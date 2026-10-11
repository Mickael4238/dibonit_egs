use d_db_l; // Importe votre bibliothèque comme le ferait un utilisateur externe

#[test]
fn test_flux_complet() {
    let resultat = d_db_l::whoami();
    assert_eq!(resultat, "d_db_lib");
}