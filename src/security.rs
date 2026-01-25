use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Clone)]
pub enum SecurityCheckStatus {
    Disabled,
    CompletedNoFindings,
    CompletedWithFindings,
    #[allow(dead_code)]
    Failed(String),
}

/// Checks if a filename contains suspicious patterns
pub fn check_suspicious_filename(path: &Path) -> bool {
    let filename = path.to_string_lossy().to_lowercase();
    filename.contains("secret")
        || filename.contains("password")
        || filename.contains("credential")
        || filename.contains("token")
        || filename.contains("key")
        || filename.contains("auth")
        || filename.contains(".env")
        || filename.contains("config")
}

/// Returns a list of sensitive keywords to look for
fn get_sensitive_keywords() -> Vec<String> {
    vec![
        // API Keys and Tokens
        "api_key".to_string(),
        "api_token".to_string(),
        "app_key".to_string(),
        "app_token".to_string(),
        "secret_key".to_string(),
        // AWS Keys
        "aws_access_key".to_string(),
        "aws_secret_key".to_string(),
        // Private Keys
        "private key".to_string(),
        "begin private key".to_string(),
        // Passwords
        "password=".to_string(),
        "passwd=".to_string(),
        "pwd=".to_string(),
        // Firebase keys
        "firebase_key".to_string(),
        // Generic Auth Tokens
        "auth_token".to_string(),
        "bearer token".to_string(),
        // Connection Strings
        "connection_string".to_string(),
        "mongodb://".to_string(),
        "postgres://".to_string(),
        "mysql://".to_string(),
        "redis://".to_string(),
    ]
}

/// Check if content contains sensitive information
pub fn check_sensitive_content(content: &str) -> bool {
    let sensitive_keywords = get_sensitive_keywords();
    let content_lower = content.to_lowercase();

    for keyword in sensitive_keywords {
        if content_lower.contains(&keyword) {
            return true;
        }
    }

    false
}

/// Recursively checks a directory for security issues
#[allow(dead_code)]
pub fn perform_security_check(path: &Path) -> Result<Vec<PathBuf>> {
    let mut suspicious_files = Vec::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() {
            // Check filename
            if check_suspicious_filename(path) {
                suspicious_files.push(path.to_path_buf());
                continue;
            }

            // Check content (skip binary files)
            let mime = tree_magic_mini::from_filepath(path);
            if mime.is_some_and(|m| m.starts_with("text/")) {
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        if check_sensitive_content(&content) {
                            suspicious_files.push(path.to_path_buf());
                        }
                    }
                    Err(_) => {
                        // Read error - skip content check
                        continue;
                    }
                }
            }
        }
    }

    Ok(suspicious_files)
}
