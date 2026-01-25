use anyhow::Result;
use remix::formatter::{format_json, format_markdown, format_sqlite_sql, format_text, format_toon};
use remix::packer::{FileContent, PackedRepository, RepositorySummary};
use remix::security::SecurityCheckStatus;

#[test]
fn test_format_toon_basic() -> Result<()> {
    let repo = PackedRepository {
        summary: RepositorySummary {
            file_count: 2,
            directory_count: 1,
            total_size: 100,
            extensions: vec!["txt".to_string(), "rs".to_string()],
            binary_file_count: 0,
        },
        files: vec![
            FileContent {
                relative_path: "file1.txt".to_string(),
                content: "Hello".to_string(),
                size: 5,
                extension: "txt".to_string(),
                is_binary: false,
            },
            FileContent {
                relative_path: "file2.rs".to_string(),
                content: "fn main() {}".to_string(),
                size: 11,
                extension: "rs".to_string(),
                is_binary: false,
            },
        ],
        instruction: Some("Test instruction".to_string()),
        suspicious_files: None,
        security_check_status: remix::security::SecurityCheckStatus::CompletedNoFindings,
        binary_files: None,
    };

    let result = format_toon(&repo)?;
    assert!(!result.is_empty());
    assert!(result.contains("files[2]"));
    assert!(result.contains("file_count: 2"));
    Ok(())
}

#[test]
fn test_format_markdown() -> Result<()> {
    let repo = PackedRepository {
        summary: RepositorySummary {
            file_count: 1,
            directory_count: 0,
            total_size: 10,
            extensions: vec!["md".to_string()],
            binary_file_count: 0,
        },
        files: vec![FileContent {
            relative_path: "README.md".to_string(),
            content: "# Hello\nWorld".to_string(),
            size: 10,
            extension: "md".to_string(),
            is_binary: false,
        }],
        instruction: Some("Test instruction".to_string()),
        suspicious_files: None,
        security_check_status: SecurityCheckStatus::CompletedNoFindings,
        binary_files: None,
    };

    let result = format_markdown(&repo);
    assert!(result.contains("# User Instruction"));
    assert!(result.contains("Test instruction"));
    assert!(result.contains("- **Files:** 1"));
    assert!(result.contains("### README.md"));
    assert!(result.contains("# Hello"));
    Ok(())
}

#[test]
fn test_format_json() -> Result<()> {
    let repo = PackedRepository {
        summary: RepositorySummary {
            file_count: 1,
            directory_count: 0,
            total_size: 5,
            extensions: vec!["txt".to_string()],
            binary_file_count: 0,
        },
        files: vec![FileContent {
            relative_path: "test.txt".to_string(),
            content: "Hello".to_string(),
            size: 5,
            extension: "txt".to_string(),
            is_binary: false,
        }],
        instruction: None,
        suspicious_files: None,
        security_check_status: SecurityCheckStatus::CompletedNoFindings,
        binary_files: None,
    };

    let result = format_json(&repo)?;
    println!("JSON result: {}", result);
    // Should be valid JSON
    let json: serde_json::Value = serde_json::from_str(&result)?;
    assert_eq!(json["summary"]["file_count"], 1);
    assert_eq!(json["files"][0]["relative_path"], "test.txt");
    assert_eq!(json["files"][0]["content"], "Hello");
    Ok(())
}

#[test]
fn test_format_text() -> Result<()> {
    let repo = PackedRepository {
        summary: RepositorySummary {
            file_count: 1,
            directory_count: 0,
            total_size: 5,
            extensions: vec!["txt".to_string()],
            binary_file_count: 0,
        },
        files: vec![FileContent {
            relative_path: "test.txt".to_string(),
            content: "Hello".to_string(),
            size: 5,
            extension: "txt".to_string(),
            is_binary: false,
        }],
        instruction: Some("Instruction".to_string()),
        suspicious_files: None,
        security_check_status: SecurityCheckStatus::CompletedNoFindings,
        binary_files: None,
    };

    let result = format_text(&repo);
    assert!(result.contains("USER INSTRUCTION:"));
    assert!(result.contains("Instruction"));
    assert!(result.contains("Files: 1"));
    assert!(result.contains("FILE: test.txt"));
    assert!(result.contains("Hello"));
    Ok(())
}

#[test]
fn test_format_toon_empty_repo() -> Result<()> {
    let repo = PackedRepository {
        summary: RepositorySummary {
            file_count: 0,
            directory_count: 0,
            total_size: 0,
            extensions: vec![],
            binary_file_count: 0,
        },
        files: vec![],
        instruction: None,
        suspicious_files: None,
        security_check_status: remix::security::SecurityCheckStatus::CompletedNoFindings,
        binary_files: None,
    };

    let result = format_toon(&repo)?;
    assert!(!result.is_empty());
    assert!(result.contains("file_count: 0"));
    Ok(())
}

#[test]
fn test_format_sqlite_basic() -> Result<()> {
    let repo = PackedRepository {
        summary: RepositorySummary {
            file_count: 2,
            directory_count: 1,
            total_size: 100,
            extensions: vec!["txt".to_string(), "rs".to_string()],
            binary_file_count: 0,
        },
        files: vec![
            FileContent {
                relative_path: "file1.txt".to_string(),
                content: "Hello".to_string(),
                size: 5,
                extension: "txt".to_string(),
                is_binary: false,
            },
            FileContent {
                relative_path: "file2.rs".to_string(),
                content: "fn main() {}".to_string(),
                size: 11,
                extension: "rs".to_string(),
                is_binary: false,
            },
        ],
        instruction: Some("Test instruction".to_string()),
        suspicious_files: None,
        security_check_status: remix::security::SecurityCheckStatus::CompletedNoFindings,
        binary_files: None,
    };

    let result = format_sqlite_sql(&repo)?;
    assert!(!result.is_empty());

    // Verify SQL structure
    assert!(result.contains("-- SQLite SQL script generated by remix"));
    assert!(result.contains("CREATE TABLE remix_runs"));
    assert!(result.contains("CREATE TABLE remix_summary"));
    assert!(result.contains("CREATE TABLE remix_files"));
    assert!(result.contains("CREATE TABLE remix_extensions"));

    // Verify data insertion
    assert!(result.contains("INSERT INTO remix_runs"));
    assert!(result.contains("INSERT INTO remix_summary"));
    assert!(result.contains("INSERT INTO remix_files"));
    assert!(result.contains("INSERT INTO remix_extensions"));

    // Verify file content is present
    assert!(result.contains("Hello"));
    assert!(result.contains("fn main() {}"));

    Ok(())
}

#[test]
fn test_format_sqlite_escaping() -> Result<()> {
    let repo = PackedRepository {
        summary: RepositorySummary {
            file_count: 1,
            directory_count: 0,
            total_size: 20,
            extensions: vec!["txt".to_string()],
            binary_file_count: 0,
        },
        files: vec![FileContent {
            relative_path: "test.txt".to_string(),
            content: "O'Reilly and a newline\ninside".to_string(),
            size: 25,
            extension: "txt".to_string(),
            is_binary: false,
        }],
        instruction: Some("Instruction with 'quotes'".to_string()),
        suspicious_files: None,
        security_check_status: SecurityCheckStatus::CompletedNoFindings,
        binary_files: None,
    };

    let result = format_sqlite_sql(&repo)?;
    assert!(!result.is_empty());

    // Verify single quotes are escaped (doubled)
    assert!(result.contains("O''Reilly"));
    assert!(result.contains("''quotes''"));

    // Verify newlines are preserved in the content
    assert!(result.contains("newline"));

    // Verify transaction wrapping
    assert!(result.contains("BEGIN TRANSACTION"));
    assert!(result.contains("COMMIT TRANSACTION"));

    Ok(())
}

#[test]
fn test_format_sqlite_empty_repo() -> Result<()> {
    let repo = PackedRepository {
        summary: RepositorySummary {
            file_count: 0,
            directory_count: 0,
            total_size: 0,
            extensions: vec![],
            binary_file_count: 0,
        },
        files: vec![],
        instruction: None,
        suspicious_files: None,
        security_check_status: remix::security::SecurityCheckStatus::CompletedNoFindings,
        binary_files: None,
    };

    let result = format_sqlite_sql(&repo)?;
    assert!(!result.is_empty());
    assert!(result.contains("CREATE TABLE remix_runs"));
    assert!(result.contains("INSERT INTO remix_runs"));
    Ok(())
}

#[test]
fn test_format_sqlite_with_security_findings() -> Result<()> {
    let repo = PackedRepository {
        summary: RepositorySummary {
            file_count: 1,
            directory_count: 0,
            total_size: 10,
            extensions: vec!["env".to_string()],
            binary_file_count: 0,
        },
        files: vec![FileContent {
            relative_path: ".env".to_string(),
            content: "API_KEY=secret".to_string(),
            size: 15,
            extension: "env".to_string(),
            is_binary: false,
        }],
        instruction: None,
        suspicious_files: Some(vec![".env".to_string()]),
        security_check_status: SecurityCheckStatus::CompletedWithFindings,
        binary_files: None,
    };

    let result = format_sqlite_sql(&repo)?;
    assert!(!result.is_empty());

    // Verify suspicious files table is created and populated
    assert!(result.contains("CREATE TABLE remix_suspicious_files"));
    assert!(result.contains("INSERT INTO remix_suspicious_files"));
    assert!(result.contains(".env"));
    assert!(result.contains("CompletedWithFindings"));

    Ok(())
}

#[test]
fn test_format_sqlite_with_binary_files() -> Result<()> {
    let repo = PackedRepository {
        summary: RepositorySummary {
            file_count: 1,
            directory_count: 0,
            total_size: 100,
            extensions: vec!["png".to_string()],
            binary_file_count: 1,
        },
        files: vec![],
        instruction: None,
        suspicious_files: None,
        security_check_status: SecurityCheckStatus::CompletedNoFindings,
        binary_files: Some(vec!["image.png".to_string()]),
    };

    let result = format_sqlite_sql(&repo)?;
    assert!(!result.is_empty());

    // Verify binary files table is created and populated
    assert!(result.contains("CREATE TABLE remix_binary_files"));
    assert!(result.contains("INSERT INTO remix_binary_files"));
    assert!(result.contains("image.png"));

    Ok(())
}
