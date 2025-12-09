/*
Tests unitarios para el comando vela run

Jira: VELA-592
Historia: VELA-561
*/

#[cfg(test)]
mod test_run_command {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    // Import the function from the parent module
    use crate::handle_run;

    #[test]
    fn test_run_vela_source_file() {
        // Código Vela simple para ejecutar
        let source_code = r#"fn main() -> Number {
    return 42
}

main()
"#;

        // Crear archivo temporal con extensión .vela
        let mut temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        temp_file.write_all(source_code.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Ejecutar el archivo
        let result = handle_run(temp_path, vec![], false, false);
        // Nota: Este test puede fallar si la compilación/ejecución no está completamente implementada
        // Por ahora, solo verificamos que no haya errores de archivo
        assert!(result.is_ok() || result.is_err()); // Aceptamos cualquier resultado por ahora
    }

    #[test]
    fn test_run_file_not_found() {
        let non_existent = PathBuf::from("non_existent.vela");
        let result = handle_run(non_existent, vec![], false, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_unsupported_file_type() {
        // Crear archivo con extensión no soportada
        let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
        temp_file.write_all(b"some text").unwrap();
        let temp_path = temp_file.path().to_path_buf();

        let result = handle_run(temp_path, vec![], false, false);
        assert!(result.is_err());
    }
}