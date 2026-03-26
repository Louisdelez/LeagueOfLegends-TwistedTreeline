# League of Legends — Twisted Treeline 3v3

Open source recreation of the **Twisted Treeline** (3v3) game mode, removed from League of Legends in November 2019. Built from scratch in **Rust** with the **Bevy 0.18** game engine.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.94-orange.svg)
![Bevy](https://img.shields.io/badge/bevy-0.18-green.svg)

## Screenshots

*Coming soon*

## Features

### Map & Environment
- Full 3D Twisted Treeline map from patch 9.22 NVR data
- 78 original textures (structures, trees, walls, mushrooms, spider dens)
- Correct positions for all turrets, inhibitors, nexuses, altars
- Speed shrine, health relics, Vilemaw spawn point
- 2 lanes (top/bottom) with full minion waypoint paths

### Gameplay
- **15 playable champions** with real stats from patch 4.20 data: Annie, Garen, Ashe, Darius, Lux, Thresh, Jinx, Yasuo, Master Yi, Jax, Teemo, Singed, Tryndamere, Mordekaiser, Poppy
- **45 items** with real stats and prices (Doran's, boots, core AD/AP/Tank items)
- **4 abilities per champion** (Q/W/E/R) — skillshots, AOE zones, dashes, shields
- **9 summoner spells** — Flash, Ignite, Heal, Barrier, Exhaust, Ghost, Cleanse, Teleport, Smite
- **5 rune paths** with 15 keystones

### Combat
- Auto-attack system with damage calculation (armor/MR penetration)
- CC system: Stun, Slow, Root, Silence (blocks movement/attacks/abilities)
- Bounty/kill gold with real LoL formula (+16.5%/kill streak, -20%/death streak, first blood +100g)
- Shared XP within 1250 range
- Level 1-18 with XP thresholds
- Death timers scaling with level and game time

### AI & Objectives
- **5 bot AI** (2 allies + 3 enemies) with lane/fight/retreat behavior
- **6 jungle camps** (Golems, Wolves, Wraiths) — spawn at 1:05, respawn every 75s
- **Vilemaw** epic boss — spawns at 10:00, grants team buff
- **2 altars** — 9s capture, +10% MS / +1% HP restore buffs
- **Turret AI** with aggro swap when ally attacked
- **Inhibitor destruction** spawns super minions, respawns after 5 minutes
- **Win condition** — nexus destruction triggers Victory/Defeat screen

### Client UI (styled like real LoL client)
- Home screen with featured content, friends list, navigation
- Lobby with queue selection
- Champion select with timer, team slots, spell selection
- Loading screen with player cards
- Post-game screen with real KDA/CS/Gold stats
- Profile, Collection, Settings screens
- Real LoL fonts (Beaufort for LoL, Spiegel)

### Networking
- UDP game server (authoritative)
- Client-server protocol
- Single-player with bots (default)

## Architecture

```
project/crates/
  sg-core/        — Shared types, components, constants, runes, spells
  sg-gameplay/    — Combat math, economy, leveling, items, abilities, champions
  sg-ai/          — Bot AI, minion AI, turret AI, jungle AI
  sg-map/         — Map layout, lane waypoints, spawn positions
  sg-protocol/    — Network protocol (client <-> server)
  sg-server/      — Authoritative UDP game server
  sg-client/      — Bevy app with 15 plugins (map, camera, combat, abilities, etc.)
```

## Build & Run

### Prerequisites
- Rust 1.94+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- System libs (Debian/Ubuntu): `libwayland-dev libudev-dev libasound2-dev`

### Build
```bash
cd project
cargo build --bin sg-client --bin sg-server
ln -sf $(pwd)/assets $(pwd)/target/debug/assets
```

### Run
```bash
# Start server
./target/debug/sg-server &

# Start client
./target/debug/sg-client
```

### Controls
| Key | Action |
|-----|--------|
| Right click | Move |
| A | Ability Q |
| W | Ability W |
| E | Ability E |
| R | Ability R (Ultimate) |
| P | Toggle shop |
| B | Recall (8s channel) |
| Mouse wheel | Zoom in/out |
| Screen edges | Pan camera |

## Data Sources

Game data is extracted from real League of Legends files for faithful recreation:

- **Champion stats** — [LS4-3x3](https://github.com/) patch 4.20 server data (459 champion stat files)
- **Item stats** — LS4-3x3 item database (850+ items)
- **Map geometry** — NVR format from patch 9.22 (last patch with Twisted Treeline)
- **Map textures** — 78 DDS textures extracted from Map10 WAD
- **Bounty formula** — `bounty.json` from LS4-3x3 globals
- **XP curve** — `ExpCurve.json` from Map10 data
- **UI assets** — Extracted from official LoL client WAD files (fonts, icons, backgrounds)

## Roadmap

- [ ] Native NVR/DDS loader in Rust (replace GLB conversion)
- [ ] Full terrain rendering with heightmap
- [ ] More champions (goal: 30+)
- [ ] Complete item build paths
- [ ] Minimap overlay
- [ ] Multiplayer (client-server networking)
- [ ] Spectator mode
- [ ] Champion animations
- [ ] Particle effects
- [ ] Sound effects for abilities/combat

## Contributing

Contributions welcome! This is a community project to bring back the Twisted Treeline.

Areas that need help:
- **3D artists** — Champion models, terrain, props
- **Gameplay programmers** — Champion abilities, item effects
- **Network engineers** — Multiplayer implementation
- **Sound designers** — Combat sounds, ambient music

## Disclaimer

This project is a fan-made recreation for educational purposes. It is not affiliated with or endorsed by Riot Games. League of Legends and Twisted Treeline are trademarks of Riot Games, Inc. Game assets used are extracted from publicly available game files for interoperability purposes.

## License

[MIT](LICENSE)
