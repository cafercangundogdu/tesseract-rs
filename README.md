# tesseract-rs

`tesseract-rs` is a Rust binding for Tesseract OCR with built-in compilation of Tesseract and Leptonica libraries. This project aims to provide a safe and idiomatic Rust interface to Tesseract's functionality while handling the complexity of compiling the underlying C++ libraries.

## Features

- Safe Rust bindings for Tesseract OCR
- Built-in compilation of Tesseract and Leptonica
- Automatic download of Tesseract training data (English and Turkish)
- High-level Rust API for common OCR tasks
- Caching of compiled libraries for faster subsequent builds
- Support for multiple operating systems (Linux, macOS, Windows)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tesseract-rs = { version = "0.1.18", features = ["build-tesseract"] }
```

For development and testing, you'll also need these dependencies:

```toml
[dev-dependencies]
image = "0.25.5"
imageproc = "0.25.0"
```

## System Requirements

To build this crate, you need:

- A C++ compiler (e.g., gcc, clang)
- CMake
- Internet connection (for downloading Tesseract training data)
- Rust 1.83.0 or later

## Environment Variables

The following environment variables affect the build and test process:

### Build Variables

- `CARGO_CLEAN`: If set, cleans the cache directory before building
- `RUSTC_WRAPPER`: If set to "sccache", enables compiler caching with sccache
- `CC`: Compiler selection for C code (affects Linux builds)
- `HOME` (Unix) or `APPDATA` (Windows): Used to determine cache directory location

### Test Variables

- `TESSDATA_PREFIX` (Optional): Path to override the default tessdata directory. If not set, the crate will use its default cache directory.

## Cache and Data Directories

The crate uses the following directory structure based on your operating system:

- macOS: `~/Library/Application Support/tesseract-rs`
- Linux: `~/.tesseract-rs`
- Windows: `%APPDATA%/tesseract-rs`

The cache includes:

- Compiled Tesseract and Leptonica libraries
- Downloaded training data (eng.traineddata, tur.traineddata) in the `tessdata` subdirectory
- Third-party source code

The training data files are automatically downloaded and placed in the appropriate `tessdata` subdirectory during the build process. You don't need to manually set up the tessdata directory unless you want to use a custom location.

## Testing

The project includes several integration tests that verify OCR functionality. To run the tests:

1. Ensure you have the required test dependencies:

   ```toml
   [dev-dependencies]
   image = "0.25.5"
   imageproc = "0.25.0"
   ```

2. Run the tests:
   ```bash
   cargo test
   ```

Note: Setting `TESSDATA_PREFIX` is optional. If not set, the tests will use the default tessdata directory in the cache location. If you want to use a custom tessdata directory, you can set it:

```bash
# Linux/macOS
export TESSDATA_PREFIX=/path/to/custom/tessdata

# Windows (PowerShell)
$env:TESSDATA_PREFIX="C:\path\to\custom\tessdata"
```

Available test cases:

- `test_multiple_languages_with_lstm`: Tests LSTM engine with multiple languages
- `test_ocr_on_real_image`: Tests OCR on a sample English text image
- `test_multiple_languages`: Tests recognition of mixed English and Turkish text
- `test_digit_recognition`: Tests digit-only recognition with whitelist
- `test_error_handling`: Tests error cases and invalid inputs

Test images are located in the `tests/test_images/` directory:

- `sample_text.png`: English text sample
- `multilang_sample.png`: Mixed English and Turkish text
- Additional test images can be added to this directory

## Usage

Here's a basic example of how to use `tesseract-rs`:

````rust
use std::path::PathBuf;
use std::error::Error;
use tesseract_rs::TesseractAPI;

fn get_default_tessdata_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
        PathBuf::from(home_dir)
            .join("Library")
            .join("Application Support")
            .join("tesseract-rs")
            .join("tessdata")
    } else if cfg!(target_os = "linux") {
        let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
        PathBuf::from(home_dir)
            .join(".tesseract-rs")
            .join("tessdata")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(std::env::var("APPDATA").expect("APPDATA environment variable not set"))
            .join("tesseract-rs")
            .join("tessdata")
    } else {
        panic!("Unsupported operating system");
    }
}

fn get_tessdata_dir() -> PathBuf {
    match std::env::var("TESSDATA_PREFIX") {
        Ok(dir) => {
            let path = PathBuf::from(dir);
            println!("Using TESSDATA_PREFIX directory: {:?}", path);
            path
        }
        Err(_) => {
            let default_dir = get_default_tessdata_dir();
            println!(
                "TESSDATA_PREFIX not set, using default directory: {:?}",
                default_dir
            );
            default_dir
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize Tesseract API
    let api = TesseractAPI::new()?;

    // Get tessdata directory (uses default location or TESSDATA_PREFIX if set)
    let tessdata_dir = get_tessdata_dir();
    api.init(tessdata_dir.to_str().unwrap(), "eng")?;

    // Create a simple test image (8x8 pixels, black text on white background)
    let image_data: Vec<u8> = vec![
        255, 255, 255, 255, 255, 255, 255, 255,
        255, 0,   0,   0,   0,   0,   255, 255,
        255, 0,   255, 255, 255, 0,   255, 255,
        255, 0,   255, 255, 255, 0,   255, 255,
        255, 0,   255, 255, 255, 0,   255, 255,
        255, 0,   255, 255, 255, 0,   255, 255,
        255, 0,   0,   0,   0,   0,   255, 255,
        255, 255, 255, 255, 255, 255, 255, 255,
    ];

    // Set the image data (8x8 pixels, 1 byte per pixel, 8 bytes per row)
    api.set_image(&image_data, 8, 8, 1, 8)?;

    // Set whitelist for digits only
    api.set_variable("tessedit_char_whitelist", "0123456789")?;

    // Get the recognized text
    let text = api.get_utf8_text()?;
    println!("Recognized text: {}", text.trim());

    Ok(())
}

## Advanced Usage

The API provides additional functionality for more complex OCR tasks:

```rust
use tesseract_rs::TesseractAPI;

fn main() -> Result<(), Box<dyn Error>> {
    let mut api = TesseractAPI::new()?;

    // Initialize with Turkish language
    api.init(None, "tur")?;

    // Configure OCR settings
    api.set_variable("tessedit_pageseg_mode", "1")?; // Automatic page segmentation

    // Get iterators for detailed analysis
    let (page_iter, result_iter) = api.get_iterators()?;

    // ... process results

    Ok(())
}
````

## Building

The crate will automatically download and compile Tesseract and Leptonica during the build process. This may take some time on the first build, but subsequent builds will use the cached libraries.

To clean the cache and force a rebuild:

```bash
CARGO_CLEAN=1 cargo build
```

## Documentation

For more detailed information, please check the [API documentation](https://docs.rs/tesseract-rs).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributors

- [Cafer Can Gündoğdu](https://github.com/cafercangundogdu)

## Contribution

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements

This project uses [Tesseract OCR](https://github.com/tesseract-ocr/tesseract) and [Leptonica](http://leptonica.org/). We are grateful to the maintainers and contributors of these projects.
