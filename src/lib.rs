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
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when using the Tesseract API.
#[derive(Error, Debug)]
pub enum TesseractError {
    #[error("Failed to initialize Tesseract")]
    InitError,
    #[error("Failed to set image")]
    SetImageError,
    #[error("Failed to perform OCR")]
    OcrError,
    #[error("Invalid UTF-8 in Tesseract output")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for Tesseract operations.
pub type Result<T> = std::result::Result<T, TesseractError>;

/// Raw bindings to the Tesseract C API.
#[cfg(feature = "build-tesseract")]
#[link(name = "tesseract")]
extern "C" {
    fn TessBaseAPICreate() -> *mut libc::c_void;
    fn TessBaseAPIDelete(handle: *mut libc::c_void);
    fn TessBaseAPIInit3(handle: *mut libc::c_void, datapath: *const c_char, language: *const c_char) -> c_int;
    fn TessBaseAPISetImage(handle: *mut libc::c_void, imagedata: *const u8, width: c_int, height: c_int, bytes_per_pixel: c_int, bytes_per_line: c_int);
    fn TessBaseAPIGetUTF8Text(handle: *mut libc::c_void) -> *mut c_char;
    fn TessBaseAPIAllWordConfidences(handle: *mut libc::c_void) -> *const c_int;
    fn TessBaseAPISetVariable(handle: *mut libc::c_void, name: *const c_char, value: *const c_char) -> c_int;
}


/// Main interface to the Tesseract OCR engine.
#[cfg(feature = "build-tesseract")]
pub struct TesseractAPI {
    handle: *mut libc::c_void,
}

#[cfg(feature = "build-tesseract")]
impl TesseractAPI {
    /// Creates a new instance of the Tesseract API.
    pub fn new() -> Self {
        let handle = unsafe { TessBaseAPICreate() };
        TesseractAPI { handle }
    }

    /// Initializes the Tesseract engine with the specified datapath and language.
    ///
    /// # Arguments
    ///
    /// * `datapath` - Path to the directory containing Tesseract data files.
    /// * `language` - Language code (e.g., "eng" for English, "tur" for Turkish).
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initialization is successful, otherwise returns an error.
    pub fn init<P: AsRef<Path>>(&self, datapath: P, language: &str) -> Result<()> {
        let datapath = CString::new(datapath.as_ref().to_str().unwrap()).unwrap();
        let language = CString::new(language).unwrap();
        let result = unsafe {
            TessBaseAPIInit3(self.handle, datapath.as_ptr(), language.as_ptr())
        };
        if result != 0 {
            Err(TesseractError::InitError)
        } else {
            Ok(())
        }
    }

    /// Sets the image for OCR processing.
    ///
    /// # Arguments
    ///
    /// * `image_data` - Raw image data.
    /// * `width` - Width of the image.
    /// * `height` - Height of the image.
    /// * `bytes_per_pixel` - Number of bytes per pixel (e.g., 3 for RGB, 1 for grayscale).
    /// * `bytes_per_line` - Number of bytes per line (usually width * bytes_per_pixel, but might be padded).
    pub fn set_image(&self, image_data: &[u8], width: i32, height: i32, bytes_per_pixel: i32, bytes_per_line: i32) {
        unsafe {
            TessBaseAPISetImage(
                self.handle,
                image_data.as_ptr(),
                width,
                height,
                bytes_per_pixel,
                bytes_per_line,
            );
        }
    }

    /// Performs OCR on the set image and returns the recognized text.
    ///
    /// # Returns
    ///
    /// Returns the recognized text as a String if successful, otherwise returns an error.
    pub fn get_utf8_text(&self) -> Result<String> {
        let text_ptr = unsafe { TessBaseAPIGetUTF8Text(self.handle) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { libc::free(text_ptr as *mut libc::c_void) };
        Ok(result)
    }

    /// Gets the confidence values for all recognized words.
    ///
    /// # Returns
    ///
    /// Returns a vector of confidence values (0-100) for each recognized word.
    pub fn get_word_confidences(&self) -> Vec<i32> {
        let confidences_ptr = unsafe { TessBaseAPIAllWordConfidences(self.handle) };
        let mut confidences = Vec::new();
        let mut i = 0;
        while unsafe { *confidences_ptr.offset(i) } != -1 {
            confidences.push(unsafe { *confidences_ptr.offset(i) });
            i += 1;
        }
        confidences
    }

    /// Sets a Tesseract variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the variable.
    /// * `value` - Value to set.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if setting the variable is successful, otherwise returns an error.
    pub fn set_variable(&self, name: &str, value: &str) -> Result<()> {
        let name = CString::new(name).unwrap();
        let value = CString::new(value).unwrap();
        let result = unsafe {
            TessBaseAPISetVariable(self.handle, name.as_ptr(), value.as_ptr())
        };
        if result != 1 {
            Err(TesseractError::InitError)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "build-tesseract")]
impl Drop for TesseractAPI {
    fn drop(&mut self) {
        unsafe { TessBaseAPIDelete(self.handle) };
    }
}

#[cfg(test)]
#[cfg(feature = "build-tesseract")]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    fn setup() -> TesseractAPI {
        let api = TesseractAPI::new();
        let tessdata_dir = env::var("TESSDATA_PREFIX").expect("TESSDATA_PREFIX not set");
        println!("Fount tessdata_dir: {}", tessdata_dir.to_string());
        api.init(tessdata_dir, "eng").expect("Failed to initialize Tesseract");
        api
    }

    #[test]
    fn test_create_and_init() {
        let _api = setup();
    }

    #[test]
    fn test_set_variable() {
        let api = setup();
        assert!(api.set_variable("tessedit_char_whitelist", "0123456789").is_ok());
    }

    #[test]
    fn test_invalid_init() {
        let api = TesseractAPI::new();
        assert!(api.init("/invalid/path", "invalidlang").is_err());
    }

    #[test]
    fn test_get_utf8_text_empty_image() {
        let api = setup();
        // Set an empty image
        api.set_image(&[], 0, 0, 1, 0);
        let result = api.get_utf8_text();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_get_word_confidences_empty_image() {
        let api = setup();
        // Set an empty image
        api.set_image(&[], 0, 0, 1, 0);
        let confidences = api.get_word_confidences();
        assert!(confidences.is_empty());
    }

    #[test]
    fn test_set_variable_invalid() {
        let api = setup();
        assert!(api.set_variable("invalid_variable", "value").is_err());
    }

    #[test]
    fn test_simple_image_recognition() {
        let api = setup();

        // Create a simple 3x3 black and white image with a "1"
        let image_data: Vec<u8> = vec![
            0xFF, 0x00, 0xFF,
            0x00, 0x00, 0xFF,
            0x00, 0x00, 0xFF,
        ];

        api.set_image(&image_data, 3, 3, 1, 3);
        api.set_variable("tessedit_char_whitelist", "0123456789").unwrap();

        let result = api.get_utf8_text().unwrap();
        assert!(result.contains("1"));
    }
}