#![cfg_attr(not(feature = "build-tesseract"), allow(unused_variables, dead_code))]

//! # tesseract-rs
//!
//! `tesseract-rs` provides safe Rust bindings for Tesseract OCR with built-in compilation
//! of Tesseract and Leptonica libraries. This crate aims to make OCR functionality
//! easily accessible in Rust projects while handling the complexity of interfacing
//! with the underlying C++ libraries.
//!
//! ## Usage
//!
//! Here's a basic example of how to use `tesseract-rs`:
//!
//! ```rust
//! use tesseract_rs::TesseractAPI;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let api = TesseractAPI::new();
//!     api.init("path/to/tessdata", "eng")?;
//!
//!     // Assume we have a 3x3 black and white image with a "1"
//!     let image_data: Vec<u8> = vec![
//!         0xFF, 0x00, 0xFF,
//!         0x00, 0x00, 0xFF,
//!         0x00, 0x00, 0xFF,
//!     ];
//!
//!     api.set_image(&image_data, 3, 3, 1, 3);
//!     api.set_variable("tessedit_char_whitelist", "0123456789")?;
//!
//!     let text = api.get_utf8_text()?;
//!     println!("Recognized text: {}", text);
//!
//!     Ok(())
//! }
//! ```
pub use error::{Result, TesseractError};
mod error;
mod page_iterator;
pub use page_iterator::PageIterator;
mod result_iterator;
pub use result_iterator::ResultIterator;
mod choice_iterator;
pub use choice_iterator::ChoiceIterator;
mod monitor;
pub use monitor::TessMonitor;
mod result_renderer;
pub use result_renderer::TessResultRenderer;
mod mutable_iterator;
pub use mutable_iterator::MutableIterator;
mod enums;
pub use enums::{TessPageIteratorLevel, TessPageSegMode, TessPolyBlockType};
mod api;
pub use api::TesseractAPI;
