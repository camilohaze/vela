/*!
Cross-platform abstraction layer for desktop applications

This module provides unified APIs across Windows, macOS, and Linux platforms.
*/

use anyhow::Result;

/// File filter for file dialogs
#[derive(Debug, Clone)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// Message box types
#[derive(Debug, Clone)]
pub enum MessageType {
    Info,
    Warning,
    Error,
    Question,
}

/// Message box buttons
#[derive(Debug, Clone)]
pub enum MessageButtons {
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
}

/// Message box result
#[derive(Debug, Clone)]
pub enum MessageResult {
    Ok,
    Cancel,
    Yes,
    No,
}

/// UI theme
#[derive(Debug, Clone)]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// Initialize platform-specific resources
pub fn init() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        crate::platform::windows::init()
    }

    #[cfg(target_os = "macos")]
    {
        crate::platform::macos::init()
    }

    #[cfg(target_os = "linux")]
    {
        crate::platform::linux::init()
    }
}

/// Get platform-specific data directory
pub fn get_data_directory() -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        Ok(crate::platform::windows::get_data_directory("vela")?.to_string_lossy().to_string())
    }

    #[cfg(target_os = "macos")]
    {
        Ok(crate::platform::macos::get_data_directory("vela")?.to_string_lossy().to_string())
    }

    #[cfg(target_os = "linux")]
    {
        Ok(crate::platform::linux::get_data_directory("vela")?.to_string_lossy().to_string())
    }
}

/// Show a message box
pub fn show_message_box(
    title: &str,
    message: &str,
    message_type: MessageType,
    buttons: MessageButtons,
) -> Result<MessageResult> {
    #[cfg(target_os = "windows")]
    {
        crate::platform::windows::show_message_box(title, message, message_type, buttons)
    }

    #[cfg(target_os = "macos")]
    {
        crate::platform::macos::show_message_box(title, message, message_type, buttons)
    }

    #[cfg(target_os = "linux")]
    {
        crate::platform::linux::show_message_box(title, message, message_type, buttons)
    }
}

// Re-export platform-specific modules
pub mod windows;
pub mod macos;
pub mod linux;