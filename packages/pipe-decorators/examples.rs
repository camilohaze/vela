//! Examples demonstrating the @pipe decorator usage

/// Frontend UI Pipe Example
/// This pipe formats currency values in templates
#[pipe(name = "currency", pure = true)]
pub struct CurrencyPipe;

impl CurrencyPipe {
    pub fn transform(&self, value: f64, currency: &str) -> String {
        format!("{} {:.2}", currency, value)
    }
}

/// Frontend Date Pipe Example
/// This pipe formats dates in templates
#[pipe(name = "date", pure = false)]
pub struct DatePipe;

impl DatePipe {
    pub fn transform(&self, timestamp: i64, format: &str) -> String {
        // In a real implementation, this would use chrono or similar
        format!("{} ({})", timestamp, format)
    }
}

/// Backend HTTP Pipe Example
/// This pipe validates and transforms user data
#[pipe(ValidationPipe, TransformPipe)]
pub struct UserPipe;

impl UserPipe {
    pub fn validate(&self, data: &serde_json::Value) -> Result<(), Vec<String>> {
        let errors = Vec::new();

        // Check if required fields exist
        if !data.get("name").is_some() {
            errors.push("name is required".to_string());
        }

        if !data.get("email").is_some() {
            errors.push("email is required".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn transform(&self, data: serde_json::Value) -> Result<serde_json::Value, String> {
        // Add default values or transform data
        let mut transformed = data.as_object().unwrap().clone();

        // Add timestamps
        transformed.insert("created_at".to_string(), serde_json::json!(chrono::Utc::now().timestamp()));
        transformed.insert("updated_at".to_string(), serde_json::json!(chrono::Utc::now().timestamp()));

        Ok(serde_json::Value::Object(transformed))
    }
}

/// Backend File Upload Pipe Example
/// This pipe handles file upload validation
#[pipe(ValidationPipe)]
pub struct FileUploadPipe;

impl FileUploadPipe {
    pub fn validate(&self, file_data: &Vec<u8>) -> Result<(), Vec<String>> {
        let errors = Vec::new();

        // Check file size (max 10MB)
        if file_data.len() > 10 * 1024 * 1024 {
            errors.push("File too large (max 10MB)".to_string());
        }

        // Check file type (simple check)
        if file_data.len() > 0 {
            let first_bytes = &file_data[0..4];
            if first_bytes != b"\x89PNG" && first_bytes != b"\xFF\xD8\xFF" {
                errors.push("Only PNG and JPEG files allowed".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Example of how these pipes would be used in Vela code
///
/// Frontend usage (in templates):
/// ```vela
/// // In a Vela component
/// template: """
///   <div>
///     <p>Price: {{product.price | currency:'USD'}}</p>
///     <p>Date: {{product.created_at | date:'short'}}</p>
///   </div>
/// """
/// ```
///
/// Backend usage (in controllers):
/// ```vela
/// @controller("/api")
/// class ApiController {
///   @post("/users")
///   async fn create_user(@body user: User) -> Result<User> {
///     // user data is automatically validated and transformed by UserPipe
///     // ...
///   }
///
///   @post("/upload")
///   async fn upload_file(@file file: Vec<u8>) -> Result<String> {
///     // file is automatically validated by FileUploadPipe
///     // ...
///   }
/// }
/// ```
pub mod examples {
    pub use super::*;
}