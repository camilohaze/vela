/*
Tests de integración para comandos CLI de debugging tools

Historia: VELA-142 (EPIC-14 DevTools & Debugging)
Tarea: TASK-142
Fecha: 2025-12-14

Tests para:
- Integración completa de comandos CLI
- Validación end-to-end de debugging tools
- Manejo de archivos de entrada/salida
*/

#[cfg(test)]
mod test_debugging_cli_integration {
    use std::fs;
    use std::process::Command;
    use tempfile::NamedTempFile;
    use vela_tooling::cli::commands::{execute_inspect_signals, InspectSignalsArgs};

    #[test]
    fn test_inspect_signals_cli_with_temp_file() {
        // Crear archivo temporal con programa Vela de prueba
        let temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        let vela_program = r#"
// Programa Vela simple para testing
state counter: Number = 0

computed doubled: Number {
  return counter * 2
}

effect {
  print("Counter: ${counter}, Doubled: ${doubled}")
}
"#;

        fs::write(&temp_file, vela_program).unwrap();

        // Ejecutar comando inspect-signals
        let args = InspectSignalsArgs {
            input_file: Some(temp_file.path().to_path_buf()),
            output_format: "text".to_string(),
            output_file: None,
        };

        let result = execute_inspect_signals(args);
        assert!(result.is_ok(), "CLI command should succeed with valid input");

        let output = result.unwrap();
        assert!(!output.is_empty(), "Output should not be empty");
        assert!(output.contains("counter"), "Output should contain signal names");
    }

    #[test]
    fn test_inspect_signals_cli_json_output() {
        let temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        let vela_program = r#"
state name: String = "Vela"
state count: Number = 42
"#;

        fs::write(&temp_file, vela_program).unwrap();

        let args = InspectSignalsArgs {
            input_file: Some(temp_file.path().to_path_buf()),
            output_format: "json".to_string(),
            output_file: None,
        };

        let result = execute_inspect_signals(args);
        assert!(result.is_ok(), "JSON output should succeed");

        let output = result.unwrap();
        // Verificar que es JSON válido
        assert!(output.trim_start().starts_with('{'), "Output should start with JSON object");
        assert!(output.contains("name"), "JSON should contain signal names");
        assert!(output.contains("count"), "JSON should contain signal names");
    }

    #[test]
    fn test_inspect_signals_cli_graphviz_output() {
        let temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        let vela_program = r#"
state a: Number = 1
computed b: Number { return a + 1 }
computed c: Number { return b * 2 }
"#;

        fs::write(&temp_file, vela_program).unwrap();

        let args = InspectSignalsArgs {
            input_file: Some(temp_file.path().to_path_buf()),
            output_format: "graphviz".to_string(),
            output_file: None,
        };

        let result = execute_inspect_signals(args);
        assert!(result.is_ok(), "Graphviz output should succeed");

        let output = result.unwrap();
        assert!(output.contains("digraph"), "Graphviz output should contain digraph");
        assert!(output.contains("->"), "Graphviz should contain edges");
    }

    #[test]
    fn test_inspect_signals_cli_output_to_file() {
        let input_file = NamedTempFile::with_suffix(".vela").unwrap();
        let output_file = NamedTempFile::with_suffix(".txt").unwrap();

        let vela_program = r#"state test: String = "hello""#;
        fs::write(&input_file, vela_program).unwrap();

        let args = InspectSignalsArgs {
            input_file: Some(input_file.path().to_path_buf()),
            output_format: "text".to_string(),
            output_file: Some(output_file.path().to_path_buf()),
        };

        let result = execute_inspect_signals(args);
        assert!(result.is_ok(), "Output to file should succeed");

        // Verificar que archivo de salida existe y tiene contenido
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        assert!(!output_content.is_empty(), "Output file should not be empty");
        assert!(output_content.contains("test"), "Output file should contain signal name");
    }

    #[test]
    fn test_inspect_signals_cli_invalid_input_file() {
        let args = InspectSignalsArgs {
            input_file: Some(std::path::PathBuf::from("nonexistent.vela")),
            output_format: "text".to_string(),
            output_file: None,
        };

        let result = execute_inspect_signals(args);
        assert!(result.is_err(), "Should fail with nonexistent input file");
    }

    #[test]
    fn test_inspect_signals_cli_invalid_format() {
        let temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        let vela_program = r#"state x: Number = 1"#;
        fs::write(&temp_file, vela_program).unwrap();

        let args = InspectSignalsArgs {
            input_file: Some(temp_file.path().to_path_buf()),
            output_format: "invalid".to_string(),
            output_file: None,
        };

        let result = execute_inspect_signals(args);
        assert!(result.is_err(), "Should fail with invalid output format");
    }

    #[test]
    fn test_inspect_signals_cli_empty_program() {
        let temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        fs::write(&temp_file, "").unwrap(); // Archivo vacío

        let args = InspectSignalsArgs {
            input_file: Some(temp_file.path().to_path_buf()),
            output_format: "text".to_string(),
            output_file: None,
        };

        let result = execute_inspect_signals(args);
        // Programa vacío debería ser válido (sin señales)
        assert!(result.is_ok(), "Empty program should be handled gracefully");

        let output = result.unwrap();
        assert!(output.contains("No reactive signals found") ||
                output.trim().is_empty(),
                "Empty program should report no signals or empty output");
    }

    #[test]
    fn test_inspect_signals_cli_large_program() {
        let temp_file = NamedTempFile::with_suffix(".vela").unwrap();

        // Crear programa grande con muchas señales
        let mut vela_program = String::new();
        for i in 0..100 {
            vela_program.push_str(&format!("state signal_{}: Number = {}\n", i, i));
            vela_program.push_str(&format!("computed computed_{}: Number {{ return signal_{} * 2 }}\n", i, i));
        }

        fs::write(&temp_file, vela_program).unwrap();

        let args = InspectSignalsArgs {
            input_file: Some(temp_file.path().to_path_buf()),
            output_format: "json".to_string(),
            output_file: None,
        };

        let result = execute_inspect_signals(args);
        assert!(result.is_ok(), "Large program should be handled");

        let output = result.unwrap();
        // Verificar que contiene algunas de las señales
        assert!(output.contains("signal_0"), "Should contain first signal");
        assert!(output.contains("computed_99"), "Should contain last computed");
    }

    #[test]
    fn test_inspect_signals_cli_malformed_program() {
        let temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        let malformed_program = r#"
// Programa malformado
state x: Number = "string"  // Error de tipos
computed y { return x }     // Sintaxis incorrecta
"#;

        fs::write(&temp_file, malformed_program).unwrap();

        let args = InspectSignalsArgs {
            input_file: Some(temp_file.path().to_path_buf()),
            output_format: "text".to_string(),
            output_file: None,
        };

        // El comando debería manejar errores de compilación gracefully
        let result = execute_inspect_signals(args);
        // Puede fallar o dar warning, pero no debería panic
        // Aceptamos tanto éxito como error controlado
        if result.is_err() {
            let error = result.unwrap_err().to_string();
            assert!(!error.contains("panic"), "Should not panic on malformed input");
        }
    }
}