//! Embedded tessdata functionality for tesseract-rs
//!
//! This module provides functionality to embed Tesseract training data directly
//! into the binary, eliminating the need to ship separate tessdata files.

use crate::{Result, TesseractAPI};

// Include the generated embedded tessdata
include!(concat!(env!("OUT_DIR"), "/embedded_tessdata.rs"));

impl TesseractAPI {
    /// Initialize Tesseract with embedded training data for the specified language.
    ///
    /// This method uses tessdata that has been embedded into the binary at compile time,
    /// eliminating the need for external tessdata files.
    ///
    /// # Arguments
    ///
    /// * `language` - The language code (e.g., "eng", "tur")
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initialization is successful, otherwise returns an error.
    /// Returns an error if the requested language is not embedded.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tesseract_rs::TesseractAPI;
    ///
    /// let api = TesseractAPI::new()?;
    /// api.init_embedded("eng")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn init_embedded(&self, language: &str) -> Result<()> {
        let tessdata = EMBEDDED_TESSDATA
            .get(language)
            .ok_or_else(|| crate::TesseractError::InitError)?;

        self.init_5(tessdata, tessdata.len() as i32, language, 3, &[])
    }

    /// Get a list of available embedded languages.
    ///
    /// # Returns
    ///
    /// A vector of language codes that are embedded in the binary.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tesseract_rs::TesseractAPI;
    ///
    /// let api = TesseractAPI::new()?;
    /// let languages = api.embedded_languages();
    /// println!("Available embedded languages: {:?}", languages);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn embedded_languages(&self) -> Vec<&'static str> {
        EMBEDDED_TESSDATA.available_languages()
    }
}

/// Get the embedded tessdata for a specific language.
///
/// This is a low-level function that returns the raw tessdata bytes.
/// Most users should use `TesseractAPI::init_embedded()` instead.
///
/// # Arguments
///
/// * `language` - The language code (e.g., "eng", "tur")
///
/// # Returns
///
/// Returns `Some(&[u8])` if the language is embedded, `None` otherwise.
///
/// # Example
///
/// ```rust,no_run
/// use tesseract_rs::get_embedded_tessdata;
///
/// if let Some(eng_data) = get_embedded_tessdata("eng") {
///     println!("English tessdata size: {} bytes", eng_data.len());
/// }
/// ```
pub fn get_embedded_tessdata(language: &str) -> Option<&'static [u8]> {
    EMBEDDED_TESSDATA.get(language)
}

/// Get a list of all available embedded languages.
///
/// # Returns
///
/// A vector of language codes that are embedded in the binary.
///
/// # Example
///
/// ```rust,no_run
/// use tesseract_rs::embedded_languages;
///
/// let languages = embedded_languages();
/// println!("Available embedded languages: {:?}", languages);
/// ```
pub fn embedded_languages() -> Vec<&'static str> {
    EMBEDDED_TESSDATA.available_languages()
}
