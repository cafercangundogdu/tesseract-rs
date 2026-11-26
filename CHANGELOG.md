# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.2] - 2026-11-26

### Fixed
- The bundled build failed on Fedora, RHEL, AlmaLinux (including the
  manylinux_2_28 images) and other distributions where CMake installs into
  `lib64`. Leptonica 1.87.0 and Tesseract 5.5.2 put their libraries there,
  while the build script looks in `lib`, so linking stopped with
  "could not find native static library `leptonica`". The install directory
  is now pinned to `lib`. Versions 0.3.0 to 0.4.1 are affected.

## [0.4.1] - 2026-11-26

### Security
- Raised the `zip` build-dependency to 9.x (minimum 9.0.5). `zip` 8.x is in the
  affected range of several advisories, and GHSA-m665-v7pm-7grj is only fixed
  in 9.0.5 (#39). zip 9.8 and later need Rust 1.89; on Rust 1.88 pin an older
  9.x with `cargo update -p zip --precise 9.7.3`.

### Fixed
- `embed-tessdata` only ever downloaded `eng` and `tur`, so any other language
  in `TESSERACT_EMBED_LANGUAGES` was left out of the binary with nothing more
  than a build warning. Every listed language is now downloaded when it is
  missing, and a language that cannot be found fails the build (#37).
- Changing `TESSERACT_EMBED_LANGUAGES` now takes effect without a clean build.
- Tessdata downloads check the HTTP status and are written atomically, so a
  failed or interrupted download no longer leaves a broken `.traineddata`
  behind.
- On MSVC the bundled libraries now follow the `crt-static` target feature
  (`/MT` or `/MD`) instead of always using the dynamic C runtime (#35).
- The shared build cache was keyed on the library file name only, so libraries
  built for another crate version, target or C runtime could be reused. Build
  output now lives in `build/<crate version>/<target>/` inside the cache
  directory (#35).
- `CARGO_CLEAN=1` had no effect unless the build script happened to rerun for
  another reason. It now reliably rebuilds the libraries for the current
  target.

### Changed
- With the Visual Studio CMake generator the bundled libraries are always built
  in the Release configuration, as on every other platform. Debug cargo
  profiles previously built them unoptimized and against the debug C runtime.
- The `cache/`, `leptonica/`, `tesseract/` and `third_party/` directories at the
  top level of the cache directory are no longer used. They can be deleted
  unless an older version of this crate is still built on the same machine.

### Contributors
- Reported by [@logan-markewich](https://github.com/logan-markewich) and
  [@Phil-Fan](https://github.com/Phil-Fan) (#39),
  [@w-charlton-virifides](https://github.com/w-charlton-virifides) (#37) and
  [@Waquo](https://github.com/Waquo) (#35).

## [0.4.0] - 2026-07-31

### Added
- `use-system-tesseract` feature: link against a system-installed Tesseract
  (Homebrew, `libtesseract-dev`, ...) via pkg-config instead of compiling the
  bundled sources — builds go from minutes to seconds.
- `tesseract-rs-cli` — a new command-line OCR tool (`tesseract-rs <image>`),
  shipped as a workspace crate. Install via `cargo install tesseract-rs-cli`
  (bundled Tesseract) or `cargo install tesseract-rs-cli --no-default-features
  --features use-system-tesseract` (system Tesseract).

### Fixed
- `process_pages()` crashed with SIGSEGV on tesseract 5.5.3: the FFI
  signature of `TessBaseAPIProcessPages` declared a `char *` return type,
  but the C API returns `BOOL`. On success the value `1` was treated as a
  pointer and dereferenced. The signature now returns `c_int` and the text
  is fetched via `GetUTF8Text`.

## [0.3.1] - 2026-07-31

### Fixed
- Fixed the macOS build failing on Xcode 26+ runners: `std::filesystem` was
  reported unavailable (introduced in macOS 10.15) because no deployment
  target was set and the runner's default dropped below 10.15. The build now
  pins `CMAKE_OSX_DEPLOYMENT_TARGET` / `-mmacosx-version-min` to 10.15 (#32).

### Contributors
- Reported by [@pndaza](https://github.com/pndaza) (#32).

## [0.3.0] - 2026-07-08

### BREAKING CHANGES
- Raised the minimum supported Rust version (MSRV) to **1.88** (required by the
  upgraded dependencies).
- `get_int_variable()`, `get_bool_variable()` and `get_double_variable()` now
  return the actual variable value and `Err(GetVariableError)` when the
  variable is not found, instead of the C `BOOL` success flag.

### Added
- FreeBSD support — the build and full test suite now run on FreeBSD in CI
  (via a FreeBSD VM).
- `embed-tessdata` feature: embed `.traineddata` directly into the compiled
  binary and load it at runtime with `init_embedded()` / `embedded_languages()`.

### Changed
- Upgraded the bundled Tesseract **5.3.4 → 5.5.2** and Leptonica
  **1.84.1 → 1.87.0**, now compiled with C++17.
- Upgraded all dependencies to their latest versions (thiserror 2, reqwest
  0.13, zip 7 deflate-only, imageproc 0.27, etc.).

### Fixed
- Fixed a SIGSEGV caused by truncated `TessBaseAPIInit4` / `TessBaseAPIInit5`
  FFI signatures (missing trailing parameters).
- Fixed the `TessBaseAPIGet{Int,Bool,Double}Variable` FFI signatures (missing
  output pointer), which returned wrong values and could corrupt memory.
- Fixed the Windows debug build failing to locate the `d`-suffixed static
  libraries (#17).

### Contributors
- FreeBSD support and the `embed-tessdata` feature were contributed by
  [@mwstowe](https://github.com/mwstowe) (#20).
- The deflate-only `zip` configuration was contributed by
  [@YageGeng](https://github.com/YageGeng) (#25).
- The Windows debug-library and C++17 build fixes were cross-checked against
  the maintained [xberg-tesseract](https://github.com/xberg-io/xberg) fork.

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