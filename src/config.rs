use crate::cli::Cli;
use anyhow::{Context, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const CONFIG_FILENAME: &str = "remix.config.json";
const DEFAULT_MAX_FILE_SIZE: u64 = 100_000; // 100KB

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputConfig {
    /// Output format (md, json, txt, toon)
    #[serde(default = "default_format")]
    pub format: String,

    /// Whether to open the output file after generation
    #[serde(default)]
    pub open_file: bool,

    /// Output file path
    #[serde(default = "default_output_path")]
    pub path: String,

    /// Path to instruction file to include in output
    pub instruction_file_path: Option<String>,

    /// Whether to remove comments from supported file types
    #[serde(default)]
    pub remove_comments: bool,
}

fn default_format() -> String {
    "txt".to_string()
}

fn default_output_path() -> String {
    "./remix-output.txt".to_string()
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            open_file: false,
            path: default_output_path(),
            instruction_file_path: None,
            remove_comments: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgnoreConfig {
    /// Use patterns from .gitignore files
    #[serde(default = "default_use_gitignore")]
    pub use_gitignore: bool,

    /// Use default ignore patterns for common exclusions
    #[serde(default = "default_use_default_patterns")]
    pub use_default_patterns: bool,

    /// Use patterns from .remixignore file if present
    #[serde(default = "default_use_mixignore")]
    pub use_mixignore: bool,

    /// Custom ignore patterns
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}

fn default_use_gitignore() -> bool {
    true
}

fn default_use_default_patterns() -> bool {
    true
}

fn default_use_mixignore() -> bool {
    true
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            use_gitignore: default_use_gitignore(),
            use_default_patterns: default_use_default_patterns(),
            use_mixignore: default_use_mixignore(),
            custom_patterns: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    /// Enable security check for sensitive information
    #[serde(default = "default_enable_security_check")]
    pub enable_security_check: bool,
}

fn default_enable_security_check() -> bool {
    true
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_security_check: default_enable_security_check(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Patterns to include (glob syntax)
    #[serde(default)]
    pub include: Vec<String>,

    /// Ignore configuration
    #[serde(default)]
    pub ignore: IgnoreConfig,

    /// Maximum file size in bytes
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,

    /// Whether to compress code
    #[serde(default)]
    pub compress: bool,

    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,

    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,

    /// User instruction to add at the top of the output
    pub instruction: Option<String>,
}

fn default_max_file_size() -> u64 {
    DEFAULT_MAX_FILE_SIZE
}

impl Default for Config {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            ignore: IgnoreConfig {
                use_gitignore: default_use_gitignore(),
                use_default_patterns: default_use_default_patterns(),
                use_mixignore: default_use_mixignore(),
                custom_patterns: vec![
                    "node_modules/".to_string(),
                    "package-lock.json".to_string(),
                    "**/package-lock.json".to_string(),
                    "**/node_modules/".to_string(),
                    "**/bun.lockb".to_string(),
                    "bun.lockb".to_string(),
                    "bun.lock".to_string(),
                    ".conda/".to_string(),
                    "**/.conda/".to_string(),
                    ".venv/".to_string(),
                    "**/.venv/".to_string(),
                    ".mamba/".to_string(),
                    "**/.mamba/".to_string(),
                    ".pyenv/".to_string(),
                    "**/.pyenv/".to_string(),
                    ".git/".to_string(),
                    "**/.git/".to_string(),
                    ".gitignore".to_string(),
                    "**/.gitignore".to_string(),
                    ".gitattributes".to_string(),
                    "**/.gitattributes".to_string(),
                    ".github/".to_string(),
                    "**/.github/".to_string(),
                    ".gitmodules".to_string(),
                    "**/.gitmodules".to_string(),
                    ".gitkeep".to_string(),
                    "**/.gitkeep".to_string(),
                    "target/".to_string(),
                    "**/target/".to_string(),
                    "dist/".to_string(),
                    "**/dist/".to_string(),
                    "build/".to_string(),
                    "**/build/".to_string(),
                    "**/*.log".to_string(),
                    "**/Cargo.lock".to_string(),
                    "**/.env".to_string(),
                    "**/*.exe".to_string(),
                    "**/*.o".to_string(),
                    "**/*.so".to_string(),
                    "**/*.dylib".to_string(),
                    "**/*.dll".to_string(),
                    "**/*.lib".to_string(),
                    "**/*.a".to_string(),
                    "**/*.lib".to_string(),
                ],
            },
            max_file_size: default_max_file_size(),
            compress: false,
            security: SecurityConfig::default(),
            output: OutputConfig::default(),
            instruction: None,
        }
    }
}

impl Config {
    /// Merge CLI arguments with configuration
    pub fn merge_with_cli(&self, cli: &Cli) -> Self {
        let mut config = self.clone();

        // Override with CLI options if provided
        if let Some(patterns) = cli.include_patterns() {
            config.include = patterns;
        }

        if let Some(patterns) = cli.ignore_patterns() {
            config.ignore.custom_patterns = patterns;
        }

        if let Some(max_size) = cli.max_file_size {
            config.max_file_size = max_size;
        }

        if cli.compress {
            config.compress = true;
        }

        if cli.skip_sensitive_check {
            config.security.enable_security_check = false;
        }

        if cli.no_gitignore {
            config.ignore.use_gitignore = false;
        }

        if cli.no_default_patterns {
            config.ignore.use_default_patterns = false;
        }

        if let Some(format) = &cli.format {
            config.output.format = format.clone();
        }

        // Adjust output path extension based on format if no custom output path provided
        if cli.output.is_none() {
            let extension = match config.output.format.as_str() {
                "md" | "markdown" => "md",
                "json" => "json",
                "txt" | "text" => "txt",
                "toon" => "toon",
                _ => "md",
            };
            let path_buf = PathBuf::from(&config.output.path);
            let new_path = path_buf.with_extension(extension);
            config.output.path = new_path.to_string_lossy().to_string();
        }

        if let Some(output_path) = &cli.output {
            config.output.path = output_path.to_string_lossy().to_string();
        }

        if cli.open {
            config.output.open_file = true;
        }

        if cli.remove_comments {
            config.output.remove_comments = true;
        }

        if let Some(instruction) = &cli.instruction {
            config.instruction = Some(instruction.clone());
        }

        if let Some(instruction_file) = &cli.instruction_file {
            config.output.instruction_file_path =
                Some(instruction_file.to_string_lossy().to_string());
        }

        config
    }
}

/// Find and load the configuration file
pub fn find_and_load_config() -> Result<Config> {
    let current_dir = std::env::current_dir()?;
    find_and_load_config_at(&current_dir)
}

/// Load configuration from a file
pub fn load_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read config file: {}", path.display()))?;

    let config: Config = serde_json::from_str(&content)
        .context(format!("Failed to parse config file: {}", path.display()))?;

    Ok(config)
}

/// Find and load the configuration file from a specific directory
pub fn find_and_load_config_at(dir: &Path) -> Result<Config> {
    let config_path = dir.join(CONFIG_FILENAME);

    if config_path.exists() {
        info!("Found configuration file: {}", config_path.display());
        load_config(&config_path)
    } else {
        // Don't warn here as it might be common to not have a config in the target dir
        Ok(Config::default())
    }
}

/// Initialize a new configuration file
pub fn init_config() -> Result<()> {
    let config = Config::default();
    let current_dir = std::env::current_dir()?;
    let config_path = current_dir.join(CONFIG_FILENAME);

    if config_path.exists() {
        warn!(
            "Configuration file already exists: {}",
            config_path.display()
        );
        return Ok(());
    }

    let json =
        serde_json::to_string_pretty(&config).context("Failed to serialize configuration")?;

    let mut file = fs::File::create(&config_path).context(format!(
        "Failed to create config file: {}",
        config_path.display()
    ))?;

    file.write_all(json.as_bytes()).context(format!(
        "Failed to write to config file: {}",
        config_path.display()
    ))?;

    info!("Created configuration file: {}", config_path.display());

    Ok(())
}
