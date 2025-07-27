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
tesseract-rs = { version = "0.1.20", features = ["build-tesseract"] }
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

```rust
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
    let api = TesseractAPI::new()?;

    // Get tessdata directory (uses default location or TESSDATA_PREFIX if set)
    let tessdata_dir = get_tessdata_dir();
    api.init(tessdata_dir.to_str().unwrap(), "eng")?;

    let width = 24;
    let height = 24;
    let bytes_per_pixel = 1;
    let bytes_per_line = width * bytes_per_pixel;

    // Initialize image data with all white pixels
    let mut image_data = vec![255u8; width * height];

    // Draw number 9 with clearer distinction
    for y in 4..19 {
        for x in 7..17 {
            // Top bar
            if y == 4 && x >= 8 && x <= 15 {
                image_data[y * width + x] = 0;
            }
            // Top curve left side
            if y >= 4 && y <= 10 && x == 7 {
                image_data[y * width + x] = 0;
            }
            // Top curve right side
            if y >= 4 && y <= 11 && x == 16 {
                image_data[y * width + x] = 0;
            }
            // Middle bar
            if y == 11 && x >= 8 && x <= 15 {
                image_data[y * width + x] = 0;
            }
            // Bottom right vertical line
            if y >= 11 && y <= 18 && x == 16 {
                image_data[y * width + x] = 0;
            }
            // Bottom bar
            if y == 18 && x >= 8 && x <= 15 {
                image_data[y * width + x] = 0;
            }
        }
    }

    // Set the image data
    api.set_image(
        &image_data,
        width.try_into().unwrap(),
        height.try_into().unwrap(),
        bytes_per_pixel.try_into().unwrap(),
        bytes_per_line.try_into().unwrap(),
    )?;

    // Set whitelist for digits only
    api.set_variable("tessedit_char_whitelist", "0123456789")?;

    // Set PSM mode to single character
    api.set_variable("tessedit_pageseg_mode", "10")?;

    // Get the recognized text
    let text = api.get_utf8_text()?;
    println!("Recognized text: {}", text.trim());

    Ok(())
}
```

## Advanced Usage

The API provides additional functionality for more complex OCR tasks, including thread-safe operations:

```rust
use tesseract_rs::TesseractAPI;
use std::sync::Arc;
use std::thread;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new()?;

    // Initialize the main API
    api.init(tessdata_dir.to_str().unwrap(), "eng")?;
    api.set_variable("tessedit_pageseg_mode", "1")?;

    // Load and prepare image data
    let (image_data, width, height) = load_test_image("sample_text.png")?;

    // Share image data across threads
    let image_data = Arc::new(image_data);
    let mut handles = vec![];

    // Spawn multiple threads for parallel OCR processing
    for _ in 0..3 {
        let api_clone = api.clone(); // Clones the API with all configurations
        let image_data = Arc::clone(&image_data);

        let handle = thread::spawn(move || {
            // Set image in each thread
            let res = api_clone.set_image(
                &image_data,
                width as i32,
                height as i32,
                3,
                3 * width as i32,
            );
            assert!(res.is_ok());

            // Perform OCR in parallel
            let text = api_clone.get_utf8_text()
                .expect("Failed to get text");
            println!("Thread result: {}", text);
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

// Helper function to get tessdata directory
fn get_tessdata_dir() -> PathBuf {
    // ... (implementation as shown in basic example)
}

// Helper function to load test image
fn load_test_image(filename: &str) -> Result<(Vec<u8>, u32, u32), Box<dyn Error>> {
    let img = image::open(filename)?
        .to_rgb8();
    let (width, height) = img.dimensions();
    Ok((img.into_raw(), width, height))
}
```

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

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Start for Contributors

1. Fork and clone the repository
2. Install development dependencies:
   ```bash
   ./setup-hooks.sh
   ```
3. Make your changes following our commit message format
4. Run tests: `cargo test`
5. Submit a Pull Request

Our commit messages follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

## Acknowledgements

This project uses [Tesseract OCR](https://github.com/tesseract-ocr/tesseract) and [Leptonica](http://leptonica.org/). We are grateful to the maintainers and contributors of these projects.

```

```
