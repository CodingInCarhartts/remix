use crate::config::OutputConfig;
use crate::packer::PackedRepository;
use crate::security::SecurityCheckStatus;
use crate::utils::{format_size, open_file};
use anyhow::{Context, Result};
use log::{info, warn};
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn output_result(repo: &PackedRepository, config: &OutputConfig) -> Result<()> {
    let output_path = &config.path;
    let format = &config.format;

    info!("Generating output in {} format to {}", format, output_path);

    let content = match format.as_str() {
        "md" | "markdown" => format_markdown(repo),
        "json" => format_json(repo)?,
        "txt" | "text" => format_text(repo),
        "toon" => format_toon(repo)?,
        "sql" => format_sqlite_sql(repo)?,
        _ => {
            warn!("Unknown format '{}', defaulting to markdown", format);
            format_markdown(repo)
        }
    };

    // Write the output to a file
    let mut file = fs::File::create(output_path)
        .context(format!("Failed to create output file: {}", output_path))?;

    file.write_all(content.as_bytes())
        .context(format!("Failed to write to output file: {}", output_path))?;

    info!("Output written to {}", output_path);

    // Open the file if requested
    if config.open_file {
        info!("Opening output file");
        open_file(output_path).context(format!("Failed to open output file: {}", output_path))?;
    }

    Ok(())
}

pub fn format_markdown(repo: &PackedRepository) -> String {
    let mut output = String::new();

    // Add user instruction if provided
    if let Some(instruction) = &repo.instruction {
        output.push_str("# User Instruction\n\n");
        output.push_str(instruction);
        output.push_str("\n\n");
    }

    // Add repository summary
    output.push_str("# Repository Summary\n\n");
    output.push_str(&format!("- **Files:** {}\n", repo.summary.file_count));
    output.push_str(&format!(
        "- **Directories:** {}\n",
        repo.summary.directory_count
    ));
    output.push_str(&format!(
        "- **Total Size:** {}\n",
        format_size(repo.summary.total_size)
    ));
    output.push_str(&format!(
        "- **Binary Files:** {}\n",
        repo.summary.binary_file_count
    ));

    if !repo.summary.extensions.is_empty() {
        output.push_str(&format!(
            "- **Extensions:** {}\n",
            repo.summary.extensions.join(", ")
        ));
    }

    // Add security check results if available
    match &repo.security_check_status {
        SecurityCheckStatus::Disabled => {
            output.push_str("## Security Check\n\n");
            output.push_str("🔒 **Security check was disabled**\n");
        }
        SecurityCheckStatus::CompletedNoFindings => {
            output.push_str("## Security Check\n\n");
            output.push_str("✅ **Security check completed - no suspicious files found**\n");
        }
        SecurityCheckStatus::CompletedWithFindings => {
            output.push_str("\n## Security Check Results\n\n");
            output.push_str(&format!(
                "⚠️ **{} suspicious file(s) detected that may contain sensitive information:**\n\n",
                repo.suspicious_files.as_ref().map_or(0, |v| v.len())
            ));

            for (i, file) in repo
                .suspicious_files
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .enumerate()
            {
                output.push_str(&format!("{}. `{}`\n", i + 1, file));
            }

            output
                .push_str("\n> **Note:** Please review these files before sharing this output.\n");
        }
        SecurityCheckStatus::Failed(error) => {
            output.push_str("## Security Check\n\n");
            output.push_str(&format!("❌ **Security check failed**: {}\n", error));
        }
    }

    // Add binary files list if available
    if let Some(binary_files) = &repo.binary_files {
        if !binary_files.is_empty() {
            output.push_str("\n## Binary Files\n\n");
            output.push_str(
                "The following binary files were detected but not included in the content:\n\n",
            );

            for (i, file) in binary_files.iter().enumerate() {
                output.push_str(&format!("{}. `{}`\n", i + 1, file));
            }
        }
    }

    output.push_str("\n# Files\n\n");

    // Group files by directory
    let mut files_by_dir: std::collections::BTreeMap<String, Vec<&crate::packer::FileContent>> =
        std::collections::BTreeMap::new();

    for file in &repo.files {
        let path = Path::new(&file.relative_path);
        let parent = path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        files_by_dir.entry(parent).or_default().push(file);
    }

    // Output files by directory
    for (dir, files) in &files_by_dir {
        if dir != "." {
            output.push_str(&format!("## {}/\n\n", dir));
        } else {
            output.push_str("## Root Directory\n\n");
        }

        for file in files {
            let filename = Path::new(&file.relative_path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| file.relative_path.clone());

            output.push_str(&format!("### {}\n\n", filename));

            // Add file metadata
            output.push_str(&format!("- **Path:** {}\n", file.relative_path));
            output.push_str(&format!("- **Size:** {}\n", format_size(file.size)));

            if !file.extension.is_empty() {
                output.push_str(&format!("- **Type:** {}\n", file.extension));
            }

            output.push_str("\n```");

            // Add language hint for syntax highlighting
            if !file.extension.is_empty() {
                output.push_str(&file.extension);
            }

            output.push('\n');
            output.push_str(&file.content);
            output.push('\n');
            output.push_str("```\n\n");
        }
    }

    output
}

pub fn format_json(repo: &PackedRepository) -> Result<String> {
    serde_json::to_string_pretty(repo).context("Failed to serialize repository to JSON")
}

pub fn format_toon(repo: &PackedRepository) -> Result<String> {
    let value = serde_json::to_value(repo).context("Failed to convert repository to JSON value")?;
    rtoon::encode_default(&value).context("Failed to encode repository to TOON")
}

pub fn format_text(repo: &PackedRepository) -> String {
    let mut output = String::new();

    // Add user instruction if provided
    if let Some(instruction) = &repo.instruction {
        output.push_str("USER INSTRUCTION:\n\n");
        output.push_str(instruction);
        output.push_str("\n\n");
    }

    // Add repository summary
    output.push_str("REPOSITORY SUMMARY:\n\n");
    output.push_str(&format!("Files: {}\n", repo.summary.file_count));
    output.push_str(&format!("Directories: {}\n", repo.summary.directory_count));
    output.push_str(&format!(
        "Total Size: {}\n",
        format_size(repo.summary.total_size)
    ));
    output.push_str(&format!(
        "Binary Files: {}\n",
        repo.summary.binary_file_count
    ));

    if !repo.summary.extensions.is_empty() {
        output.push_str(&format!(
            "Extensions: {}\n",
            repo.summary.extensions.join(", ")
        ));
    }

    // Add security check results if available
    match &repo.security_check_status {
        SecurityCheckStatus::Disabled => {
            output.push_str("SECURITY CHECK:\n\n");
            output.push_str("Security check was disabled.\n\n");
        }
        SecurityCheckStatus::CompletedNoFindings => {
            output.push_str("SECURITY CHECK:\n\n");
            output.push_str("Security check completed - no suspicious files found.\n\n");
        }
        SecurityCheckStatus::CompletedWithFindings => {
            output.push_str("SECURITY CHECK:\n\n");
            output.push_str("WARNING: ");
            output.push_str(&format!(
                "{} suspicious file(s) detected that may contain sensitive information:\n\n",
                repo.suspicious_files.as_ref().map_or(0, |v| v.len())
            ));

            for (i, file) in repo
                .suspicious_files
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .enumerate()
            {
                output.push_str(&format!("{}. {}\n", i + 1, file));
            }

            output.push_str("\nPlease review these files before sharing this output.\n");
        }
        SecurityCheckStatus::Failed(error) => {
            output.push_str("SECURITY CHECK:\n\n");
            output.push_str(&format!("Security check failed: {}\n", error));
        }
    }

    // Add binary files list if available
    if let Some(binary_files) = &repo.binary_files {
        if !binary_files.is_empty() {
            output.push_str("\nBINARY FILES:\n\n");
            output.push_str(
                "The following binary files were detected but not included in the content:\n\n",
            );

            for (i, file) in binary_files.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, file));
            }
        }
    }

    output.push_str("\nFILES:\n\n");

    // Output each file
    for file in &repo.files {
        output.push_str(&format!("FILE: {}\n", file.relative_path));
        output.push_str(&format!("SIZE: {}\n", format_size(file.size)));

        if !file.extension.is_empty() {
            output.push_str(&format!("TYPE: {}\n", file.extension));
        }

        output.push_str("\nCONTENT:\n");
        output.push_str(&file.content);
        output.push_str("\n\n");
        output.push_str("--------------------------------\n\n");
    }

    output
}

/// Escapes a string for use in SQLite single-quoted strings.
/// Doubles single quotes and preserves newlines.
fn escape_sqlite_string(s: &str) -> String {
    s.replace('\'', "''")
}

/// Formats the repository as a SQLite SQL script.
pub fn format_sqlite_sql(repo: &PackedRepository) -> Result<String> {
    let mut output = String::new();

    // Header comment
    output.push_str("-- SQLite SQL script generated by remix\n");
    output.push_str("-- This script creates tables for a packed repository and inserts data.\n");
    output.push_str("-- SQLite dialect: compatible with SQLite 3.0+\n");
    output.push_str(
        "-- WARNING: This script drops existing 'remix_*' tables before recreating them.\n",
    );
    output.push_str("-- Run this script in an empty database or one without 'remix_*' tables.\n");
    output.push('\n');

    // Enable foreign keys
    output.push_str("PRAGMA foreign_keys = ON;\n\n");

    // Begin transaction
    output.push_str("BEGIN TRANSACTION;\n\n");

    // Drop existing tables (in reverse dependency order)
    output.push_str("-- Drop existing tables\n");
    output.push_str("DROP TABLE IF EXISTS remix_suspicious_files;\n");
    output.push_str("DROP TABLE IF EXISTS remix_binary_files;\n");
    output.push_str("DROP TABLE IF EXISTS remix_files;\n");
    output.push_str("DROP TABLE IF EXISTS remix_extensions;\n");
    output.push_str("DROP TABLE IF EXISTS remix_summary;\n");
    output.push_str("DROP TABLE IF EXISTS remix_runs;\n\n");

    // Create tables
    output.push_str("-- Create remix_runs table\n");
    output.push_str("CREATE TABLE remix_runs (\n");
    output.push_str("    id INTEGER PRIMARY KEY,\n");
    output.push_str("    created_at TEXT NOT NULL,\n");
    output.push_str("    instruction TEXT,\n");
    output.push_str("    security_check_status TEXT NOT NULL,\n");
    output.push_str("    security_error TEXT\n");
    output.push_str(");\n\n");

    output.push_str("-- Create remix_summary table\n");
    output.push_str("CREATE TABLE remix_summary (\n");
    output.push_str("    run_id INTEGER NOT NULL REFERENCES remix_runs(id),\n");
    output.push_str("    file_count INTEGER NOT NULL,\n");
    output.push_str("    directory_count INTEGER NOT NULL,\n");
    output.push_str("    total_size INTEGER NOT NULL,\n");
    output.push_str("    binary_file_count INTEGER NOT NULL\n");
    output.push_str(");\n\n");

    output.push_str("-- Create remix_extensions table\n");
    output.push_str("CREATE TABLE remix_extensions (\n");
    output.push_str("    run_id INTEGER NOT NULL REFERENCES remix_runs(id),\n");
    output.push_str("    extension TEXT NOT NULL\n");
    output.push_str(");\n\n");

    output.push_str("-- Create remix_files table\n");
    output.push_str("CREATE TABLE remix_files (\n");
    output.push_str("    id INTEGER PRIMARY KEY,\n");
    output.push_str("    run_id INTEGER NOT NULL REFERENCES remix_runs(id),\n");
    output.push_str("    relative_path TEXT NOT NULL,\n");
    output.push_str("    extension TEXT NOT NULL,\n");
    output.push_str("    size INTEGER NOT NULL,\n");
    output.push_str("    is_binary INTEGER NOT NULL,\n");
    output.push_str("    content TEXT NOT NULL\n");
    output.push_str(");\n\n");

    output.push_str("-- Create remix_binary_files table\n");
    output.push_str("CREATE TABLE remix_binary_files (\n");
    output.push_str("    run_id INTEGER NOT NULL REFERENCES remix_runs(id),\n");
    output.push_str("    relative_path TEXT NOT NULL\n");
    output.push_str(");\n\n");

    output.push_str("-- Create remix_suspicious_files table\n");
    output.push_str("CREATE TABLE remix_suspicious_files (\n");
    output.push_str("    run_id INTEGER NOT NULL REFERENCES remix_runs(id),\n");
    output.push_str("    relative_path TEXT NOT NULL\n");
    output.push_str(");\n\n");

    // Insert data - use run_id = 1 for simplicity
    let run_id = 1;

    // Insert remix_runs
    let created_at = chrono::Utc::now().to_rfc3339();
    let security_status = match &repo.security_check_status {
        SecurityCheckStatus::Disabled => "Disabled",
        SecurityCheckStatus::CompletedNoFindings => "CompletedNoFindings",
        SecurityCheckStatus::CompletedWithFindings => "CompletedWithFindings",
        SecurityCheckStatus::Failed(_) => "Failed",
    };
    let security_error = match &repo.security_check_status {
        SecurityCheckStatus::Failed(e) => Some(e.as_str()),
        _ => None,
    };

    output.push_str("-- Insert remix_runs\n");
    output.push_str("INSERT INTO remix_runs (id, created_at, instruction, security_check_status, security_error)\n");
    output.push_str("VALUES (\n");
    output.push_str(&format!("    {}, -- id\n", run_id));
    output.push_str(&format!(
        "    '{}', -- created_at\n",
        escape_sqlite_string(&created_at)
    ));
    if let Some(instruction) = &repo.instruction {
        output.push_str(&format!(
            "    '{}', -- instruction\n",
            escape_sqlite_string(instruction)
        ));
    } else {
        output.push_str("    NULL, -- instruction\n");
    }
    output.push_str(&format!(
        "    '{}', -- security_check_status\n",
        security_status
    ));
    if let Some(error) = security_error {
        output.push_str(&format!(
            "    '{}' -- security_error\n",
            escape_sqlite_string(error)
        ));
    } else {
        output.push_str("    NULL -- security_error\n");
    }
    output.push_str(");\n\n");

    // Insert remix_summary
    output.push_str("-- Insert remix_summary\n");
    output.push_str("INSERT INTO remix_summary (run_id, file_count, directory_count, total_size, binary_file_count)\n");
    output.push_str("VALUES (\n");
    output.push_str(&format!("    {}, -- run_id\n", run_id));
    output.push_str(&format!("    {}, -- file_count\n", repo.summary.file_count));
    output.push_str(&format!(
        "    {}, -- directory_count\n",
        repo.summary.directory_count
    ));
    output.push_str(&format!("    {}, -- total_size\n", repo.summary.total_size));
    output.push_str(&format!(
        "    {} -- binary_file_count\n",
        repo.summary.binary_file_count
    ));
    output.push_str(");\n\n");

    // Insert remix_extensions
    if !repo.summary.extensions.is_empty() {
        output.push_str("-- Insert remix_extensions\n");
        for ext in &repo.summary.extensions {
            output.push_str(&format!(
                "INSERT INTO remix_extensions (run_id, extension) VALUES ({}, '{}');\n",
                run_id,
                escape_sqlite_string(ext)
            ));
        }
        output.push('\n');
    }

    // Insert remix_files
    if !repo.files.is_empty() {
        output.push_str("-- Insert remix_files\n");
        for (i, file) in repo.files.iter().enumerate() {
            output.push_str("INSERT INTO remix_files (id, run_id, relative_path, extension, size, is_binary, content) VALUES (\n");
            output.push_str(&format!("    {}, -- id\n", i + 1));
            output.push_str(&format!("    {}, -- run_id\n", run_id));
            output.push_str(&format!(
                "    '{}', -- relative_path\n",
                escape_sqlite_string(&file.relative_path)
            ));
            output.push_str(&format!(
                "    '{}', -- extension\n",
                escape_sqlite_string(&file.extension)
            ));
            output.push_str(&format!("    {}, -- size\n", file.size));
            output.push_str(&format!(
                "    {}, -- is_binary\n",
                if file.is_binary { 1 } else { 0 }
            ));
            output.push_str(&format!(
                "    '{}' -- content\n",
                escape_sqlite_string(&file.content)
            ));
            output.push_str(");\n");
        }
        output.push('\n');
    }

    // Insert remix_binary_files
    if let Some(binary_files) = &repo.binary_files {
        if !binary_files.is_empty() {
            output.push_str("-- Insert remix_binary_files\n");
            for bf in binary_files {
                output.push_str(&format!(
                    "INSERT INTO remix_binary_files (run_id, relative_path) VALUES ({}, '{}');\n",
                    run_id,
                    escape_sqlite_string(bf)
                ));
            }
            output.push('\n');
        }
    }

    // Insert remix_suspicious_files
    if let Some(suspicious_files) = &repo.suspicious_files {
        if !suspicious_files.is_empty() {
            output.push_str("-- Insert remix_suspicious_files\n");
            for sf in suspicious_files {
                output.push_str(&format!(
                    "INSERT INTO remix_suspicious_files (run_id, relative_path) VALUES ({}, '{}');\n",
                    run_id,
                    escape_sqlite_string(sf)
                ));
            }
            output.push('\n');
        }
    }

    // Commit transaction
    output.push_str("COMMIT TRANSACTION;\n");

    Ok(output)
}
