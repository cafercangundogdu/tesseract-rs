# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-23

### BREAKING CHANGES
- Removed `Clone` impl for `TesseractAPI` — use `try_clone()` instead
- Removed `MutableIterator` type from public API
- Removed duplicate `analyze_layout()` — use `analyse_layout()` (matches C API)
- Removed `get_mutable_iterator()` — use `get_iterator()`
- Changed `is_valid_word()` return type from `Result<i32>` to `Result<bool>`
- Changed `PageIterator` methods to return `Result<T>` instead of bare types
- Changed `TessMonitor` methods to return `Result<T>` instead of bare types
- Changed `TessResultRenderer` methods to return `Result<T>` instead of bare types
- Removed duplicate `MutexError` variant — use `MutexLockError`
- Deprecated `get_word_confidences()` — use `all_word_confidences()`

### Fixed
- **Critical:** Use-after-free in `process_pages()` CString handling
- **Critical:** Undefined behavior in `detect_os()` freeing Tesseract's static pointer
- **Critical:** Unsafe `transmute` replaced with safe `from_int()` enum conversions
- **Critical:** Memory leak in `get_word_confidences()` (never called `TessDeleteIntArray`)
- **Critical:** `ChoiceIterator` incorrectly freed internal Tesseract pointer via `TessDeleteText`
- Fixed `print_variables_to_file()` inverted success/error logic
- Fixed `get_unichar()` panicking on mutex lock failure
- Fixed inconsistent mutex error handling (some modules panicked, others returned errors)
- Fixed `Drop` implementations to use `if let Ok()` pattern (no panic on poisoned mutex)
- Fixed null pointer comparison using `==` instead of `.is_null()`

### Added
- `TesseractAPI::try_clone()` — fallible clone that returns `Result`
- `ResultIterator::get_choice_iterator()` — access alternative recognition choices
- Re-exported `TessOrientation`, `TessWritingDirection`, `TessTextlineOrder`, `TessParagraphJustification`
- Consistent `#[cfg(feature = "build-tesseract")]` on all extern blocks
- 179 tests (from 14) covering all modules with 91.6% coverage
- End-to-end tests simulating real-world usage scenarios
- Shared test helpers module

### Removed
- `src/mutable_iterator.rs` — was a complete duplicate of `ResultIterator`
- Unnecessary clippy allow attributes

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