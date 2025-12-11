//! Tests for the @pipe decorator

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_currency_pipe() {
        let pipe = CurrencyPipe;
        let result = pipe.transform(123.45, "USD");
        assert_eq!(result, "USD 123.45");
    }

    #[test]
    fn test_date_pipe() {
        let pipe = DatePipe;
        let result = pipe.transform(1640995200, "short");
        assert_eq!(result, "1640995200 (short)");
    }

    #[test]
    fn test_user_pipe_validation_success() {
        let pipe = UserPipe;
        let valid_data = json!({
            "name": "John Doe",
            "email": "john@example.com"
        });

        let result = pipe.validate(&valid_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_user_pipe_validation_failure() {
        let pipe = UserPipe;
        let invalid_data = json!({}); // Missing required fields

        let result = pipe.validate(&invalid_data);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() >= 2); // Should have errors for name and email
        assert!(errors.iter().any(|e| e.contains("name")));
        assert!(errors.iter().any(|e| e.contains("email")));
    }

    #[test]
    fn test_user_pipe_transform() {
        let pipe = UserPipe;
        let input_data = json!({
            "name": "John Doe",
            "email": "john@example.com"
        });

        let result = pipe.transform(input_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.get("created_at").is_some());
        assert!(transformed.get("updated_at").is_some());
        assert_eq!(transformed["name"], "John Doe");
        assert_eq!(transformed["email"], "john@example.com");
    }

    #[test]
    fn test_file_upload_pipe_valid_file() {
        let pipe = FileUploadPipe;
        // Simulate a small PNG file
        let valid_file = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header

        let result = pipe.validate(&valid_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_upload_pipe_invalid_size() {
        let pipe = FileUploadPipe;
        // Create a file larger than 10MB
        let large_file = vec![0; 11 * 1024 * 1024];

        let result = pipe.validate(&large_file);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("too large")));
    }

    #[test]
    fn test_file_upload_pipe_invalid_type() {
        let pipe = FileUploadPipe;
        // Invalid file type (not PNG or JPEG)
        let invalid_file = vec![0x00, 0x01, 0x02, 0x03];

        let result = pipe.validate(&invalid_file);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("PNG and JPEG")));
    }

    #[test]
    fn test_empty_file() {
        let pipe = FileUploadPipe;
        let empty_file = vec![];

        let result = pipe.validate(&empty_file);
        assert!(result.is_err());
    }
}