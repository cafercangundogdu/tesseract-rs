use crate::error::{Result, TesseractError};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

use crate::result_iterator::{
    TessResultIteratorConfidence, TessResultIteratorGetUTF8Text, TessResultIteratorNext,
    TessResultIteratorSymbolIsDropcap, TessResultIteratorSymbolIsSubscript,
    TessResultIteratorSymbolIsSuperscript, TessResultIteratorWordFontAttributes,
    TessResultIteratorWordIsFromDictionary, TessResultIteratorWordIsNumeric,
    TessResultIteratorWordRecognitionLanguage,
};

pub struct MutableIterator {
    handle: *mut c_void,
}

impl MutableIterator {
    /// Creates a new instance of the MutableIterator.
    ///
    /// # Arguments
    ///
    /// * `handle` - Pointer to the MutableIterator.
    pub fn new(handle: *mut c_void) -> Self {
        MutableIterator { handle }
    }

    /// Gets the UTF-8 text for the current iterator.
    ///
    /// # Arguments
    ///
    /// * `level` - Level of the text.
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

    /// Gets the confidence of the current iterator.
    ///
    /// # Arguments
    ///
    /// * `level` - Level of the confidence.
    pub fn confidence(&self, level: i32) -> f32 {
        unsafe { TessResultIteratorConfidence(self.handle, level) }
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

    /// Checks if the current word is from the dictionary.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current word is from the dictionary, otherwise returns `false`.
    pub fn word_is_from_dictionary(&self) -> bool {
        unsafe { TessResultIteratorWordIsFromDictionary(self.handle) != 0 }
    }

    /// Checks if the current word is numeric.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current word is numeric, otherwise returns `false`.
    pub fn word_is_numeric(&self) -> bool {
        unsafe { TessResultIteratorWordIsNumeric(self.handle) != 0 }
    }

    /// Checks if the current symbol is superscript.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current symbol is superscript, otherwise returns `false`.
    pub fn symbol_is_superscript(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsSuperscript(self.handle) != 0 }
    }

    /// Checks if the current symbol is subscript.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current symbol is subscript, otherwise returns `false`.
    pub fn symbol_is_subscript(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsSubscript(self.handle) != 0 }
    }

    /// Checks if the current symbol is dropcap.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current symbol is dropcap, otherwise returns `false`.
    pub fn symbol_is_dropcap(&self) -> bool {
        unsafe { TessResultIteratorSymbolIsDropcap(self.handle) != 0 }
    }

    /// Gets the next iterator.
    ///
    /// # Arguments
    ///
    /// * `level` - Level of the iterator.
    ///
    /// # Returns
    ///
    /// Returns `true` if the next iterator is successful, otherwise returns `false`.
    pub fn next(&self, level: i32) -> bool {
        unsafe { TessResultIteratorNext(self.handle, level) != 0 }
    }

    /// Sets the value for the current iterator.
    ///
    /// # Arguments
    ///
    /// * `level` - Level of the value.
    /// * `value` - Value to set.
    ///
    /// # Returns
    ///
    /// Returns `true` if setting the value is successful, otherwise returns `false`.
    pub fn set_value(&self, level: i32, value: &str) -> Result<bool> {
        let c_value = CString::new(value).unwrap();
        let result = unsafe { TessMutableIteratorSetValue(self.handle, level, c_value.as_ptr()) };
        Ok(result != 0)
    }

    /// Deletes the MutableIterator.
    ///
    /// # Returns
    ///
    /// Returns `true` if deleting the MutableIterator is successful, otherwise returns `false`.
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

extern "C" {
    fn TessMutableIteratorSetValue(
        handle: *mut c_void,
        level: c_int,
        value: *const c_char,
    ) -> c_int;
    fn TessMutableIteratorDelete(handle: *mut c_void) -> c_int;
    fn TessResultIteratorDelete(handle: *mut c_void);
    fn TessDeleteText(text: *mut c_char);
}
