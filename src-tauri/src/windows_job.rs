//! Keep yt-dlp descendants owned until output readers have finished.
use std::{ffi::c_void, os::windows::io::AsRawHandle, process::Child};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> *mut c_void;
    fn AssignProcessToJobObject(job: *mut c_void, process: *mut c_void) -> i32;
    fn TerminateJobObject(job: *mut c_void, exit_code: u32) -> i32;
    fn CloseHandle(handle: *mut c_void) -> i32;
}

pub struct WindowsJob(*mut c_void);

impl WindowsJob {
    pub fn attach(child: &Child) -> Result<Self, String> {
        // An unnamed, non-inheritable job belongs exclusively to this guard.
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle.is_null() {
            return Err(format!(
                "Could not create process job: {}",
                std::io::Error::last_os_error()
            ));
        }
        let job = Self(handle);
        if unsafe { AssignProcessToJobObject(handle, child.as_raw_handle()) } == 0 {
            return Err(format!(
                "Could not assign process job: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(job)
    }
}

impl Drop for WindowsJob {
    fn drop(&mut self) {
        // Also stop descendants retaining pipes after the original process exits.
        unsafe {
            TerminateJobObject(self.0, 1);
            CloseHandle(self.0);
        }
    }
}
