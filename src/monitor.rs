use std::os::raw::{c_int, c_void};

pub struct TessMonitor {
    handle: *mut c_void,
}

impl TessMonitor {
    /// Creates a new instance of the TessMonitor.
    ///
    /// # Returns
    ///
    /// Returns the new instance of the TessMonitor.
    pub fn new() -> Self {
        let handle = unsafe { TessMonitorCreate() };
        TessMonitor { handle }
    }

    /// Sets the deadline for the monitor.
    ///
    /// # Arguments
    ///
    /// * `deadline` - Deadline in milliseconds.
    pub fn set_deadline(&mut self, deadline: i32) {
        unsafe { TessMonitorSetDeadlineMSecs(self.handle, deadline) };
    }

    /// Gets the progress of the monitor.
    ///
    /// # Returns
    ///
    /// Returns the progress as an `i32`.
    pub fn get_progress(&self) -> i32 {
        unsafe { TessMonitorGetProgress(self.handle) }
    }
}

impl Drop for TessMonitor {
    fn drop(&mut self) {
        unsafe { TessMonitorDelete(self.handle) };
    }
}

extern "C" {
    pub fn TessMonitorCreate() -> *mut c_void;
    pub fn TessMonitorDelete(monitor: *mut c_void);
    pub fn TessMonitorSetDeadlineMSecs(monitor: *mut c_void, deadline: c_int);
    pub fn TessMonitorGetProgress(monitor: *mut c_void) -> c_int;
}
