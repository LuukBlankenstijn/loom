pub mod sys;

use std::{
    ffi::{CStr, CString},
    fmt,
    marker::PhantomData,
    ptr::{self, NonNull},
    rc::Rc,
};

use glib_sys::{GError, g_error_free, gboolean};
use sys as lightdm_sys;

#[derive(Debug)]
pub struct GreeterError(String);

impl fmt::Display for GreeterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GreeterError: {}", self.0)
    }
}

impl std::error::Error for GreeterError {}

pub struct Greeter {
    ptr: NonNull<lightdm_sys::LightDMGreeter>,
    // GObject/LightDM types are not generally Send/Sync; Rc makes us !Send + !Sync.
    _not_send_sync: PhantomData<Rc<()>>,
}

#[allow(dead_code)]
impl Greeter {
    /// Construct a new LightDMGreeter GObject.
    pub fn new() -> Result<Self, GreeterError> {
        unsafe {
            let ptr = lightdm_sys::lightdm_greeter_new();
            if ptr.is_null() {
                Err(GreeterError("lightdm_greeter_new returned NULL".into()))
            } else {
                // Safety: we just checked for null.
                Ok(Greeter {
                    ptr: NonNull::new_unchecked(ptr),
                    _not_send_sync: PhantomData,
                })
            }
        }
    }

    pub fn connect_to_daemon(&self) -> Result<(), GreeterError> {
        unsafe {
            let mut error: *mut GError = ptr::null_mut();

            let ok: gboolean =
                lightdm_sys::lightdm_greeter_connect_to_daemon_sync(self.ptr.as_ptr(), &mut error);

            handle_gboolean(ok, error, "connect_to_daemon_sync")
        }
    }

    pub fn authenticate(&self, username: &str) -> Result<(), GreeterError> {
        let username = CString::new(username)
            .map_err(|_| GreeterError("username contained a NUL byte".into()))?;

        unsafe {
            let mut error: *mut GError = ptr::null_mut();
            let ok = lightdm_sys::lightdm_greeter_authenticate(
                self.ptr.as_ptr(),
                username.as_ptr(),
                &mut error,
            );

            handle_gboolean(ok, error, "authenticate")
        }
    }

    pub fn authenticate_as_guest(&self) -> Result<(), GreeterError> {
        unsafe {
            let mut error: *mut GError = ptr::null_mut();
            let ok =
                lightdm_sys::lightdm_greeter_authenticate_as_guest(self.ptr.as_ptr(), &mut error);

            handle_gboolean(ok, error, "authenticate_as_guest")
        }
    }

    pub fn authenticate_autologin(&self) -> Result<(), GreeterError> {
        unsafe {
            let mut error: *mut GError = ptr::null_mut();
            let ok =
                lightdm_sys::lightdm_greeter_authenticate_autologin(self.ptr.as_ptr(), &mut error);

            handle_gboolean(ok, error, "authenticate_autologin")
        }
    }

    pub fn respond(&self, response: &str) -> Result<(), GreeterError> {
        let response = CString::new(response)
            .map_err(|_| GreeterError("response contained a NUL byte".into()))?;

        unsafe {
            let mut error: *mut GError = ptr::null_mut();
            let ok = lightdm_sys::lightdm_greeter_respond(
                self.ptr.as_ptr(),
                response.as_ptr(),
                &mut error,
            );

            handle_gboolean(ok, error, "respond")
        }
    }

    pub fn cancel_authentication(&self) -> Result<(), GreeterError> {
        unsafe {
            let mut error: *mut GError = ptr::null_mut();
            let ok =
                lightdm_sys::lightdm_greeter_cancel_authentication(self.ptr.as_ptr(), &mut error);

            handle_gboolean(ok, error, "cancel_authentication")
        }
    }

    pub fn in_authentication(&self) -> bool {
        unsafe { lightdm_sys::lightdm_greeter_get_in_authentication(self.ptr.as_ptr()) != 0 }
    }

    pub fn is_authenticated(&self) -> bool {
        unsafe { lightdm_sys::lightdm_greeter_get_is_authenticated(self.ptr.as_ptr()) != 0 }
    }

    pub fn authentication_user(&self) -> Option<String> {
        unsafe {
            let ptr = lightdm_sys::lightdm_greeter_get_authentication_user(self.ptr.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }

    pub fn start_session(&self, session: Option<&str>) -> Result<(), GreeterError> {
        let session_cstring = match session {
            Some(session) => Some(
                CString::new(session)
                    .map_err(|_| GreeterError("session contained a NUL byte".into()))?,
            ),
            None => None,
        };

        unsafe {
            let mut error: *mut GError = ptr::null_mut();
            let ok = lightdm_sys::lightdm_greeter_start_session_sync(
                self.ptr.as_ptr(),
                session_cstring
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                &mut error,
            );

            handle_gboolean(ok, error, "start_session_sync")
        }
    }

    /// Convenience helpers for a couple of common hints.
    pub fn default_session_hint(&self) -> Option<String> {
        unsafe {
            let ptr = lightdm_sys::lightdm_greeter_get_default_session_hint(self.ptr.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }

    pub fn autologin_user_hint(&self) -> Option<String> {
        unsafe {
            let ptr = lightdm_sys::lightdm_greeter_get_autologin_user_hint(self.ptr.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }
}

impl Drop for Greeter {
    fn drop(&mut self) {
        unsafe {
            gobject_sys::g_object_unref(self.ptr.as_ptr() as *mut _);
        }
    }
}

unsafe fn c_error_to_string(error: *mut GError) -> String {
    // Safety: LightDM promises a valid, NUL-terminated message on GError.
    unsafe { CStr::from_ptr((*error).message) }
        .to_string_lossy()
        .into_owned()
}

fn handle_gboolean(ok: gboolean, error: *mut GError, context: &str) -> Result<(), GreeterError> {
    if ok == 0 {
        if !error.is_null() {
            // Safety: error is expected to be a valid GError when non-null.
            let msg = unsafe { c_error_to_string(error) };
            unsafe { g_error_free(error) };
            Err(GreeterError(format!("{context} failed: {msg}")))
        } else {
            Err(GreeterError(format!("{context} failed (no GError)")))
        }
    } else {
        Ok(())
    }
}
