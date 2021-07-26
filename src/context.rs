use crate::handle::{Handle, HandleMut};

/// A `libgphoto2` library context.
pub struct Context {
    context: *mut crate::gphoto2::GPContext,
}

impl Context {
    /// Creates a new context.
    pub fn new() -> crate::Result<Context> {
        let ptr = unsafe { crate::gphoto2::gp_context_new() };

        if !ptr.is_null() {
            Ok(Context { context: ptr })
        } else {
            Err(crate::error::from_libgphoto2(
                crate::gphoto2::GP_ERROR_NO_MEMORY,
            ))
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            crate::gphoto2::gp_context_unref(self.context);
        }
    }
}

#[doc(hidden)]
impl Handle<crate::gphoto2::GPContext> for Context {
    unsafe fn as_ptr(&self) -> *const crate::gphoto2::GPContext {
        self.context
    }
}

#[doc(hidden)]
impl HandleMut<crate::gphoto2::GPContext> for Context {
    unsafe fn as_mut_ptr(&mut self) -> *mut crate::gphoto2::GPContext {
        self.context
    }
}
