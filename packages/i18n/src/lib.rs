/*!
# Vela Internationalization (i18n) System

This crate provides comprehensive internationalization support for Vela applications,
including locale management, translation loading, string interpolation, and localized
formatting.

## Features

- **Locale Management**: Support for multiple locales with fallback chains
- **Translation Loading**: JSON/YAML translation files with caching
- **String Interpolation**: Variable substitution with pluralization support
- **Localized Formatting**: Date, number, and currency formatting by locale
- **Decorator System**: `@i18n` decorator for translation classes
- **Hot Reload**: Automatic translation reloading during development

## Example

```rust
use vela_i18n::{Translator, Locale};

// Create a runtime for async operations
let rt = tokio::runtime::Runtime::new().unwrap();

// Execute async operations in the runtime
rt.block_on(async {
    // Create translator
    let mut translator = Translator::new();

    // Set primary locale
    let locale = Locale::from("en-US").unwrap();
    translator.set_locale(locale).await?;

    // Load translations
    translator.load_translations_from_dir("translations/").await?;

    // Simple translation
    let greeting = translator.translate("greeting.hello", &[]).await?;

    // Translation with variables
    let welcome = translator.translate("messages.user_greeting", &[
        ("name", "Alice")
    ]).await?;

    Ok::<(), Box<dyn std::error::Error>>(())
})?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
*/

pub mod locale;
pub mod loader;
pub mod interpolator;
pub mod formatter;
pub mod translator;
pub mod decorator;
pub mod hot_reload;
pub mod pluralization;
pub mod error;

// Re-export main types
pub use locale::Locale;
pub use translator::Translator;
pub use formatter::Formatter;
pub use error::{I18nError, Result};

// Re-export commonly used types
pub use interpolator::{InterpolationContext, Interpolator};
pub use loader::{TranslationLoader, TranslationMap};
pub use decorator::I18nDecorator;

// Include comprehensive tests
#[cfg(test)]
mod tests;