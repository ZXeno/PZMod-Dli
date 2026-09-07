use reqwest::Url;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{config::Config, zomboid};

const SCMD_LINUX_URL: &str = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz";
const SCMD_WINDOWS_URL: &str = "https://client-update.steamstatic.com/installer/steamcmd.zip";

pub fn init_scmd() -> Result<(), String> {
    if check_scmd_exists() {
        return Ok(());
    }

    let scmd_path = get_scmd_path()?;
    let parent = scmd_path
        .parent()
        .ok_or_else(|| format!("SteamCMD path {} has no parent directory", scmd_path.display()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("cannot create directory {}: {e}", parent.display()))?;

    acquire_scmd()?;
    Ok(())
}

pub fn run_scmd(item_ids: &Vec<String>, config: &Config) -> Result<(), String> {
    let staging_dir = config.target_dir.join(".pzmdli-staging");
    let mut steamcmd = build_steamcmd_command(&config, &staging_dir, &item_ids);

    // todo: execute steamcmd
    println!("starting steamcmd...");
    let status = steamcmd
       .status()
       .expect("failed to launch steamcmd. check the configured path");

    if !status.success() {
        eprintln!("steamcmd exited with a failure status: {status}");
        std::process::exit(1);
    }

    // validate all mods downloaded and move them if they did
    for item in item_ids {
        process_mod(&item, &staging_dir, &config);
    }

    if staging_dir.exists() {
       println!("cleaning up...");
       std::fs::remove_dir_all(&staging_dir).expect("could not remove download staging directory");
   }

    Ok(())
}

pub fn check_scmd_exists() -> bool {
    match get_scmd_path() {
        Ok(scmd_path) => scmd_path.exists(),
        Err(_) => false,
    }
}

fn process_mod(item: &String, staging_dir: &PathBuf, config: &Config) {
    // get the item directory
   let item_dir = build_workshop_item_folder_path(item, &staging_dir, &config);
   println!("processing {item}");
   let exists = item_dir.exists();
   if !exists {
       eprintln!("Couldn't validate the path for {item} - skipping");
       return
   }

   let item_mods_dir = item_dir.join("mods");
   let mods_dir_contents: Vec<PathBuf> = list_dirs(&item_mods_dir).expect("could not enumerate the mod's content directory");
   let mut mod_dir_name: String = String::new();
   for m in mods_dir_contents {
       let mod_name = match m.file_name().and_then(|n| n.to_str()) {
           Some(name) => name,
           None => {
               eprintln!("cannot determine mod name from {}", m.display());
               std::process::exit(1);
           }
       };
       mod_dir_name = String::from(mod_name);
       break;
   }

   println!("found mod directory: {}", mod_dir_name);
   let actual_mod_dir = item_mods_dir.join(&mod_dir_name);
   let target_mod_dir = zomboid::zomboid_base_dir().expect("could not get zomboid directory");
   if target_mod_dir.exists() {
       println!("removing existing folder for {mod_dir_name}");
       std::fs::remove_dir_all(&target_mod_dir).expect("could not remove existing mod directory");
   }

   match std::fs::rename(actual_mod_dir, target_mod_dir) {
       Ok(_) => println!("installed {}", mod_dir_name),
       Err(e) => {
           eprintln!("failed to install mod with error: {e}")
       }
   };
}

fn list_dirs(dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(dir)
        .map_err(|e| format!("cannot read directory {}: {e}", dir.display()))?
    {
        let entry = entry.map_err(|e| format!("cannot read entry in {}: {e}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    Ok(dirs)
}

fn build_workshop_item_folder_path(item_id: &String, staging_dir: &PathBuf, config: &Config) -> PathBuf {
    let item_path: PathBuf = staging_dir
        .join("steamapps")
        .join("workshop")
        .join("content")
        .join(&config.pz_app_id)
        .join(item_id);

    item_path
}

fn build_steamcmd_command(config: &Config, staging_dir: &PathBuf, item_ids: &Vec<String>) -> Command {
    let scmd_path = get_scmd_path().expect("unable to retrieve scmd path!");
    let mut steamcmd = Command::new(scmd_path);
    
    // set the staging directory for scmd to download to.
    steamcmd.arg("+force_install_dir");
    steamcmd.arg(staging_dir.to_str().unwrap_or_default());

    // set the login to anonymous
    steamcmd.arg("+login");
    steamcmd.arg("anonymous");

    // build the item download list
    for item in item_ids {
        steamcmd.arg("+workshop_download_item");
        steamcmd.arg(&config.pz_app_id);
        steamcmd.arg(item);
    }

    steamcmd.arg("+quit");
    steamcmd
}

#[cfg(target_os = "windows")]
fn acquire_scmd() -> Result<(), String> {
    let dest = get_scmd_path()?;
    if dest.exists() {
        return Ok(());
    }

    let zip_path = download_scmd(SCMD_WINDOWS_URL)?;
    extract_zip(&zip_path, dest.parent().unwrap_or(Path::new(".")))?;
    // The archive has served its purpose; the extracted files are the deliverable.
    std::fs::remove_file(&zip_path)
        .map_err(|e| format!("cannot remove downloaded archive {}: {e}", zip_path.display()))?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn acquire_scmd() -> Result<(), String> {
    let dest = get_scmd_path()?;
    if dest.exists() {
        return Ok(());
    }

    let archive_path = download_scmd(SCMD_LINUX_URL)?;
    extract_tar_gz(&archive_path, dest.parent().unwrap_or(Path::new(".")))?;
    std::fs::remove_file(&archive_path)
        .map_err(|e| format!("cannot remove downloaded archive {}: {e}", archive_path.display()))?;
    Ok(())
}

fn download_scmd(url: &str) -> Result<PathBuf, String> {
    let dest = get_scmd_path()?;
    let parent = dest
        .parent()
        .ok_or_else(|| format!("SteamCMD path {} has no parent directory", dest.display()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("cannot create directory {}: {e}", parent.display()))?;

    let file_name = url
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("cannot derive a file name from {url}"))?;
    let archive_path = parent.join(file_name);
    let tmp_path = parent.join(format!("{file_name}.tmp"));

    // Validate the URL up front so a bad URL fails before any file is touched.
    let url = Url::parse(url).map_err(|e| format!("invalid SteamCMD URL {url}: {e}"))?;

    let mut response = reqwest::blocking::get(url.as_str())
        .map_err(|e| format!("request to {url} failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("{url} returned HTTP {}", response.status()));
    }

    let mut file = std::fs::File::create(&tmp_path)
        .map_err(|e| format!("cannot create {}: {e}", tmp_path.display()))?;

    response
        .copy_to(&mut file)
        .map_err(|e| {
            // A failed body write leaves a partial file; drop it.
            let _ = std::fs::remove_file(&tmp_path);
            format!("writing {} failed: {e}", tmp_path.display())
        })?;

    // Rename is atomic on the same filesystem: a crash mid-download can
    // leave the .tmp behind, never a truncated archive at the real name.
    std::fs::rename(&tmp_path, &archive_path)
        .map_err(|e| format!("cannot finalize {}: {e}", archive_path.display()))?;

    Ok(archive_path)
}

#[cfg(target_os = "windows")]
fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive_path)
        .map_err(|e| format!("cannot open {}: {e}", archive_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("cannot read zip {}: {e}", archive_path.display()))?;

    // mangled_name sanitizes path traversal ("zip-slip") entries.
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("cannot read zip entry {i} in {}: {e}", archive_path.display()))?;
        let out_path = dest_dir.join(entry.mangled_name());
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| format!("cannot create directory {}: {e}", out_path.display()))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot create directory {}: {e}", parent.display()))?;
            }
            let mut out_file = std::fs::File::create(&out_path)
                .map_err(|e| format!("cannot create {}: {e}", out_path.display()))?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|e| format!("cannot extract zip entry {}: {e}", entry.name()))?;
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive_path)
        .map_err(|e| format!("cannot open {}: {e}", archive_path.display()))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    // unpack guards against path traversal and unsafe symlinks.
    archive
        .unpack(dest_dir)
        .map_err(|e| format!("cannot extract {}: {e}", archive_path.display()))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn get_scmd_path() -> Result<PathBuf, String> {
    let base = Config::config_dir()?;
    Ok(base.join("steamcmd.exe"))
}

#[cfg(not(target_os = "windows"))]
fn get_scmd_path() -> Result<PathBuf, String> {
    let base = Config::config_dir()?;
    Ok(base.join("steamcmd"))
}
