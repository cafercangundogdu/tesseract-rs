use crate::api::TessDeleteText;
use crate::error::{Result, TesseractError};
use std::ffi::CStr;
use std::os::raw::{c_char, c_float, c_int, c_void};

pub struct ChoiceIterator {
    handle: *mut c_void,
}

impl ChoiceIterator {
    /// Creates a new instance of the ChoiceIterator.
    ///
    /// # Arguments
    ///
    /// * `handle` - Pointer to the ChoiceIterator.
    pub fn new(handle: *mut c_void) -> Self {
        ChoiceIterator { handle }
    }

    /// Gets the next choice.
    ///
    /// # Returns
    ///
    /// Returns `true` if the next choice is successful, otherwise returns `false`.
    pub fn next(&self) -> bool {
        unsafe { TessChoiceIteratorNext(self.handle) != 0 }
    }

    /// Gets the UTF-8 text for the current choice.
    ///
    /// # Returns
    ///
    /// Returns the UTF-8 text as a `String` if successful, otherwise returns an error.
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

    /// Gets the confidence of the current choice.
    ///
    /// # Returns
    ///
    /// Returns the confidence as a `f32`.
    pub fn confidence(&self) -> f32 {
        unsafe { TessChoiceIteratorConfidence(self.handle) }
    }
}

impl Drop for ChoiceIterator {
    fn drop(&mut self) {
        unsafe { TessChoiceIteratorDelete(self.handle) };
    }
}

extern "C" {
    fn TessChoiceIteratorDelete(handle: *mut c_void);
    fn TessChoiceIteratorNext(handle: *mut c_void) -> c_int;
    fn TessChoiceIteratorGetUTF8Text(handle: *mut c_void) -> *mut c_char;
    fn TessChoiceIteratorConfidence(handle: *mut c_void) -> c_float;
}
