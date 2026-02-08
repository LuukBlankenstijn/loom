use ini::Ini;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub enum SessionType {
    Wayland,
    X11,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub slug: Option<String>,
    pub name: String,
    pub command: String,
    pub session_type: SessionType,
    pub xdg_desktop_names: Option<String>,
}

impl Session {
    pub fn get_session_command(&self) -> (String, Vec<String>) {
        let mut env = Vec::new();

        if let Some(ref slug) = self.slug {
            env.push(format!("XDG_SESSION_DESKTOP={}", slug));
            env.push(format!("DESKTOP_SESSION={}", slug));
        }

        match self.session_type {
            SessionType::Wayland => {
                env.push("XDG_SESSION_TYPE=wayland".to_string());
            }
            SessionType::X11 => {
                env.push("XDG_SESSION_TYPE=x11".to_string());
            }
        }

        if let Some(ref desktop_names) = self.xdg_desktop_names {
            env.push(format!("XDG_CURRENT_DESKTOP={}", desktop_names));
        }

        (self.command.clone(), env)
    }
}

pub fn get_sessions() -> Result<Vec<Session>, Box<dyn Error>> {
    let mut files = vec![];

    for (path, session_type) in build_session_paths().iter() {
        if let Ok(entries) = fs::read_dir(path) {
            files.extend(
                entries
                    .flat_map(|entry| {
                        entry.map(|entry| load_desktop_file(entry.path(), *session_type))
                    })
                    .flatten()
                    .flatten(),
            );
        }
    }

    files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(files)
}

fn load_desktop_file<P>(
    path: P,
    session_type: SessionType,
) -> Result<Option<Session>, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let desktop = Ini::load_from_file(path.as_ref())?;
    let section = desktop
        .section(Some("Desktop Entry"))
        .ok_or("no Desktop Entry section in desktop file")?;

    if let Some("true") = section.get("Hidden") {
        return Ok(None);
    }

    if let Some("true") = section.get("NoDisplay") {
        return Ok(None);
    }

    let slug = path
        .as_ref()
        .file_stem()
        .map(|slug| slug.to_string_lossy().to_string());

    let name = section
        .get("Name")
        .ok_or("no Name property in desktop file")?;

    let exec = section
        .get("Exec")
        .ok_or("no Exec property in desktop file")?;

    let xdg_desktop_names = section.get("DesktopNames").map(str::to_string);

    Ok(Some(Session {
        slug,
        name: name.to_string(),
        command: exec.to_string(),
        session_type,
        xdg_desktop_names,
    }))
}

// Build session paths from XDG_DATA_DIRS
fn build_session_paths() -> Vec<(PathBuf, SessionType)> {
    let data_dirs =
        env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

    let mut paths = Vec::new();

    for data_dir in data_dirs.split(':') {
        paths.push((
            PathBuf::from(data_dir).join("wayland-sessions"),
            SessionType::Wayland,
        ));
        paths.push((PathBuf::from(data_dir).join("xsessions"), SessionType::X11));
    }

    paths
}
