/*
Tests unitarios para el comando vela fmt

Jira: VELA-592
Historia: VELA-561
*/

#[cfg(test)]
mod test_fmt_command {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    // Import the function from the parent module
    use crate::handle_fmt;

    #[test]
    fn test_format_basic_syntax() {
        // Código mal formateado
        let input_code = r#"fn add(a: Number,b: Number)->Number{
if a>0{
return a+b
}else{
return 0
}
}"#;

        // Crear archivo temporal con extensión .vela
        let mut temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        temp_file.write_all(input_code.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Ejecutar formateo
        let result = handle_fmt(vec![temp_path.clone()], false);
        assert!(result.is_ok());

        // Verificar contenido formateado
        let formatted_content = fs::read_to_string(&temp_path).unwrap();

        // Verificar que se aplicaron las reglas básicas
        assert!(formatted_content.contains("a: Number, b: Number"));
        assert!(formatted_content.contains(" -> Number"));
        assert!(formatted_content.contains("a > 0"));
        assert!(formatted_content.contains("a + b"));
    }

    #[test]
    fn test_format_check_mode() {
        // Código que NO necesita formateo
        let input_code = "a, b";

        // Crear archivo temporal con extensión .vela
        let temp_path = std::env::temp_dir().join("test_fmt.vela");
        std::fs::write(&temp_path, input_code).unwrap();

        // Ejecutar en modo check - debería pasar
        let result = handle_fmt(vec![temp_path], true);
        // std::fs::remove_file(&temp_path).unwrap(); // cleanup
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_already_formatted() {
        // Código ya bien formateado
        let input_code = r#"fn add(a: Number, b: Number) -> Number {
    if a > 0 {
        return a + b
    } else {
        return 0
    }
}"#;

        // Crear archivo temporal con extensión .vela
        let mut temp_file = NamedTempFile::with_suffix(".vela").unwrap();
        temp_file.write_all(input_code.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Leer contenido original
        let original_content = fs::read_to_string(&temp_path).unwrap();

        // Ejecutar formateo
        let result = handle_fmt(vec![temp_path.clone()], false);
        assert!(result.is_ok());

        // Verificar que no cambió
        let final_content = fs::read_to_string(&temp_path).unwrap();
        assert_eq!(original_content, final_content);
    }

    #[test]
    fn test_format_non_vela_file() {
        // Crear archivo .txt
        let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
        temp_file.write_all(b"some text content").unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Debería procesar sin error pero ignorar el archivo
        let result = handle_fmt(vec![temp_path.clone()], false);
        assert!(result.is_ok());

        // El contenido no debería cambiar
        let content = fs::read_to_string(&temp_path).unwrap();
        assert_eq!(content, "some text content");
    }

    #[test]
    fn test_format_file_not_found() {
        let non_existent = PathBuf::from("non_existent.vela");
        let result = handle_fmt(vec![non_existent], false);
        assert!(result.is_err());
    }
}