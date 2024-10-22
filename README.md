# tesseract-rs

`tesseract-rs` is a Rust binding for Tesseract OCR with built-in compilation of Tesseract and Leptonica libraries. This project aims to provide a safe and idiomatic Rust interface to Tesseract's functionality while handling the complexity of compiling the underlying C++ libraries.

## Features

- Safe Rust bindings for Tesseract OCR
- Built-in compilation of Tesseract and Leptonica
- Automatic download of Tesseract training data
- High-level Rust API for common OCR tasks

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tesseract-rs = { version = "0.1.18", features = ["build-tesseract"] }
```

## System Requirements

To build this crate, you need:

- A C++ compiler (e.g., gcc, clang)
- CMake
- Internet connection (for downloading Tesseract training data)

## Usage

Here's a basic example of how to use `tesseract-rs`:

```rust
use tesseract_rs::TesseractAPI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = TesseractAPI::new();
    api.init("path/to/tessdata", "eng")?;

    // Assume we have a 3x3 black and white image with a "1"
    let image_data: Vec<u8> = vec![
        0xFF, 0x00, 0xFF,
        0x00, 0x00, 0xFF,
        0x00, 0x00, 0xFF,
    ];

    api.set_image(&image_data, 3, 3, 1, 3);
    api.set_variable("tessedit_char_whitelist", "0123456789")?;

    let text = api.get_utf8_text()?;
    println!("Recognized text: {}", text);

    Ok(())
}
```

## Building

The crate will automatically download and compile Tesseract and Leptonica during the build process. This may take some time on the first build.

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
