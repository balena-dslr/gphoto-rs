use std::ffi::CString;
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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
    pub fn create(path: &Path) -> crate::Result<Self> {
        use ::libc::{O_CREAT, O_EXCL, O_RDWR};

        let cstr = match CString::new(path.as_os_str().as_bytes()) {
            Ok(s) => s,
            Err(_) => {
                return Err(crate::error::from_libgphoto2(
                    crate::gphoto2::GP_ERROR_BAD_PARAMETERS,
                ))
            }
        };

        let fd = unsafe { ::libc::open(cstr.as_ptr(), O_CREAT | O_EXCL | O_RDWR, 0o644) };
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
                    ::libc::close(fd);
                }

                Err(crate::error::from_libgphoto2(err))
            }
        }
    }
}

impl Media for FileMedia {
    #[doc(hidden)]
    unsafe fn as_mut_ptr(&mut self) -> *mut crate::gphoto2::CameraFile {
        self.file
    }
}
