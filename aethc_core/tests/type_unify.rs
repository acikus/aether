use aethc_core::type_::{Type};

#[test]
fn unify_int_float() {
    let t = Type::unify(&Type::Int, &Type::Float).unwrap();
    assert_eq!(t, Type::Float);
}
