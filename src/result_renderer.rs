use crate::error::{Result, TesseractError};
use crate::TesseractAPI;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

pub struct TessResultRenderer {
    handle: *mut c_void,
}

impl TessResultRenderer {
    /// Creates a new instance of the TessResultRenderer.
    ///
    /// # Arguments
    ///
    /// * `outputbase` - Output base path.
    ///
    /// # Returns
    ///
    /// Returns the new instance of the TessResultRenderer.
    pub fn new_text_renderer(outputbase: &str) -> Result<Self> {
        let outputbase = CString::new(outputbase).unwrap();
        let handle = unsafe { TessTextRendererCreate(outputbase.as_ptr()) };
        if handle.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(TessResultRenderer { handle })
        }
    }

    /// Creates a new instance of the TessResultRenderer for HOCR.
    ///
    /// # Arguments
    ///
    /// * `outputbase` - Output base path.
    ///
    /// # Returns
    ///
    /// Returns the new instance of the TessResultRenderer.
    pub fn new_hocr_renderer(outputbase: &str) -> Result<Self> {
        let outputbase = CString::new(outputbase).unwrap();
        let handle = unsafe { TessHOcrRendererCreate(outputbase.as_ptr()) };
        if handle.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(TessResultRenderer { handle })
        }
    }

    /// Creates a new instance of the TessResultRenderer for PDF.
    ///
    /// # Arguments
    ///
    /// * `outputbase` - Output base path.
    /// * `datadir` - Data directory path.
    /// * `textonly` - Whether to include text only.
    ///
    /// # Returns
    ///
    /// Returns the new instance of the TessResultRenderer.
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

    /// Begins a new document.
    ///
    /// # Arguments
    ///
    /// * `title` - Title of the document.
    ///
    /// # Returns
    ///
    /// Returns `true` if the document was created successfully, otherwise returns `false`.
    pub fn begin_document(&self, title: &str) -> bool {
        let title = CString::new(title).unwrap();
        unsafe { TessResultRendererBeginDocument(self.handle, title.as_ptr()) != 0 }
    }

    /// Adds an image to the document.
    ///
    /// # Arguments
    ///
    /// * `api` - The TesseractAPI instance.
    ///
    /// # Returns
    ///
    /// Returns `true` if the image was added successfully, otherwise returns `false`.
    pub fn add_image(&self, api: &TesseractAPI) -> bool {
        let api_handle = api.handle.lock().unwrap();
        unsafe { TessResultRendererAddImage(self.handle, *api_handle) != 0 }
    }

    /// Ends the document.
    ///
    /// # Returns
    ///
    /// Returns `true` if the document was ended successfully, otherwise returns `false`.
    pub fn end_document(&self) -> bool {
        unsafe { TessResultRendererEndDocument(self.handle) != 0 }
    }

    /// Gets the extension of the document.
    ///
    /// # Returns
    ///
    /// Returns the extension as a `String` if successful, otherwise returns an error.
    pub fn get_extension(&self) -> Result<String> {
        let ext_ptr = unsafe { TessResultRendererExtention(self.handle) };
        if ext_ptr.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            let c_str = unsafe { CStr::from_ptr(ext_ptr) };
            Ok(c_str.to_str()?.to_owned())
        }
    }

    /// Gets the title of the document.
    ///
    /// # Returns
    ///
    /// Returns the title as a `String` if successful, otherwise returns an error.
    pub fn get_title(&self) -> Result<String> {
        let title_ptr = unsafe { TessResultRendererTitle(self.handle) };
        if title_ptr.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            let c_str = unsafe { CStr::from_ptr(title_ptr) };
            Ok(c_str.to_str()?.to_owned())
        }
    }

    /// Gets the number of images in the document.
    ///
    /// # Returns
    ///
    /// Returns the number of images as an `i32`.
    pub fn get_image_num(&self) -> i32 {
        unsafe { TessResultRendererImageNum(self.handle) }
    }
}

impl Drop for TessResultRenderer {
    fn drop(&mut self) {
        unsafe { TessDeleteResultRenderer(self.handle) };
    }
}

extern "C" {
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
}
