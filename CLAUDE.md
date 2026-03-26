# League of Legends — Twisted Treeline Recreation

## Project Overview
Faithful recreation of the removed League of Legends Twisted Treeline (3v3) game mode as a standalone game built with Rust and Bevy 0.18.

## Architecture
Modular crate structure under `project/crates/`:
- **sg-core** — Shared types, components, constants, runes, summoner spells
- **sg-gameplay** — Combat math, economy, leveling, items, abilities, buffs, champion definitions
- **sg-ai** — Minion, turret, and jungle camp AI (targeting priorities)
- **sg-map** — Map layout with exact TT coordinates, lane waypoints, spawn positions
- **sg-protocol** — Network protocol messages (client ↔ server)
- **sg-server** — Authoritative UDP game server (`cargo run --bin sg-server`)
- **sg-client** — Bevy app with 15 plugins: map, camera, input, movement, spawn, combat, ability, objectives, fog, shop, audio, hud, debug, net, menu (`cargo run --bin sg-client`)

## Build & Run
```bash
cd project
cargo build --bin sg-client --bin sg-server
# Assets symlink required:
ln -sf $(pwd)/assets $(pwd)/target/debug/assets
# Run server then client:
./target/debug/sg-server &
./target/debug/sg-client
```
System deps: `libwayland-client`, `libudev`, `libasound` (symlinks may be needed on Debian 13).

## Key Conventions
- All game balance data comes from real LoL TT sources in `sources/LS4-3x3/`
- Menu UI uses real LoL launcher assets (Beaufort/Spiegel fonts, backgrounds, icons) from `assets/launcher-assets/`
- The menu is integrated in-game (fullscreen Bevy UI), not a separate launcher
- Champion/item/turret stats should match real patch 4.20 TT data
- ECS pattern: data in components (sg-core), logic in systems (sg-client plugins)
- GameSet ordering: Input → AI → Movement → Combat → Spawn

## Reference Data
- Champion stats (459 units): `sources/LS4-3x3/gameserver/Content/LeagueSandbox-Default/Stats/`
- Items (850+): `sources/LS4-3x3/gameserver/Content/LeagueSandbox-Default/Items/`
- Map config: `assets/tt-reference/map-config/map10.bin.json`
- Bounty/gold: `sources/LS4-3x3/gameserver/Content/LeagueSandbox-Default/Globals/bounty.json`
- XP curve: `sources/LS4-3x3/.../Maps/Map10/ExpCurve.json`

## Current State
- ✅ 3D TT map loaded (59MB GLB), correct positions for turrets/inhibs/nexuses
- ✅ Minion waves (2 lanes), basic combat, 3 champion archetypes, abilities (Q/W/E/R)
- ✅ Vilemaw boss, altars, health relics, speed shrine
- ✅ Gold/XP system, fog of war, shop (10 items), camera controls
- ✅ Menu system styled like real LoL client
- ❌ Missing: jungle camps, bot AI, CC application, item stat bonuses, win conditions, bounty system, real champion roster, animations, audio
