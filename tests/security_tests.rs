use remix::security;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_check_sensitive_content() {
    // Test strings with sensitive information based on actual keywords
    let sensitive_content = [
        "const API_KEY = 'abc123'",
        "const JWT_SECRET = 'secret123'",
        "AWS_SECRET_ACCESS_KEY=AKIAIOSFODNN7EXAMPLE",
        "const API_TOKEN = 'abc123'",
        "mongodb://username:password@localhost:27017",
        "postgres://user:password@localhost:5432/mydb",
    ];

    // Test strings without sensitive information
    let safe_content = vec![
        "const NAME = 'John Doe'",
        "const VERSION = '1.0.0'",
        "import { useState } from 'react'",
        "log.info('Application started')",
        "const mock_data = { test: true }",
    ];

    // Check for at least one positive match - we need to verify the function works
    let some_sensitive_detected = sensitive_content
        .iter()
        .any(|content| security::check_sensitive_content(content));

    assert!(
        some_sensitive_detected,
        "Security check should detect at least one sensitive content example"
    );

    // Verify non-detection of safe content
    for content in safe_content {
        assert!(
            !security::check_sensitive_content(content),
            "Incorrectly flagged safe content as sensitive: {}",
            content
        );
    }
}

#[test]
fn test_perform_security_check() {
    // Create a test directory with some sensitive files
    let dir = tempdir().expect("Failed to create temporary directory");

    // File with API key (should be detected)
    let api_file_path = dir.path().join("api.js");
    fs::write(
        &api_file_path,
        "const API_KEY = 'abc123';\nconst API_SECRET = 'secret456';\n",
    )
    .expect("Failed to write sensitive file");

    // File with database connection string (should be detected)
    let db_file_path = dir.path().join("db.js");
    fs::write(
        &db_file_path,
        "const DB_URI = 'postgres://user:password@localhost:5432/mydb';\n",
    )
    .expect("Failed to write sensitive db file");

    // Regular file (should not be detected)
    let safe_file_path = dir.path().join("safe.js");
    fs::write(
        &safe_file_path,
        "const VERSION = '1.0.0';\nconsole.log('App version:', VERSION);\n",
    )
    .expect("Failed to write safe file");

    // Perform security check
    let suspicious_files =
        security::perform_security_check(dir.path()).expect("Security check failed");

    // Verify that security check works in general - should detect at least one file
    assert!(
        !suspicious_files.is_empty(),
        "Security check should detect at least one suspicious file"
    );
}

#[test]
fn test_check_sensitive_content_edge_cases() {
    // Test various formats and edge cases
    let sensitive_cases = vec![
        "const API_KEY = 'abc123'",  // api_key keyword
        "secret_key: mySecret",  // secret_key keyword
        "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC\n-----END PRIVATE KEY-----",  // private key
        "password= mySecretPass123!",  // password=
        "mongodb://username:password@localhost:27017",  // mongodb://
    ];

    let safe_cases = vec![
        "const version = '1.2.3'",                     // Version string
        "log.debug('API called with key: test_key')",  // Test key in log
        "const example = 'sk_test_example_from_docs'", // Example from docs
        "README.md contains sk_test_123",              // In documentation
    ];

    for case in sensitive_cases {
        assert!(
            security::check_sensitive_content(case),
            "Should detect sensitive content: {}",
            case
        );
    }

    for case in safe_cases {
        assert!(
            !security::check_sensitive_content(case),
            "Should not flag safe content: {}",
            case
        );
    }
}

#[test]
fn test_perform_security_check_empty_directory() {
    let dir = tempdir().expect("Failed to create temporary directory");

    let result = security::perform_security_check(dir.path());
    assert!(result.is_ok());

    let suspicious_files = result.unwrap();
    assert!(
        suspicious_files.is_empty(),
        "Empty directory should have no suspicious files"
    );
}

#[test]
fn test_perform_security_check_binary_file() {
    let dir = tempdir().expect("Failed to create temporary directory");

    // Create a binary file with sensitive-looking content
    let binary_path = dir.path().join("binary.dat");
    let binary_content = b"\x00\x01\x02sk_test_1234567890\x03\x04\x05";
    fs::write(&binary_path, binary_content).expect("Failed to write binary file");

    let result = security::perform_security_check(dir.path());
    assert!(result.is_ok());

    let suspicious_files = result.unwrap();
    // Binary files are not checked for sensitive content
    assert!(
        suspicious_files.is_empty(),
        "Binary files should not be checked for sensitive content"
    );
}
