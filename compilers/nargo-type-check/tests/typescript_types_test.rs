use nargo_type_check::{Type, TypeChecker, TypeScriptStmtHandler};

#[test]
fn test_parse_conditional_type() {
    let checker = TypeChecker::new();
    let type_str = "T extends U ? X : Y";
    let ty = checker.parse_type(type_str);

    match ty {
        Type::Conditional(check_type, extends_type, true_type, false_type) => {
            assert_eq!(*check_type, Type::Interface("T".to_string()));
            assert_eq!(*extends_type, Type::Interface("U".to_string()));
            assert_eq!(*true_type, Type::Interface("X".to_string()));
            assert_eq!(*false_type, Type::Interface("Y".to_string()));
        }
        _ => panic!("Expected Conditional type, got {:?}", ty),
    }
}

#[test]
fn test_parse_mapped_type() {
    let checker = TypeChecker::new();
    let type_str = "{ [K in keyof T]: T[K] }";
    let ty = checker.parse_type(type_str);

    match ty {
        Type::Mapped(key_var, source_type, mapped_type) => {
            assert_eq!(key_var, "K");
            assert_eq!(*source_type, Type::KeyOf(Box::new(Type::Interface("T".to_string()))));
            match *mapped_type {
                Type::IndexAccess(obj_type, index_type) => {
                    assert_eq!(*obj_type, Type::Interface("T".to_string()));
                    assert_eq!(*index_type, Type::Interface("K".to_string()));
                }
                _ => panic!("Expected IndexAccess type, got {:?}", mapped_type),
            }
        }
        _ => panic!("Expected Mapped type, got {:?}", ty),
    }
}

#[test]
fn test_parse_keyof_type() {
    let checker = TypeChecker::new();
    let type_str = "keyof T";
    let ty = checker.parse_type(type_str);

    match ty {
        Type::KeyOf(target_type) => {
            assert_eq!(*target_type, Type::Interface("T".to_string()));
        }
        _ => panic!("Expected KeyOf type, got {:?}", ty),
    }
}

#[test]
fn test_parse_index_access_type() {
    let checker = TypeChecker::new();
    let type_str = "T[K]";
    let ty = checker.parse_type(type_str);

    match ty {
        Type::IndexAccess(obj_type, index_type) => {
            assert_eq!(*obj_type, Type::Interface("T".to_string()));
            assert_eq!(*index_type, Type::Interface("K".to_string()));
        }
        _ => panic!("Expected IndexAccess type, got {:?}", ty),
    }
}

#[test]
fn test_parse_literal_types() {
    let checker = TypeChecker::new();

    // 测试字符串字面量类型
    let str_literal = checker.parse_type("\"hello\"");
    match str_literal {
        Type::Literal(value) => assert_eq!(value, "hello"),
        _ => panic!("Expected Literal type, got {:?}", str_literal),
    }

    // 测试数字字面量类型
    let num_literal = checker.parse_type("42");
    match num_literal {
        Type::Literal(value) => assert_eq!(value, "42"),
        _ => panic!("Expected Literal type, got {:?}", num_literal),
    }

    // 测试布尔字面量类型
    let bool_literal = checker.parse_type("true");
    match bool_literal {
        Type::Literal(value) => assert_eq!(value, "true"),
        _ => panic!("Expected Literal type, got {:?}", bool_literal),
    }
}
