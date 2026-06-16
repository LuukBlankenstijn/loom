use std::collections::HashMap;
use std::fs;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use zbus::zvariant::OwnedObjectPath;
use zbus::{Connection, proxy};
#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait Manager {
    fn list_sessions_ex(
        &self,
    ) -> zbus::Result<
        Vec<(
            String,
            u32,
            String,
            String,
            u32,
            String,
            String,
            bool,
            u64,
            OwnedObjectPath,
        )>,
    >;
}

#[proxy(
    interface = "org.freedesktop.login1.Session",
    default_service = "org.freedesktop.login1"
)]
trait Session {
    #[zbus(property)]
    fn state(&self) -> zbus::Result<String>;

    #[zbus(property, name = "Type")]
    fn type_(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn remote(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn display(&self) -> zbus::Result<String>;
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    id: String,
    uid: u32,
    user: String,
    session_type: String,
    display: String,
    class: String,
    state: String,
}

impl SessionInfo {
    fn priority(&self) -> Option<u8> {
        match (self.class.as_str(), self.state.as_str()) {
            ("user", "active") => Some(0),
            ("greeter", "active") => Some(1),
            ("user", _) => Some(2),
            ("greeter", _) => Some(3),
            _ => None,
        }
    }
}

pub async fn find_target_session(conn: &Connection) -> Result<SessionInfo> {
    let manager = ManagerProxy::new(conn).await?;
    let sessions = manager.list_sessions_ex().await?;

    let mut candidates = Vec::new();

    for (id, uid, user, seat, _leader, class, _tty, _idle, _idle_since, path) in sessions {
        // Seat-less sessions are not graphical (SSH, user managers, etc.)
        if seat.is_empty() {
            continue;
        }
        // Only consider user/greeter sessions
        if !matches!(class.as_str(), "user" | "greeter") {
            continue;
        }

        let session = SessionProxy::builder(conn).path(path)?.build().await?;

        if session.remote().await.unwrap_or(false) {
            continue;
        }

        candidates.push(SessionInfo {
            id,
            uid,
            user,
            class,
            state: session.state().await.unwrap_or_default(),
            session_type: session.type_().await.unwrap_or_default(),
            display: session.display().await.unwrap_or_default(),
        });
    }

    let chosen = candidates
        .into_iter()
        .filter_map(|s| s.priority().map(|p| (p, s)))
        .min_by_key(|(p, _)| *p)
        .map(|(_, s)| s)
        .ok_or_else(|| anyhow!("no suitable graphical session found"))?;

    tracing::info!(
        session_id = %chosen.id,
        user = %chosen.user,
        uid = chosen.uid,
        class = %chosen.class,
        state = %chosen.state,
        session_type = %chosen.session_type,
        display = %chosen.display,
        "selected graphical session for registration tool",
    );

    Ok(chosen)
}

/// Reads the real uid and gid of a process from `/proc/<pid>/status`.
fn read_proc_ids(pid: u32) -> Option<(u32, u32)> {
    let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    let field = |key: &str| {
        status
            .lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
    };
    Some((field("Uid:")?, field("Gid:")?))
}

fn read_proc_environ(pid: u32) -> Option<HashMap<String, String>> {
    let data = fs::read(format!("/proc/{pid}/environ")).ok()?;
    let mut env = HashMap::new();
    for entry in data.split(|&b| b == 0) {
        if let Ok(s) = std::str::from_utf8(entry) {
            if let Some((k, v)) = s.split_once('=') {
                env.insert(k.to_string(), v.to_string());
            }
        }
    }
    Some(env)
}

/// A graphical session's real gid and harvested environment, taken from a live
/// process owned by the session user.
struct HarvestedSession {
    gid: u32,
    env: HashMap<String, String>,
}

/// Scans `/proc` for a process owned by `uid` that carries a usable graphical
/// environment, returning its gid and environment. Prefers a Wayland process,
/// falling back to any X11/runtime-bearing one.
fn harvest_user_session(uid: u32) -> Option<HarvestedSession> {
    let mut best: Option<HarvestedSession> = None;

    for entry in fs::read_dir("/proc").ok()?.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let Some((puid, pgid)) = read_proc_ids(pid) else {
            continue;
        };
        if puid != uid {
            continue;
        }
        let Some(env) = read_proc_environ(pid) else {
            continue;
        };

        let has_wayland = env.contains_key("WAYLAND_DISPLAY");
        let has_x11 = env.contains_key("DISPLAY");
        let has_runtime = env.contains_key("XDG_RUNTIME_DIR");

        if has_wayland && has_runtime {
            return Some(HarvestedSession { gid: pgid, env });
        }
        if (has_x11 || has_runtime) && best.is_none() {
            best = Some(HarvestedSession { gid: pgid, env });
        }
    }

    best
}

/// Launches `program` inside the target session's compositor, dropping
/// privileges to the session user. `extra_env` is added on top of the harvested
/// graphical environment (used to pass the backend server/token via env rather
/// than argv, which would be world-readable in `/proc/<pid>/cmdline`).
pub fn launch_gui(
    session: &SessionInfo,
    program: &str,
    extra_env: &[(&str, &str)],
) -> Result<Child> {
    let harvested = harvest_user_session(session.uid).ok_or_else(|| {
        anyhow!(
            "no graphical process found for uid {} ({})",
            session.uid,
            session.user
        )
    })?;
    let user_env = &harvested.env;

    let mut cmd = Command::new(program);
    // Inherit the target user's actual session environment — Wayland display,
    // XDG dirs, D-Bus, and crucially the NixOS GPU driver discovery (drivers
    // live under /run/opengl-driver and are found via the session's env). A
    // hand-picked allowlist dropped those, so Mesa couldn't find any DRI driver
    // (libEGL "fd -1") and even software rendering failed. env_clear() first so
    // none of loomd's root environment leaks in.
    cmd.env_clear();
    cmd.envs(user_env);

    if !user_env.contains_key("PATH") {
        cmd.env(
            "PATH",
            "/run/current-system/sw/bin:/run/wrappers/bin:/usr/bin:/bin",
        );
    }

    // A greeter system user's HOME is the immutable /var/empty, so WebKitGTK/GTK
    // writing their cache/config there fails with EPERM. When the home is
    // unusable, redirect HOME and the XDG base dirs to the writable per-user
    // runtime dir.
    let harvested_home = user_env.get("HOME").map(String::as_str).unwrap_or("");
    let home_unusable = matches!(harvested_home, "" | "/var/empty" | "/nonexistent" | "/dev/null");
    if home_unusable
        && let Some(rt) = user_env.get("XDG_RUNTIME_DIR")
    {
        cmd.env("HOME", rt);
        let base = format!("{rt}/loom-registration");
        cmd.env("XDG_DATA_HOME", format!("{base}/data"));
        cmd.env("XDG_CONFIG_HOME", format!("{base}/config"));
        cmd.env("XDG_CACHE_HOME", format!("{base}/cache"));
        cmd.env("XDG_STATE_HOME", format!("{base}/state"));
    }

    for (key, val) in extra_env {
        cmd.env(key, val);
    }

    // Drop privileges before exec, while still root and in the right order:
    // install the target user's real groups (so the GUI keeps video/render/
    // input/audio access — NOT setgroups(0)), then setgid, then setuid (after
    // setuid the process can no longer change groups). std's `uid`/`gid` setters
    // run *after* `pre_exec`, so we do the whole drop here ourselves. The group
    // list is resolved before fork because `setgroups` is async-signal-safe but
    // the NSS lookup behind `getgrouplist` is not.
    let user_cstr =
        std::ffi::CString::new(session.user.as_bytes()).context("session user has a NUL byte")?;
    let groups = supplementary_groups(&user_cstr, harvested.gid);
    let uid = session.uid as libc::uid_t;
    let gid = harvested.gid as libc::gid_t;
    unsafe {
        cmd.pre_exec(move || {
            if libc::setgroups(groups.len() as _, groups.as_ptr()) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::setgid(gid) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::setuid(uid) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    cmd.spawn().context("failed to spawn GUI process")
}

/// Resolves the full group list for `user` (primary `gid` plus supplementary
/// groups), like `initgroups`. Computed before fork so the `pre_exec` closure
/// only needs the async-signal-safe `setgroups`.
fn supplementary_groups(user: &std::ffi::CStr, gid: u32) -> Vec<libc::gid_t> {
    let mut count: libc::c_int = 16;
    let mut groups: Vec<libc::gid_t> = vec![0; count as usize];
    loop {
        let rc = unsafe {
            libc::getgrouplist(
                user.as_ptr(),
                gid as libc::gid_t,
                groups.as_mut_ptr(),
                &mut count,
            )
        };
        if rc >= 0 {
            groups.truncate(count as usize);
            return groups;
        }
        // Buffer was too small; `count` now holds the required length.
        groups.resize(count as usize, 0);
    }
}

/// Sends SIGTERM, waits up to `grace`, then escalates to SIGKILL if needed.
/// Async so it never blocks the runtime worker.
pub async fn terminate_child(child: &mut Child, grace: Duration) -> Result<()> {
    let pid = child.id() as i32;
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    let step = Duration::from_millis(100);
    let mut waited = Duration::ZERO;
    while waited < grace {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        tokio::time::sleep(step).await;
        waited += step;
    }

    // Grace expired — force kill and reap without blocking.
    child.kill()?;
    for _ in 0..20 {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        tokio::time::sleep(step).await;
    }
    Ok(())
}
