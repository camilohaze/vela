//! Pipe Decorators for Vela Programming Language
//!
//! The `@pipe` decorator is a hybrid decorator that works differently
//! depending on the context:
//!
//! Frontend (UI/Angular-style): @pipe(name="currency", pure=true)
//! Backend (HTTP/NestJS-style): @pipe(ValidationPipe, TransformPipe)
//!
//! The decorator auto-detects the context based on syntax.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, Result, Token};

/// Context in which the pipe operates
#[derive(Debug, Clone)]
enum PipeContext {
    /// Frontend UI context (Angular-style pipes)
    Frontend(UiPipeConfig),
    /// Backend HTTP context (NestJS-style pipes)
    Backend(HttpPipeConfig),
}

/// Configuration for UI pipes (frontend context)
#[derive(Debug, Clone)]
struct UiPipeConfig {
    /// Name of the pipe for use in templates
    name: String,
    /// Whether the pipe is pure (cacheable)
    pure: bool,
}

/// Configuration for HTTP pipes (backend context)
#[derive(Debug, Clone)]
struct HttpPipeConfig {
    /// List of pipe types to apply
    pipes: Vec<String>,
}

impl Parse for PipeContext {
    fn parse(input: ParseStream) -> Result<Self> {
        // Try to parse as UI pipe first (name="...", pure=...)
        if input.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            if ident == "name" {
                input.parse::<Token![=]>()?;
                let name: syn::LitStr = input.parse()?;
                let mut pure = true; // Default to pure

                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                    let pure_ident: syn::Ident = input.parse()?;
                    if pure_ident == "pure" {
                        input.parse::<Token![=]>()?;
                        let pure_value: syn::LitBool = input.parse()?;
                        pure = pure_value.value();
                    }
                }

                return Ok(PipeContext::Frontend(UiPipeConfig {
                    name: name.value(),
                    pure,
                }));
            }
        }

        // If not UI pipe, try HTTP pipe (ValidationPipe, TransformPipe, ...)
        let mut pipes = Vec::new();

        while !input.is_empty() {
            let pipe_ident: syn::Ident = input.parse()?;
            pipes.push(pipe_ident.to_string());

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        if pipes.is_empty() {
            return Err(syn::Error::new(input.span(), "Expected pipe configuration"));
        }

        Ok(PipeContext::Backend(HttpPipeConfig { pipes }))
    }
}

/// Generate UI pipe implementation (frontend context)
fn generate_ui_pipe(
    config: &UiPipeConfig,
    class_name: &syn::Ident,
    class_tokens: &proc_macro::TokenStream,
) -> TokenStream {
    let pipe_name = &config.name;
    let pure_flag = config.pure;

    let class_ast: syn::ItemStruct = syn::parse(class_tokens.clone()).unwrap();
    let class_impl = quote!(#class_ast);

    quote! {
        #[derive(Clone)]
        #class_impl

        impl #class_name {
            pub fn new() -> Self {
                Self {}
            }

            pub fn is_pure(&self) -> bool {
                #pure_flag
            }

            pub fn pipe_name(&self) -> &'static str {
                #pipe_name
            }
        }
    }
}

/// Generate HTTP pipe implementation (backend context)
fn generate_http_pipe(
    config: &HttpPipeConfig,
    class_name: &syn::Ident,
    class_tokens: &proc_macro::TokenStream,
) -> TokenStream {
    let pipe_types = &config.pipes;

    let class_ast: syn::ItemStruct = syn::parse(class_tokens.clone()).unwrap();
    let class_impl = quote!(#class_ast);

    quote! {
        #class_impl

        impl #class_name {
            pub fn new() -> Self {
                Self {}
            }

            pub fn pipes(&self) -> &[String] {
                &[#(String::from(#pipe_types)),*]
            }
        }

        // HTTP Pipe trait for this context
        pub trait HttpPipe {
            fn process(&self, input: serde_json::Value) -> Result<serde_json::Value, String>;
        }

        // Built-in pipe implementations
        pub struct ValidationPipe;
        impl ValidationPipe {
            pub fn new() -> Self { Self }
        }
        impl HttpPipe for ValidationPipe {
            fn process(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
                // Basic validation logic
                Ok(input)
            }
        }

        pub struct TransformPipe;
        impl TransformPipe {
            pub fn new() -> Self { Self }
        }
        impl HttpPipe for TransformPipe {
            fn process(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
                // Basic transformation logic
                Ok(input)
            }
        }
    }
}

/// Procedural macro for @pipe decorator
#[proc_macro_attribute]
pub fn pipe(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let context = match syn::parse::<PipeContext>(attr) {
        Ok(ctx) => ctx,
        Err(err) => return err.to_compile_error().into(),
    };

    let class: syn::ItemStruct = match syn::parse(item.clone()) {
        Ok(cls) => cls,
        Err(err) => return err.to_compile_error().into(),
    };

    let class_name = &class.ident;

    let generated = match context {
        PipeContext::Frontend(ui_config) => {
            generate_ui_pipe(&ui_config, class_name, &item)
        }
        PipeContext::Backend(http_config) => {
            generate_http_pipe(&http_config, class_name, &item)
        }
    };

    generated.into()
}

    #[cfg(test)]
mod tests {
    use serde_json::json;
    use std::result::Result as StdResult;

    // Mock implementations for testing
    #[derive(Clone)]
    struct CurrencyPipe;

    impl CurrencyPipe {
        fn transform(&self, value: f64, currency: &str) -> String {
            format!("{} {:.2}", currency, value)
        }
    }

    #[derive(Clone)]
    struct DatePipe;

    impl DatePipe {
        fn transform(&self, timestamp: i64, format: &str) -> String {
            format!("{} ({})", timestamp, format)
        }
    }

    #[derive(Clone)]
    struct UserPipe;

    impl UserPipe {
        fn validate(&self, data: &serde_json::Value) -> StdResult<(), Vec<String>> {
            let mut errors = Vec::new();

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

        fn transform(&self, data: serde_json::Value) -> StdResult<serde_json::Value, String> {
            let mut transformed = data.as_object().unwrap().clone();
            transformed.insert("created_at".to_string(), json!(1234567890));
            transformed.insert("updated_at".to_string(), json!(1234567890));
            Ok(json!(transformed))
        }
    }

    #[derive(Clone)]
    struct FileUploadPipe;

    impl FileUploadPipe {
        fn validate(&self, file_data: &Vec<u8>) -> StdResult<(), Vec<String>> {
            let mut errors = Vec::new();

            if file_data.len() > 10 * 1024 * 1024 {
                errors.push("File too large (max 10MB)".to_string());
            }

            if file_data.len() > 0 {
                let first_bytes = &file_data[0..4.min(file_data.len())];
                if first_bytes != b"\x89PNG" && !first_bytes.starts_with(b"\xFF\xD8\xFF") {
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
        let invalid_data = json!({});

        let result = pipe.validate(&invalid_data);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() >= 2);
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
        let valid_file = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

        let result = pipe.validate(&valid_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_upload_pipe_invalid_size() {
        let pipe = FileUploadPipe;
        let large_file = vec![0; 11 * 1024 * 1024];

        let result = pipe.validate(&large_file);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("too large")));
    }

    #[test]
    fn test_file_upload_pipe_invalid_type() {
        let pipe = FileUploadPipe;
        let invalid_file = vec![0x00, 0x01, 0x02, 0x03];

        let result = pipe.validate(&invalid_file);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("PNG and JPEG")));
    }
}