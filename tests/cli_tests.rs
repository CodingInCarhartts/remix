use clap::Parser;
use remix::cli::Cli;

#[test]
fn test_cli_parsing() {
    // Test basic CLI parsing
    let cli = Cli::parse_from(["remix", "path/to/repo"]);
    assert_eq!(cli.path, Some("path/to/repo".to_string()));
    assert!(cli.config.is_none());
    assert!(!cli.init);
    assert!(cli.include.is_none());
    assert!(cli.ignore.is_none());
    assert!(!cli.compress);
    assert!(!cli.skip_sensitive_check);
    assert!(!cli.remove_comments);
}

#[test]
fn test_cli_include_ignore_patterns() {
    // Test include patterns parsing
    let cli = Cli::parse_from([
        "remix",
        "--include",
        "*.rs,*.md",
        "--ignore",
        "target/,node_modules/",
    ]);

    let include_patterns = cli.include_patterns().unwrap();
    let ignore_patterns = cli.ignore_patterns().unwrap();

    assert_eq!(include_patterns.len(), 2);
    assert_eq!(include_patterns[0], "*.rs");
    assert_eq!(include_patterns[1], "*.md");

    assert_eq!(ignore_patterns.len(), 2);
    assert_eq!(ignore_patterns[0], "target/");
    assert_eq!(ignore_patterns[1], "node_modules/");
}

#[test]
fn test_cli_remote_repo() {
    // Test remote repository options
    let cli = Cli::parse_from([
        "remix",
        "--remote",
        "username/repo",
        "--remote-branch",
        "main",
    ]);

    assert_eq!(cli.remote, Some("username/repo".to_string()));
    assert_eq!(cli.remote_branch, Some("main".to_string()));
}

#[test]
fn test_cli_format_validation() {
    // Test valid formats
    for format in ["md", "markdown", "json", "txt", "text", "toon"] {
        let cli = Cli::parse_from(["remix", "--format", format]);
        assert_eq!(cli.format, Some(format.to_string()));
    }
}

#[test]
fn test_cli_edge_cases() {
    // Test empty path
    let cli = Cli::parse_from(["remix", ""]);
    assert_eq!(cli.path, Some("".to_string()));

    // Test path with special characters
    let cli = Cli::parse_from(["remix", "/path/with spaces/and@symbols#"]);
    assert_eq!(cli.path, Some("/path/with spaces/and@symbols#".to_string()));

    // Test very long path
    let long_path = "a".repeat(1000);
    let cli = Cli::parse_from(["remix", &long_path]);
    assert_eq!(cli.path, Some(long_path));
}

#[test]
fn test_cli_all_options() {
    // Test all options together
    let cli = Cli::parse_from([
        "remix",
        "/test/path",
        "--config",
        "config.json",
        "--init",
        "--include",
        "*.rs,*.toml",
        "--ignore",
        "target/**,*.log",
        "--max-file-size",
        "50000",
        "--output",
        "output.md",
        "--format",
        "json",
        "--compress",
        "--skip-sensitive-check",
        "--remote",
        "https://github.com/user/repo",
        "--remote-branch",
        "develop",
        "--open",
        "--instruction",
        "Test instruction",
        "--instruction-file",
        "instructions.txt",
        "--remove-comments",
        "--no-gitignore",
        "--no-default-patterns",
    ]);

    assert_eq!(cli.path, Some("/test/path".to_string()));
    assert_eq!(cli.config, Some("config.json".into()));
    assert!(cli.init);
    assert_eq!(cli.include, Some("*.rs,*.toml".to_string()));
    assert_eq!(cli.ignore, Some("target/**,*.log".to_string()));
    assert_eq!(cli.max_file_size, Some(50000));
    assert_eq!(cli.output, Some("output.md".into()));
    assert_eq!(cli.format, Some("json".to_string()));
    assert!(cli.compress);
    assert!(cli.skip_sensitive_check);
    assert_eq!(cli.remote, Some("https://github.com/user/repo".to_string()));
    assert_eq!(cli.remote_branch, Some("develop".to_string()));
    assert!(cli.open);
    assert_eq!(cli.instruction, Some("Test instruction".to_string()));
    assert_eq!(cli.instruction_file, Some("instructions.txt".into()));
    assert!(cli.remove_comments);
    assert!(cli.no_gitignore);
    assert!(cli.no_default_patterns);
}

#[test]
fn test_cli_pattern_parsing_edge_cases() {
    // Test empty patterns
    let cli = Cli::parse_from(["remix", "--include", "", "--ignore", ""]);
    assert!(cli.include_patterns().unwrap().is_empty());
    assert!(cli.ignore_patterns().unwrap().is_empty());

    // Test patterns with spaces
    let cli = Cli::parse_from(["remix", "--include", " *.rs , *.md "]);
    let patterns = cli.include_patterns().unwrap();
    assert_eq!(patterns.len(), 2);
    assert_eq!(patterns[0], "*.rs");
    assert_eq!(patterns[1], "*.md");

    // Test single pattern
    let cli = Cli::parse_from(["remix", "--include", "*.rs"]);
    let patterns = cli.include_patterns().unwrap();
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0], "*.rs");
}
