use clap::Parser;
use remix::cli::Cli;
use remix::config::{load_config, Config, OutputConfig};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let config = Config::default();

    // Test default values
    assert!(config.include.is_empty());
    assert_eq!(config.max_file_size, 100_000); // 100KB
    assert!(!config.compress);
    assert!(config.security.enable_security_check);
    assert_eq!(config.output.format, "txt");
    assert!(!config.output.open_file);
    assert_eq!(config.output.path, "./remix-output.txt");
}

#[test]
fn test_output_config_default() {
    let output_config = OutputConfig::default();

    assert_eq!(output_config.format, "txt");
    assert!(!output_config.open_file);
    assert_eq!(output_config.path, "./remix-output.txt");
    assert!(output_config.instruction_file_path.is_none());
    assert!(!output_config.remove_comments);
}

#[test]
fn test_load_config_valid() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.json");

    let config_content = r#"
    {
        "include": ["*.rs", "*.toml"],
        "ignore": {
            "custom_patterns": ["target/**"]
        },
        "max_file_size": 50000,
        "compress": true,
        "security": {
            "enable_security_check": false
        },
        "output": {
            "format": "json",
            "open_file": true,
            "path": "custom_output.json",
            "remove_comments": true
        }
    }
    "#;

    fs::write(&config_path, config_content).unwrap();

    let config = load_config(&config_path).unwrap();

    assert_eq!(
        config.include,
        vec!["*.rs".to_string(), "*.toml".to_string()]
    );
    assert_eq!(config.ignore.custom_patterns, vec!["target/**".to_string()]);
    assert_eq!(config.max_file_size, 50000);
    assert!(config.compress);
    assert!(!config.security.enable_security_check);
    assert_eq!(config.output.format, "json");
    assert!(config.output.open_file);
    assert_eq!(config.output.path, "custom_output.json");
    assert!(config.output.remove_comments);
}

#[test]
fn test_load_config_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid_config.json");

    fs::write(&config_path, "{ invalid json }").unwrap();

    let result = load_config(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_load_config_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("missing.json");

    let result = load_config(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_config_merge_with_cli() {
    let config = Config::default();
    let cli = Cli::parse_from([
        "remix",
        "--include",
        "*.rs",
        "--format",
        "toon",
        "--output",
        "output.toon",
        "--compress",
        "--remove-comments",
    ]);

    let merged = config.merge_with_cli(&cli);

    assert_eq!(merged.include, vec!["*.rs".to_string()]);
    assert_eq!(merged.output.format, "toon");
    assert_eq!(merged.output.path, "output.toon");
    assert!(merged.compress);
    assert!(merged.output.remove_comments);
}

#[test]
fn test_config_merge_overrides() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config_content = r#"
    {
        "output": {
            "format": "md",
            "path": "default.md"
        }
    }
    "#;

    fs::write(&config_path, config_content).unwrap();

    let config = load_config(&config_path).unwrap();
    let cli = Cli::parse_from(["remix", "--format", "json", "--output", "cli.json"]);

    let merged = config.merge_with_cli(&cli);

    assert_eq!(merged.output.format, "json");
    assert_eq!(merged.output.path, "cli.json");
}
