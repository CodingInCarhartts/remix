use remix::config::Config;
use remix::packer::pack_repository;
mod common;

#[tokio::test]
async fn test_empty_include_yields_empty_result() {
    let test_dir = common::create_test_repo();
    // common::create_test_repo creates some rust files.

    let config = Config {
        include: vec!["nonexistent.rs".to_string()],
        ..Default::default()
    };
    let result = pack_repository(test_dir.path(), &config).await.unwrap();
    assert!(
        result.files.is_empty(),
        "Result should be empty when include patterns match nothing, found: {:?}",
        result.files
    );
}

#[tokio::test]
async fn test_explicit_binary_include() {
    let test_dir = common::create_test_repo();
    let bin_path = test_dir.path().join("logo.png");
    // Create a fake binary file
    std::fs::write(&bin_path, vec![0u8, 1u8, 2u8, 3u8]).unwrap();

    let config = Config {
        include: vec!["logo.png".to_string()],
        ..Default::default()
    };
    let result = pack_repository(test_dir.path(), &config).await.unwrap();

    assert_eq!(result.files.len(), 1, "Should include exactly 1 file");
    assert_eq!(result.files[0].relative_path, "logo.png");
    assert!(result.files[0].is_binary);
}

#[tokio::test]
async fn test_security_scan_integration() {
    let test_dir = common::create_test_repo();
    let secret_path = test_dir.path().join("secret_config.txt");
    std::fs::write(&secret_path, "AWS_ACCESS_KEY=AKIA1234567890").unwrap();

    let config = Config::default(); // Security check enabled by default

    let result = pack_repository(test_dir.path(), &config).await.unwrap();

    // File should be skipped from content
    assert!(
        !result
            .files
            .iter()
            .any(|f| f.relative_path == "secret_config.txt"),
        "Sensitive file should not be in content"
    );

    // Should be in suspicious files
    match &result.suspicious_files {
        Some(files) => {
            assert!(
                files.contains(&"secret_config.txt".to_string()),
                "Sensitive file should be marked suspicious"
            );
        }
        None => panic!("Should have detected suspicious files"),
    }
}
