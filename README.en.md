[简体中文（默认）](README.md) | **English**

<img width="1920" height="1080" alt="Azeroth Survivors gameplay" src="https://github.com/user-attachments/assets/fa574bb6-7bd4-4e78-8b72-14c58e3c716e" />

<div align="center">

# Azeroth Survivors

**A local survival roguelite on a from-scratch WoW 1.12.1 client engine, built with Rust + [Bevy](https://bevy.org).**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE-MIT)

</div>

## About

This repository, [sqhome-jin/azeroth-survivors-re-cn](https://github.com/sqhome-jin/azeroth-survivors-re-cn),
is a fork of [OutThisLife/azeroth-survivors-re](https://github.com/OutThisLife/azeroth-survivors-re)
for local development and future extensions, with Simplified Chinese as the default documentation language.
This branch adds zhCN localization: Chinese character/battleground selection, the Survivors combat HUD and upgrade cards by default, plus locale-aware DBC and FrameXML text. Some custom or legacy hard-coded strings may still appear in English; changing the UI locale also requires matching client data.

> **Original Game and Credits:** The original Azeroth Survivors is by **[Aethus](https://wow.aethus.pl/)**.
> Play the online version at [the original website](https://wow.aethus.pl/), read the
> [announcement](https://www.reddit.com/r/wowservers/comments/1x0yidl/azeroth_survivors_roguelike_wow_singleplayer_game/),
> or join [the original game's Discord](https://discord.gg/UzxaZuSAk).
> This repository is maintained independently; it should not be treated as the online version's complete source or a feature-synchronized release.

Pick a class and a battleground, then survive: move with WASD, cast abilities automatically,
collect experience, choose upgrades, and face timed bosses.
Gameplay needs no game server, login, or account. The browser build still needs a local HTTP server
to serve the page and game assets; the initial build downloads dependencies and game data.

## Contents

- [Current Features](#current-features)
- [Quick Start](#quick-start)
- [Controls](#controls)
- [Client Locale](#client-locale)
- [Game Data](#game-data)
- [Architecture and Development Docs](#architecture-and-development-docs)
- [Documentation Languages](#documentation-languages)
- [License and Disclaimer](#license-and-disclaimer)

## Current Features

<table>
  <tr>
    <td width="50%"><img src="docs/screenshots/menu.jpg" alt="Character and battleground selection"></td>
    <td width="50%"><img src="docs/screenshots/combat.jpg" alt="A Mage fighting in Elwynn Forest"></td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/screenshots/level-up.jpg" alt="Ability upgrade cards at level-up"></td>
    <td width="50%"><img src="docs/screenshots/results.jpg" alt="Results after a run"></td>
  </tr>
</table>

- **Character and battleground selection:** eight races, both genders, valid race/class combinations,
  and sixteen battlegrounds, with your last selection remembered.
- **Local survival combat:** automatic abilities, enemy waves, experience pickups, and timed bosses,
  with themed enemy and boss rosters for each battleground.
- **Level-up cards:** ability and passive upgrades, rarity colors, before/after values, and deal-in and hover animations.
- **Class mobility:** unlocked at level 5 and activated with F toward the cursor, including Blink, Charge, Sprint, and more.
- **Camera controls:** locked top-down by default; C unlocks rotation and tilt, and the wheel zooms out to 70 yards.
- **Post-processing:** G toggles sharpening and color adjustment.
- **Chinese localization:** zhCN is the default for Survivors UI and client-provided localized text; some custom strings may remain in English.

## Quick Start

### Requirements

- Install Rust through [rustup](https://rustup.rs/); [rust-toolchain.toml](rust-toolchain.toml) pins the project version.
- `make`, `curl`, `unzip`, and Bash. macOS also needs Xcode Command Line Tools.
- A browser with **WebGPU** in a secure context (`localhost` or HTTPS). WebGL2 does not support full world rendering.
- On Windows, use Git Bash. These Bash scripts cannot be run directly in PowerShell or cmd.

If Rust was just installed and `rustup: command not found` appears, reopen the terminal or run this in Bash / Zsh:

```bash
source "$HOME/.cargo/env"
```

### Browser Build

```bash
git clone https://github.com/sqhome-jin/azeroth-survivors-re-cn.git
cd azeroth-survivors-re-cn
make build
make start
```

Open <http://127.0.0.1:8090/>. The initial game-data download is about 5.5 GB,
in addition to build dependencies and tools.
The browser build defaults to zhCN; append `?locale=enUS` to use the English UI. Provide client data for the matching locale as well.

`make build` runs these repeatable steps in order, skipping the relevant downloads or installations when already complete:

| Step | Purpose |
| --- | --- |
| [scripts/fetch-data.sh](scripts/fetch-data.sh) | Download the WoW 1.12.1 client and extract its MPQs |
| [scripts/web-setup.sh](scripts/web-setup.sh) | Set up the WASM target, wasm-bindgen, WASI SDK, and Binaryen |
| [scripts/web-build.sh](scripts/web-build.sh) | Build the browser client into `web/dist/` |

### Desktop Build

```bash
make native
```

If you already have game data, point directly at your own Data directory:

```bash
WOW_DATA=/path/to/WoW/Data cargo run --release -p warcraft-survivors
```

Run `make help` for all targets. **Use `make clean` carefully: it deletes downloaded game data and browser build output.**

## Controls

| Key | Action |
| --- | --- |
| WASD | Move |
| Space | Jump |
| F | Class mobility, unlocked at level 5 |
| Mouse wheel | Zoom |
| C | Toggle camera lock |
| Esc | Pause |
| 1 / 2 / 3 | Choose a level-up card |
| G | Toggle post-processing |

See [CONTROLS.md](CONTROLS.md) for the full English reference.

## Client Locale

The client defaults to `zhCN`. Select a browser locale with `?locale=zhCN` or `?locale=enUS`; on desktop, set `WOW_LOCALE=zhCN` or `WOW_LOCALE=enUS`. The locale selects which strings the UI reads; it does not convert client assets. Match `WOW_LOCALE` to the client data passed through `WOW_DATA` or `--data`. Individual DBC strings missing in the selected locale fall back to enUS.

## Game Data

The repository includes no Blizzard game asset files: models, textures, sounds, maps,
MPQ contents, and FrameXML are not bundled. Runtime requires a WoW 1.12.1 client's `Data` directory.

The default script downloads the **Simplified Chinese WoW 1.12.1 (build 5875, zhCN)** client from
[archive.org](https://archive.org/details/WoW_1.12.1-5875) and extracts MPQs into `wow-data/Data/`.
That directory is Git-ignored; downloads are resumable and skipped when data is already present.
You are responsible for ensuring you have permission to obtain and use game data from any source.

### Use Existing Data

For desktop, set `WOW_DATA`. For the browser build, pass `--data` directly to the host:

```bash
cargo run --release -p wenilla-host -- --www web/dist --data /path/to/WoW/Data
```

The local host's asset endpoints have no login protection. **Do not expose them directly to the public internet.**

### Change the Download Source

```bash
make build WOW_DATA_URL=https://your-mirror.example/client.zip
```

Alternatively, edit `DEFAULT_DATA_URL` in [scripts/fetch-data.sh](scripts/fetch-data.sh).
The URL must point directly to a ZIP containing the client's `Data/*.MPQ` files.

## Architecture and Development Docs

Gameplay lives primarily in [crates/benilla-app/src/survivors](crates/benilla-app/src/survivors).
Desktop and browser launchers share the same Rust logic, built on two upstream projects:

- [benilla](https://github.com/samwhosung/benilla): a from-scratch WoW 1.12.1 client.
- [wenilla](https://github.com/Arnesen/wenilla): the browser WASM port and associated host / realm services.

| Document | Contents |
| --- | --- |
| [Chinese Developer Guide](docs/PROJECT-GUIDE.zh-CN.md) | Architecture, modules, runtime flow, extension points, and verification guidance |
| [AGENTS.md](AGENTS.md) | Repository working conventions |
| [docs/METHOD.md](docs/METHOD.md) | Engine development methods and constraints |
| [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) | Upstream engine contribution guidelines |
| [docs/MAP.md](docs/MAP.md) | Generated engine code index |
| [web/README.md](web/README.md) | Browser integration and JavaScript bridge documentation |

Some inherited documents describe the original client or other upstream components rather than this fork's
feature set. Read them alongside current source and the Chinese guide.

## Documentation Languages

- [README.md](README.md): Simplified Chinese, the default project entry point.
- [README.en.md](README.en.md): English, with matching sections and core information.
- Keep both READMEs in sync when changing installation commands, feature scope, or important notices.
  Link additional translations from the top of each version.

## License and Disclaimer

The original Azeroth Survivors is by **Aethus**. This project is not affiliated with or endorsed by Blizzard Entertainment.
No Blizzard game asset files are bundled. Screenshots show artwork rendered from external client data;
that artwork belongs to Blizzard Entertainment. World of Warcraft is a trademark of Blizzard Entertainment, Inc.

The engine code is available under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option.
Vendored components in [third_party](third_party) retain their upstream licenses.
The code licenses do not grant redistribution rights for Blizzard assets.