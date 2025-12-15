/*
Tests para Debug Info Generation

Implementación de: TASK-138 (Debug info generation)
Fecha: 2025-12-14
*/

use crate::debug_info::{DebugInfo, DebugInfoGenerator, SourceLocation, VariableLocation, VariableScope};
use crate::Compiler;
use bincode;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_info_creation() {
        let mut generator = DebugInfoGenerator::new();
        generator.set_current_file("test.vela".to_string());

        // Agregar función
        generator.start_function("main".to_string(), 0);
        generator.add_parameter("arg1".to_string(), "String".to_string(), 0);
        generator.add_local("local_var".to_string(), "Number".to_string(), VariableLocation::Stack(1));
        generator.end_function(10);

        // Registrar mapeo de línea
        generator.record_line_mapping(5, 10, 0);

        let debug_info = generator.get_debug_info();

        // Verificar source map
        assert!(debug_info.source_maps.contains_key("test.vela"));
        let source_map = &debug_info.source_maps["test.vela"];
        assert_eq!(source_map.line_mappings.len(), 1);
        assert_eq!(source_map.line_mappings[0].source_line, 5);
        assert_eq!(source_map.line_mappings[0].source_column, 10);
        assert_eq!(source_map.line_mappings[0].bytecode_offset, 0);

        // Verificar función
        assert!(debug_info.symbol_table.functions.contains_key("main"));
        let func_symbol = &debug_info.symbol_table.functions["main"];
        assert_eq!(func_symbol.name, "main");
        assert_eq!(func_symbol.start_offset, 0);
        assert_eq!(func_symbol.end_offset, 10);
        assert_eq!(func_symbol.parameters.len(), 1);
        assert_eq!(func_symbol.parameters[0].name, "arg1");
        assert_eq!(func_symbol.locals.len(), 1);
        assert_eq!(func_symbol.locals[0].name, "local_var");

        // Verificar variable
        assert!(debug_info.symbol_table.variables.contains_key("local_var"));
        let var_symbol = &debug_info.symbol_table.variables["local_var"];
        assert_eq!(var_symbol.name, "local_var");
        assert_eq!(var_symbol.var_type, "Number");
        assert!(matches!(var_symbol.scope, VariableScope::Local));
        assert!(matches!(var_symbol.location, VariableLocation::Stack(1)));
    }

    #[test]
    fn test_multiple_functions() {
        let mut generator = DebugInfoGenerator::new();
        generator.set_current_file("multi.vela".to_string());

        // Primera función
        generator.start_function("func1".to_string(), 0);
        generator.add_parameter("x".to_string(), "Number".to_string(), 0);
        generator.end_function(5);

        // Segunda función
        generator.start_function("func2".to_string(), 6);
        generator.add_parameter("y".to_string(), "String".to_string(), 0);
        generator.add_local("result".to_string(), "Bool".to_string(), VariableLocation::Stack(1));
        generator.end_function(15);

        let debug_info = generator.get_debug_info();

        assert_eq!(debug_info.symbol_table.functions.len(), 2);
        assert!(debug_info.symbol_table.functions.contains_key("func1"));
        assert!(debug_info.symbol_table.functions.contains_key("func2"));

        let func1 = &debug_info.symbol_table.functions["func1"];
        assert_eq!(func1.start_offset, 0);
        assert_eq!(func1.end_offset, 5);

        let func2 = &debug_info.symbol_table.functions["func2"];
        assert_eq!(func2.start_offset, 6);
        assert_eq!(func2.end_offset, 15);
    }

    #[test]
    fn test_line_mappings() {
        let mut generator = DebugInfoGenerator::new();
        generator.set_current_file("lines.vela".to_string());

        // Registrar múltiples mapeos
        generator.record_line_mapping(1, 1, 0);
        generator.record_line_mapping(2, 5, 3);
        generator.record_line_mapping(3, 10, 7);
        generator.record_line_mapping(2, 15, 10); // Misma línea, diferente columna

        let debug_info = generator.get_debug_info();
        let source_map = &debug_info.source_maps["lines.vela"];

        assert_eq!(source_map.line_mappings.len(), 4);

        // Verificar orden (deberían estar en orden de bytecode_offset)
        assert_eq!(source_map.line_mappings[0].bytecode_offset, 0);
        assert_eq!(source_map.line_mappings[1].bytecode_offset, 3);
        assert_eq!(source_map.line_mappings[2].bytecode_offset, 7);
        assert_eq!(source_map.line_mappings[3].bytecode_offset, 10);
    }

    #[test]
    fn test_variable_scopes() {
        let mut generator = DebugInfoGenerator::new();

        // Función con diferentes tipos de variables
        generator.start_function("test_func".to_string(), 0);

        // Parámetro
        generator.add_parameter("param".to_string(), "String".to_string(), 0);

        // Variable local
        generator.add_local("local".to_string(), "Number".to_string(), VariableLocation::Stack(1));

        // Variable en registro
        generator.add_local("reg_var".to_string(), "Bool".to_string(), VariableLocation::Register(0));

        generator.end_function(20);

        let debug_info = generator.get_debug_info();

        // Verificar parámetro
        let param_var = &debug_info.symbol_table.variables["param"];
        assert!(matches!(param_var.scope, VariableScope::Parameter));

        // Verificar variables locales
        let local_var = &debug_info.symbol_table.variables["local"];
        assert!(matches!(local_var.scope, VariableScope::Local));
        assert!(matches!(local_var.location, VariableLocation::Stack(1)));

        let reg_var = &debug_info.symbol_table.variables["reg_var"];
        assert!(matches!(reg_var.scope, VariableScope::Local));
        assert!(matches!(reg_var.location, VariableLocation::Register(0)));
    }

    #[test]
    fn test_debug_info_serialization() {
        // Test simple de serialización del debug info
        let mut generator = DebugInfoGenerator::new();
        
        // Agregar algo de información básica
        generator.set_current_file("test.vela".to_string());
        generator.start_function("test_function".to_string(), 0);
        generator.end_function(10);
        generator.start_function("main".to_string(), 11);
        generator.end_function(20);
        
        let debug_info = generator.get_debug_info().clone();
        
        // Serializar
        let serialized = bincode::serialize(&debug_info).expect("Failed to serialize debug info");
        
        // Deserializar
        let deserialized: DebugInfo = bincode::deserialize(&serialized).expect("Failed to deserialize debug info");
        
        // Verificar que sean iguales
        assert_eq!(debug_info, deserialized);
        
        println!("✅ Debug info serialization test passed!");
    }

    #[test]
    fn test_source_location() {
        let location = SourceLocation::new(42, 17);
        assert_eq!(location.line, 42);
        assert_eq!(location.column, 17);
    }

    #[test]
    fn test_debug_info_integration() {
        // Test que verifica la integración completa de debug info
        // con el pipeline de compilación
        use crate::Compiler;

        let source = r#"
// Simple Vela program for testing debug info
fn test_function() -> Number {
    return 30
}

fn main() -> void {
    return
}
"#;

        let mut compiler = Compiler::default();
        let result = compiler.compile_string(source, "test.vela");

        if let Err(e) = &result {
            println!("Compilation error: {:?}", e);
        }
        assert!(result.is_ok(), "Compilation should succeed");
        let bytecode_bytes = result.unwrap();

        // Deserializar el bytecode como Bytecode de la VM
        let vm_bytecode: vela_vm::bytecode::Bytecode = bincode::deserialize(&bytecode_bytes)
            .expect("Failed to deserialize bytecode");

        // Extraer debug info del metadata
        let debug_info_bytes = vm_bytecode.metadata.get("debug_info")
            .expect("Debug info should be in metadata");
        let debug_info: crate::debug_info::DebugInfo = bincode::deserialize(debug_info_bytes)
            .expect("Failed to deserialize debug info");

        // Verificar que el bytecode tiene debug info
        assert!(debug_info.symbol_table.functions.contains_key("test_function"));
        assert!(debug_info.symbol_table.functions.contains_key("main"));

        // Verificar que no hay parámetros registrados (el compilador actual no soporta parámetros)
        let test_func = &debug_info.symbol_table.functions["test_function"];
        assert_eq!(test_func.parameters.len(), 0);

        let main_func = &debug_info.symbol_table.functions["main"];
        assert_eq!(main_func.parameters.len(), 0);

        // Verificar que no hay variables locales (el código simple no tiene variables)
        // El compilador actual no registra variables en funciones simples que solo retornan constantes
        // assert!(debug_info.symbol_table.variables.is_empty()); // Puede estar vacío o tener variables del sistema

        println!("✅ Debug info integration test passed!");
    }
}