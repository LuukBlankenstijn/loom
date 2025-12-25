use zbus::interface;

pub trait GreeterServiceBackend: Send + Sync {
    fn set_wallpaper_source(&self, url: String);
    fn set_countdown_endtime(&self, end_time: i64) -> zbus::fdo::Result<()>;
    fn disable_countdown(&self);
    fn login(&self);
}

pub struct GreeterService<B: 'static> {
    backend: B,
}

impl<B> GreeterService<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
}

/// D-Bus service definition and generated proxy.
#[interface(
    name = "nl.luukblankenstijn.ContestGreeterService",
    proxy(
        gen_blocking = false,
        default_path = "/nl/luukblankenstijn/ContestGreeterService",
        default_service = "nl.luukblankenstijn.ContestGreeterService",
    )
)]
impl<B: GreeterServiceBackend> GreeterService<B> {
    /// Sets the source of the wallpaper to use. Can be an http url
    /// (if the machine has internet) or a local filepath.
    async fn set_wallpaper_source(&self, url: String) {
        self.backend.set_wallpaper_source(url);
    }

    /// Sets the time the countdown (if enabled) will end, and the session will be started.
    /// The i64 argument is the miliseconds since epoch.
    async fn set_countdown_endtime(&self, end_time: i64) -> zbus::fdo::Result<()> {
        self.backend.set_countdown_endtime(end_time)
    }

    /// Disable the countdown and the subsequent login by removing the endtime.
    async fn disable_countdown(&self) {
        self.backend.disable_countdown();
    }

    /// Unlocks the machine and starts the default session.
    /// This only works when a username and password have been configured for the greeter.
    async fn login(&self) {
        self.backend.login();
    }
}
