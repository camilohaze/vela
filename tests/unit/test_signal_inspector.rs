/*
Tests unitarios para el signal inspector

Historia: VELA-142 (EPIC-14 DevTools & Debugging)
Tarea: TASK-142
Fecha: 2025-12-14

Tests para:
- execute_inspect_signals function
- Formatos de salida (text, json, graphviz)
- Manejo de errores
- Integración con VM y heap
*/

#[cfg(test)]
mod test_signal_inspector {
    use std::path::Path;
    use std::fs;
    use tempfile::NamedTempFile;
    use vela_tooling::cli::commands::execute_inspect_signals;
    use vela_vm::{VirtualMachine, gc::GcHeap};

    // Programa Vela simple para testing
    const SIMPLE_VELA_PROGRAM: &str = r#"
import 'stdlib:test'

fn main() -> void {
    result: Number = 42
    print("Test program")
}
"#;

    #[test]
    fn test_inspect_signals_text_format() {
        // Crear archivo temporal con programa Vela
        let mut temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        fs::write(&temp_file, SIMPLE_VELA_PROGRAM).unwrap();

        // Ejecutar inspección en formato texto
        let result = execute_inspect_signals(
            temp_file.path().to_str().unwrap(),
            "text"
        );

        assert!(result.is_ok(), "Signal inspection should succeed");
        let output = result.unwrap();

        // Verificar que contiene información esperada
        assert!(output.contains("Reactive Objects Found"), "Should contain header");
        assert!(output.contains("Signals: 0"), "Should show zero signals");
        assert!(output.contains("Computed values: 0"), "Should show zero computed values");
    }

    #[test]
    fn test_inspect_signals_json_format() {
        // Crear archivo temporal con programa Vela
        let mut temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        fs::write(&temp_file, SIMPLE_VELA_PROGRAM).unwrap();

        // Ejecutar inspección en formato JSON
        let result = execute_inspect_signals(
            temp_file.path().to_str().unwrap(),
            "json"
        );

        assert!(result.is_ok(), "Signal inspection should succeed");
        let output = result.unwrap();

        // Verificar que es JSON válido
        assert!(output.trim().starts_with('{'), "Should start with JSON object");
        assert!(output.contains("\"signals\""), "Should contain signals array");
        assert!(output.contains("\"computed\""), "Should contain computed array");
        assert!(output.contains("[]"), "Should have empty arrays for no reactive objects");

        // Verificar que es JSON válido parseando
        let _: serde_json::Value = serde_json::from_str(&output)
            .expect("Output should be valid JSON");
    }

    #[test]
    fn test_inspect_signals_graphviz_format() {
        // Crear archivo temporal con programa Vela
        let mut temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        fs::write(&temp_file, SIMPLE_VELA_PROGRAM).unwrap();

        // Ejecutar inspección en formato GraphViz
        let result = execute_inspect_signals(
            temp_file.path().to_str().unwrap(),
            "graphviz"
        );

        assert!(result.is_ok(), "Signal inspection should succeed");
        let output = result.unwrap();

        // Verificar que contiene elementos GraphViz
        assert!(output.contains("digraph ReactiveGraph"), "Should contain GraphViz header");
        assert!(output.contains("rankdir=LR"), "Should contain layout directive");
        assert!(output.contains("node [shape=box]"), "Should contain node styling");
    }

    #[test]
    fn test_inspect_signals_invalid_format() {
        // Crear archivo temporal con programa Vela
        let mut temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        fs::write(&temp_file, SIMPLE_VELA_PROGRAM).unwrap();

        // Ejecutar inspección con formato inválido
        let result = execute_inspect_signals(
            temp_file.path().to_str().unwrap(),
            "invalid_format"
        );

        assert!(result.is_err(), "Should fail with invalid format");
        let error = result.unwrap_err();
        assert!(error.to_string().contains("format"), "Error should mention format");
    }

    #[test]
    fn test_inspect_signals_nonexistent_file() {
        // Ejecutar inspección con archivo inexistente
        let result = execute_inspect_signals(
            "/nonexistent/file.vela",
            "text"
        );

        assert!(result.is_err(), "Should fail with nonexistent file");
    }

    #[test]
    fn test_inspect_signals_compilation_error() {
        // Crear archivo con código Vela inválido
        let invalid_program = "invalid vela code {{{";
        let mut temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        fs::write(&temp_file, invalid_program).unwrap();

        // Ejecutar inspección
        let result = execute_inspect_signals(
            temp_file.path().to_str().unwrap(),
            "text"
        );

        // Debería fallar por error de compilación
        assert!(result.is_err(), "Should fail with compilation error");
    }

    #[test]
    fn test_vm_heap_integration() {
        // Test de integración VM-heap
        let heap = GcHeap::new();
        let vm = VirtualMachine::new_with_heap(heap);

        // Verificar que VM tiene acceso al heap
        assert!(vm.get_reactive_objects().is_ok(), "VM should provide reactive objects access");

        let reactive_objects = vm.get_reactive_objects().unwrap();
        assert_eq!(reactive_objects.len(), 0, "Empty VM should have no reactive objects");
    }

    #[test]
    fn test_reactive_objects_info_structure() {
        // Test de la estructura ReactiveObjectsInfo
        use vela_tooling::cli::commands::ReactiveObjectsInfo;

        let info = ReactiveObjectsInfo {
            signals: vec![],
            computed: vec![],
        };

        // Verificar serialización JSON
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"signals\":[]"), "Should serialize signals array");
        assert!(json.contains("\"computed\":[]"), "Should serialize computed array");
    }
}