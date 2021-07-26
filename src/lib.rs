extern crate gphoto2_sys as gphoto2;
extern crate libc;

pub use crate::abilities::{
    Abilities, CameraOperation, DeviceType, DriverStatus, FileOperation, FolderOperation,
};
pub use crate::camera::{Camera, CameraFile};
pub(crate) use crate::context::Context;
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::media::{FileMedia, Media};
pub use crate::port::{Port, PortType};
pub use crate::storage::{AccessType, FilesystemType, Storage, StorageType};
pub use crate::version::{libgphoto2_version, LibraryVersion};

#[macro_use]
mod error;
mod abilities;
mod camera;
mod context;
mod media;
mod port;
mod storage;
mod version;

// internal
mod handle;
