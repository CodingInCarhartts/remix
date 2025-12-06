use crate::config::Config;
use anyhow::{Context, Result};
use glob::{glob_with, MatchOptions};
use ignore::{
    gitignore::{Gitignore, GitignoreBuilder},
    WalkBuilder,
};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tree_magic_mini as tree_magic;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub size: u64,
    pub mime_type: String,
    pub is_binary: bool,
}

impl FileInfo {
    pub fn new(path: PathBuf, base_path: &Path) -> Result<Self> {
        let metadata = fs::metadata(&path)
            .context(format!("Failed to get metadata for {}", path.display()))?;

        let size = metadata.len();

        // Detect the MIME type of the file
        let mime_type = tree_magic::from_filepath(&path).unwrap_or("application/octet-stream");

        // Determine if the file is binary based on its MIME type
        let is_binary = mime_type.starts_with("application/")
            && !mime_type.contains("json")
            && !mime_type.contains("xml")
            && !mime_type.contains("javascript")
            && !mime_type.contains("typescript")
            || mime_type.starts_with("image/")
            || mime_type.starts_with("audio/")
            || mime_type.starts_with("video/");

        let relative_path = path.strip_prefix(base_path).unwrap_or(&path).to_path_buf();

        Ok(Self {
            path,
            relative_path,
            size,
            mime_type: mime_type.to_string(),
            is_binary,
        })
    }
}

/// Read a .remixignore file if it exists
fn read_mixignore(base_path: &Path) -> Option<Gitignore> {
    let mixignore_path = base_path.join(".remixignore");
    if mixignore_path.exists() {
        debug!("Found .remixignore file");
        let mut builder = GitignoreBuilder::new(base_path);
        if builder
            .add_line(
                None,
                &fs::read_to_string(mixignore_path).unwrap_or_default(),
            )
            .is_ok()
        {
            return builder.build().ok();
        }
    }
    None
}

/// Get the default ignore patterns
fn get_default_ignore_patterns() -> Vec<&'static str> {
    vec![
        "node_modules/",
        ".git/",
        ".gitignore",
        ".gitattributes",
        ".github/",
        ".gitmodules",
        ".gitkeep",
        "target/",
        "dist/",
        "build/",
        "**/*.log",
        "**/Cargo.lock",
        "**/.env",
        "**/*.exe",
        "**/*.o",
        "**/*.so",
        "**/*.dll",
        "**/*.dylib",
        "**/*.zip",
        "**/*.tar",
        "**/*.gz",
        "**/*.rar",
        "**/*.7z",
        "**/*.jar",
        "**/*.class",
        "**/*.pyc",
        "**/__pycache__/",
        "**/.idea/",
        "**/.vscode/",
        "**/node_modules/",
        "**/vendor/",
        "**/bin/",
        "**/obj/",
        "**/build/",
    ]
}

/// Normalize a path string for glob matching
fn normalize_path(path: &str) -> String {
    // Convert Windows backslashes to forward slashes for consistent glob matching
    path.replace('\\', "/")
}

/// Check if a path should be ignored based on common patterns
/// Used both by scanner and security check
pub fn should_ignore_common(path: &Path) -> bool {
    let path_str = normalize_path(&path.to_string_lossy());

    // Common directories that should always be ignored
    if path_str.contains("/target/")
        || path_str.starts_with("target/")
        || path_str.contains("/.git/")
        || path_str.starts_with(".git/")
        || path_str.contains("/node_modules/")
        || path_str.starts_with("node_modules/")
        || path_str.contains("/dist/")
        || path_str.starts_with("dist/")
        || path_str.contains("/build/")
        || path_str.starts_with("build/")
    {
        return true;
    }

    // Common files that should always be ignored
    if path_str.ends_with(".exe")
        || path_str.ends_with(".o")
        || path_str.ends_with(".obj")
        || path_str.ends_with(".dll")
        || path_str.ends_with(".so")
        || path_str.ends_with(".dylib")
        || path_str.ends_with(".class")
        || path_str.ends_with(".jar")
        || path_str.ends_with(".war")
        || path_str.ends_with(".zip")
        || path_str.ends_with(".tar")
        || path_str.ends_with(".gz")
        || path_str.ends_with(".rar")
        || path_str.ends_with(".7z")
        || path_str.ends_with(".pyc")
    {
        return true;
    }

    false
}

pub fn scan_repository(base_path: &Path, config: &Config) -> Result<Vec<FileInfo>> {
    info!("Scanning repository at {}", base_path.display());

    // Create a progress bar for scanning
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message("Scanning repository files...");

    // Initialize the ignore system with the appropriate layers
    let should_ignore = |path: &Path| -> bool {
        // First check common patterns that should always be ignored
        if should_ignore_common(path) {
            return true;
        }

        let relative_path = path.strip_prefix(base_path).unwrap_or(path);
        let path_str = normalize_path(&relative_path.to_string_lossy());

        // Layer 1 (highest priority): Custom ignore patterns
        if !config.ignore.custom_patterns.is_empty() {
            for pattern in &config.ignore.custom_patterns {
                // Normalize the pattern too
                let pattern = normalize_path(pattern);

                if glob_match::glob_match(&pattern, &path_str) {
                    debug!(
                        "Ignoring '{}' due to custom pattern '{}'",
                        path_str, pattern
                    );
                    return true;
                }
            }
        }

        // Layer 2: .remixignore file
        if config.ignore.use_mixignore {
            if let Some(ref mixignore) = read_mixignore(base_path) {
                let result = mixignore.matched_path_or_any_parents(relative_path, false);
                if result.is_ignore() {
                    debug!("Ignoring '{}' due to .remixignore", path_str);
                    return true;
                }
            }
        }

        // Layer 3: Default ignore patterns if enabled
        if config.ignore.use_default_patterns {
            for pattern in get_default_ignore_patterns() {
                if glob_match::glob_match(pattern, &path_str) {
                    debug!(
                        "Ignoring '{}' due to default pattern '{}'",
                        path_str, pattern
                    );
                    return true;
                }
            }
        }

        false
    };

    // Use the ignore crate to build a walker that respects .gitignore if enabled
    let mut walker = WalkBuilder::new(base_path);
    walker.hidden(false); // Include hidden files/directories

    // Add standard directories to always ignore
    walker.filter_entry(|entry| {
        let path = entry.path();

        // Skip target/ and .git/ directories completely
        if (path.ends_with("target") || path.ends_with(".git"))
            && entry.file_type().is_some_and(|ft| ft.is_dir())
        {
            debug!("Ignoring directory: {}", path.display());
            return false;
        }

        true
    });

    if config.ignore.use_gitignore {
        walker.git_ignore(true);
        walker.git_global(true);
        walker.git_exclude(true);
    } else {
        walker.git_ignore(false);
        walker.git_global(false);
        walker.git_exclude(false);
    }

    // Filter files using our multi-layered ignore system
    spinner.set_message("Collecting files...");
    let mut files: Vec<PathBuf> = walker
        .build()
        .filter_map(|entry| {
            spinner.tick();
            entry.ok()
        })
        .filter(|entry| {
            let path = entry.path();

            // Skip if not a regular file
            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                return false;
            }

            // Use our custom ignore function
            !should_ignore(path)
        })
        .map(|entry| entry.into_path())
        .collect();

    // Apply include patterns if specified
    if !config.include.is_empty() {
        spinner.set_message(format!("Applying include patterns: {:?}", config.include));

        let mut included_files = HashSet::new();
        let options = MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        for pattern in &config.include {
            // Normalize pattern for consistent matching
            let normalized_pattern = normalize_path(pattern);
            let full_pattern = base_path.join(&normalized_pattern);
            let pattern_str = full_pattern.to_string_lossy().to_string();

            match glob_with(&pattern_str, options) {
                Ok(entries) => {
                    for entry in entries.filter_map(Result::ok) {
                        included_files.insert(entry);
                    }
                }
                Err(e) => {
                    warn!("Invalid include pattern '{}': {}", pattern, e);
                }
            }
        }

        // Only keep files that match the include patterns
        // Fix: If include patterns are provided but no files match, we should have an empty list
        if !included_files.is_empty() {
            files.retain(|path| included_files.contains(path));
        } else {
            files.clear();
        }
    }

    spinner.set_message("Processing file information...");

    // Process file information in parallel
    let file_infos: Vec<FileInfo> = files
        .par_iter()
        .filter_map(|path| {
            match FileInfo::new(path.clone(), base_path) {
                Ok(info) => {
                    // Filter out files larger than the max size
                    if info.size > config.max_file_size {
                        debug!(
                            "Skipping large file: {} ({} bytes)",
                            info.path.display(),
                            info.size
                        );
                        None
                    } else if info.is_binary && config.include.is_empty() {
                        // Only skip binary files if no specific includes were provided.
                        // If the user explicitly included files, we assume they want them (even if binary).
                        debug!(
                            "Skipping binary file: {} ({})",
                            info.path.display(),
                            info.mime_type
                        );
                        None
                    } else {
                        Some(info)
                    }
                }
                Err(e) => {
                    warn!("Error processing file {}: {}", path.display(), e);
                    None
                }
            }
        })
        .collect();

    spinner.finish_with_message(format!("Found {} files to process", file_infos.len()));
    info!("Found {} files to process", file_infos.len());

    Ok(file_infos)
}
