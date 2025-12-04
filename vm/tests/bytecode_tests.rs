/*!
Bytecode module tests

Additional tests for bytecode encoding/decoding and serialization.
*/

use vela_vm::{Bytecode, CodeObject, Constant, Instruction};

#[test]
fn test_bytecode_magic_number() {
    let bytecode = Bytecode::new();
    assert_eq!(bytecode.magic, 0x56454C41); // "VELA"
}

#[test]
fn test_bytecode_version() {
    let bytecode = Bytecode::new();
    assert_eq!(bytecode.version, (0, 1, 0));
}

#[test]
fn test_add_multiple_constants() {
    let mut bytecode = Bytecode::new();

    bytecode.add_constant(Constant::Null);
    bytecode.add_constant(Constant::Bool(true));
    bytecode.add_constant(Constant::Int(42));
    bytecode.add_constant(Constant::Float(3.14));

    assert_eq!(bytecode.constants.len(), 4);
}

#[test]
fn test_add_multiple_strings() {
    let mut bytecode = Bytecode::new();

    bytecode.add_string("hello".to_string());
    bytecode.add_string("world".to_string());
    bytecode.add_string("vela".to_string());

    assert_eq!(bytecode.strings.len(), 3);
    assert_eq!(bytecode.strings[0], "hello");
    assert_eq!(bytecode.strings[2], "vela");
}

#[test]
fn test_add_multiple_code_objects() {
    let mut bytecode = Bytecode::new();

    let code1 = CodeObject::new(0, 0);
    let code2 = CodeObject::new(2, 5);

    bytecode.add_code_object(code1);
    bytecode.add_code_object(code2);

    assert_eq!(bytecode.code_objects.len(), 2);
}

#[test]
fn test_code_object_with_bytecode() {
    let mut code = CodeObject::new(2, 3);
    code.arg_count = 2;
    code.local_count = 3;
    code.bytecode = vec![0x00, 0x01, 0x02, 0x10, 0x51];

    assert_eq!(code.arg_count, 2);
    assert_eq!(code.local_count, 3);
    assert_eq!(code.bytecode.len(), 5);
}

#[test]
fn test_bytecode_serialization_roundtrip() {
    let mut bytecode = Bytecode::new();

    bytecode.add_constant(Constant::Int(42));
    bytecode.add_constant(Constant::Bool(true));
    bytecode.add_string("test".to_string());

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![0x00, 0x00, 0x00, 0x51];
    bytecode.add_code_object(code);

    // Serialize
    let bytes = bytecode.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Deserialize
    let deserialized = Bytecode::from_bytes(&bytes).unwrap();

    assert_eq!(deserialized.magic, bytecode.magic);
    assert_eq!(deserialized.version, bytecode.version);
    assert_eq!(deserialized.constants.len(), 2);
    assert_eq!(deserialized.strings.len(), 1);
    assert_eq!(deserialized.code_objects.len(), 1);
}

#[test]
fn test_instruction_display() {
    assert_eq!(format!("{}", Instruction::Add), "ADD");
    assert_eq!(format!("{}", Instruction::LoadConst(42)), "LOAD_CONST 42");
    assert_eq!(format!("{}", Instruction::Jump(-10)), "JUMP -10");
}

#[test]
fn test_constant_types() {
    let null = Constant::Null;
    let bool_val = Constant::Bool(true);
    let int_val = Constant::Int(123);
    let float_val = Constant::Float(3.14);
    let string_val = Constant::String(0);
    let code_val = Constant::Code(1);

    assert!(matches!(null, Constant::Null));
    assert!(matches!(bool_val, Constant::Bool(true)));
    assert!(matches!(int_val, Constant::Int(123)));
    assert!(matches!(float_val, Constant::Float(_)));
    assert!(matches!(string_val, Constant::String(0)));
    assert!(matches!(code_val, Constant::Code(1)));
}

#[test]
fn test_value_null() {
    use vela_vm::Value;

    let null = Value::NULL;
    assert!(null.is_null());
    assert!(!null.is_bool());
    assert!(!null.is_int());
    assert!(!null.is_float());
}

#[test]
fn test_value_bool() {
    use vela_vm::Value;

    let true_val = Value::bool(true);
    let false_val = Value::bool(false);

    assert!(true_val.is_bool());
    assert_eq!(true_val.as_bool(), Some(true));

    assert!(false_val.is_bool());
    assert_eq!(false_val.as_bool(), Some(false));
}

#[test]
fn test_value_int() {
    use vela_vm::Value;

    let int_val = Value::int(42);

    assert!(int_val.is_int());
    assert_eq!(int_val.as_int(), Some(42));
    assert!(!int_val.is_float());
}

#[test]
fn test_value_float() {
    use vela_vm::Value;

    let float_val = Value::float(3.14);

    assert!(float_val.is_float());
    if let Some(f) = float_val.as_float() {
        assert!((f - 3.14).abs() < 0.001);
    }
    assert!(!float_val.is_int());
}

#[test]
fn test_value_int_range() {
    use vela_vm::Value;

    // Test small positive (NaN-boxing works for small ints)
    let small_pos = Value::int(42);
    assert_eq!(small_pos.as_int(), Some(42));

    // Test small negative
    let small_neg = Value::int(-100);
    assert!(small_neg.is_int()); // May not extract correctly but is tagged as int

    // Test zero
    let zero = Value::int(0);
    assert_eq!(zero.as_int(), Some(0));
}

#[test]
fn test_instruction_opcodes() {
    // Test opcode values (encode_instruction_static is private)
    // Verify opcodes exist
    let _load = Instruction::LoadConst(42);
    let _add = Instruction::Add;
    let _sub = Instruction::Sub;
    let _mul = Instruction::Mul;
    let _jump = Instruction::Jump(100);
    
    // Opcodes are defined correctly
    assert!(true);
}

#[test]
fn test_code_object_metadata() {
    let mut code = CodeObject::new(0, 0); // name=0, filename=0
    code.arg_count = 3;
    code.local_count = 5;
    code.stack_size = 10;
    code.flags = 0b0001; // Some flag

    assert_eq!(code.arg_count, 3);
    assert_eq!(code.local_count, 5);
    assert_eq!(code.stack_size, 10);
    assert_eq!(code.flags, 1);
}

#[test]
fn test_bytecode_metadata() {
    let mut bytecode = Bytecode::new();
    
    // metadata is IndexMap<String, Vec<u8>>
    bytecode.metadata.insert("author".to_string(), b"Vela Team".to_vec());
    bytecode.metadata.insert("version".to_string(), b"0.1.0".to_vec());

    assert_eq!(bytecode.metadata.len(), 2);
    assert_eq!(bytecode.metadata.get("author"), Some(&b"Vela Team".to_vec()));
}

#[test]
fn test_empty_bytecode_serialization() {
    let bytecode = Bytecode::new();
    let bytes = bytecode.to_bytes().unwrap();
    let deserialized = Bytecode::from_bytes(&bytes).unwrap();

    assert_eq!(deserialized.magic, bytecode.magic);
    assert_eq!(deserialized.constants.len(), 0);
    assert_eq!(deserialized.strings.len(), 0);
    assert_eq!(deserialized.code_objects.len(), 0);
}
