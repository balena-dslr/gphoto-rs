use std::error::Error as StdError;
use std::ffi::CStr;
use std::fmt;
use std::result::Result as StdResult;
use std::str;

use ::libc::c_int;

/// A specialized `Result` type for working with gphoto2.
pub type Result<T> = StdResult<T, Error>;

/// Types of errors reported by gphoto2.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ErrorKind {
    /// A parameter was incorrect.
    InvalidInput,

    /// Functionality not supported.
    NotSupported,

    /// Corrupted data received.
    CorruptedData,

    /// Specified camera model was not found.
    ModelNotFound,

    /// File already exists.
    FileExists,

    /// Directory already exists.
    DirectoryExists,

    /// Directory was not found.
    DirectoryNotFound,

    /// File was not found.
    FileNotFound,

    /// Camera is busy.
    CameraBusy,

    /// Path is not absolute.
    PathNotAbsolute,

    /// Operation was canceled.
    Cancel,

    /// An error was reported by the camera.
    CameraError,

    /// An error was reported by the operating system.
    OSFailure,

    /// Not enough space when uploading a file.
    NoSpace,

    /// An unspecified error occured.
    Other,
}

/// An error type for working with gphoto2.
#[derive(Debug)]
pub struct Error {
    pub(crate) err: c_int,
}

impl Error {
    /// Returns the kind of error.
    pub fn kind(&self) -> ErrorKind {
        match self.err {
            crate::gphoto2::GP_ERROR_BAD_PARAMETERS => ErrorKind::InvalidInput,
            crate::gphoto2::GP_ERROR_NOT_SUPPORTED => ErrorKind::NotSupported,
            crate::gphoto2::GP_ERROR_CORRUPTED_DATA => ErrorKind::CorruptedData,
            crate::gphoto2::GP_ERROR_FILE_EXISTS => ErrorKind::FileExists,
            crate::gphoto2::GP_ERROR_MODEL_NOT_FOUND => ErrorKind::ModelNotFound,
            crate::gphoto2::GP_ERROR_DIRECTORY_NOT_FOUND => ErrorKind::DirectoryNotFound,
            crate::gphoto2::GP_ERROR_FILE_NOT_FOUND => ErrorKind::FileNotFound,
            crate::gphoto2::GP_ERROR_DIRECTORY_EXISTS => ErrorKind::DirectoryExists,
            crate::gphoto2::GP_ERROR_CAMERA_BUSY => ErrorKind::CameraBusy,
            crate::gphoto2::GP_ERROR_PATH_NOT_ABSOLUTE => ErrorKind::PathNotAbsolute,
            crate::gphoto2::GP_ERROR_CANCEL => ErrorKind::Cancel,
            crate::gphoto2::GP_ERROR_CAMERA_ERROR => ErrorKind::CameraError,
            crate::gphoto2::GP_ERROR_OS_FAILURE => ErrorKind::OSFailure,
            crate::gphoto2::GP_ERROR_NO_SPACE => ErrorKind::NoSpace,
            crate::gphoto2::GP_ERROR => ErrorKind::Other,
            _ => ErrorKind::Other,
        }
    }

    /// Returns an error message.
    pub fn message(&self) -> &'static str {
        unsafe {
            str::from_utf8_unchecked(
                CStr::from_ptr(crate::gphoto2::gp_result_as_string(self.err)).to_bytes(),
            )
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        fmt.write_str(self.message())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        self.message()
    }
}

#[doc(hidden)]
pub fn from_libgphoto2(err: c_int) -> Error {
    Error { err }
}

#[doc(hidden)]
macro_rules! try_unsafe {
    ($x:expr) => {
        match unsafe { $x } {
            crate::gphoto2::GP_OK => (),
            err => return Err(crate::error::from_libgphoto2(err)),
        }
    };
}
