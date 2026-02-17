//! Session deletion module
//!
//! Provides functionality to permanently delete Claude Code sessions
//! by removing the JSONL file and any associated companion directory.

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::command;

lazy_static! {
    /// Regex for validating JSONL filename pattern (alphanumeric, underscore, hyphen only)
    static ref FILENAME_REGEX: Regex = Regex::new(r"^[A-Za-z0-9_-]+$").unwrap();
}

/// Result structure for delete operations
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteSessionResult {
    pub success: bool,
    pub file_path: String,
    pub companion_dir_deleted: bool,
}

/// Deletes a Claude Code session file and its optional companion directory.
///
/// # Arguments
/// * `file_path` - Absolute path to the session JSONL file
///
/// # Returns
/// * `Ok(DeleteSessionResult)` - Success with deletion details
/// * `Err(String)` - Error description
///
/// # Security
/// - Path must be absolute
/// - No symlinks allowed
/// - File must be within ~/.claude directory
/// - Filename must match safe pattern
#[command]
pub async fn delete_session(file_path: String) -> Result<DeleteSessionResult, String> {
    let file_path_buf = std::path::PathBuf::from(&file_path);

    // 1. Validate file exists
    if !file_path_buf.exists() {
        return Err(format!("Session file not found: {file_path}"));
    }

    // 2. Validate path is within ~/.claude (reuse security checks from rename module)
    validate_delete_path(&file_path)?;

    // 3. Delete the JSONL file
    fs::remove_file(&file_path_buf).map_err(|e| format!("Failed to delete session file: {e}"))?;

    // 4. Delete companion directory if it exists (same name without .jsonl extension)
    let companion_dir = file_path_buf.with_extension("");
    let companion_dir_deleted = if companion_dir.is_dir() {
        fs::remove_dir_all(&companion_dir).map_err(|e| {
            format!("Session file deleted but failed to remove companion directory: {e}")
        })?;
        true
    } else {
        false
    };

    Ok(DeleteSessionResult {
        success: true,
        file_path,
        companion_dir_deleted,
    })
}

/// Validates that the file path is safe for deletion.
///
/// Security checks:
/// 1. Path must be absolute
/// 2. No symlinks in any path component
/// 3. Filename must match safe pattern
/// 4. File must be within ~/.claude directory
fn validate_delete_path(file_path: &str) -> Result<(), String> {
    let file_path_buf = std::path::PathBuf::from(file_path);

    // 1. Require absolute path
    if !file_path_buf.is_absolute() {
        return Err("File path must be absolute".to_string());
    }

    // 2. Block symlinks in path components
    let mut current = file_path_buf.as_path();
    while let Some(parent) = current.parent() {
        if parent.as_os_str().is_empty() {
            break;
        }
        if let Ok(metadata) = fs::symlink_metadata(parent) {
            if metadata.file_type().is_symlink() {
                return Err("Symlinks are not allowed in path".to_string());
            }
        }
        current = parent;
    }

    // Check the file itself for symlinks
    if let Ok(metadata) = fs::symlink_metadata(&file_path_buf) {
        if metadata.file_type().is_symlink() {
            return Err("File path cannot be a symlink".to_string());
        }
    }

    // 3. Validate filename pattern
    if let Some(filename) = file_path_buf.file_stem() {
        let filename_str = filename.to_string_lossy();
        if !FILENAME_REGEX.is_match(&filename_str) {
            return Err(
                "Filename must contain only alphanumeric characters, underscores, and hyphens"
                    .to_string(),
            );
        }
    } else {
        return Err("Invalid filename".to_string());
    }

    // 4. Verify file is within ~/.claude
    let canonical_path = file_path_buf
        .canonicalize()
        .map_err(|e| format!("Failed to resolve path: {e}"))?;

    let home_dir = dirs::home_dir().ok_or_else(|| "Cannot determine home directory".to_string())?;

    let claude_dir = home_dir.join(".claude");

    // Canonicalize claude_dir too so both paths use the same format
    // (on Windows, canonicalize adds \\?\ prefix)
    let canonical_claude_dir = claude_dir.canonicalize().unwrap_or(claude_dir);

    if !canonical_path.starts_with(&canonical_claude_dir) {
        return Err("File path must be within ~/.claude directory".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_delete_path_rejects_relative_path() {
        let result = validate_delete_path("relative/path/file.jsonl");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be absolute"));
    }

    #[test]
    fn test_validate_delete_path_rejects_non_claude_directory() {
        let result = validate_delete_path("/tmp/validfilename.jsonl");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_delete_path_valid_path() {
        if let Some(home) = dirs::home_dir() {
            let claude_projects = home.join(".claude/projects");
            if claude_projects.exists() {
                if let Ok(projects) = fs::read_dir(&claude_projects) {
                    for project in projects.flatten() {
                        if project.path().is_dir() {
                            if let Ok(files) = fs::read_dir(project.path()) {
                                for file in files.flatten() {
                                    let path = file.path();
                                    if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                                        let test_path = path.to_string_lossy().to_string();
                                        let result = validate_delete_path(&test_path);
                                        assert!(
                                            result.is_ok(),
                                            "Validation failed for valid path {test_path}: {result:?}"
                                        );
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
