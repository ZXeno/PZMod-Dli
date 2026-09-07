
pub mod config;

use std::{
    path::{self, PathBuf}
};
use config::Config;
use std::process::Command;

const PZ_APP_ID: &str = "108600";

fn main() {
    println!("Welcome to the no-steam PZ Mod steam workshop item Downloader and Installer!");
    println!("");
    println!("This will download project zomboid mods from the steam workshop directly to a directory of your choice.");
    println!("");

    // get config. if no previous config or its broken, create a new one
    let mut config: Config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            if Config::exists() {
                eprintln!("Existing config is broken. Starting a fresh one. Reason: {e}");
            } else {
                print_general_help();
            }
            Config::default()
        }
    };

    // get the steamcmd path
    if !config.steamcmd_path.exists() {
        let steamcmd_path_string: String = read_input("Path to SteamCMD binary: ");
        config.steamcmd_path = path::PathBuf::from(steamcmd_path_string);
    }

    // check if the config has the install path
    if !config.target_dir.exists() {
        let target_path_string = read_input("Target mod install path: ");
        config.target_dir = path::PathBuf::from(target_path_string);
    }

    Config::save(&config).expect("Unable to save config for next time.");

    // get the item list from args and check empty, if empty prompt for csv list
    println!("parsing items list...");
    let mut item_ids: Vec<String> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--items" => {
                let raw = args.next().ok_or("--items requires a value").expect("--items requires a value");
                item_ids = parse_list_arg(&raw).expect("The list was incorrectly formatted. Please provide a comma-separated list of workshop item ids");
            }
            _ => eprintln!("unknown argument: {arg}"),
        }
    }

    if item_ids.iter().count() <= 0 {
        eprintln!("no workshop item ids were provided! please provide a list of item ids.");
        eprintln!("usage: pzmod-dli --items 12345678");
        std::process::exit(1)
    }

        // set the target directory for the base install dir.
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
    for item in &item_ids {
        process_mod(&item, &staging_dir, &config);
    }

    println!("Done!");

    if staging_dir.exists() {
       println!("cleaning up...");
       std::fs::remove_dir_all(&staging_dir).expect("could not remove download staging directory directory");
   }
}

fn process_mod(item: &String, staging_dir: &PathBuf, config: &Config) {
    // get the item directory
   let item_dir = build_workshop_item_folder_path(item, &staging_dir);
   println!("processing {item}");
   let exists = item_dir.exists();
   if !exists {
       eprintln!("Couldn't validate the path for {item} - skipping");
       return
   }

   let mods_dir = item_dir.join("mods");   let mods_dir_contents: Vec<PathBuf> = list_dirs(&mods_dir).expect("could not enumerate the mod's content directory");
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
   let actual_mod_dir = mods_dir.join(&mod_dir_name);
   //if config.target_dir.ends_with("mods") { config.target_dir } else { config.target_dir.join("mods") };
   let target_mod_dir = if config.target_dir.ends_with("mods") { 
       config.target_dir.join(&mod_dir_name)
   } else { 
       config.target_dir.join("mods").join(&mod_dir_name)
   };

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

fn build_workshop_item_folder_path(item_id: &String, staging_dir: &PathBuf) -> PathBuf {
    let item_path: PathBuf = staging_dir
        .join("steamapps")
        .join("workshop")
        .join("content")
        .join(PZ_APP_ID)
        .join(item_id);

    item_path
}

fn build_steamcmd_command(config: &Config, staging_dir: &PathBuf, item_ids: &Vec<String>) -> Command {
    let mut steamcmd = Command::new(config.steamcmd_path.clone());
    steamcmd.arg("+force_install_dir");
    steamcmd.arg(staging_dir.to_str().unwrap_or_default());

    // set the login to anonymous
    steamcmd.arg("+login");
    steamcmd.arg("anonymous");

    for item in item_ids {
        steamcmd.arg("+workshop_download_item");
        steamcmd.arg(PZ_APP_ID);
        steamcmd.arg(item);
    }

    steamcmd.arg("+quit");
    steamcmd
}

fn read_input(msg: &str) -> String {
    let stdin = std::io::stdin();
    println!("{}", msg);
    let mut input_buf = String::new();
    stdin.read_line(&mut input_buf).expect("Failed to read user input! Error: {err}");
    input_buf.trim().to_string()
}

fn parse_list_arg(raw: &str) -> Result<Vec<String>, String> {
    let items: Vec<String> = raw.split(',').map(str::trim).map(String::from).collect();
    if items.iter().any(String::is_empty) {
        return Err(format!("empty item in list: '{raw}'"));
    }
    Ok(items)
}

fn print_general_help() {
    println!("");
    println!("You will need to download SteamCMD for your OS first. You can do that here: https://developer.valvesoftware.com/wiki/SteamCMD");
    println!("Once you have that installed, this program will prompt you for your steamcmd path if you have not previously provided one.");
    println!("Then, you will be prompted for a comma-separated list of workshop item IDs.");
    println!("Then, if you haven't specified it before, you will be prompted for an install location to install the mods to.");
    println!("");
    println!("You'll want to specify the root install folder - it'll look for the `mods` folder inside that. So if you're installing to, say, `C:\\games\\zomboid\\mods\\`, you'll specify ``C:\\games\\zomboid\\` instead.");
    println!("");
    println!("The config will be saved to the same location as this application.");
    println!("");
}
