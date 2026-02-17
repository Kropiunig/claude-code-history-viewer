//! Session resume module
//!
//! Provides functionality to continue a Claude Code session
//! by opening a terminal with `claude --resume <session-id>`.

use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;
use tauri::command;

lazy_static! {
    /// Regex for validating session ID (UUID format: alphanumeric and hyphens)
    static ref SESSION_ID_REGEX: Regex = Regex::new(r"^[A-Za-z0-9_-]+$").unwrap();
}

/// Opens a terminal and resumes the given Claude Code session.
///
/// # Arguments
/// * `session_id` - The actual session ID (UUID) to resume
///
/// # Security
/// - Session ID is validated against a safe pattern
/// - Only `claude --resume` command is executed
#[command]
pub async fn resume_session(session_id: String) -> Result<(), String> {
    // Validate session ID format
    if session_id.is_empty() || !SESSION_ID_REGEX.is_match(&session_id) {
        return Err("Invalid session ID format".to_string());
    }

    open_terminal_with_command(&format!("claude --resume {session_id}"))
}

/// Opens a platform-specific terminal with the given command.
fn open_terminal_with_command(cmd: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // On Windows, open a new cmd.exe window with the command
        Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", cmd])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, use osascript to open Terminal.app
        let script = format!(
            "tell application \"Terminal\"\n  activate\n  do script \"{}\"\nend tell",
            cmd.replace('\\', "\\\\").replace('"', "\\\"")
        );
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {e}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try common terminal emulators in order of preference
        let terminals = [
            ("x-terminal-emulator", vec!["-e", cmd]),
            ("gnome-terminal", vec!["--", "bash", "-c", cmd]),
            ("konsole", vec!["-e", cmd]),
            ("xfce4-terminal", vec!["-e", cmd]),
            ("xterm", vec!["-e", cmd]),
        ];

        for (terminal, args) in &terminals {
            if Command::new(terminal).args(args).spawn().is_ok() {
                return Ok(());
            }
        }

        return Err("No supported terminal emulator found".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_session_id() {
        assert!(SESSION_ID_REGEX.is_match("2df568e6-f193-4037-a3ba-a8f901ebc722"));
    }

    #[test]
    fn test_invalid_session_id_with_spaces() {
        assert!(!SESSION_ID_REGEX.is_match("invalid session id"));
    }

    #[test]
    fn test_invalid_session_id_with_special_chars() {
        assert!(!SESSION_ID_REGEX.is_match("test;rm -rf /"));
    }

    #[test]
    fn test_empty_session_id() {
        assert!(!SESSION_ID_REGEX.is_match(""));
    }
}
