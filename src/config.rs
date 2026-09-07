
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "pzmdli.toml";
const APP_DIR_NAME: &str = "pzmod-dli";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub steamcmd_path: PathBuf,
    pub target_dir: PathBuf,
}

impl Config {
    pub fn exists() -> bool {
        match Config::config_path() {
            Ok(path) => path.exists(),
            Err(_) => false,
        }
    }

    pub fn load() -> Result<Config, String> {
        let path = Config::config_path()?;
        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("cannot read config {}: {e}", path.display()))?;
        toml::from_str(&raw).map_err(|e| format!("cannot parse config {}: {e}", path.display()))
    }

    pub fn save(new_config: &Config) -> Result<(), String> {
        let path = Config::config_path()?;
        let raw = toml::to_string_pretty(new_config)
            .map_err(|e| format!("cannot serialize config: {e}"))?;

        // First launch may be the first write: create the config directory.
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)
                .map_err(|e| format!("cannot create config dir {}: {e}", dir.display()))?;
        }

        // Write to a temp sibling first, then rename, so a crash mid-write
        // cannot truncate the existing config.
        let tmp = path.with_extension("toml.tmp");
        fs::write(&tmp, raw).map_err(|e| format!("cannot write config {}: {e}", tmp.display()))?;
        fs::rename(&tmp, &path)
            .map_err(|e| format!("cannot finalize config {}: {e}", path.display()))?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf, String> {
        Ok(Config::config_dir()?.join(CONFIG_FILE_NAME))
    }

    pub fn backup_path() -> Result<PathBuf, String> {
        Ok(Config::config_dir()?.join(format!("{}.bak", CONFIG_FILE_NAME)))
    }

    /// Windows: %APPDATA%\pzmod-dli
    /// Linux (XDG): $XDG_CONFIG_HOME/pzmod-dli, else $HOME/.config/pzmod-dli
    fn config_dir() -> Result<PathBuf, String> {
        let base = Config::config_base()?;
        Ok(base.join(APP_DIR_NAME))
    }

    #[cfg(target_os = "windows")]
    fn config_base() -> Result<PathBuf, String> {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or_else(|| "APPDATA is not set; cannot determine config directory".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    fn config_base() -> Result<PathBuf, String> {
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
            .map(|home| home.join(".config"))
            .ok_or_else(|| "HOME is not set; cannot determine config directory".to_string())
    }
}
