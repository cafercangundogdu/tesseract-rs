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
//! use std::path::PathBuf;
//! use tesseract_rs::TesseractAPI;
//!
//! fn get_default_tessdata_dir() -> PathBuf {
//!     if cfg!(target_os = "macos") {
//!         let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
//!         PathBuf::from(home_dir)
//!             .join("Library")
//!             .join("Application Support")
//!             .join("tesseract-rs")
//!             .join("tessdata")
//!     } else if cfg!(target_os = "linux") {
//!         let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
//!         PathBuf::from(home_dir)
//!             .join(".tesseract-rs")
//!             .join("tessdata")
//!     } else if cfg!(target_os = "windows") {
//!         PathBuf::from(std::env::var("APPDATA").expect("APPDATA environment variable not set"))
//!             .join("tesseract-rs")
//!             .join("tessdata")
//!     } else {
//!         panic!("Unsupported operating system");
//!     }
//! }
//!
//! fn get_tessdata_dir() -> PathBuf {
//!     match std::env::var("TESSDATA_PREFIX") {
//!         Ok(dir) => {
//!             let path = PathBuf::from(dir);
//!             println!("Using TESSDATA_PREFIX directory: {:?}", path);
//!             path
//!         }
//!         Err(_) => {
//!             let default_dir = get_default_tessdata_dir();
//!             println!(
//!                 "TESSDATA_PREFIX not set, using default directory: {:?}",
//!                 default_dir
//!             );
//!             default_dir
//!         }
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let api = TesseractAPI::new();
//!     let tessdata_dir = get_tessdata_dir();
//!     api.init(tessdata_dir.to_str().unwrap(), "eng")?;
//!
//!     // Assume we have a 3x3 black and white image with a "1"
//!     let image_data: Vec<u8> = vec![
//!         0xFF, 0x00, 0xFF,
//!         0x00, 0x00, 0xFF,
//!         0x00, 0x00, 0xFF,
//!     ];
//!
//!     api.set_image(&image_data, 3, 3, 1, 3)?;
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
