/*!
Localized formatting for dates, numbers, and currencies.
*/

use chrono::{DateTime, Utc, Local, TimeZone, Datelike, Timelike};
use rust_decimal::Decimal;
use num_format::{Locale as NumFormatLocale, ToFormattedString};
use icu_locid::Locale as IcuLocale;
use icu_datetime::{DateTimeFormatter, DateTimeFormatterOptions, options::length};
use icu_decimal::{FixedDecimalFormatter, options::FixedDecimalFormatterOptions};
use icu_calendar::{DateTime as IcuDateTime, Gregorian};
use icu_list::{ListFormatter, ListLength};
use icu_provider::DataLocale;
use fixed_decimal::FixedDecimal;
use std::str::FromStr;
use crate::error::{I18nError, Result};
use crate::locale::{Locale, LocaleConfig, LocaleManager};

/// Localized formatter for dates, numbers, and currencies
#[derive(Debug, Clone)]
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
        Self {
            locale_manager,
        }
    }

    /// Format a date according to locale using ICU4X
    pub fn format_date(&self, date: &str, locale: &Locale) -> Result<String> {
        let datetime = self.parse_date(date)?;
        let icu_locale = self.locale_to_icu_locale(locale);

        // Create ICU4X DateTime from chrono DateTime
        let icu_datetime = icu_calendar::DateTime::try_new_gregorian_datetime(
            datetime.year(),
            datetime.month() as u8,
            datetime.day() as u8,
            datetime.hour() as u8,
            datetime.minute() as u8,
            datetime.second() as u8,
        ).map_err(|e| I18nError::formatting_error(format!("Invalid date components: {}", e)))?;

        // Convert to AnyCalendar for ICU4X formatter
        let icu_datetime = icu_datetime.to_any();

        // Get formatter for this locale
        let formatter = self.create_date_formatter(&icu_locale, DateFormatStyle::Medium)?;

        // Format using ICU4X
        let formatted_result = formatter.format(&icu_datetime);
        let formatted = formatted_result
            .map_err(|e| I18nError::formatting_error(format!("Failed to format date: {}", e)))?
            .to_string();
        Ok(formatted)
    }

    /// Format a date with a specific format style
    pub fn format_date_with_style(&self, date: &str, locale: &Locale, style: DateFormatStyle) -> Result<String> {
        let datetime = self.parse_date(date)?;
        let icu_locale = self.locale_to_icu_locale(locale);

        // Create ICU4X DateTime from chrono DateTime
        let icu_datetime = icu_calendar::DateTime::try_new_gregorian_datetime(
            datetime.year(),
            datetime.month() as u8,
            datetime.day() as u8,
            datetime.hour() as u8,
            datetime.minute() as u8,
            datetime.second() as u8,
        ).map_err(|e| I18nError::formatting_error(format!("Invalid date components: {}", e)))?;

        // Convert to AnyCalendar for ICU4X formatter
        let icu_datetime = icu_datetime.to_any();

        // Get formatter for this locale and style
        let formatter = self.create_date_formatter(&icu_locale, style)?;

        // Format using ICU4X
        let formatted_result = formatter.format(&icu_datetime);
        let formatted = formatted_result
            .map_err(|e| I18nError::formatting_error(format!("Failed to format date: {}", e)))?
            .to_string();
        Ok(formatted)
    }

    /// Format a number according to locale using ICU4X
    pub fn format_number(&self, number: &str, locale: &Locale) -> Result<String> {
        let icu_locale = self.locale_to_icu_locale(locale);

        // Parse number to FixedDecimal
        let decimal: FixedDecimal = number.parse()
            .map_err(|e| I18nError::formatting_error(format!("Invalid number format: {}", e)))?;

        // Get formatter for this locale
        let formatter = self.create_decimal_formatter(&icu_locale)?;

        // Format using ICU4X
        let formatted = formatter.format(&decimal).to_string();
        Ok(formatted)
    }

    /// Helper method to format numbers with custom separators
    fn format_number_with_separator(&self, num: f64, thousand_sep: &str, decimal_sep: &str) -> Result<String> {
        // Handle integer part
        let integer_part = num.trunc() as i64;
        let decimal_part = (num.fract() * 100.0).round() as i32;

        // Format integer part with thousand separators
        let integer_str = self.format_integer_with_separator(integer_part.abs(), thousand_sep);

        // Add sign if negative
        let sign = if integer_part < 0 { "-" } else { "" };

        // Format decimal part
        if decimal_part > 0 {
            Ok(format!("{}{}{}{:02}", sign, integer_str, decimal_sep, decimal_part))
        } else {
            Ok(format!("{}{}", sign, integer_str))
        }
    }

    /// Format integer with thousand separators
    fn format_integer_with_separator(&self, num: i64, separator: &str) -> String {
        let num_str = num.to_string();
        let mut result = String::new();
        let chars: Vec<char> = num_str.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 {
                result.push_str(separator);
            }
            result.push(ch);
        }

        result
    }

    /// Format a currency amount according to locale using ICU4X numbers + localized symbols
    pub fn format_currency(&self, amount: &str, currency_code: &str, locale: &Locale) -> Result<String> {
        let icu_locale = self.locale_to_icu_locale(locale);

        // Parse amount to FixedDecimal
        let decimal: FixedDecimal = amount.parse()
            .map_err(|e| I18nError::formatting_error(format!("Invalid amount format: {}", e)))?;

        // Get formatter for this locale
        let formatter = self.create_decimal_formatter(&icu_locale)?;

        // Format the number part using ICU4X
        let formatted_number = formatter.format(&decimal).to_string();

        // Get localized currency configuration
        let config = self.locale_manager.get_config_with_fallback(locale);

        // Get currency symbol based on currency code
        let symbol = self.get_currency_symbol(currency_code, locale);

        // Combine number and symbol based on locale preferences
        match config.currency_formats.symbol_position {
            crate::locale::CurrencyPosition::Before => {
                Ok(format!("{}{}", symbol, formatted_number))
            }
            crate::locale::CurrencyPosition::After => {
                Ok(format!("{}{}", formatted_number, symbol))
            }
        }
    }

    /// Format a list according to locale using ICU4X
    pub fn format_list(&self, items: &[String], style: ListFormatStyle, locale: &Locale) -> Result<String> {
        if items.is_empty() {
            return Ok(String::new());
        }

        if items.len() == 1 {
            return Ok(items[0].clone());
        }

        let icu_locale = self.locale_to_icu_locale(locale);
        let length = match style {
            ListFormatStyle::And => ListLength::Wide,
            ListFormatStyle::Or => ListLength::Wide,
        };

        // Get formatter for this locale and style
        let formatter = self.create_list_formatter(&icu_locale, length)?;

        // Convert items to ICU4X compatible format
        let icu_items: Vec<&str> = items.iter().map(|s| s.as_str()).collect();

        // Format using ICU4X
        let formatted = formatter.format(icu_items.iter()).to_string();
        Ok(formatted)
    }

    /// Parse a date string into a DateTime
    fn parse_date(&self, date_str: &str) -> Result<DateTime<Utc>> {
        // Try parsing as NaiveDate first (date only)
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let naive_datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
            return Ok(DateTime::from_naive_utc_and_offset(naive_datetime, Utc));
        }

        // Try different datetime formats
        let datetime_formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S%.fZ",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y/%m/%d",
            "%d/%m/%Y",
            "%m/%d/%Y",
        ];

        for format in &datetime_formats {
            if let Ok(datetime) = DateTime::parse_from_str(date_str, format) {
                return Ok(datetime.with_timezone(&Utc));
            }
        }

        // Try parsing other date formats without time
        let date_formats = [
            "%B %e, %Y",
            "%b %e, %Y",
            "%Y/%m/%d",
            "%d/%m/%Y",
            "%m/%d/%Y",
        ];

        for format in &date_formats {
            if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, format) {
                let naive_datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
                return Ok(DateTime::from_naive_utc_and_offset(naive_datetime, Utc));
            }
        }

        // Try parsing as timestamp
        if let Ok(timestamp) = date_str.parse::<i64>() {
            if let Some(datetime) = DateTime::from_timestamp(timestamp, 0) {
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

impl Formatter {

    /// Convert our Locale to ICU4X Locale
    fn locale_to_icu_locale(&self, locale: &Locale) -> IcuLocale {
        // Convert locale string to ICU4X format
        // ICU4X expects BCP-47 format like "en-US", "es-ES", etc.
        let locale_str = locale.to_string();
        IcuLocale::from_str(&locale_str).unwrap_or_else(|_| IcuLocale::UND)
    }

    /// Create ICU4X date formatter for the given locale and style
    fn create_date_formatter(&self, icu_locale: &IcuLocale, style: DateFormatStyle) -> Result<DateTimeFormatter> {
        let options = match style {
            DateFormatStyle::Short => DateTimeFormatterOptions::Length(length::Bag::from_date_style(length::Date::Short)),
            DateFormatStyle::Medium => DateTimeFormatterOptions::Length(length::Bag::from_date_style(length::Date::Medium)),
            DateFormatStyle::Long => DateTimeFormatterOptions::Length(length::Bag::from_date_style(length::Date::Long)),
            DateFormatStyle::Full => DateTimeFormatterOptions::Length(length::Bag::from_date_style(length::Date::Full)),
        };

        DateTimeFormatter::try_new(&DataLocale::from(icu_locale.clone()), options)
            .map_err(|e| I18nError::formatting_error(format!("Failed to create date formatter: {}", e)))
    }

    /// Create ICU4X decimal formatter for the given locale
    fn create_decimal_formatter(&self, icu_locale: &IcuLocale) -> Result<FixedDecimalFormatter> {
        let options = FixedDecimalFormatterOptions::default();
        FixedDecimalFormatter::try_new(&DataLocale::from(icu_locale.clone()), options)
            .map_err(|e| I18nError::formatting_error(format!("Failed to create decimal formatter: {}", e)))
    }

    /// Create ICU4X list formatter for the given locale and length
    fn create_list_formatter(&self, icu_locale: &IcuLocale, length: ListLength) -> Result<ListFormatter> {
        ListFormatter::try_new_and_with_length(&DataLocale::from(icu_locale.clone()), length)
            .map_err(|e| I18nError::formatting_error(format!("Failed to create list formatter: {}", e)))
    }

    /// Get currency symbol for a given currency code and locale
    fn get_currency_symbol(&self, currency_code: &str, locale: &Locale) -> String {
        // For now, use simple currency symbols
        // In a full implementation, this would use ICU4X currency data
        match currency_code {
            "USD" => "$",
            "EUR" => "€",
            "GBP" => "£",
            "JPY" => "¥",
            "CNY" => "¥",
            "KRW" => "₩",
            "INR" => "₹",
            "BRL" => "R$",
            "MXN" => "$",
            "CAD" => "$",
            "AUD" => "$",
            _ => currency_code, // Fallback to currency code
        }.to_string()
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
        let result = formatter.format_list(&items, ListFormatStyle::And, &locale).unwrap();
        assert_eq!(result, "apple, banana, and cherry");
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