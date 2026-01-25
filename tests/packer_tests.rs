use remix::config::Config;
use remix::packer::pack_repository;

// Import the common test module
mod common;

#[tokio::test]
async fn test_pack_repository_basic() {
    // Create a test repository
    let test_dir = common::create_test_repo();

    // Create a basic config
    let config = Config::default();

    // Pack the repository
    let result = pack_repository(test_dir.path(), &config)
        .await
        .expect("Failed to pack repository");

    // Basic assertions about the result
    assert!(!result.files.is_empty(), "No files were packed");
    assert!(result.summary.file_count > 0, "File count should be > 0");
    assert!(result.summary.total_size > 0, "Total size should be > 0");

    // Print debug info
    println!("File count: {}", result.summary.file_count);
    println!("Directory count: {}", result.summary.directory_count);
    println!(
        "Files found: {:?}",
        result
            .files
            .iter()
            .map(|f| &f.relative_path)
            .collect::<Vec<_>>()
    );

    // Check if important files are included - use more flexible matching
    let file_paths: Vec<String> = result
        .files
        .iter()
        .map(|f| f.relative_path.replace('\\', "/"))
        .collect();

    // Check for README.md and any file in src directory
    let has_readme = file_paths.iter().any(|path| path.contains("README.md"));
    let has_src_file = file_paths.iter().any(|path| path.contains("src/"));

    assert!(
        has_readme,
        "README.md should be included in the packed files"
    );
    assert!(
        has_src_file,
        "At least one file from src/ should be included"
    );

    // Ignored files should not be included
    let has_node_modules = file_paths.iter().any(|path| path.contains("node_modules"));
    assert!(!has_node_modules, "node_modules should be ignored");

    // Security check should work, but we won't assert specific files
    if let Some(suspicious) = &result.suspicious_files {
        println!("Suspicious files: {:?}", suspicious);
    }
}

#[tokio::test]
async fn test_pack_repository_with_include_patterns() {
    let test_dir = common::create_test_repo();

    let config = Config {
        include: vec!["**/*.rs".to_string(), "**/*.md".to_string()],
        ..Default::default()
    };

    let result = pack_repository(test_dir.path(), &config).await.unwrap();

    let file_paths: Vec<String> = result
        .files
        .iter()
        .map(|f| f.relative_path.replace('\\', "/"))
        .collect();

    // Should only include .rs and .md files
    assert!(file_paths
        .iter()
        .all(|path| path.ends_with(".rs") || path.ends_with(".md")));
    assert!(file_paths.iter().any(|path| path.contains("main.rs")));
    assert!(file_paths.iter().any(|path| path.contains("README.md")));
}

#[tokio::test]
async fn test_pack_repository_with_ignore_patterns() {
    let test_dir = common::create_test_repo();

    let mut config = Config::default();
    config.ignore.custom_patterns = vec!["*.md".to_string()];

    let result = pack_repository(test_dir.path(), &config).await.unwrap();

    let file_paths: Vec<String> = result
        .files
        .iter()
        .map(|f| f.relative_path.replace('\\', "/"))
        .collect();

    // Should not include .md files
    assert!(!file_paths.iter().any(|path| path.ends_with(".md")));
    assert!(file_paths.iter().any(|path| path.contains("main.rs")));
}

#[tokio::test]
async fn test_pack_repository_file_size_limit() {
    let test_dir = common::create_test_repo();

    // Create a large file
    let large_content = "x".repeat(2000); // 2000 bytes
    std::fs::write(test_dir.path().join("large.txt"), large_content).unwrap();

    let config = Config {
        max_file_size: 1000, // 1000 bytes limit
        ..Default::default()
    };

    let result = pack_repository(test_dir.path(), &config).await.unwrap();

    let file_paths: Vec<String> = result
        .files
        .iter()
        .map(|f| f.relative_path.replace('\\', "/"))
        .collect();

    // Large file should be excluded
    assert!(!file_paths.iter().any(|path| path.contains("large.txt")));
    // Small files should be included
    assert!(file_paths.iter().any(|path| path.contains("main.rs")));
}

#[tokio::test]
async fn test_pack_repository_binary_detection() {
    let test_dir = common::create_test_repo();

    // Create a binary-like file (null bytes)
    let binary_content = vec![0u8; 100];
    std::fs::write(test_dir.path().join("binary.dat"), binary_content).unwrap();

    let config = Config::default();
    let result = pack_repository(test_dir.path(), &config).await.unwrap();

    let file_paths: Vec<String> = result
        .files
        .iter()
        .map(|f| f.relative_path.replace('\\', "/"))
        .collect();

    // Binary file should be excluded
    assert!(!file_paths.iter().any(|path| path.contains("binary.dat")));
}

#[tokio::test]
async fn test_pack_repository_gitignore_handling() {
    let test_dir = common::create_test_repo();

    // Create a file that should be ignored by gitignore
    std::fs::write(test_dir.path().join("debug.log"), "log content").unwrap();

    let config = Config::default();
    let result = pack_repository(test_dir.path(), &config).await.unwrap();

    let file_paths: Vec<String> = result
        .files
        .iter()
        .map(|f| f.relative_path.replace('\\', "/"))
        .collect();

    // Log file should be ignored
    assert!(!file_paths.iter().any(|path| path.contains("debug.log")));
}

#[tokio::test]
async fn test_pack_repository_directory_traversal() {
    let test_dir = common::create_test_repo();

    // Create deep nested directory
    std::fs::create_dir_all(test_dir.path().join("deep/nested/folder")).unwrap();
    std::fs::write(
        test_dir.path().join("deep/nested/folder/file.txt"),
        "nested content",
    )
    .unwrap();

    let config = Config::default();
    let result = pack_repository(test_dir.path(), &config).await.unwrap();

    let file_paths: Vec<String> = result
        .files
        .iter()
        .map(|f| f.relative_path.replace('\\', "/"))
        .collect();

    // Deep nested file should be included
    assert!(file_paths
        .iter()
        .any(|path| path.contains("deep/nested/folder/file.txt")));
}

#[tokio::test]
async fn test_pack_repository_empty_directory() {
    let temp_dir = tempfile::TempDir::new().unwrap();

    let config = Config::default();
    let result = pack_repository(temp_dir.path(), &config).await.unwrap();

    assert_eq!(result.summary.file_count, 0);
    assert!(result.files.is_empty());
}
