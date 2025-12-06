# Project Context

## Purpose
Remix is a high-performance Rust implementation of repomix that packs entire repositories into single, AI-friendly files. It's designed to prepare codebases for analysis by Large Language Models (LLMs) like Claude, ChatGPT, and others. The tool processes local or remote repositories, applies intelligent filtering, performs security checks, and outputs in multiple formats optimized for AI consumption.

## Tech Stack
- **Rust 2021 Edition** - Core language for performance and safety
- **Tokio** - Async runtime for concurrent file processing
- **Clap** - Command-line argument parsing with derive features
- **Serde** - JSON serialization/deserialization for configuration
- **Rayon** - Data parallelism for CPU-intensive tasks
- **Git2** - Git repository operations and remote cloning
- **Regex** - Pattern matching for file filtering
- **Walkdir** - Directory traversal and file discovery
- **Ignore** - Gitignore-style pattern matching
- **Reqwest** - HTTP client for remote repository operations
- **Indicatif** - Progress bars and spinners for UX
- **Console** - Terminal styling and colors
- **Tree Magic Mini** - Binary file detection
- **TOON (rtoon)** - Token-efficient data serialization

## Project Conventions

### Code Style
- **Rustfmt** - Use `cargo fmt` for consistent formatting
- **Clippy** - All warnings must be addressed (`cargo clippy -- -D warnings`)
- **Naming**: snake_case for variables/functions, PascalCase for types
- **Error Handling**: Use `anyhow::Result<T>` for application errors, `thiserror` for library errors
- **Async/Await**: Prefer async functions with `.await` for I/O operations
- **Module Organization**: One module per major feature area in `src/`

### Architecture Patterns
- **Modular Design**: Separate modules for CLI, config, scanning, packing, formatting, security
- **Async Pipeline**: File processing uses async streams for memory efficiency
- **Configuration Layering**: CLI args → config file → defaults (CLI takes precedence)
- **Multi-layer Filtering**: Gitignore → mixignore → custom patterns → defaults
- **Parallel Processing**: Rayon for CPU-bound tasks, Tokio for I/O-bound tasks
- **Error Propagation**: Use `context()` for error chain clarity

### Testing Strategy
- **Unit Tests**: Module-specific tests in `tests/` directory
- **Integration Tests**: End-to-end workflow testing
- **Mocking**: Use `mockall` for external dependencies
- **Test Fixtures**: `assert_fs` and `tempfile` for isolated test environments
- **Test Cases**: `test-case` crate for parameterized tests
- **CI Testing**: All tests run on Ubuntu, Windows builds verified

### Git Workflow
- **Main Branch**: `main` (primary development)
- **Release Tags**: Semantic versioning (`v1.2.3`)
- **PR Process**: Feature branches → PR → CI checks → merge
- **Commit Style**: Conventional commits encouraged but not enforced
- **CI/CD**: GitHub Actions with test, clippy, format, and cross-platform builds

## Domain Context

### Repository Packing Workflow
1. **Input**: Local directory path or remote Git URL
2. **Scanning**: Recursive file discovery with filtering
3. **Security**: Optional sensitive content detection
4. **Processing**: Parallel file reading and optional comment removal
5. **Output**: Formatted as Markdown, JSON, plain text, or TOON

### File Filtering Priority
1. Include patterns (whitelist)
2. Gitignore patterns
3. Mixignore patterns (`.mixignore`)
4. Custom ignore patterns
5. Default ignore patterns (node_modules, target, etc.)

### Output Formats
- **Markdown**: Syntax-highlighted code blocks with file tree
- **JSON**: Structured data with base64-encoded content
- **TOON**: Token-efficient format for LLM prompts
- **Text**: Plain format with minimal metadata

## Important Constraints
- **Memory Efficiency**: Process large repositories without loading everything into memory
- **Performance**: Parallel processing for speed, async I/O for responsiveness
- **Security**: Never expose sensitive data in output unless explicitly disabled
- **Cross-Platform**: Must work on Linux, macOS, and Windows
- **Backward Compatibility**: Configuration file format must remain stable
- **Token Optimization**: Output formats designed to minimize LLM token usage

## External Dependencies
- **Git Repositories**: GitHub, GitLab, and other Git hosting services via HTTPS
- **File System**: Local filesystem access with proper permission handling
- **Temporary Storage**: System temp directories for remote repo cloning
- **Network**: HTTP/HTTPS for remote repository access
- **Environment**: Standard environment variables for configuration
