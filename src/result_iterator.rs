use crate::api::TessDeleteText;
use crate::enums::TessPageIteratorLevel;
use crate::error::{Result, TesseractError};
use std::ffi::CStr;
use std::os::raw::{c_char, c_float, c_int, c_void};

pub struct ResultIterator {
    handle: *mut c_void,
}

impl ResultIterator {
    /// Creates a new instance of the ResultIterator.
    ///
    /// # Arguments
    ///
    /// * `handle` - Pointer to the ResultIterator.
    ///
    /// # Returns
    ///
    /// Returns the new instance of the ResultIterator.
    pub fn new(handle: *mut c_void) -> Self {
        ResultIterator { handle }
    }

    /// Gets the UTF-8 text of the current iterator.
    ///
    /// # Arguments
    ///
    /// * `level` - Level of the text.
    ///
    /// # Returns
    ///
    /// Returns the UTF-8 text as a `String` if successful, otherwise returns an error.
    pub fn get_utf8_text(&self, level: TessPageIteratorLevel) -> Result<String> {
        let text_ptr = unsafe { TessResultIteratorGetUTF8Text(self.handle, level as c_int) };
        if text_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        unsafe { TessDeleteText(text_ptr as *mut c_char) };
        Ok(result)
    }

    /// Gets the confidence of the current iterator.
    ///
    /// # Arguments
    ///
    /// * `level` - Level of the confidence.
    ///
    /// # Returns
    ///
    /// Returns the confidence as a `f32`.
    pub fn confidence(&self, level: TessPageIteratorLevel) -> f32 {
        unsafe { TessResultIteratorConfidence(self.handle, level as c_int) }
    }

    /// Gets the recognition language of the current iterator.
    ///
    /// # Returns
    ///
    /// Returns the recognition language as a `String` if successful, otherwise returns an error.
    pub fn word_recognition_language(&self) -> Result<String> {
        let lang_ptr = unsafe { TessResultIteratorWordRecognitionLanguage(self.handle) };
        if lang_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        let c_str = unsafe { CStr::from_ptr(lang_ptr) };
        Ok(c_str.to_str()?.to_owned())
    }

    /// Gets the font attributes of the current iterator.
    ///
    /// # Returns
    ///
    /// Returns the font attributes as a tuple if successful, otherwise returns an error.
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

    /// Checks if the current iterator is from the dictionary.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current iterator is from the dictionary, otherwise returns `false`.
    pub fn word_is_from_dictionary(&self) -> bool {
        unsafe { TessResultIteratorWordIsFromDictionary(self.handle) != 0 }
    }

    /// Checks if the current iterator is numeric.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current iterator is numeric, otherwise returns `false`.
    pub fn word_is_numeric(&self) -> bool {
        unsafe { TessResultIteratorWordIsNumeric(self.handle) != 0 }
    }

    /// Checks if the current iterator is superscript.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current iterator is superscript, otherwise returns `false`.
    pub fn symbol_is_superscript(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsSuperscript(self.handle) != 0 }
    }

    /// Checks if the current iterator is subscript.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current iterator is subscript, otherwise returns `false`.
    pub fn symbol_is_subscript(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsSubscript(self.handle) != 0 }
    }

    /// Checks if the current iterator is dropcap.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current iterator is dropcap, otherwise returns `false`.
    pub fn symbol_is_dropcap(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsDropcap(self.handle) != 0 }
    }

    /// Moves to the next iterator.
    ///
    /// # Arguments
    ///
    /// * `level` - Level of the next iterator.
    ///
    /// # Returns
    ///
    /// Returns `true` if the next iterator exists, otherwise returns `false`.
    pub fn next(&self, level: TessPageIteratorLevel) -> bool {
        unsafe { TessResultIteratorNext(self.handle, level as c_int) != 0 }
    }
}

impl Drop for ResultIterator {
    fn drop(&mut self) {
        unsafe { TessResultIteratorDelete(self.handle) };
    }
}

#[cfg(feature = "build-tesseract")]
#[link(name = "tesseract")]
extern "C" {
    pub fn TessResultIteratorDelete(handle: *mut c_void);
    pub fn TessResultIteratorGetUTF8Text(handle: *mut c_void, level: c_int) -> *mut c_char;
    pub fn TessResultIteratorConfidence(handle: *mut c_void, level: c_int) -> c_float;
    pub fn TessResultIteratorWordRecognitionLanguage(handle: *mut c_void) -> *const c_char;
    pub fn TessResultIteratorWordFontAttributes(
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
    pub fn TessResultIteratorWordIsFromDictionary(handle: *mut c_void) -> c_int;
    pub fn TessResultIteratorWordIsNumeric(handle: *mut c_void) -> c_int;
    pub fn TessResultIteratorSymbolIsSuperscript(handle: *mut c_void) -> c_int;
    pub fn TessResultIteratorSymbolIsSubscript(handle: *mut c_void) -> c_int;
    pub fn TessResultIteratorSymbolIsDropcap(handle: *mut c_void) -> c_int;
    pub fn TessResultIteratorNext(handle: *mut c_void, level: c_int) -> c_int;
}
