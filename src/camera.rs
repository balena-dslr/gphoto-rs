#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
#[cfg(feature = "std")]
use std::borrow::Cow;

use core::mem::MaybeUninit;
use cstr_core::{CStr, CString};

use gphoto2_sys::CameraWidgetType;
use libc::c_char;

use crate::abilities::Abilities;
use crate::context::Context;
use crate::media::Media;
use crate::port::Port;
use crate::storage::Storage;

use crate::handle::prelude::*;

/// A structure representing a camera connected to the system.
pub struct Camera {
    camera: *mut crate::gphoto2::Camera,
    context: Context,
}

impl Drop for Camera {
    fn drop(&mut self) {
        unsafe {
            crate::gphoto2::gp_camera_unref(self.camera);
            crate::gphoto2::gp_context_unref(self.context.context);
        }
    }
}

impl Camera {
    /// Opens the first detected camera.
    pub fn autodetect() -> crate::Result<Self> {
        let context = crate::Context::new()?;

        let mut ptr = MaybeUninit::uninit();

        let camera = unsafe {
            match crate::gphoto2::gp_camera_new(&mut *ptr.as_mut_ptr()) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            ptr.assume_init()
        };

        let mut camera = Camera { camera, context };

        try_unsafe!(crate::gphoto2::gp_camera_init(
            camera.camera,
            camera.context.as_mut_ptr()
        ));

        Ok(camera)
    }

    /// Captures an image.
    pub fn capture_image(&mut self) -> crate::Result<CameraFile> {
        let mut file_path = MaybeUninit::uninit();

        let file_path = unsafe {
            match crate::gphoto2::gp_camera_capture(
                self.camera,
                crate::gphoto2::GP_CAPTURE_IMAGE,
                &mut *file_path.as_mut_ptr(),
                self.context.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            file_path.assume_init()
        };
        unsafe {
            crate::gphoto2::gp_camera_exit(self.camera, self.context.context);
        }
        Ok(CameraFile { inner: file_path })
    }

    /// Set a setting to a specific value
    pub fn set_setting(&mut self) -> crate::Result<()> {
        let mut widget_ptr = MaybeUninit::uninit();
        let label = CString::new("").unwrap();
        let label: *const c_char = label.as_ptr() as *const c_char;
        let window = unsafe {
            match crate::gphoto2::gp_widget_new(
                CameraWidgetType::GP_WIDGET_WINDOW,
                label,
                &mut *widget_ptr.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            widget_ptr.assume_init()
        };
        // TODO actually set values
        // TODO wrap widget to something useful
        unsafe {
            match crate::gphoto2::gp_camera_set_config(
                self.camera,
                window,
                self.context.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => Ok(()),
                err => Err(crate::error::from_libgphoto2(err)),
            }
        }
    }

    /// Downloads a file from the camera.
    pub fn download<T: Media>(
        &mut self,
        source: &CameraFile,
        destination: &mut T,
        file_type: Option<crate::CameraFileType>,
    ) -> crate::Result<()> {
        let file_type = if let Some(file_type) = file_type {
            file_type
        } else {
            crate::gphoto2::GP_FILE_TYPE_NORMAL
        };
        try_unsafe! {
            crate::gphoto2::gp_camera_file_get(self.camera,
                                          source.inner.folder.as_ptr(),
                                          source.inner.name.as_ptr(),
                                          file_type,
                                          destination.as_mut_ptr(),
                                          self.context.as_mut_ptr())
        };
        unsafe {
            crate::gphoto2::gp_camera_exit(self.camera, self.context.context);
        }

        Ok(())
    }

    /// Returns information about the port the camera is connected to.
    pub fn port(&self) -> Port {
        let mut ptr = MaybeUninit::uninit();

        let port_info = unsafe {
            assert_eq!(
                crate::gphoto2::GP_OK,
                crate::gphoto2::gp_camera_get_port_info(self.camera, &mut *ptr.as_mut_ptr())
            );

            ptr.assume_init()
        };
        unsafe {
            crate::gphoto2::gp_camera_exit(self.camera, self.context.context);
        }
        crate::port::from_libgphoto2(self, port_info)
    }

    /// Retrieves the camera's abilities.
    pub fn abilities(&self) -> Abilities {
        let mut abilities = MaybeUninit::uninit();

        let abilities = unsafe {
            assert_eq!(
                crate::gphoto2::GP_OK,
                crate::gphoto2::gp_camera_get_abilities(self.camera, &mut *abilities.as_mut_ptr())
            );
            abilities.assume_init()
        };

        unsafe {
            crate::gphoto2::gp_camera_exit(self.camera, self.context.context);
        }
        crate::abilities::from_libgphoto2(abilities)
    }

    /// Retrieves information about the camera's storage.
    ///
    /// Returns a `Vec` containing one `Storage` for each filesystem on the device.
    pub fn storage(&mut self) -> crate::Result<Vec<Storage>> {
        let mut ptr = MaybeUninit::uninit();
        let mut len = MaybeUninit::uninit();

        let (storage, len) = unsafe {
            match crate::gphoto2::gp_camera_get_storageinfo(
                self.camera,
                &mut *ptr.as_mut_ptr(),
                &mut *len.as_mut_ptr(),
                self.context.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            (ptr.assume_init(), len.assume_init())
        };
        let storage = storage as *mut Storage;
        let length = len as usize;

        unsafe {
            crate::gphoto2::gp_camera_exit(self.camera, self.context.context);
        }
        Ok(unsafe { Vec::from_raw_parts(storage, length, length) })
    }

    /// Returns the camera's summary.
    ///
    /// The summary typically contains non-configurable information about the camera, such as
    /// manufacturer and number of pictures taken.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the summary could not be retrieved:
    ///
    /// * `NotSupported` if there is no summary available for the camera.
    /// * `CorruptedData` if the summary is invalid UTF-8.
    pub fn summary(&mut self) -> crate::Result<String> {
        let mut summary = MaybeUninit::uninit();

        let summary = unsafe {
            match crate::gphoto2::gp_camera_get_summary(
                self.camera,
                &mut *summary.as_mut_ptr(),
                self.context.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            summary.assume_init()
        };

        unsafe {
            crate::gphoto2::gp_camera_exit(self.camera, self.context.context);
        }
        util::camera_text_to_string(summary)
    }

    /// Returns the camera's manual.
    ///
    /// The manual contains information about using the camera.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the manual could not be retrieved:
    ///
    /// * `NotSupported` if there is no manual available for the camera.
    /// * `CorruptedData` if the summary is invalid UTF-8.
    pub fn manual(&mut self) -> crate::Result<String> {
        let mut manual = MaybeUninit::uninit();

        let manual = unsafe {
            match crate::gphoto2::gp_camera_get_manual(
                self.camera,
                &mut *manual.as_mut_ptr(),
                self.context.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            manual.assume_init()
        };

        unsafe {
            crate::gphoto2::gp_camera_exit(self.camera, self.context.context);
        }
        util::camera_text_to_string(manual)
    }

    /// Returns information about the camera driver.
    ///
    /// This text typically contains information about the driver's author, acknowledgements, etc.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the about text could not be retrieved:
    ///
    /// * `NotSupported` if there is no about text available for the camera's driver.
    /// * `CorruptedData` if the summary is invalid UTF-8.
    pub fn about_driver(&mut self) -> crate::Result<String> {
        let mut about = MaybeUninit::uninit();

        let about = unsafe {
            match crate::gphoto2::gp_camera_get_about(
                self.camera,
                &mut *about.as_mut_ptr(),
                self.context.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            about.assume_init()
        };

        unsafe {
            crate::gphoto2::gp_camera_exit(self.camera, self.context.context);
        }
        util::camera_text_to_string(about)
    }
}

/// A file stored on a camera's storage.
pub struct CameraFile {
    inner: crate::gphoto2::CameraFilePath,
}

impl CameraFile {
    /// Returns the directory that the file is stored in.
    pub fn directory(&self) -> Cow<str> {
        unsafe { String::from_utf8_lossy(CStr::from_ptr(self.inner.folder.as_ptr()).to_bytes()) }
    }

    /// Returns the name of the file without the directory.
    pub fn basename(&self) -> Cow<str> {
        unsafe { String::from_utf8_lossy(CStr::from_ptr(self.inner.name.as_ptr()).to_bytes()) }
    }
}

mod util {
    use cstr_core::CStr;

    pub fn camera_text_to_string(camera_text: crate::gphoto2::CameraText) -> crate::Result<String> {
        let c_str = unsafe { CStr::from_ptr(camera_text.text.as_ptr()) };

        let rust_str: &str = c_str
            .to_str()
            .map_err(|_| crate::error::Error { err: -1 })?;

        Ok(rust_str.to_owned())
    }
}
