use std::borrow::Cow;
use std::ffi::CStr;
use std::mem::MaybeUninit;

use crate::abilities::Abilities;
use crate::context::Context;
use crate::media::Media;
use crate::port::Port;
use crate::storage::Storage;

use crate::handle::prelude::*;

/// A structure representing a camera connected to the system.
pub struct Camera {
    camera: *mut crate::gphoto2::Camera,
}

impl Drop for Camera {
    fn drop(&mut self) {
        unsafe {
            crate::gphoto2::gp_camera_unref(self.camera);
        }
    }
}

impl Camera {
    /// Opens the first detected camera.
    pub fn autodetect(context: &mut Context) -> crate::Result<Self> {
        let mut ptr = MaybeUninit::uninit();

        let camera = unsafe {
            match crate::gphoto2::gp_camera_new(&mut *ptr.as_mut_ptr()) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            ptr.assume_init()
        };

        let camera = Camera { camera };

        try_unsafe!(crate::gphoto2::gp_camera_init(
            camera.camera,
            context.as_mut_ptr()
        ));

        Ok(camera)
    }

    /// Captures an image.
    pub fn capture_image(&mut self, context: &mut Context) -> crate::Result<CameraFile> {
        let mut file_path = MaybeUninit::uninit();

        let file_path = unsafe {
            match crate::gphoto2::gp_camera_capture(
                self.camera,
                crate::gphoto2::GP_CAPTURE_IMAGE,
                &mut *file_path.as_mut_ptr(),
                context.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            file_path.assume_init()
        };
        Ok(CameraFile { inner: file_path })
    }

    /// Downloads a file from the camera.
    pub fn download<T: Media>(
        &mut self,
        context: &mut Context,
        source: &CameraFile,
        destination: &mut T,
    ) -> crate::Result<()> {
        try_unsafe! {
            crate::gphoto2::gp_camera_file_get(self.camera,
                                          source.inner.folder.as_ptr(),
                                          source.inner.name.as_ptr(),
                                          crate::gphoto2::GP_FILE_TYPE_NORMAL,
                                          destination.as_mut_ptr(),
                                          context.as_mut_ptr())
        };

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

        crate::abilities::from_libgphoto2(abilities)
    }

    /// Retrieves information about the camera's storage.
    ///
    /// Returns a `Vec` containing one `Storage` for each filesystem on the device.
    pub fn storage(&mut self, context: &mut Context) -> crate::Result<Vec<Storage>> {
        let mut ptr = MaybeUninit::uninit();
        let mut len = MaybeUninit::uninit();

        let (storage, len) = unsafe {
            match crate::gphoto2::gp_camera_get_storageinfo(
                self.camera,
                &mut *ptr.as_mut_ptr(),
                &mut *len.as_mut_ptr(),
                context.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            (ptr.assume_init(), len.assume_init())
        };
        let storage = storage as *mut Storage;
        let length = len as usize;

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
    pub fn summary(&mut self, context: &mut Context) -> crate::Result<String> {
        let mut summary = MaybeUninit::uninit();

        let summary = unsafe {
            match crate::gphoto2::gp_camera_get_summary(
                self.camera,
                &mut *summary.as_mut_ptr(),
                context.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            summary.assume_init()
        };

        println!("Debug before free?");
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
    pub fn manual(&mut self, context: &mut Context) -> crate::Result<String> {
        let mut manual = MaybeUninit::uninit();

        let manual = unsafe {
            match crate::gphoto2::gp_camera_get_manual(
                self.camera,
                &mut *manual.as_mut_ptr(),
                context.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            manual.assume_init()
        };

        println!("Debug before free manual?");
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
    pub fn about_driver(&mut self, context: &mut Context) -> crate::Result<String> {
        let mut about = MaybeUninit::uninit();

        let about = unsafe {
            match crate::gphoto2::gp_camera_get_about(
                self.camera,
                &mut *about.as_mut_ptr(),
                context.as_mut_ptr(),
            ) {
                crate::gphoto2::GP_OK => (),
                err => return Err(crate::error::from_libgphoto2(err)),
            }
            about.assume_init()
        };

        println!("Debug before free about?");
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
    use std::ffi::CStr;

    pub fn camera_text_to_string(
        mut camera_text: crate::gphoto2::CameraText,
    ) -> crate::Result<String> {
        let length = unsafe { CStr::from_ptr(camera_text.text.as_ptr()).to_bytes().len() };

        let vec = unsafe {
            Vec::<u8>::from_raw_parts(
                camera_text.text.as_mut_ptr() as *mut u8,
                length,
                camera_text.text.len(),
            )
        };

        String::from_utf8(vec)
            .map_err(|_| crate::error::from_libgphoto2(crate::gphoto2::GP_ERROR_CORRUPTED_DATA))
    }
}
