# PZMod-Dli

`pzmod-dli` is a tool that downloads Project Zomboid mods from the Steam Workshop and installs them into your mods folder. It works both with and without a Steam install of Project Zomboid for both Linux and Windows.

This tool wraps [SteamCMD](https://developer.valvesoftware.com/wiki/SteamCMD) for downloads of workshop items.

## Requirements

- Download [SteamCMD](https://developer.valvesoftware.com/wiki/SteamCMD) for your OS. Get it at the official link or from your distro's package manager. The page lists the install steps for Windows, Linux, and macOS. SteamCMD is Valve's command-line version of the Steam client. PZMod-Dli runs it with an anonymous login.
- Note the full path to the `steamcmd` executable. PZMod-Dli asks for it on first run.
- No Steam account is required to use this tool. (some mods may require Steam to work, which is outside the scope of this tool)

## Installation

Download the correct archive for your platform from the [releases page](https://github.com/ZXeno/pzmod-dli/releases):

- `pzmod-dli-linux-x86_64.tar.gz`
- `pzmod-dli-windows-x86_64.zip`

Unpack the archive. Put the binary wherever you like.

## Usage

```
pzmod-dli --items <id>[,<id>...]
```

`--items` takes one workshop item ID or a comma-separated list of IDs. Find the ID in the workshop item's URL: `https://steamcommunity.com/sharedfiles/filedetails/?id=3792740760` means `--items 3792740760`.

Example:

```
pzmod-dli --items 3792740760,3790573197,3794379457
```

On the first run, pzmod-dli tool prompts for two paths:

1. **Path to SteamCMD binary** — the full path to the `steamcmd` executable from the Requirements section.
2. **Target mod install path** — the root of your Project Zomboid install, not the `mods` folder itself. The tool looks for `mods` inside it. If you give a path that already ends in `mods`, the tool uses it directly.

Example target paths:

- Windows: `C:\Program Files (x86)\Steam\steamapps\common\ProjectZomboid`
- Linux: `~/.local/share/Steam/steamapps/common/ProjectZomboid`

The tool saves both paths to its config file and never asks again.

### What happens during a run

1. SteamCMD downloads each item into a staging folder (`.pzmdli-staging`, inside the target directory).
2. The tool locates the mod folder inside each download.
3. If the mod already exists in the target `mods` folder, the old folder is deleted.
4. The new mod folder is moved into place.
5. The staging folder is deleted.

A failed item prints a message and the run continues with the remaining items.

## Configuration

The config file is named `pzmdli.toml`:

- Windows: `%APPDATA%\pzmod-dli\pzmdli.toml`
- Linux: `$XDG_CONFIG_HOME/pzmod-dli/pzmdli.toml`, or `~/.config/pzmod-dli/pzmdli.toml` when `XDG_CONFIG_HOME` is unset

Delete the file to make the tool prompt for both paths again. A broken file makes the tool start a fresh config.

## Building from source

Requires a Rust toolchain (stable).

```
cargo build --release
```

The binary lands in `target/release/`.

