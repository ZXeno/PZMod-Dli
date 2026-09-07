
pub mod config;
pub mod steamcmd;
pub mod zomboid;

use config::Config;

fn main() {
    println!("Welcome to the no-steam PZ Mod steam workshop item Downloader and Installer!");
    println!("");
    println!("This will download project zomboid mods from the steam workshop directly to a directory of your choice.");
    println!("");

    // get config. if no previous config or its broken, create a new one
    let config: Config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            if Config::exists() {
                eprintln!("Existing config is broken. Starting a fresh one. Reason: {e}");
            } else {
                print_general_help();
            }
            let cfg = Config::new();
            Config::save(&cfg).expect("Unable to save config");
            cfg
        }
    };

    // initialize SteamCMD, which will also acquire it if it is missing
    steamcmd::init_scmd().expect("steamcmd init failed.");


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

    // run the core process
    steamcmd::run_scmd(&item_ids, &config);

    println!("Done!");
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
    println!("Downloading SteamCMD directly from Valve first.");
    println!("");
    println!("After, it will parse your list of workshop item ids. You need to have provided them in a comma seaparted list via the '--items' param.");
    println!("Mods will be installed to your Zomboid folder (located in your user directory).");
    println!("");
    println!("The config will be saved to the same location as this application.");
    println!("");
}
