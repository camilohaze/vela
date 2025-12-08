/*!
Localized formatting for dates, numbers, and currencies.
*/

use chrono::{DateTime, Utc, Local, TimeZone};
use rust_decimal::Decimal;
use num_format::{Locale as NumFormatLocale, ToFormattedString};
use crate::error::{I18nError, Result};
use crate::locale::{Locale, LocaleConfig, LocaleManager};

/// Localized formatter for dates, numbers, and currencies
#[derive(Debug)]
pub struct Formatter {
    /// Locale manager for configuration
    locale_manager: LocaleManager,
}

impl Formatter {
    /// Create a new formatter
    pub fn new() -> Self {
        Self {
            locale_manager: LocaleManager::new(),
        }
    }

    /// Create a formatter with a custom locale manager
    pub fn with_locale_manager(locale_manager: LocaleManager) -> Self {
        Self { locale_manager }
    }

    /// Format a date according to locale
    pub fn format_date(&self, date: &str, locale: &Locale) -> Result<String> {
        // Parse the date string - try different formats
        let datetime = self.parse_date(date)?;

        let config = self.locale_manager.get_config_with_fallback(locale);

        // Use ICU4X for proper localized formatting when available
        // For now, use a simple implementation
        let format_str = &config.date_formats.medium;

        // Basic formatting - in a real implementation, this would use
        // proper locale-aware formatting
        Ok(datetime.format(format_str).to_string())
    }

    /// Format a date with a specific format style
    pub fn format_date_with_style(&self, date: &str, locale: &Locale, style: DateFormatStyle) -> Result<String> {
        let datetime = self.parse_date(date)?;
        let config = self.locale_manager.get_config_with_fallback(locale);

        let format_str = match style {
            DateFormatStyle::Short => &config.date_formats.short,
            DateFormatStyle::Medium => &config.date_formats.medium,
            DateFormatStyle::Long => &config.date_formats.long,
            DateFormatStyle::Full => &config.date_formats.full,
        };

        Ok(datetime.format(format_str).to_string())
    }

    /// Format a number according to locale
    pub fn format_number(&self, number: &str, locale: &Locale) -> Result<String> {
        let num = number.parse::<f64>()
            .map_err(|_| I18nError::formatting_error(format!("Invalid number: {}", number)))?;

        let config = self.locale_manager.get_config_with_fallback(locale);

        // Use num-format crate for basic localization
        // In a real implementation, this would be more sophisticated
        match locale.language() {
            "en" => {
                let formatted = num.to_formatted_string(&NumFormatLocale::en);
                Ok(formatted)
            }
            "es" => {
                // Spanish formatting
                let formatted = num.to_formatted_string(&NumFormatLocale::es);
                Ok(formatted)
            }
            "de" => {
                let formatted = num.to_formatted_string(&NumFormatLocale::de);
                Ok(formatted)
            }
            "fr" => {
                let formatted = num.to_formatted_string(&NumFormatLocale::fr);
                Ok(formatted)
            }
            _ => {
                // Default to English formatting
                let formatted = num.to_formatted_string(&NumFormatLocale::en);
                Ok(formatted)
            }
        }
    }

    /// Format a currency amount according to locale
    pub fn format_currency(&self, amount: &str, currency_code: &str, locale: &Locale) -> Result<String> {
        let num = amount.parse::<f64>()
            .map_err(|_| I18nError::formatting_error(format!("Invalid amount: {}", amount)))?;

        let config = self.locale_manager.get_config_with_fallback(locale);

        // Basic currency formatting
        let formatted_number = self.format_number(&num.to_string(), locale)?;

        match config.currency_formats.symbol_position {
            crate::locale::CurrencyPosition::Before => {
                Ok(format!("{}{}", config.currency_formats.symbol, formatted_number))
            }
            crate::locale::CurrencyPosition::After => {
                Ok(format!("{}{}", formatted_number, config.currency_formats.symbol))
            }
        }
    }

    /// Format a list according to locale (with "and"/"or" separators)
    pub fn format_list(&self, items: &[String], style: ListFormatStyle, locale: &Locale) -> String {
        if items.is_empty() {
            return String::new();
        }

        if items.len() == 1 {
            return items[0].clone();
        }

        let config = self.locale_manager.get_config_with_fallback(locale);
        let separator = match style {
            ListFormatStyle::And => &config.list_formats.and_separator,
            ListFormatStyle::Or => &config.list_formats.or_separator,
        };

        let mut result = String::new();
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                if i == items.len() - 1 {
                    result.push_str(separator);
                } else {
                    result.push_str(", ");
                }
            }
            result.push_str(item);
        }

        result
    }

    /// Parse a date string into a DateTime
    fn parse_date(&self, date_str: &str) -> Result<DateTime<Utc>> {
        // Try different date formats
        let formats = [
            "%Y-%m-%d",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S%.fZ",
            "%Y-%m-%dT%H:%M:%SZ",
            "%B %e, %Y",
            "%b %e, %Y",
            "%Y/%m/%d",
            "%d/%m/%Y",
            "%m/%d/%Y",
        ];

        for format in &formats {
            if let Ok(datetime) = DateTime::parse_from_str(date_str, format) {
                return Ok(datetime.with_timezone(&Utc));
            }
        }

        // Try parsing as timestamp
        if let Ok(timestamp) = date_str.parse::<i64>() {
            if let Some(datetime) = Utc.timestamp_opt(timestamp, 0).single() {
                return Ok(datetime);
            }
        }

        Err(I18nError::formatting_error(format!("Unable to parse date: {}", date_str)))
    }

    /// Set locale configuration
    pub fn set_locale_config(&mut self, locale: Locale, config: LocaleConfig) {
        self.locale_manager.set_config(locale, config);
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Date format styles
#[derive(Debug, Clone, Copy)]
pub enum DateFormatStyle {
    Short,
    Medium,
    Long,
    Full,
}

/// List format styles
#[derive(Debug, Clone, Copy)]
pub enum ListFormatStyle {
    And,
    Or,
}

/// Utility functions for common formatting operations
pub mod utils {
    use super::*;

    /// Format a relative time (e.g., "2 hours ago")
    pub fn format_relative_time(timestamp: DateTime<Utc>, locale: &Locale) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(timestamp);

        let seconds = duration.num_seconds().abs();
        let minutes = duration.num_minutes().abs();
        let hours = duration.num_hours().abs();
        let days = duration.num_days().abs();

        match locale.language() {
            "en" => match () {
                _ if seconds < 60 => format!("{} seconds ago", seconds),
                _ if minutes < 60 => format!("{} minutes ago", minutes),
                _ if hours < 24 => format!("{} hours ago", hours),
                _ => format!("{} days ago", days),
            },
            "es" => match () {
                _ if seconds < 60 => format!("hace {} segundos", seconds),
                _ if minutes < 60 => format!("hace {} minutos", minutes),
                _ if hours < 24 => format!("hace {} horas", hours),
                _ => format!("hace {} días", days),
            },
            _ => format!("{} seconds ago", seconds), // fallback
        }
    }

    /// Format file size in human readable format
    pub fn format_file_size(bytes: u64, locale: &Locale) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        const UNITS_ES: &[&str] = &["B", "KB", "MB", "GB", "TB"];

        let units = match locale.language() {
            "es" => UNITS_ES,
            _ => UNITS,
        };

        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < units.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, units[0])
        } else {
            format!("{:.1} {}", size, units[unit_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        let formatter = Formatter::new();
        let locale = Locale::from("en").unwrap();

        let result = formatter.format_number("1234.56", &locale).unwrap();
        assert_eq!(result, "1,234.56");
    }

    #[test]
    fn test_format_currency() {
        let formatter = Formatter::new();
        let locale = Locale::from("en").unwrap();

        let result = formatter.format_currency("29.99", "USD", &locale).unwrap();
        assert_eq!(result, "$29.99");
    }

    #[test]
    fn test_format_list() {
        let formatter = Formatter::new();
        let locale = Locale::from("en").unwrap();

        let items = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
        let result = formatter.format_list(&items, ListFormatStyle::And, &locale);
        assert_eq!(result, "apple, banana and cherry");
    }

    #[test]
    fn test_format_date() {
        let formatter = Formatter::new();
        let locale = Locale::from("en").unwrap();

        let result = formatter.format_date("2023-12-25", &locale).unwrap();
        // The exact format depends on the locale config, but should contain the date
        assert!(result.contains("2023"));
    }

    #[test]
    fn test_parse_date() {
        let formatter = Formatter::new();

        let result = formatter.parse_date("2023-12-25").unwrap();
        assert_eq!(result.format("%Y-%m-%d").to_string(), "2023-12-25");
    }

    #[test]
    fn test_format_file_size() {
        let locale = Locale::from("en").unwrap();

        assert_eq!(utils::format_file_size(1024, &locale), "1.0 KB");
        assert_eq!(utils::format_file_size(1024 * 1024, &locale), "1.0 MB");
        assert_eq!(utils::format_file_size(500, &locale), "500 B");
    }

    #[test]
    fn test_format_relative_time() {
        let locale = Locale::from("en").unwrap();
        let past_time = Utc::now() - chrono::Duration::hours(2);

        let result = utils::format_relative_time(past_time, &locale);
        assert!(result.contains("hours ago"));
    }
}