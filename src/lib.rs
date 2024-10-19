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
use std::os::raw::{c_char, c_double, c_float, c_int, c_void};
use std::path::Path;
use std::sync::{Arc, Mutex};
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
    #[error("Mutex lock error")]
    MutexLockError,
    #[error("Failed to set variable")]
    SetVariableError,
    #[error("Failed to get variable")]
    GetVariableError,
    #[error("Null pointer error")]
    NullPointerError,
    #[error("Invalid parameter")]
    InvalidParameterError,
    #[error("Failed to analyse layout")]
    AnalyseLayoutError,
    #[error("Failed to process pages")]
    ProcessPagesError,
    #[error("I/O error")]
    IoError,
}

/// Result type for Tesseract operations.
pub type Result<T> = std::result::Result<T, TesseractError>;

/// Main interface to the Tesseract OCR engine.
#[cfg(feature = "build-tesseract")]
pub struct TesseractAPI {
    handle: Arc<Mutex<*mut c_void>>,
}

unsafe impl Send for TesseractAPI {}
unsafe impl Sync for TesseractAPI {}

#[cfg(feature = "build-tesseract")]
impl TesseractAPI {
    /// Creates a new instance of the Tesseract API.
    pub fn new() -> Self {
        let handle = unsafe { TessBaseAPICreate() };
        TesseractAPI {
            handle: Arc::new(Mutex::new(handle)),
        }
    }

    pub fn version() -> String {
        let version = unsafe { TessVersion() };
        unsafe { CStr::from_ptr(version) }
            .to_string_lossy()
            .into_owned()
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
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let result = unsafe { TessBaseAPIInit3(*handle, datapath.as_ptr(), language.as_ptr()) };
        if result != 0 {
            Err(TesseractError::InitError)
        } else {
            Ok(())
        }
    }

    /// Gets the confidence values for all recognized words.
    ///
    /// # Returns
    ///
    /// Returns a vector of confidence values (0-100) for each recognized word.
    pub fn get_word_confidences(&self) -> Result<Vec<i32>> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;

        let confidences_ptr = unsafe { TessBaseAPIAllWordConfidences(*handle) };
        let mut confidences = Vec::new();
        let mut i = 0;
        while unsafe { *confidences_ptr.offset(i) } != -1 {
            confidences.push(unsafe { *confidences_ptr.offset(i) });
            i += 1;
        }
        Ok(confidences)
    }

    pub fn mean_text_conf(&self) -> Result<i32> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        Ok(unsafe { TessBaseAPIMeanTextConf(*handle) })
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
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let result = unsafe { TessBaseAPISetVariable(*handle, name.as_ptr(), value.as_ptr()) };
        if result != 1 {
            Err(TesseractError::SetVariableError)
        } else {
            Ok(())
        }
    }

    pub fn get_string_variable(&self, name: &str) -> Result<String> {
        let name = CString::new(name).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let value_ptr = unsafe { TessBaseAPIGetStringVariable(*handle, name.as_ptr()) };
        if value_ptr.is_null() {
            return Err(TesseractError::GetVariableError);
        }
        let c_str = unsafe { CStr::from_ptr(value_ptr) };
        Ok(c_str.to_str()?.to_owned())
    }

    pub fn get_int_variable(&self, name: &str) -> Result<i32> {
        let name = CString::new(name).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        Ok(unsafe { TessBaseAPIGetIntVariable(*handle, name.as_ptr()) })
    }

    pub fn get_bool_variable(&self, name: &str) -> Result<bool> {
        let name = CString::new(name).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        Ok(unsafe { TessBaseAPIGetBoolVariable(*handle, name.as_ptr()) } != 0)
    }

    pub fn get_double_variable(&self, name: &str) -> Result<f64> {
        let name = CString::new(name).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        Ok(unsafe { TessBaseAPIGetDoubleVariable(*handle, name.as_ptr()) })
    }

    pub fn set_page_seg_mode(&self, mode: i32) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPISetPageSegMode(*handle, mode) };
        Ok(())
    }

    pub fn get_page_seg_mode(&self) -> Result<i32> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        Ok(unsafe { TessBaseAPIGetPageSegMode(*handle) })
    }

    pub fn recognize(&self) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let result = unsafe { TessBaseAPIRecognize(*handle, std::ptr::null_mut()) };
        if result != 0 {
            Err(TesseractError::OcrError)
        } else {
            Ok(())
        }
    }

    pub fn get_hocr_text(&self, page: i32) -> Result<String> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let text_ptr = unsafe { TessBaseAPIGetHOCRText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    pub fn get_alto_text(&self, page: i32) -> Result<String> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let text_ptr = unsafe { TessBaseAPIGetAltoText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    pub fn get_tsv_text(&self, page: i32) -> Result<String> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let text_ptr = unsafe { TessBaseAPIGetTsvText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    pub fn set_input_name(&self, name: &str) -> Result<()> {
        let name = CString::new(name).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPISetInputName(*handle, name.as_ptr()) };
        Ok(())
    }

    pub fn get_input_name(&self) -> Result<String> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let name_ptr = unsafe { TessBaseAPIGetInputName(*handle) };
        if name_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        let c_str = unsafe { CStr::from_ptr(name_ptr) };
        Ok(c_str.to_str()?.to_owned())
    }

    pub fn get_datapath(&self) -> Result<String> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let path_ptr = unsafe { TessBaseAPIGetDatapath(*handle) };
        if path_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        let c_str = unsafe { CStr::from_ptr(path_ptr) };
        Ok(c_str.to_str()?.to_owned())
    }

    pub fn get_source_y_resolution(&self) -> Result<i32> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        Ok(unsafe { TessBaseAPIGetSourceYResolution(*handle) })
    }

    pub fn get_thresholded_image(&self) -> Result<*mut c_void> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let pix = unsafe { TessBaseAPIGetThresholdedImage(*handle) };
        if pix.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(pix)
        }
    }

    pub fn get_box_text(&self, page: i32) -> Result<String> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let text_ptr = unsafe { TessBaseAPIGetBoxText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    pub fn get_lstm_box_text(&self, page: i32) -> Result<String> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let text_ptr = unsafe { TessBaseAPIGetLSTMBoxText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    pub fn get_word_str_box_text(&self, page: i32) -> Result<String> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let text_ptr = unsafe { TessBaseAPIGetWordStrBoxText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    pub fn get_unlv_text(&self) -> Result<String> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let text_ptr = unsafe { TessBaseAPIGetUNLVText(*handle) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    pub fn all_word_confidences(&self) -> Result<Vec<i32>> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let confidences_ptr = unsafe { TessBaseAPIAllWordConfidences(*handle) };
        if confidences_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        let mut confidences = Vec::new();
        let mut i = 0;
        while unsafe { *confidences_ptr.offset(i) } != -1 {
            confidences.push(unsafe { *confidences_ptr.offset(i) });
            i += 1;
        }
        unsafe { TessDeleteIntArray(confidences_ptr) };
        Ok(confidences)
    }

    pub fn adapt_to_word_str(&self, mode: i32, wordstr: &str) -> Result<bool> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let wordstr = CString::new(wordstr).unwrap();
        let result = unsafe { TessBaseAPIAdaptToWordStr(*handle, mode, wordstr.as_ptr()) };
        Ok(result != 0)
    }

    pub fn detect_os(&self) -> Result<(i32, f32, String, f32)> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let mut orient_deg = 0;
        let mut orient_conf = 0.0;
        let mut script_name_ptr = std::ptr::null_mut();
        let mut script_conf = 0.0;
        let result = unsafe {
            TessBaseAPIDetectOrientationScript(
                *handle,
                &mut orient_deg,
                &mut orient_conf,
                &mut script_name_ptr,
                &mut script_conf,
            )
        };
        if result == 0 {
            return Err(TesseractError::OcrError);
        }
        let script_name = if !script_name_ptr.is_null() {
            let c_str = unsafe { CStr::from_ptr(script_name_ptr) };
            let result = c_str.to_str()?.to_owned();
            unsafe { TessDeleteText(script_name_ptr) };
            result
        } else {
            String::new()
        };
        Ok((orient_deg, orient_conf, script_name, script_conf))
    }

    pub fn set_min_orientation_margin(&self, margin: f64) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPISetMinOrientationMargin(*handle, margin) };
        Ok(())
    }

    pub fn get_page_iterator(&self) -> Result<PageIterator> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let iterator = unsafe { TessBaseAPIGetIterator(*handle) };
        if iterator.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        Ok(PageIterator { handle: iterator })
    }

    pub fn set_input_image(&self, pix: *mut c_void) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPISetInputImage(*handle, pix) };
        Ok(())
    }

    pub fn get_input_image(&self) -> Result<*mut c_void> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let pix = unsafe { TessBaseAPIGetInputImage(*handle) };
        if pix.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(pix)
        }
    }

    pub fn set_output_name(&self, name: &str) -> Result<()> {
        let name = CString::new(name).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPISetOutputName(*handle, name.as_ptr()) };
        Ok(())
    }

    pub fn set_debug_variable(&self, name: &str, value: &str) -> Result<()> {
        let name = CString::new(name).unwrap();
        let value = CString::new(value).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let result = unsafe { TessBaseAPISetDebugVariable(*handle, name.as_ptr(), value.as_ptr()) };
        if result != 1 {
            Err(TesseractError::SetVariableError)
        } else {
            Ok(())
        }
    }

    pub fn print_variables_to_file(&self, filename: &str) -> Result<()> {
        let filename = CString::new(filename).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let result = unsafe { TessBaseAPIPrintVariablesToFile(*handle, filename.as_ptr()) };
        if result != 0 {
            Err(TesseractError::IoError)
        } else {
            Ok(())
        }
    }

    pub fn init_for_analyse_page(&self) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPIInitForAnalysePage(*handle) };
        Ok(())
    }

    pub fn read_config_file(&self, filename: &str) -> Result<()> {
        let filename = CString::new(filename).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPIReadConfigFile(*handle, filename.as_ptr()) };
        Ok(())
    }

    pub fn read_debug_config_file(&self, filename: &str) -> Result<()> {
        let filename = CString::new(filename).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPIReadDebugConfigFile(*handle, filename.as_ptr()) };
        Ok(())
    }

    pub fn get_thresholded_image_scale_factor(&self) -> Result<i32> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        Ok(unsafe { TessBaseAPIGetThresholdedImageScaleFactor(*handle) })
    }

    pub fn process_pages(
        &self,
        filename: &str,
        retry_config: Option<&str>,
        timeout_millisec: i32,
    ) -> Result<String> {
        let filename = CString::new(filename).unwrap();
        let retry_config = retry_config.map(|s| CString::new(s).unwrap());
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let result = unsafe {
            TessBaseAPIProcessPages(
                *handle,
                filename.as_ptr(),
                retry_config.map_or(std::ptr::null(), |rc| rc.as_ptr()),
                timeout_millisec,
                std::ptr::null_mut(), // renderer
            )
        };
        if result.is_null() {
            Err(TesseractError::ProcessPagesError)
        } else {
            let c_str = unsafe { CStr::from_ptr(result) };
            let output = c_str.to_str()?.to_owned();
            unsafe { TessDeleteText(result) };
            Ok(output)
        }
    }

    pub fn get_init_languages_as_string(&self) -> Result<String> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let result = unsafe { TessBaseAPIGetInitLanguagesAsString(*handle) };
        if result.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            let c_str = unsafe { CStr::from_ptr(result) };
            Ok(c_str.to_str()?.to_owned())
        }
    }

    pub fn get_loaded_languages(&self) -> Result<Vec<String>> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let vec_ptr = unsafe { TessBaseAPIGetLoadedLanguagesAsVector(*handle) };
        self.string_vec_to_rust(vec_ptr)
    }

    pub fn get_available_languages(&self) -> Result<Vec<String>> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let vec_ptr = unsafe { TessBaseAPIGetAvailableLanguagesAsVector(*handle) };
        self.string_vec_to_rust(vec_ptr)
    }

    fn string_vec_to_rust(&self, vec_ptr: *mut *mut c_char) -> Result<Vec<String>> {
        if vec_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        let mut result = Vec::new();
        let mut i = 0;
        loop {
            let str_ptr = unsafe { *vec_ptr.offset(i) };
            if str_ptr.is_null() {
                break;
            }
            let c_str = unsafe { CStr::from_ptr(str_ptr) };
            result.push(c_str.to_str()?.to_owned());
            i += 1;
        }
        unsafe { TessDeleteTextArray(vec_ptr) };
        Ok(result)
    }

    pub fn clear_adaptive_classifier(&self) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPIClearAdaptiveClassifier(*handle) };
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPIClear(*handle) };
        Ok(())
    }

    pub fn end(&self) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPIEnd(*handle) };
        Ok(())
    }

    pub fn is_valid_word(&self, word: &str) -> Result<i32> {
        let word = CString::new(word).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        Ok(unsafe { TessBaseAPIIsValidWord(*handle, word.as_ptr()) })
    }

    pub fn get_text_direction(&self) -> Result<(i32, f32)> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let mut out_degrees = 0;
        let mut out_confidence = 0.0;
        unsafe {
            TessBaseAPIGetTextDirection(*handle, &mut out_degrees, &mut out_confidence);
        }
        Ok((out_degrees, out_confidence))
    }

    pub fn init_1(&self, datapath: &str, language: &str, oem: i32, configs: &[&str]) -> Result<()> {
        let datapath = CString::new(datapath).unwrap();
        let language = CString::new(language).unwrap();
        let config_ptrs: Vec<_> = configs.iter().map(|&s| CString::new(s).unwrap()).collect();
        let config_ptr_ptrs: Vec<_> = config_ptrs.iter().map(|cs| cs.as_ptr()).collect();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let result = unsafe {
            TessBaseAPIInit1(
                *handle,
                datapath.as_ptr(),
                language.as_ptr(),
                oem,
                config_ptr_ptrs.as_ptr(),
                config_ptrs.len() as c_int,
            )
        };
        if result != 0 {
            Err(TesseractError::InitError)
        } else {
            Ok(())
        }
    }

    pub fn init_2(&self, datapath: &str, language: &str, oem: i32) -> Result<()> {
        let datapath = CString::new(datapath).unwrap();
        let language = CString::new(language).unwrap();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let result =
            unsafe { TessBaseAPIInit2(*handle, datapath.as_ptr(), language.as_ptr(), oem) };
        if result != 0 {
            Err(TesseractError::InitError)
        } else {
            Ok(())
        }
    }

    pub fn init_4(&self, datapath: &str, language: &str, oem: i32, configs: &[&str]) -> Result<()> {
        let datapath = CString::new(datapath).unwrap();
        let language = CString::new(language).unwrap();
        let config_ptrs: Vec<_> = configs.iter().map(|&s| CString::new(s).unwrap()).collect();
        let config_ptr_ptrs: Vec<_> = config_ptrs.iter().map(|cs| cs.as_ptr()).collect();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let result = unsafe {
            TessBaseAPIInit4(
                *handle,
                datapath.as_ptr(),
                language.as_ptr(),
                oem,
                config_ptr_ptrs.as_ptr(),
                config_ptrs.len() as c_int,
            )
        };
        if result != 0 {
            Err(TesseractError::InitError)
        } else {
            Ok(())
        }
    }

    pub fn init_5(
        &self,
        data: &[u8],
        data_size: i32,
        language: &str,
        oem: i32,
        configs: &[&str],
    ) -> Result<()> {
        let language = CString::new(language).unwrap();
        let config_ptrs: Vec<_> = configs.iter().map(|&s| CString::new(s).unwrap()).collect();
        let config_ptr_ptrs: Vec<_> = config_ptrs.iter().map(|cs| cs.as_ptr()).collect();
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let result = unsafe {
            TessBaseAPIInit5(
                *handle,
                data.as_ptr(),
                data_size,
                language.as_ptr(),
                oem,
                config_ptr_ptrs.as_ptr(),
                config_ptrs.len() as c_int,
            )
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
    pub fn set_image(
        &self,
        image_data: &[u8],
        width: i32,
        height: i32,
        bytes_per_pixel: i32,
        bytes_per_line: i32,
    ) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe {
            TessBaseAPISetImage(
                *handle,
                image_data.as_ptr(),
                width,
                height,
                bytes_per_pixel,
                bytes_per_line,
            );
        }
        Ok(())
    }

    pub fn set_image_2(&self, pix: *mut c_void) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPISetImage2(*handle, pix) };
        Ok(())
    }

    pub fn set_source_resolution(&self, ppi: i32) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPISetSourceResolution(*handle, ppi) };
        Ok(())
    }

    pub fn set_rectangle(&self, left: i32, top: i32, width: i32, height: i32) -> Result<()> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        unsafe { TessBaseAPISetRectangle(*handle, left, top, width, height) };
        Ok(())
    }

    /// Performs OCR on the set image and returns the recognized text.
    ///
    /// # Returns
    ///
    /// Returns the recognized text as a String if successful, otherwise returns an error.
    pub fn get_utf8_text(&self) -> Result<String> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let text_ptr = unsafe { TessBaseAPIGetUTF8Text(*handle) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    pub fn get_iterator(&self) -> Result<ResultIterator> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let iterator = unsafe { TessBaseAPIGetIterator(*handle) };
        if iterator.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(ResultIterator::new(iterator))
        }
    }

    pub fn get_mutable_iterator(&self) -> Result<ResultIterator> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let iterator = unsafe { TessBaseAPIGetMutableIterator(*handle) };
        if iterator.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(ResultIterator::new(iterator))
        }
    }

    pub fn analyse_layout(&self) -> Result<PageIterator> {
        let handle = self
            .handle
            .lock()
            .map_err(|_| TesseractError::MutexLockError)?;
        let iterator = unsafe { TessBaseAPIAnalyseLayout(*handle) };
        if iterator.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(PageIterator::new(iterator))
        }
    }

    pub fn get_unichar(unichar_id: i32) -> Result<String> {
        let char_ptr = unsafe { TessGetUnichar(unichar_id) };
        if char_ptr.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            let c_str = unsafe { CStr::from_ptr(char_ptr) };
            Ok(c_str.to_str()?.to_owned())
        }
    }
}

#[cfg(feature = "build-tesseract")]
impl Drop for TesseractAPI {
    fn drop(&mut self) {
        if let Ok(handle) = self.handle.lock() {
            unsafe { TessBaseAPIDelete(*handle) };
        }
    }
}

pub struct PageIterator {
    handle: *mut c_void,
}

impl PageIterator {
    pub fn new(handle: *mut c_void) -> Self {
        PageIterator { handle }
    }

    pub fn begin(&self) {
        unsafe { TessPageIteratorBegin(self.handle) };
    }

    pub fn next(&self, level: i32) -> bool {
        unsafe { TessPageIteratorNext(self.handle, level) != 0 }
    }

    pub fn is_at_beginning_of(&self, level: i32) -> bool {
        unsafe { TessPageIteratorIsAtBeginningOf(self.handle, level) != 0 }
    }

    pub fn is_at_final_element(&self, level: i32, element: i32) -> bool {
        unsafe { TessPageIteratorIsAtFinalElement(self.handle, level, element) != 0 }
    }

    pub fn bounding_box(&self, level: i32) -> Result<(i32, i32, i32, i32)> {
        let mut left = 0;
        let mut top = 0;
        let mut right = 0;
        let mut bottom = 0;
        let result = unsafe {
            TessPageIteratorBoundingBox(
                self.handle,
                level,
                &mut left,
                &mut top,
                &mut right,
                &mut bottom,
            )
        };
        if result == 0 {
            Err(TesseractError::InvalidParameterError)
        } else {
            Ok((left, top, right, bottom))
        }
    }

    pub fn block_type(&self) -> i32 {
        unsafe { TessPageIteratorBlockType(self.handle) }
    }

    pub fn baseline(&self, level: i32) -> Result<(i32, i32, i32, i32)> {
        let mut x1 = 0;
        let mut y1 = 0;
        let mut x2 = 0;
        let mut y2 = 0;
        let result = unsafe {
            TessPageIteratorBaseline(self.handle, level, &mut x1, &mut y1, &mut x2, &mut y2)
        };
        if result == 0 {
            Err(TesseractError::InvalidParameterError)
        } else {
            Ok((x1, y1, x2, y2))
        }
    }

    pub fn orientation(&self) -> Result<(i32, i32, i32, f32)> {
        let mut orientation = 0;
        let mut writing_direction = 0;
        let mut textline_order = 0;
        let mut deskew_angle = 0.0;
        let result = unsafe {
            TessPageIteratorOrientation(
                self.handle,
                &mut orientation,
                &mut writing_direction,
                &mut textline_order,
                &mut deskew_angle,
            )
        };
        if result == 0 {
            Err(TesseractError::InvalidParameterError)
        } else {
            Ok((orientation, writing_direction, textline_order, deskew_angle))
        }
    }
}

impl Drop for PageIterator {
    fn drop(&mut self) {
        unsafe { TessPageIteratorDelete(self.handle) };
    }
}

pub struct ResultIterator {
    handle: *mut c_void,
}

impl ResultIterator {
    pub fn new(handle: *mut c_void) -> Self {
        ResultIterator { handle }
    }

    pub fn get_utf8_text(&self, level: i32) -> Result<String> {
        let text_ptr = unsafe { TessResultIteratorGetUTF8Text(self.handle, level) };
        if text_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr as *mut c_char) };
        Ok(result)
    }

    pub fn confidence(&self, level: i32) -> f32 {
        unsafe { TessResultIteratorConfidence(self.handle, level) }
    }

    pub fn word_recognition_language(&self) -> Result<String> {
        let lang_ptr = unsafe { TessResultIteratorWordRecognitionLanguage(self.handle) };
        if lang_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        let c_str = unsafe { CStr::from_ptr(lang_ptr) };
        Ok(c_str.to_str()?.to_owned())
    }

    pub fn word_font_attributes(&self) -> Result<(bool, bool, bool, bool, bool, bool, i32, i32)> {
        let mut is_bold = 0;
        let mut is_italic = 0;
        let mut is_underlined = 0;
        let mut is_monospace = 0;
        let mut is_serif = 0;
        let mut is_smallcaps = 0;
        let mut pointsize = 0;
        let mut font_id = 0;

        let result = unsafe {
            TessResultIteratorWordFontAttributes(
                self.handle,
                &mut is_bold,
                &mut is_italic,
                &mut is_underlined,
                &mut is_monospace,
                &mut is_serif,
                &mut is_smallcaps,
                &mut pointsize,
                &mut font_id,
            )
        };

        if result == 0 {
            Err(TesseractError::InvalidParameterError)
        } else {
            Ok((
                is_bold != 0,
                is_italic != 0,
                is_underlined != 0,
                is_monospace != 0,
                is_serif != 0,
                is_smallcaps != 0,
                pointsize,
                font_id,
            ))
        }
    }

    pub fn word_is_from_dictionary(&self) -> bool {
        unsafe { TessResultIteratorWordIsFromDictionary(self.handle) != 0 }
    }

    pub fn word_is_numeric(&self) -> bool {
        unsafe { TessResultIteratorWordIsNumeric(self.handle) != 0 }
    }

    pub fn symbol_is_superscript(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsSuperscript(self.handle) != 0 }
    }

    pub fn symbol_is_subscript(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsSubscript(self.handle) != 0 }
    }

    pub fn symbol_is_dropcap(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsDropcap(self.handle) != 0 }
    }
}

impl Drop for ResultIterator {
    fn drop(&mut self) {
        unsafe { TessResultIteratorDelete(self.handle) };
    }
}

pub struct ChoiceIterator {
    handle: *mut c_void,
}

impl ChoiceIterator {
    pub fn new(handle: *mut c_void) -> Self {
        ChoiceIterator { handle }
    }

    pub fn next(&self) -> bool {
        unsafe { TessChoiceIteratorNext(self.handle) != 0 }
    }

    pub fn get_utf8_text(&self) -> Result<String> {
        let text_ptr = unsafe { TessChoiceIteratorGetUTF8Text(self.handle) };
        if text_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    pub fn confidence(&self) -> f32 {
        unsafe { TessChoiceIteratorConfidence(self.handle) }
    }
}

impl Drop for ChoiceIterator {
    fn drop(&mut self) {
        unsafe { TessChoiceIteratorDelete(self.handle) };
    }
}

pub struct TessMonitor {
    handle: *mut c_void,
}

impl TessMonitor {
    pub fn new() -> Self {
        let handle = unsafe { TessMonitorCreate() };
        TessMonitor { handle }
    }

    pub fn set_deadline(&mut self, deadline: i32) {
        unsafe { TessMonitorSetDeadlineMSecs(self.handle, deadline) };
    }

    pub fn get_progress(&self) -> i32 {
        unsafe { TessMonitorGetProgress(self.handle) }
    }
}

impl Drop for TessMonitor {
    fn drop(&mut self) {
        unsafe { TessMonitorDelete(self.handle) };
    }
}

pub struct TessResultRenderer {
    handle: *mut c_void,
}

impl TessResultRenderer {
    pub fn new_text_renderer(outputbase: &str) -> Result<Self> {
        let outputbase = CString::new(outputbase).unwrap();
        let handle = unsafe { TessTextRendererCreate(outputbase.as_ptr()) };
        if handle.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(TessResultRenderer { handle })
        }
    }

    pub fn new_hocr_renderer(outputbase: &str) -> Result<Self> {
        let outputbase = CString::new(outputbase).unwrap();
        let handle = unsafe { TessHOcrRendererCreate(outputbase.as_ptr()) };
        if handle.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(TessResultRenderer { handle })
        }
    }

    pub fn new_pdf_renderer(outputbase: &str, datadir: &str, textonly: bool) -> Result<Self> {
        let outputbase = CString::new(outputbase).unwrap();
        let datadir = CString::new(datadir).unwrap();
        let handle = unsafe {
            TessPDFRendererCreate(outputbase.as_ptr(), datadir.as_ptr(), textonly as c_int)
        };
        if handle.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(TessResultRenderer { handle })
        }
    }

    pub fn begin_document(&self, title: &str) -> bool {
        let title = CString::new(title).unwrap();
        unsafe { TessResultRendererBeginDocument(self.handle, title.as_ptr()) != 0 }
    }

    pub fn add_image(&self, api: &crate::TesseractAPI) -> bool {
        let api_handle = api.handle.lock().unwrap();
        unsafe { TessResultRendererAddImage(self.handle, *api_handle) != 0 }
    }

    pub fn end_document(&self) -> bool {
        unsafe { TessResultRendererEndDocument(self.handle) != 0 }
    }

    pub fn get_extension(&self) -> Result<String> {
        let ext_ptr = unsafe { TessResultRendererExtention(self.handle) };
        if ext_ptr.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            let c_str = unsafe { CStr::from_ptr(ext_ptr) };
            Ok(c_str.to_str()?.to_owned())
        }
    }

    pub fn get_title(&self) -> Result<String> {
        let title_ptr = unsafe { TessResultRendererTitle(self.handle) };
        if title_ptr.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            let c_str = unsafe { CStr::from_ptr(title_ptr) };
            Ok(c_str.to_str()?.to_owned())
        }
    }

    pub fn get_image_num(&self) -> i32 {
        unsafe { TessResultRendererImageNum(self.handle) }
    }

    pub fn process_pages(
        &self,
        filename: &str,
        retry_config: Option<&str>,
        timeout_millisec: i32,
        renderer: Option<&TessResultRenderer>,
    ) -> Result<String> {
        let filename = CString::new(filename).unwrap();
        let retry_config = retry_config.map(|s| CString::new(s).unwrap());
        /* let handle = self
        .handle
        .lock()
        .map_err(|_| TesseractError::MutexLockError)?; */
        let result = unsafe {
            TessBaseAPIProcessPages(
                self.handle,
                filename.as_ptr(),
                retry_config
                    .as_ref()
                    .map_or(std::ptr::null(), |rc| rc.as_ptr()),
                timeout_millisec,
                renderer.map_or(std::ptr::null_mut(), |r| r.handle),
            )
        };
        if result.is_null() {
            Err(TesseractError::ProcessPagesError)
        } else {
            let c_str = unsafe { CStr::from_ptr(result) };
            let output = c_str.to_str()?.to_owned();
            unsafe { TessDeleteText(result) };
            Ok(output)
        }
    }

    pub fn process_page(
        &self,
        pix: *mut c_void,
        page_index: i32,
        filename: &str,
        retry_config: Option<&str>,
        timeout_millisec: i32,
        renderer: Option<&TessResultRenderer>,
    ) -> Result<bool> {
        let filename = CString::new(filename).unwrap();
        let retry_config = retry_config.map(|s| CString::new(s).unwrap());
        /* let handle = self
        .handle
        .lock()
        .map_err(|_| TesseractError::MutexLockError)?; */
        let result = unsafe {
            TessBaseAPIProcessPage(
                self.handle,
                pix,
                page_index,
                filename.as_ptr(),
                retry_config
                    .as_ref()
                    .map_or(std::ptr::null(), |rc| rc.as_ptr()),
                timeout_millisec,
                renderer.map_or(std::ptr::null_mut(), |r| r.handle),
            )
        };
        Ok(result != 0)
    }
}

impl Drop for TessResultRenderer {
    fn drop(&mut self) {
        unsafe { TessDeleteResultRenderer(self.handle) };
    }
}

pub struct MutableIterator {
    handle: *mut c_void,
}

impl MutableIterator {
    pub fn new(handle: *mut c_void) -> Self {
        MutableIterator { handle }
    }

    pub fn get_utf8_text(&self, level: i32) -> Result<String> {
        let text_ptr = unsafe { TessResultIteratorGetUTF8Text(self.handle, level) };
        if text_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr as *mut c_char) };
        Ok(result)
    }

    pub fn confidence(&self, level: i32) -> f32 {
        unsafe { TessResultIteratorConfidence(self.handle, level) }
    }

    pub fn word_recognition_language(&self) -> Result<String> {
        let lang_ptr = unsafe { TessResultIteratorWordRecognitionLanguage(self.handle) };
        if lang_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        let c_str = unsafe { CStr::from_ptr(lang_ptr) };
        Ok(c_str.to_str()?.to_owned())
    }

    pub fn word_font_attributes(&self) -> Result<(bool, bool, bool, bool, bool, bool, i32, i32)> {
        let mut is_bold = 0;
        let mut is_italic = 0;
        let mut is_underlined = 0;
        let mut is_monospace = 0;
        let mut is_serif = 0;
        let mut is_smallcaps = 0;
        let mut pointsize = 0;
        let mut font_id = 0;

        let result = unsafe {
            TessResultIteratorWordFontAttributes(
                self.handle,
                &mut is_bold,
                &mut is_italic,
                &mut is_underlined,
                &mut is_monospace,
                &mut is_serif,
                &mut is_smallcaps,
                &mut pointsize,
                &mut font_id,
            )
        };

        if result == 0 {
            Err(TesseractError::InvalidParameterError)
        } else {
            Ok((
                is_bold != 0,
                is_italic != 0,
                is_underlined != 0,
                is_monospace != 0,
                is_serif != 0,
                is_smallcaps != 0,
                pointsize,
                font_id,
            ))
        }
    }

    pub fn word_is_from_dictionary(&self) -> bool {
        unsafe { TessResultIteratorWordIsFromDictionary(self.handle) != 0 }
    }

    pub fn word_is_numeric(&self) -> bool {
        unsafe { TessResultIteratorWordIsNumeric(self.handle) != 0 }
    }

    pub fn symbol_is_superscript(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsSuperscript(self.handle) != 0 }
    }

    pub fn symbol_is_subscript(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsSubscript(self.handle) != 0 }
    }

    pub fn symbol_is_dropcap(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsDropcap(self.handle) != 0 }
    }

    pub fn next(&self, level: i32) -> bool {
        unsafe { TessResultIteratorNext(self.handle, level) != 0 }
    }

    pub fn set_value(&self, level: i32, value: &str) -> Result<bool> {
        let c_value = CString::new(value).unwrap();
        let result = unsafe { TessMutableIteratorSetValue(self.handle, level, c_value.as_ptr()) };
        Ok(result != 0)
    }

    pub fn delete(&self) -> Result<bool> {
        let result = unsafe { TessMutableIteratorDelete(self.handle) };
        Ok(result != 0)
    }
}

impl Drop for MutableIterator {
    fn drop(&mut self) {
        unsafe { TessResultIteratorDelete(self.handle) };
    }
}

#[cfg(feature = "build-tesseract")]
#[link(name = "tesseract")]
extern "C" {
    fn TessBaseAPIMeanTextConf(handle: *mut c_void) -> c_int;
    fn TessBaseAPISetVariable(
        handle: *mut c_void,
        name: *const c_char,
        value: *const c_char,
    ) -> c_int;
    fn TessBaseAPIGetStringVariable(handle: *mut c_void, name: *const c_char) -> *const c_char;
    fn TessBaseAPIGetIntVariable(handle: *mut c_void, name: *const c_char) -> c_int;
    fn TessBaseAPIGetBoolVariable(handle: *mut c_void, name: *const c_char) -> c_int;
    fn TessBaseAPIGetDoubleVariable(handle: *mut c_void, name: *const c_char) -> c_double;
    fn TessBaseAPISetPageSegMode(handle: *mut c_void, mode: c_int);
    fn TessBaseAPIGetPageSegMode(handle: *mut c_void) -> c_int;
    fn TessBaseAPIRecognize(handle: *mut c_void, monitor: *mut c_void) -> c_int;
    fn TessBaseAPIGetHOCRText(handle: *mut c_void, page: c_int) -> *mut c_char;

    fn TessBaseAPIGetAltoText(handle: *mut c_void, page: c_int) -> *mut c_char;
    fn TessBaseAPIGetTsvText(handle: *mut c_void, page: c_int) -> *mut c_char;
    fn TessBaseAPIGetBoxText(handle: *mut c_void, page: c_int) -> *mut c_char;
    fn TessBaseAPIGetLSTMBoxText(handle: *mut c_void, page: c_int) -> *mut c_char;
    fn TessBaseAPIGetWordStrBoxText(handle: *mut c_void, page: c_int) -> *mut c_char;
    fn TessBaseAPIGetUNLVText(handle: *mut c_void) -> *mut c_char;
    fn TessBaseAPIAllWordConfidences(handle: *mut c_void) -> *const c_int;
    fn TessBaseAPIAdaptToWordStr(handle: *mut c_void, mode: c_int, wordstr: *const c_char)
        -> c_int;
    fn TessBaseAPIDetectOrientationScript(
        handle: *mut c_void,
        orient_deg: *mut c_int,
        orient_conf: *mut c_float,
        script_name: *mut *mut c_char,
        script_conf: *mut c_float,
    ) -> c_int;
    fn TessBaseAPISetMinOrientationMargin(handle: *mut c_void, margin: c_double);
    fn TessBaseAPIGetIterator(handle: *mut c_void) -> *mut c_void;
    fn TessBaseAPIGetMutableIterator(handle: *mut c_void) -> *mut c_void;
    fn TessPageIteratorDelete(handle: *mut c_void);
    fn TessPageIteratorBegin(handle: *mut c_void);
    fn TessPageIteratorNext(handle: *mut c_void, level: c_int) -> c_int;
    fn TessPageIteratorBoundingBox(
        handle: *mut c_void,
        level: c_int,
        left: *mut c_int,
        top: *mut c_int,
        right: *mut c_int,
        bottom: *mut c_int,
    ) -> c_int;
    fn TessResultIteratorDelete(handle: *mut c_void);
    fn TessDeleteIntArray(arr: *const c_int);
    fn TessBaseAPISetInputImage(handle: *mut c_void, pix: *mut c_void);
    fn TessBaseAPIGetInputImage(handle: *mut c_void) -> *mut c_void;
    fn TessBaseAPISetOutputName(handle: *mut c_void, name: *const c_char);
    fn TessBaseAPISetDebugVariable(
        handle: *mut c_void,
        name: *const c_char,
        value: *const c_char,
    ) -> c_int;
    fn TessBaseAPIPrintVariablesToFile(handle: *mut c_void, filename: *const c_char) -> c_int;
    fn TessBaseAPIInitForAnalysePage(handle: *mut c_void);
    fn TessBaseAPIReadConfigFile(handle: *mut c_void, filename: *const c_char);
    fn TessBaseAPIReadDebugConfigFile(handle: *mut c_void, filename: *const c_char);
    fn TessBaseAPIGetThresholdedImageScaleFactor(handle: *mut c_void) -> c_int;
    fn TessBaseAPIAnalyseLayout(handle: *mut c_void) -> *mut c_void;
    fn TessBaseAPIGetInitLanguagesAsString(handle: *mut c_void) -> *const c_char;
    fn TessBaseAPIGetLoadedLanguagesAsVector(handle: *mut c_void) -> *mut *mut c_char;
    fn TessBaseAPIGetAvailableLanguagesAsVector(handle: *mut c_void) -> *mut *mut c_char;
    fn TessBaseAPIClearAdaptiveClassifier(handle: *mut c_void);
    fn TessDeleteTextArray(arr: *mut *mut c_char);

    fn TessVersion() -> *const c_char;
    fn TessBaseAPICreate() -> *mut c_void;
    fn TessBaseAPIDelete(handle: *mut c_void);
    fn TessBaseAPIInit3(
        handle: *mut c_void,
        datapath: *const c_char,
        language: *const c_char,
    ) -> c_int;
    fn TessBaseAPIInit1(
        handle: *mut c_void,
        datapath: *const c_char,
        language: *const c_char,
        oem: c_int,
        configs: *const *const c_char,
        configs_size: c_int,
    ) -> c_int;
    fn TessBaseAPIInit2(
        handle: *mut c_void,
        datapath: *const c_char,
        language: *const c_char,
        oem: c_int,
    ) -> c_int;
    fn TessBaseAPIInit4(
        handle: *mut c_void,
        datapath: *const c_char,
        language: *const c_char,
        oem: c_int,
        configs: *const *const c_char,
        configs_size: c_int,
    ) -> c_int;
    fn TessBaseAPIInit5(
        handle: *mut c_void,
        data: *const u8,
        data_size: c_int,
        language: *const c_char,
        oem: c_int,
        configs: *const *const c_char,
        configs_size: c_int,
    ) -> c_int;
    fn TessBaseAPISetImage(
        handle: *mut c_void,
        imagedata: *const u8,
        width: c_int,
        height: c_int,
        bytes_per_pixel: c_int,
        bytes_per_line: c_int,
    );
    fn TessBaseAPISetImage2(handle: *mut c_void, pix: *mut c_void);
    fn TessBaseAPISetSourceResolution(handle: *mut c_void, ppi: c_int);
    fn TessBaseAPISetRectangle(
        handle: *mut c_void,
        left: c_int,
        top: c_int,
        width: c_int,
        height: c_int,
    );
    fn TessBaseAPIGetUTF8Text(handle: *mut c_void) -> *mut c_char;
    fn TessBaseAPIClear(handle: *mut c_void);
    fn TessBaseAPIEnd(handle: *mut c_void);
    fn TessBaseAPIIsValidWord(handle: *mut c_void, word: *const c_char) -> c_int;
    fn TessBaseAPIGetTextDirection(
        handle: *mut c_void,
        out_degrees: *mut c_int,
        out_confidence: *mut c_float,
    );
    fn TessDeleteText(text: *mut c_char);

    fn TessPageIteratorIsAtBeginningOf(handle: *mut c_void, level: c_int) -> c_int;
    fn TessPageIteratorIsAtFinalElement(handle: *mut c_void, level: c_int, element: c_int)
        -> c_int;
    fn TessPageIteratorBlockType(handle: *mut c_void) -> c_int;
    fn TessPageIteratorBaseline(
        handle: *mut c_void,
        level: c_int,
        x1: *mut c_int,
        y1: *mut c_int,
        x2: *mut c_int,
        y2: *mut c_int,
    ) -> c_int;
    fn TessPageIteratorOrientation(
        handle: *mut c_void,
        orientation: *mut c_int,
        writing_direction: *mut c_int,
        textline_order: *mut c_int,
        deskew_angle: *mut c_float,
    ) -> c_int;

    fn TessResultIteratorGetUTF8Text(handle: *mut c_void, level: c_int) -> *mut c_char;
    fn TessResultIteratorConfidence(handle: *mut c_void, level: c_int) -> c_float;
    fn TessResultIteratorWordRecognitionLanguage(handle: *mut c_void) -> *const c_char;
    fn TessResultIteratorWordFontAttributes(
        handle: *mut c_void,
        is_bold: *mut c_int,
        is_italic: *mut c_int,
        is_underlined: *mut c_int,
        is_monospace: *mut c_int,
        is_serif: *mut c_int,
        is_smallcaps: *mut c_int,
        pointsize: *mut c_int,
        font_id: *mut c_int,
    ) -> c_int;
    fn TessResultIteratorWordIsFromDictionary(handle: *mut c_void) -> c_int;
    fn TessResultIteratorWordIsNumeric(handle: *mut c_void) -> c_int;
    fn TessResultIteratorSymbolIsSuperscript(handle: *mut c_void) -> c_int;
    fn TessResultIteratorSymbolIsSubscript(handle: *mut c_void) -> c_int;
    fn TessResultIteratorSymbolIsDropcap(handle: *mut c_void) -> c_int;

    fn TessChoiceIteratorDelete(handle: *mut c_void);
    fn TessChoiceIteratorNext(handle: *mut c_void) -> c_int;
    fn TessChoiceIteratorGetUTF8Text(handle: *mut c_void) -> *mut c_char;
    fn TessChoiceIteratorConfidence(handle: *mut c_void) -> c_float;

    fn TessMonitorCreate() -> *mut c_void;
    fn TessMonitorDelete(monitor: *mut c_void);
    fn TessMonitorSetDeadlineMSecs(monitor: *mut c_void, deadline: c_int);
    fn TessMonitorGetProgress(monitor: *mut c_void) -> c_int;

    fn TessTextRendererCreate(outputbase: *const c_char) -> *mut c_void;
    fn TessHOcrRendererCreate(outputbase: *const c_char) -> *mut c_void;
    fn TessPDFRendererCreate(
        outputbase: *const c_char,
        datadir: *const c_char,
        textonly: c_int,
    ) -> *mut c_void;
    fn TessDeleteResultRenderer(renderer: *mut c_void);
    fn TessResultRendererBeginDocument(renderer: *mut c_void, title: *const c_char) -> c_int;
    fn TessResultRendererAddImage(renderer: *mut c_void, api: *mut c_void) -> c_int;
    fn TessResultRendererEndDocument(renderer: *mut c_void) -> c_int;
    fn TessResultRendererExtention(renderer: *mut c_void) -> *const c_char;
    fn TessResultRendererTitle(renderer: *mut c_void) -> *const c_char;
    fn TessResultRendererImageNum(renderer: *mut c_void) -> c_int;

    fn TessGetUnichar(unichar_id: c_int) -> *const c_char;

    fn TessBaseAPIProcessPages(
        handle: *mut c_void,
        filename: *const c_char,
        retry_config: *const c_char,
        timeout_millisec: c_int,
        renderer: *mut c_void,
    ) -> *mut c_char;
    fn TessBaseAPIProcessPage(
        handle: *mut c_void,
        pix: *mut c_void,
        page_index: c_int,
        filename: *const c_char,
        retry_config: *const c_char,
        timeout_millisec: c_int,
        renderer: *mut c_void,
    ) -> c_int;

    fn TessBaseAPIGetInputName(handle: *mut c_void) -> *const c_char;
    fn TessBaseAPISetInputName(handle: *mut c_void, name: *const c_char);
    fn TessBaseAPIGetSourceYResolution(handle: *mut c_void) -> c_int;
    fn TessBaseAPIGetDatapath(handle: *mut c_void) -> *const c_char;
    fn TessBaseAPIGetThresholdedImage(handle: *mut c_void) -> *mut c_void;
    fn TessMutableIteratorSetValue(
        handle: *mut c_void,
        level: c_int,
        value: *const c_char,
    ) -> c_int;
    fn TessMutableIteratorDelete(handle: *mut c_void) -> c_int;
    fn TessResultIteratorNext(handle: *mut c_void, level: c_int) -> c_int;

    // unimplemented functions
    /*
    fn TessHOcrRendererCreate2(outputbase: *const c_char, font_info: c_int) -> *mut c_void;
    fn TessAltoRendererCreate(outputbase: *const c_char) -> *mut c_void;
    fn TessPAGERendererCreate(outputbase: *const c_char) -> *mut c_void;
    fn TessTsvRendererCreate(outputbase: *const c_char) -> *mut c_void;
    fn TessUnlvRendererCreate(outputbase: *const c_char) -> *mut c_void;
    fn TessWordStrBoxRendererCreate(outputbase: *const c_char) -> *mut c_void;
    fn TessLSTMBoxRendererCreate(outputbase: *const c_char) -> *mut c_void;

    fn TessResultRendererInsert(renderer: *mut c_void, next: *mut c_void);
    fn TessResultRendererNext(renderer: *mut c_void) -> *mut c_void;
    fn TessBaseAPIPrintVariables(handle: *mut c_void, fp: *mut c_void);
    fn TessBaseAPIRect(
        handle: *mut c_void,
        imagedata: *const u8,
        bytes_per_pixel: c_int,
        bytes_per_line: c_int,
        left: c_int,
        top: c_int,
        width: c_int,
        height: c_int,
    ) -> *mut c_char;
    fn TessBaseAPIGetGradient(handle: *mut c_void) -> c_float;
    fn TessBaseAPIGetRegions(handle: *mut c_void, pixa: *mut *mut c_void) -> *mut c_void;
    fn TessBaseAPIGetTextlines(
        handle: *mut c_void,
        pixa: *mut *mut c_void,
        blockids: *mut *mut c_int,
    ) -> *mut c_void;
    fn TessBaseAPIGetTextlines1(
        handle: *mut c_void,
        raw_image: c_int,
        raw_padding: c_int,
        pixa: *mut *mut c_void,
        blockids: *mut *mut c_int,
        paraids: *mut *mut c_int,
    ) -> *mut c_void;
    fn TessBaseAPIGetStrips(
        handle: *mut c_void,
        pixa: *mut *mut c_void,
        blockids: *mut *mut c_int,
    ) -> *mut c_void;
    fn TessBaseAPIGetWords(handle: *mut c_void, pixa: *mut *mut c_void) -> *mut c_void;
    fn TessBaseAPIGetConnectedComponents(
        handle: *mut c_void,
        pixa: *mut *mut c_void,
    ) -> *mut c_void;
    fn TessBaseAPIGetComponentImages(
        handle: *mut c_void,
        level: c_int,
        text_only: c_int,
        pixa: *mut *mut c_void,
        blockids: *mut *mut c_int,
    ) -> *mut c_void;
    fn TessBaseAPIGetComponentImages1(
        handle: *mut c_void,
        level: c_int,
        text_only: c_int,
        raw_image: c_int,
        raw_padding: c_int,
        pixa: *mut *mut c_void,
    ) -> *mut c_void;
    fn TessBaseAPIProcessPagesWithOptions(
        handle: *mut c_void,
        filename: *const c_char,
        retry_config: *const c_char,
        timeout_millisec: c_int,
        renderer: *mut c_void,
    ) -> *mut c_char;
    fn TessBaseAPIOem(handle: *mut c_void) -> c_int;
    fn TessBaseAPIGetBlockTextOrientations(
        handle: *mut c_void,
        block_orientation: *mut *mut c_int,
        vertical_writing: *mut bool,
    );
    fn TessPageIteratorCopy(handle: *mut c_void) -> *mut c_void;
    fn TessPageIteratorGetBinaryImage(handle: *mut c_void, level: c_int) -> *mut c_void;
    fn TessPageIteratorGetImage(
        handle: *mut c_void,
        level: c_int,
        padding: c_int,
        original_image: *mut c_void,
        left: *mut c_int,
        top: *mut c_int,
    ) -> *mut c_void;
    fn TessPageIteratorParagraphInfo(
        handle: *mut c_void,
        justification: *mut c_int,
        is_list_item: *mut bool,
        is_crown: *mut bool,
        first_line_indent: *mut c_int,
    ) -> c_int;
    fn TessResultIteratorCopy(handle: *mut c_void) -> *mut c_void;
    fn TessResultIteratorGetPageIterator(handle: *mut c_void) -> *mut c_void;
    fn TessResultIteratorGetPageIteratorConst(handle: *mut c_void) -> *const c_void;
    fn TessResultIteratorGetChoiceIterator(handle: *mut c_void) -> *mut c_void;
    fn TessMonitorSetCancelFunc(monitor: *mut c_void, cancel_func: *mut c_void);
    fn TessMonitorSetCancelThis(monitor: *mut c_void, cancel_this: *mut c_void);
    fn TessMonitorGetCancelThis(monitor: *mut c_void) -> *mut c_void;
    fn TessMonitorSetProgressFunc(monitor: *mut c_void, progress_func: *mut c_void);
    */
}

#[cfg(test)]
#[cfg(feature = "build-tesseract")]
mod tests {
    use super::*;
    use std::env;

    fn setup() -> TesseractAPI {
        let api = TesseractAPI::new();
        let tessdata_dir = env::var("TESSDATA_PREFIX").expect("TESSDATA_PREFIX not set");
        println!("Fount tessdata_dir: {}", tessdata_dir.to_string());
        api.init(tessdata_dir, "eng")
            .expect("Failed to initialize Tesseract");
        api
    }

    #[test]
    fn test_create_and_init() {
        let _api = setup();
    }

    #[test]
    fn test_set_variable() {
        let api = setup();
        assert!(api
            .set_variable("tessedit_char_whitelist", "0123456789")
            .is_ok());
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
        let result = api.set_image(&[], 0, 0, 1, 0);
        assert!(result.is_ok());
        let text = api.get_utf8_text();
        assert!(text.is_ok());
        assert_eq!(text.unwrap(), "");
    }

    #[test]
    fn test_get_word_confidences_empty_image() {
        let api = setup();
        // Set an empty image
        let result = api.set_image(&[], 0, 0, 1, 0);
        assert!(result.is_ok());
        let confidences = api.get_word_confidences();
        assert!(confidences.is_ok());
        assert!(confidences.unwrap().is_empty());
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
        let image_data: Vec<u8> = vec![0xFF, 0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00, 0xFF];

        let result = api.set_image(&image_data, 3, 3, 1, 3);
        assert!(result.is_ok());
        api.set_variable("tessedit_char_whitelist", "0123456789")
            .unwrap();

        let result = api.get_utf8_text().unwrap();
        assert!(result.contains("1"));
    }
}
