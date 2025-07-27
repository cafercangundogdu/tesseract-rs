# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.20] - 2025-07-27

### Added
- Comprehensive unit tests for error handling and enums
- Benchmark tests using criterion
- Code coverage reporting with tarpaulin
- Commit message standards (Conventional Commits) 
- Pre-commit hooks with Husky for code quality
- CI/CD pipeline with clippy, rustfmt, and security audit
- Contributing guidelines

### Fixed
- Windows build issues with environment variables
- CMake policy version compatibility
- Windows library detection with multiple possible library names
- FFI binding issues by enabling legacy engine support
- Git's link.exe conflict on Windows CI
- All clippy warnings

### Changed
- Improved build script with better error handling
- Enhanced Windows support with fallback mechanisms
- Updated dependencies to latest versions

## [0.1.19] - Previous releases

- Initial release with basic Tesseract OCR bindings
- Optional built-in compilation support
- Cross-platform support (Windows, macOS, Linux)