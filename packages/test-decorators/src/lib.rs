//! Test Decorators for Vela Programming Language
//!
//! This crate provides procedural macros for test-related decorators in Vela:
//! - `@test`: Mark functions as unit tests
//! - `@describe`: Group related tests (BDD style)
//! - `@it`: Define individual test cases (BDD style)
//! - `@beforeEach`, `@afterEach`: Setup/cleanup per test
//! - `@beforeAll`, `@afterAll`: Setup/cleanup once per test suite
//! - `@context`: Alias for `@describe`

use proc_macro2::TokenStream;
use quote::{quote};
use serde::{Deserialize, Serialize};
use syn::{parse::Parse, parse::ParseStream, Result, Token};

/// Configuration for the @test decorator
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestConfig {
    /// Test name (optional, defaults to function name)
    name: Option<String>,
    /// Whether this test should be ignored
    ignore: bool,
    /// Expected panic message (for should_panic tests)
    should_panic: Option<String>,
    /// Timeout in milliseconds
    timeout: Option<u64>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            name: None,
            ignore: false,
            should_panic: None,
            timeout: None,
        }
    }
}

impl Parse for TestConfig {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut config = TestConfig::default();

        if input.is_empty() {
            return Ok(config);
        }

        // Parse key-value pairs
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "name" => {
                    let value: syn::LitStr = input.parse()?;
                    config.name = Some(value.value());
                }
                "ignore" => {
                    let value: syn::LitBool = input.parse()?;
                    config.ignore = value.value();
                }
                "should_panic" => {
                    let value: syn::LitStr = input.parse()?;
                    config.should_panic = Some(value.value());
                }
                "timeout" => {
                    let value: syn::LitInt = input.parse()?;
                    config.timeout = Some(value.base10_parse()?);
                }
                _ => return Err(syn::Error::new(key.span(), "Unknown test configuration key")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(config)
    }
}

/// Configuration for BDD test groups
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DescribeConfig {
    /// Description of the test suite
    description: String,
    /// Whether to skip this test suite
    skip: bool,
}

impl Parse for DescribeConfig {
    fn parse(input: ParseStream) -> Result<Self> {
        let description: syn::LitStr = input.parse()?;
        let mut skip = false;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let skip_ident: syn::Ident = input.parse()?;
            if skip_ident == "skip" {
                input.parse::<Token![=]>()?;
                let skip_value: syn::LitBool = input.parse()?;
                skip = skip_value.value();
            }
        }

        Ok(DescribeConfig {
            description: description.value(),
            skip,
        })
    }
}

/// Configuration for individual test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ItConfig {
    /// Description of the test case
    description: String,
    /// Whether to skip this test
    skip: bool,
    /// Timeout in milliseconds
    timeout: Option<u64>,
}

impl Parse for ItConfig {
    fn parse(input: ParseStream) -> Result<Self> {
        let description: syn::LitStr = input.parse()?;
        let mut skip = false;
        let mut timeout = None;

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "skip" => {
                    let value: syn::LitBool = input.parse()?;
                    skip = value.value();
                }
                "timeout" => {
                    let value: syn::LitInt = input.parse()?;
                    timeout = Some(value.base10_parse()?);
                }
                _ => return Err(syn::Error::new(key.span(), "Unknown 'it' configuration key")),
            }
        }

        Ok(ItConfig {
            description: description.value(),
            skip,
            timeout,
        })
    }
}

/// Configuration for hook decorators
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HookConfig {
    /// Optional timeout for the hook
    timeout: Option<u64>,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self { timeout: None }
    }
}

impl Parse for HookConfig {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut config = HookConfig::default();

        if input.is_empty() {
            return Ok(config);
        }

        // Parse optional timeout
        if input.peek(syn::Ident) {
            let key: syn::Ident = input.parse()?;
            if key == "timeout" {
                input.parse::<syn::Token![=]>()?;
                let value: syn::LitInt = input.parse()?;
                config.timeout = Some(value.base10_parse()?);
            }
        }

        Ok(config)
    }
}

/// Generate the test function wrapper
fn generate_test_function(
    config: &TestConfig,
    function_name: &syn::Ident,
    function_body: &TokenStream,
) -> TokenStream {
    let ignore_attr = if config.ignore {
        quote!(#[ignore])
    } else {
        quote!()
    };

    let should_panic_attr = if let Some(msg) = &config.should_panic {
        quote!(#[should_panic(expected = #msg)])
    } else {
        quote!()
    };

    let timeout_check = if let Some(timeout_ms) = config.timeout {
        quote! {
            let start_time = std::time::Instant::now();
            let result = std::panic::catch_unwind(|| {
                #function_body
            });
            let elapsed = start_time.elapsed().as_millis() as u64;
            if elapsed > #timeout_ms {
                panic!("Test timed out after {}ms", #timeout_ms);
            }
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    } else {
        quote! {
            #function_body
        }
    };

    // Generate the test function directly without #[test] to avoid recursion
    quote! {
        #ignore_attr
        #should_panic_attr
        fn #function_name() {
            #timeout_check
        }
    }
}

/// Generate a test module for BDD describe blocks
fn generate_describe_module(
    config: &DescribeConfig,
    module_name: &syn::Ident,
    items: &[syn::Item],
) -> TokenStream {
    let _description = &config.description;
    let skip_attr = if config.skip {
        quote!(#[cfg(feature = "skip_bdd_tests")])
    } else {
        quote!()
    };

    let mut test_functions = Vec::new();
    let mut sub_modules = Vec::new();

    for item in items {
        match item {
            syn::Item::Fn(func) => {
                // Check if function has #[it] attribute
                let has_it_attr = func.attrs.iter().any(|attr| {
                    attr.path().is_ident("it")
                });

                if has_it_attr {
                    let func_name = &func.sig.ident;
                    // Generate function without #[test] - it will be added by the it macro
                    test_functions.push(quote! {
                        fn #func_name() {
                            #func
                        }
                    });
                }
            }
            syn::Item::Mod(sub_mod) => {
                sub_modules.push(quote!(#sub_mod));
            }
            _ => {}
        }
    }

    quote! {
        #skip_attr
        mod #module_name {
            #(#test_functions)*
            #(#sub_modules)*
        }
    }
}

/// Generate individual test function for @it
fn generate_it_function(
    config: &ItConfig,
    function_name: &syn::Ident,
    function_body: &TokenStream,
) -> TokenStream {
    let description = &config.description;
    let skip_attr = if config.skip {
        quote!(#[ignore])
    } else {
        quote!()
    };

    let timeout_check = if let Some(timeout_ms) = config.timeout {
        quote! {
            let start_time = std::time::Instant::now();
            let result = std::panic::catch_unwind(|| {
                #function_body
            });
            let elapsed = start_time.elapsed().as_millis() as u64;
            if elapsed > #timeout_ms {
                panic!("Test '{}' timed out after {}ms", #description, #timeout_ms);
            }
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    } else {
        quote! {
            #function_body
        }
    };

    // Generate function without #[test] - it will be added by the macro
    quote! {
        #skip_attr
        fn #function_name() {
            println!("Running: {}", #description);
            #timeout_check
        }
    }
}

/// Generate setup function for @beforeEach
fn generate_before_each_function(
    config: &HookConfig,
    function_name: &syn::Ident,
    function_body: &TokenStream,
) -> TokenStream {
    let timeout_check = if let Some(timeout_ms) = config.timeout {
        quote! {
            let start_time = std::time::Instant::now();
            #function_body
            let elapsed = start_time.elapsed().as_millis() as u64;
            if elapsed > #timeout_ms {
                panic!("beforeEach hook timed out after {}ms", #timeout_ms);
            }
        }
    } else {
        quote!(#function_body)
    };

    quote! {
        fn #function_name() {
            #timeout_check
        }
    }
}

/// Generate cleanup function for @afterEach
fn generate_after_each_function(
    config: &HookConfig,
    function_name: &syn::Ident,
    function_body: &TokenStream,
) -> TokenStream {
    let timeout_check = if let Some(timeout_ms) = config.timeout {
        quote! {
            let start_time = std::time::Instant::now();
            #function_body
            let elapsed = start_time.elapsed().as_millis() as u64;
            if elapsed > #timeout_ms {
                eprintln!("Warning: afterEach hook timed out after {}ms", #timeout_ms);
            }
        }
    } else {
        quote!(#function_body)
    };

    quote! {
        fn #function_name() {
            #timeout_check
        }
    }
}

/// Generate suite setup function for @beforeAll
fn generate_before_all_function(
    config: &HookConfig,
    function_name: &syn::Ident,
    function_body: &TokenStream,
) -> TokenStream {
    let timeout_check = if let Some(timeout_ms) = config.timeout {
        quote! {
            static INIT: std::sync::Once = std::sync::Once::new();
            INIT.call_once(|| {
                let start_time = std::time::Instant::now();
                #function_body
                let elapsed = start_time.elapsed().as_millis() as u64;
                if elapsed > #timeout_ms {
                    panic!("beforeAll hook timed out after {}ms", #timeout_ms);
                }
            });
        }
    } else {
        quote! {
            static INIT: std::sync::Once = std::sync::Once::new();
            INIT.call_once(|| {
                #function_body
            });
        }
    };

    quote! {
        fn #function_name() {
            #timeout_check
        }
    }
}

/// Generate suite cleanup function for @afterAll
fn generate_after_all_function(
    config: &HookConfig,
    function_name: &syn::Ident,
    function_body: &TokenStream,
) -> TokenStream {
    let timeout_check = if let Some(timeout_ms) = config.timeout {
        quote! {
            static CLEANUP: std::sync::Once = std::sync::Once::new();
            CLEANUP.call_once(|| {
                let start_time = std::time::Instant::now();
                #function_body
                let elapsed = start_time.elapsed().as_millis() as u64;
                if elapsed > #timeout_ms {
                    eprintln!("Warning: afterAll hook timed out after {}ms", #timeout_ms);
                }
            });
        }
    } else {
        quote! {
            static CLEANUP: std::sync::Once = std::sync::Once::new();
            CLEANUP.call_once(|| {
                #function_body
            });
        }
    };

    quote! {
        fn #function_name() {
            #timeout_check
        }
    }
}

/// Procedural macro for @test decorator
#[proc_macro_attribute]
pub fn test(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = match syn::parse::<TestConfig>(attr) {
        Ok(cfg) => cfg,
        Err(err) => return err.to_compile_error().into(),
    };

    let function: syn::ItemFn = match syn::parse(item) {
        Ok(func) => func,
        Err(err) => return err.to_compile_error().into(),
    };

    let function_name = &function.sig.ident;
    let function_body = &function.block;

    let generated = generate_test_function(&config, function_name, &quote!(#function_body));

    // Add the #[test] attribute externally to avoid recursion
    quote! {
        #[test]
        #generated
    }.into()
}

/// Procedural macro for @describe decorator
#[proc_macro_attribute]
pub fn describe(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = match syn::parse::<DescribeConfig>(attr) {
        Ok(cfg) => cfg,
        Err(err) => return err.to_compile_error().into(),
    };

    let module: syn::ItemMod = match syn::parse(item) {
        Ok(mod_item) => mod_item,
        Err(err) => return err.to_compile_error().into(),
    };

    let module_name = &module.ident;
    let items = &module.content.as_ref().unwrap().1;

    let generated = generate_describe_module(&config, module_name, items);

    generated.into()
}

/// Procedural macro for @it decorator
#[proc_macro_attribute]
pub fn it(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = match syn::parse::<ItConfig>(attr) {
        Ok(cfg) => cfg,
        Err(err) => return err.to_compile_error().into(),
    };

    let function: syn::ItemFn = match syn::parse(item) {
        Ok(func) => func,
        Err(err) => return err.to_compile_error().into(),
    };

    let function_name = &function.sig.ident;
    let function_body = &function.block;

    let generated = generate_it_function(&config, function_name, &quote!(#function_body));

    // Add the #[test] attribute externally to avoid recursion
    quote! {
        #[test]
        #generated
    }.into()
}

/// Alias for @describe - provides better readability in some contexts
#[proc_macro_attribute]
pub fn context(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // @context is just an alias for @describe
    describe(attr, item)
}

/// Procedural macro for @beforeEach decorator
#[proc_macro_attribute]
pub fn before_each(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = match syn::parse::<HookConfig>(attr) {
        Ok(cfg) => cfg,
        Err(err) => return err.to_compile_error().into(),
    };

    let function: syn::ItemFn = match syn::parse(item) {
        Ok(func) => func,
        Err(err) => return err.to_compile_error().into(),
    };

    let function_name = &function.sig.ident;
    let function_body = &function.block;

    let generated = generate_before_each_function(&config, function_name, &quote!(#function_body));

    generated.into()
}

/// Procedural macro for @afterEach decorator
#[proc_macro_attribute]
pub fn after_each(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = match syn::parse::<HookConfig>(attr) {
        Ok(cfg) => cfg,
        Err(err) => return err.to_compile_error().into(),
    };

    let function: syn::ItemFn = match syn::parse(item) {
        Ok(func) => func,
        Err(err) => return err.to_compile_error().into(),
    };

    let function_name = &function.sig.ident;
    let function_body = &function.block;

    let generated = generate_after_each_function(&config, function_name, &quote!(#function_body));

    generated.into()
}

/// Procedural macro for @beforeAll decorator
#[proc_macro_attribute]
pub fn before_all(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = match syn::parse::<HookConfig>(attr) {
        Ok(cfg) => cfg,
        Err(err) => return err.to_compile_error().into(),
    };

    let function: syn::ItemFn = match syn::parse(item) {
        Ok(func) => func,
        Err(err) => return err.to_compile_error().into(),
    };

    let function_name = &function.sig.ident;
    let function_body = &function.block;

    let generated = generate_before_all_function(&config, function_name, &quote!(#function_body));

    generated.into()
}

/// Procedural macro for @afterAll decorator
#[proc_macro_attribute]
pub fn after_all(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = match syn::parse::<HookConfig>(attr) {
        Ok(cfg) => cfg,
        Err(err) => return err.to_compile_error().into(),
    };

    let function: syn::ItemFn = match syn::parse(item) {
        Ok(func) => func,
        Err(err) => return err.to_compile_error().into(),
    };

    let function_name = &function.sig.ident;
    let function_body = &function.block;

    let generated = generate_after_all_function(&config, function_name, &quote!(#function_body));

    generated.into()
}