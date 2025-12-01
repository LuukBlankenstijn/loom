pub mod sys;

use std::{
    cell::Cell,
    ffi::{CStr, CString},
    fmt,
    marker::PhantomData,
    ptr::{self, NonNull},
    rc::Rc,
};

use glib_sys::{GError, g_error_free, gboolean, gpointer};
use gobject_sys::{GClosure, GConnectFlags, g_signal_connect_data, g_signal_handler_disconnect};
use libc::c_ulong;
use log::{debug, error};
use sys as lightdm_sys;

#[derive(Debug)]
pub struct GreeterError(String);

impl fmt::Display for GreeterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GreeterError: {}", self.0)
    }
}

impl std::error::Error for GreeterError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptType {
    Question,
    Secret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Info,
    Error,
}

impl From<lightdm_sys::LightDMMessageType> for MessageType {
    fn from(value: lightdm_sys::LightDMMessageType) -> Self {
        match value {
            lightdm_sys::LightDMMessageType::LIGHTDM_MESSAGE_TYPE_INFO => MessageType::Info,
            lightdm_sys::LightDMMessageType::LIGHTDM_MESSAGE_TYPE_ERROR => MessageType::Error,
        }
    }
}

impl From<lightdm_sys::LightDMPromptType> for PromptType {
    fn from(value: lightdm_sys::LightDMPromptType) -> Self {
        match value {
            lightdm_sys::LightDMPromptType::LIGHTDM_PROMPT_TYPE_QUESTION => PromptType::Question,
            lightdm_sys::LightDMPromptType::LIGHTDM_PROMPT_TYPE_SECRET => PromptType::Secret,
        }
    }
}

pub struct Greeter {
    ptr: NonNull<lightdm_sys::LightDMGreeter>,
    prompt_handler: Cell<Option<c_ulong>>,
    message_handler: Cell<Option<c_ulong>>,
    auth_complete_handler: Cell<Option<c_ulong>>,
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
                    prompt_handler: Cell::new(None),
                    message_handler: Cell::new(None),
                    auth_complete_handler: Cell::new(None),
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
        Self::respond_with_ptr(self.ptr, response)
    }

    pub fn set_prompt_responder<F>(&self, callback: F)
    where
        F: Fn(&str, PromptType) + 'static,
    {
        self.disconnect_prompt_handler();
        let handler_id = unsafe { connect_show_prompt(self.ptr, callback) };
        self.prompt_handler.set(Some(handler_id));
    }

    /// Respond to any secret prompt (usually the password) with the provided value.
    pub fn respond_to_secret_prompts(&self, response: String) {
        let greeter_ptr = self.ptr;
        self.set_prompt_responder(move |prompt, prompt_type| {
            if matches!(prompt_type, PromptType::Secret) {
                debug!("Responding to prompt: {}", prompt);
                if let Err(err) = Greeter::respond_with_ptr(greeter_ptr, &response) {
                    error!("Failed to respond to prompt: {err}");
                }
            } else {
                debug!("Ignoring non-secret prompt: {}", prompt);
            }
        });
    }

    fn disconnect_prompt_handler(&self) {
        if let Some(id) = self.prompt_handler.take() {
            unsafe {
                g_signal_handler_disconnect(self.ptr.as_ptr() as *mut _, id);
            }
        }
    }

    pub fn set_message_handler<F>(&self, callback: F)
    where
        F: Fn(&str, MessageType) + 'static,
    {
        self.disconnect_message_handler();
        let handler_id = unsafe { connect_show_message(self.ptr, callback) };
        self.message_handler.set(Some(handler_id));
    }

    fn disconnect_message_handler(&self) {
        if let Some(id) = self.message_handler.take() {
            unsafe {
                g_signal_handler_disconnect(self.ptr.as_ptr() as *mut _, id);
            }
        }
    }

    pub fn set_authentication_complete_handler<F>(&self, callback: F)
    where
        F: Fn(bool) + 'static,
    {
        self.disconnect_auth_complete_handler();
        let handler_id = unsafe { connect_authentication_complete(self.ptr, callback) };
        self.auth_complete_handler.set(Some(handler_id));
    }

    fn disconnect_auth_complete_handler(&self) {
        if let Some(id) = self.auth_complete_handler.take() {
            unsafe {
                g_signal_handler_disconnect(self.ptr.as_ptr() as *mut _, id);
            }
        }
    }

    fn respond_with_ptr(
        ptr: NonNull<lightdm_sys::LightDMGreeter>,
        response: &str,
    ) -> Result<(), GreeterError> {
        let response = CString::new(response)
            .map_err(|_| GreeterError("response contained a NUL byte".into()))?;

        unsafe {
            let mut error: *mut GError = ptr::null_mut();
            let ok =
                lightdm_sys::lightdm_greeter_respond(ptr.as_ptr(), response.as_ptr(), &mut error);

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
        self.disconnect_prompt_handler();
        self.disconnect_message_handler();
        self.disconnect_auth_complete_handler();
        unsafe {
            gobject_sys::g_object_unref(self.ptr.as_ptr() as *mut _);
        }
    }
}

unsafe fn connect_show_prompt<F>(ptr: NonNull<lightdm_sys::LightDMGreeter>, callback: F) -> c_ulong
where
    F: Fn(&str, PromptType) + 'static,
{
    unsafe extern "C" fn show_prompt_trampoline<F>(
        _greeter: *mut lightdm_sys::LightDMGreeter,
        message: *const libc::c_char,
        prompt_type: lightdm_sys::LightDMPromptType,
        user_data: gpointer,
    ) where
        F: Fn(&str, PromptType) + 'static,
    {
        let callback = unsafe { &*(user_data as *const F) };
        let message = unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned();
        callback(&message, PromptType::from(prompt_type));
    }

    unsafe extern "C" fn drop_closure<F>(data: gpointer, _: *mut GClosure)
    where
        F: Fn(&str, PromptType) + 'static,
    {
        unsafe {
            drop(Box::from_raw(data as *mut F));
        }
    }

    unsafe {
        g_signal_connect_data(
            ptr.as_ptr() as *mut _,
            b"show-prompt\0".as_ptr() as *const _,
            Some(std::mem::transmute::<
                unsafe extern "C" fn(
                    *mut lightdm_sys::LightDMGreeter,
                    *const libc::c_char,
                    lightdm_sys::LightDMPromptType,
                    gpointer,
                ),
                unsafe extern "C" fn(),
            >(show_prompt_trampoline::<F>)),
            Box::into_raw(Box::new(callback)) as *mut _,
            Some(drop_closure::<F>),
            GConnectFlags::default(),
        )
    }
}

unsafe fn connect_show_message<F>(ptr: NonNull<lightdm_sys::LightDMGreeter>, callback: F) -> c_ulong
where
    F: Fn(&str, MessageType) + 'static,
{
    unsafe extern "C" fn show_message_trampoline<F>(
        _greeter: *mut lightdm_sys::LightDMGreeter,
        message: *const libc::c_char,
        message_type: lightdm_sys::LightDMMessageType,
        user_data: gpointer,
    ) where
        F: Fn(&str, MessageType) + 'static,
    {
        let callback = unsafe { &*(user_data as *const F) };
        let message = unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned();
        callback(&message, MessageType::from(message_type));
    }

    unsafe extern "C" fn drop_closure<F>(data: gpointer, _: *mut GClosure)
    where
        F: Fn(&str, MessageType) + 'static,
    {
        unsafe {
            drop(Box::from_raw(data as *mut F));
        }
    }

    unsafe {
        g_signal_connect_data(
            ptr.as_ptr() as *mut _,
            b"show-message\0".as_ptr() as *const _,
            Some(std::mem::transmute::<
                unsafe extern "C" fn(
                    *mut lightdm_sys::LightDMGreeter,
                    *const libc::c_char,
                    lightdm_sys::LightDMMessageType,
                    gpointer,
                ),
                unsafe extern "C" fn(),
            >(show_message_trampoline::<F>)),
            Box::into_raw(Box::new(callback)) as *mut _,
            Some(drop_closure::<F>),
            GConnectFlags::default(),
        )
    }
}

unsafe fn connect_authentication_complete<F>(
    ptr: NonNull<lightdm_sys::LightDMGreeter>,
    callback: F,
) -> c_ulong
where
    F: Fn(bool) + 'static,
{
    unsafe extern "C" fn auth_complete_trampoline<F>(
        greeter: *mut lightdm_sys::LightDMGreeter,
        user_data: gpointer,
    ) where
        F: Fn(bool) + 'static,
    {
        let callback = unsafe { &*(user_data as *const F) };
        let authed = unsafe { lightdm_sys::lightdm_greeter_get_is_authenticated(greeter) != 0 };
        callback(authed);
    }

    unsafe extern "C" fn drop_closure<F>(data: gpointer, _: *mut GClosure)
    where
        F: Fn(bool) + 'static,
    {
        unsafe {
            drop(Box::from_raw(data as *mut F));
        }
    }

    unsafe {
        g_signal_connect_data(
            ptr.as_ptr() as *mut _,
            b"authentication-complete\0".as_ptr() as *const _,
            Some(std::mem::transmute::<
                unsafe extern "C" fn(*mut lightdm_sys::LightDMGreeter, gpointer),
                unsafe extern "C" fn(),
            >(auth_complete_trampoline::<F>)),
            Box::into_raw(Box::new(callback)) as *mut _,
            Some(drop_closure::<F>),
            GConnectFlags::default(),
        )
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
