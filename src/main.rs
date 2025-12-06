mod cli;
mod comments;
mod config;
mod formatter;
mod packer;
mod remote;
mod scanner;
mod security;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use console::style;
use env_logger::Env;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let cli = if args.len() > 1 && args[1] == "mix" {
        // When run as 'cargo mix', skip the 'mix' argument
        let filtered_args: Vec<String> = args
            .into_iter()
            .enumerate()
            .filter(|(i, arg)| *i != 1 || arg != "mix")
            .map(|(_, arg)| arg)
            .collect();
        Cli::parse_from(filtered_args)
    } else {
        Cli::parse()
    };

    // Display welcome message with spinner
    let main_spinner = ProgressBar::new_spinner();
    main_spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {wide_msg}")
            .unwrap(),
    );

    main_spinner.set_message(format!("🚀 {} starting...", style("remix").bold().green()));
    main_spinner.tick();

    // Load configuration
    // Load configuration - moved to later

    // If --init flag is set, create a new configuration file and exit
    if cli.init {
        main_spinner.set_message("Initializing configuration...");
        config::init_config()?;
        main_spinner.finish_with_message(format!(
            "{} Configuration initialized successfully",
            style("✓").bold().green()
        ));
        return Ok(());
    }

    // Determine the target path
    let target_path = if let Some(path) = &cli.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    main_spinner.set_message(format!(
        "Starting remix on {}",
        style(target_path.display()).cyan()
    ));

    info!("Starting remix on {}", target_path.display());

    // If processing a remote repository
    if let Some(remote_url) = &cli.remote {
        let branch = cli
            .remote_branch
            .as_ref()
            .map_or_else(|| "main".to_string(), |s| s.clone());
        main_spinner.set_message(format!(
            "Processing remote repository: {} ({})",
            style(remote_url).cyan(),
            style(&branch).cyan()
        ));
        info!("Processing remote repository: {} ({})", remote_url, branch);

        let temp_dir = remote::clone_repository(remote_url, &branch)
            .context("Failed to clone remote repository")?;

        main_spinner.set_message("Processing repository...");
        // Load configuration for remote
        let base_config = if let Some(ref config_path) = cli.config {
            config::load_config(config_path).context(format!(
                "Failed to load config from {}",
                config_path.display()
            ))?
        } else {
            config::find_and_load_config_at(&temp_dir).unwrap_or_default()
        };
        let merged_config = base_config.merge_with_cli(&cli);
        let result = packer::pack_repository(&temp_dir, &merged_config).await?;

        main_spinner.set_message("Formatting output...");
        formatter::output_result(&result, &merged_config.output)?;
    } else {
        // Process local repository
        main_spinner.set_message(format!(
            "Processing local repository: {}",
            style(target_path.display()).cyan()
        ));
        info!("Processing local repository: {}", target_path.display());

        main_spinner.set_message("Processing repository...");
        // Load configuration for local
        let base_config = if let Some(ref config_path) = cli.config {
            config::load_config(config_path).context(format!(
                "Failed to load config from {}",
                config_path.display()
            ))?
        } else {
            config::find_and_load_config_at(&target_path).unwrap_or_default()
        };
        let merged_config = base_config.merge_with_cli(&cli);
        let result = packer::pack_repository(&target_path, &merged_config).await?;

        main_spinner.set_message("Formatting output...");
        formatter::output_result(&result, &merged_config.output)?;
    }

    main_spinner.finish_with_message(format!(
        "{} Repository packing completed successfully",
        style("✓").bold().green()
    ));
    info!("Repository packing completed successfully");
    Ok(())
}
