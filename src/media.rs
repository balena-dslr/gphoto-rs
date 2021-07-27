#[cfg(not(feature = "std"))]
use alloc::slice;
use core::mem::MaybeUninit;
use cstr_core::CString;
use std::path::Path;
#[cfg(feature = "std")]
use std::slice;

use libc::c_ulong;

/// A trait for types that can store media.
pub trait Media {
    #[doc(hidden)]
    unsafe fn as_mut_ptr(&mut self) -> *mut crate::gphoto2::CameraFile;
}

/// Media stored as a local file.
pub struct FileMedia {
    file: *mut crate::gphoto2::CameraFile,
}

impl Drop for FileMedia {
    fn drop(&mut self) {
        unsafe {
            crate::gphoto2::gp_file_unref(self.file);
        }
    }
}

impl FileMedia {
    /// Creates a new file that stores media.
    ///
    /// This function creates a new file on disk. The file will start out empty.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the file can not be created:
    ///
    /// * `FileExists` if the file already exists.
    #[cfg(feature = "std")]
    pub fn create(path: &Path) -> crate::Result<Self> {
        let path_str = path.to_str().unwrap();
        FileMedia::create_internal(path_str)
    }

    /// Creates a new file that stores media.
    ///
    /// This function creates a new file on disk. The file will start out empty.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the file can not be created:
    ///
    /// * `FileExists` if the file already exists.
    #[cfg(not(feature = "std"))]
    fn create(path_str: &str) -> crate::Result<Self> {
        FileMedia::create_internal(path_str)
    }

    fn create_internal(path: &str) -> crate::Result<Self> {
        use libc::{O_CREAT, O_EXCL, O_RDWR};

        let cstr = match CString::new(path) {
            Ok(s) => s,
            Err(_) => {
                return Err(crate::error::from_libgphoto2(
                    crate::gphoto2::GP_ERROR_BAD_PARAMETERS,
                ))
            }
        };

        let fd = unsafe { libc::open(cstr.as_ptr(), O_CREAT | O_EXCL | O_RDWR, 0o644) };
        if fd < 0 {
            return Err(crate::error::from_libgphoto2(
                crate::gphoto2::GP_ERROR_FILE_EXISTS,
            ));
        }

        let mut ptr = MaybeUninit::uninit();

        match unsafe { crate::gphoto2::gp_file_new_from_fd(&mut *ptr.as_mut_ptr(), fd) } {
            crate::gphoto2::GP_OK => {
                let ptr = unsafe { ptr.assume_init() };
                Ok(FileMedia { file: ptr })
            }
            err => {
                unsafe {
                    libc::close(fd);
                }

                Err(crate::error::from_libgphoto2(err))
            }
        }
    }

    pub fn create_mem() -> crate::Result<Self> {
        let mut ptr = MaybeUninit::uninit();

        match unsafe { crate::gphoto2::gp_file_new(&mut *ptr.as_mut_ptr()) } {
            crate::gphoto2::GP_OK => {
                let ptr = unsafe { ptr.assume_init() };
                Ok(FileMedia { file: ptr })
            }
            err => Err(crate::error::from_libgphoto2(err)),
        }
    }

    pub fn get_data(&mut self) -> Vec<u8> {
        let mut ptr = MaybeUninit::uninit();
        let mut len: c_ulong = 0;

        let ptr = unsafe {
            crate::gphoto2::gp_file_get_data_and_size(self.file, &mut *ptr.as_mut_ptr(), &mut len);
            ptr.assume_init()
        };

        unsafe { slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() }
    }
}

impl Media for FileMedia {
    #[doc(hidden)]
    unsafe fn as_mut_ptr(&mut self) -> *mut crate::gphoto2::CameraFile {
        self.file
    }
}
