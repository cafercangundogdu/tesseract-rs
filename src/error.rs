use std::str::Utf8Error;
use thiserror::Error;

/// Errors that can occur when using the Tesseract API.
#[derive(Error, Debug)]
pub enum TesseractError {
    #[error("Failed to initialize Tesseract")]
    InitError,
    #[error("Failed to set image")]
    SetImageError,
    #[error("OCR operation failed")]
    OcrError,
    #[error("Invalid UTF-8 in Tesseract output")]
    Utf8Error(#[from] Utf8Error),
    #[error("Failed to lock mutex")]
    MutexLockError,
    #[error("Failed to set variable")]
    SetVariableError,
    #[error("Failed to get variable")]
    GetVariableError,
    #[error("Null pointer error")]
    NullPointerError,
    #[error("Invalid parameter")]
    InvalidParameterError,
    #[error("Layout analysis failed")]
    AnalyseLayoutError,
    #[error("Page processing failed")]
    ProcessPagesError,
    #[error("I/O error")]
    IoError,
    #[error("Mutex error")]
    MutexError,
    #[error("Invalid dimensions")]
    InvalidDimensions,
    #[error("Invalid bytes per pixel")]
    InvalidBytesPerPixel,
    #[error("Invalid bytes per line")]
    InvalidBytesPerLine,
    #[error("Invalid image data")]
    InvalidImageData,
    #[error("Uninitialized error")]
    UninitializedError,
}

/// Result type for Tesseract operations.
pub type Result<T> = std::result::Result<T, TesseractError>;
