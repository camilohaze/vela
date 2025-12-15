/*!
Linux-specific platform implementations (stub)
*/

use std::path::PathBuf;
use anyhow::Result;
use super::{FileFilter, MessageType, MessageButtons, MessageResult, Theme};

pub fn init() -> Result<()> {
    // Stub implementation
    Ok(())
}

pub fn get_data_directory(_app_name: &str) -> Result<PathBuf> {
    // Stub implementation
    Ok(PathBuf::from("/stub/data"))
}

pub fn get_config_directory(_app_name: &str) -> Result<PathBuf> {
    // Stub implementation
    Ok(PathBuf::from("/stub/config"))
}

pub fn get_cache_directory(_app_name: &str) -> Result<PathBuf> {
    // Stub implementation
    Ok(PathBuf::from("/stub/cache"))
}

pub fn show_file_dialog(_title: &str, _filters: &[FileFilter], _multiple: bool) -> Result<Vec<PathBuf>> {
    // Stub implementation
    Ok(vec![])
}

pub fn show_message_box(_title: &str, _message: &str, _message_type: MessageType, _buttons: MessageButtons) -> Result<MessageResult> {
    // Stub implementation
    Ok(MessageResult::Ok)
}

pub fn open_url(url: &str) -> Result<()> {
    use std::process::Command;
    Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}

pub fn get_system_theme() -> Theme {
    // Stub implementation
    Theme::System
}