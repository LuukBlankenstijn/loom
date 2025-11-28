use glib_sys::{GError, gboolean};
use libc::c_char;

#[repr(C)]
pub struct LightDMGreeter {
    // GObject, we never touch the fields directly.
    _opaque: [u8; 0],
}

#[allow(dead_code)]
unsafe extern "C" {
    pub fn lightdm_greeter_new() -> *mut LightDMGreeter;

    pub fn lightdm_greeter_connect_to_daemon_sync(
        greeter: *mut LightDMGreeter,
        error: *mut *mut GError,
    ) -> gboolean;

    pub fn lightdm_greeter_authenticate(
        greeter: *mut LightDMGreeter,
        username: *const c_char,
        error: *mut *mut GError,
    ) -> gboolean;

    pub fn lightdm_greeter_authenticate_as_guest(
        greeter: *mut LightDMGreeter,
        error: *mut *mut GError,
    ) -> gboolean;

    pub fn lightdm_greeter_authenticate_autologin(
        greeter: *mut LightDMGreeter,
        error: *mut *mut GError,
    ) -> gboolean;

    pub fn lightdm_greeter_respond(
        greeter: *mut LightDMGreeter,
        response: *const c_char,
        error: *mut *mut GError,
    ) -> gboolean;

    pub fn lightdm_greeter_cancel_authentication(
        greeter: *mut LightDMGreeter,
        error: *mut *mut GError,
    ) -> gboolean;

    pub fn lightdm_greeter_get_in_authentication(greeter: *mut LightDMGreeter) -> gboolean;

    pub fn lightdm_greeter_get_is_authenticated(greeter: *mut LightDMGreeter) -> gboolean;

    pub fn lightdm_greeter_get_authentication_user(greeter: *mut LightDMGreeter) -> *const c_char;

    pub fn lightdm_greeter_start_session_sync(
        greeter: *mut LightDMGreeter,
        session: *const c_char,
        error: *mut *mut GError,
    ) -> gboolean;

    pub fn lightdm_greeter_get_default_session_hint(greeter: *mut LightDMGreeter) -> *const c_char;

    pub fn lightdm_greeter_get_autologin_user_hint(greeter: *mut LightDMGreeter) -> *const c_char;
}
