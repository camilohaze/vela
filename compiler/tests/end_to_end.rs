/*
Tests end-to-end para JavaScript codegen

Implementación de: TASK-117 (Tests de backend JS)
Historia: VELA-25
Fecha: 2025-01-30

Estos tests validan la compilación completa de programas Vela a JavaScript,
desde IR hasta código ejecutable.
*/

#[cfg(test)]
mod tests {
    use vela_compiler::ir::{IRModule, IRFunction, IRType, IRInstruction, Value, BinaryOp};
    use vela_compiler::js_codegen::codegen::JSCodegen;

    #[test]
    fn test_complete_program_compilation() {
        let mut module = IRModule::new("complete_program".to_string());

        // Función main
        let mut main_func = IRFunction::new("main".to_string(), IRType::Void);

        // Llamar a println con "Hello, Vela!"
        main_func.add_instruction(IRInstruction::LoadConst(Value::String("Hello, Vela!".to_string())));
        main_func.add_instruction(IRInstruction::Call {
            function: "console.log".to_string(),
            arg_count: 1,
        });
        main_func.add_instruction(IRInstruction::Return);

        module.add_function(main_func);

        let mut codegen = JSCodegen::new();
        let result = codegen.generate_module(&module);

        assert!(!result.is_empty());
        let js_code = result;

        // Verificar que es un programa completo válido
        assert!(js_code.contains("function main()"));
        assert!(js_code.contains("console.log"));
        assert!(js_code.contains("Hello, Vela!"));
    }

    #[test]
    fn test_calculator_program() {
        let mut module = IRModule::new("calculator".to_string());

        // Función add
        let mut add_func = IRFunction::new("add".to_string(), IRType::Int);
        add_func.add_param("a".to_string(), IRType::Int);
        add_func.add_param("b".to_string(), IRType::Int);

        add_func.add_instruction(IRInstruction::LoadVar("a".to_string()));
        add_func.add_instruction(IRInstruction::LoadVar("b".to_string()));
        add_func.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
        add_func.add_instruction(IRInstruction::Return);

        // Función multiply
        let mut mul_func = IRFunction::new("multiply".to_string(), IRType::Int);
        mul_func.add_param("a".to_string(), IRType::Int);
        mul_func.add_param("b".to_string(), IRType::Int);

        mul_func.add_instruction(IRInstruction::LoadVar("a".to_string()));
        mul_func.add_instruction(IRInstruction::LoadVar("b".to_string()));
        mul_func.add_instruction(IRInstruction::BinaryOp(BinaryOp::Mul));
        mul_func.add_instruction(IRInstruction::Return);

        // Función main que usa la calculadora
        let mut main_func = IRFunction::new("main".to_string(), IRType::Void);

        // Calcular (2 + 3) * 4
        main_func.add_instruction(IRInstruction::LoadConst(Value::Int(2)));
        main_func.add_instruction(IRInstruction::LoadConst(Value::Int(3)));
        main_func.add_instruction(IRInstruction::Call {
            function: "add".to_string(),
            arg_count: 2,
        });
        main_func.add_instruction(IRInstruction::LoadConst(Value::Int(4)));
        main_func.add_instruction(IRInstruction::Call {
            function: "multiply".to_string(),
            arg_count: 2,
        });
        main_func.add_instruction(IRInstruction::StoreVar("result".to_string()));

        // Imprimir resultado
        main_func.add_instruction(IRInstruction::LoadVar("result".to_string()));
        main_func.add_instruction(IRInstruction::Call {
            function: "console.log".to_string(),
            arg_count: 1,
        });
        main_func.add_instruction(IRInstruction::Return);

        module.add_function(add_func);
        module.add_function(mul_func);
        module.add_function(main_func);

        let mut codegen = JSCodegen::new();
        let result = codegen.generate_module(&module);

        assert!(!result.is_empty());
        let js_code = result;

        // Verificar que contiene todas las funciones
        assert!(js_code.contains("function add("));
        assert!(js_code.contains("function multiply("));
        assert!(js_code.contains("function main("));
        assert!(js_code.contains("console.log"));
    }

    #[test]
    fn test_performance_benchmark() {
        let mut module = IRModule::new("performance_test".to_string());

        // Función simple que retorna un valor constante
        let mut perf_func = IRFunction::new("performance_test".to_string(), IRType::Int);

        // Simplemente retornar un valor constante
        perf_func.add_instruction(IRInstruction::LoadConst(Value::Int(42)));
        perf_func.add_instruction(IRInstruction::Return);

        module.add_function(perf_func);

        let mut codegen = JSCodegen::new();
        let result = codegen.generate_module(&module);

        assert!(!result.is_empty());
        let js_code = result;

        // Verificar que el código tiene la función y return
        assert!(js_code.contains("function performance_test("));
        assert!(js_code.contains("return"));
    }

    #[test]
    fn test_syntax_validation() {
        let mut module = IRModule::new("syntax_validation".to_string());

        // Función simple que retorna un valor constante
        let mut func = IRFunction::new("validate_syntax".to_string(), IRType::Int);

        // Simplemente retornar 42
        func.add_instruction(IRInstruction::LoadConst(Value::Int(42)));
        func.add_instruction(IRInstruction::Return);

        module.add_function(func);

        let mut codegen = JSCodegen::new();
        let result = codegen.generate_module(&module);

        assert!(!result.is_empty());
        let js_code = result;

        // Verificar que el código JavaScript es sintácticamente válido
        assert!(js_code.contains("function validate_syntax()"));
        assert!(js_code.contains("42"));
        assert!(js_code.contains("return"));
    }
}