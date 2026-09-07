use std::path::PathBuf;

#[cfg(target_os = "windows")]
pub fn zomboid_base_dir() -> Result<PathBuf, String> {
    std::env::home_dir()
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .map(|home| home.join("Zomboid").join("mods"))
        .ok_or_else(|| "cannot determine zomboid directory".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn zomboid_base_dir() -> Result<PathBuf, String> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg);
        if p.is_absolute() && !p.as_os_str().is_empty() {
            return Ok(p);
        }
        // XDG spec: a non-absolute XDG_CONFIG_HOME must be ignored.
    }

    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .map(|home| home.join("Zomboid").join("mods"))
        .ok_or_else(|| "HOME is not set; cannot determine config directory".to_string())
}

