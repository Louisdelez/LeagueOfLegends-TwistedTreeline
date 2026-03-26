# Shadow Grove — Project Inventory

## Sources (cloned repos)

| Repo | Path | Purpose | Status |
|------|------|---------|--------|
| LS4-3x3 | `sources/LS4-3x3/` | TT-specific server fork (C#, .NET Core 3.0) | 105+ champions, TT map script, incomplete Vilemaw AI |
| GameServer | `sources/GameServer/` | LeagueSandbox main (archived) | Reference architecture, 647 C# files |
| Fishbones | `sources/Fishbones/` | P2P launcher (TypeScript/Bun) | Active, supports TT (Map4/10), Godot 4.6 UI |
| LeagueEmulatorJS | `sources/LeagueEmulatorJS/` | JS-based emulator | Active development |
| Legends | `sources/Legends/` | LoL 4.20 server prototype (C#) | Reference only |
| My-work-on-League-Sandbox | `sources/My-work-on-League-Sandbox/` | Champion scripts fork | Active, March 2026 |

## Tools (cloned repos)

| Tool | Path | Purpose |
|------|------|---------|
| Obsidian | `tools/Obsidian/` | WAD archive explorer/extractor (C#) |
| CDTB | `tools/CDTB/` | CDN scraper + WAD extractor (Python) |
| lol2gltf | `tools/lol2gltf/` | Convert .mapgeo to glTF (Rust) |
| LoL-MAPGEO-Converter | `tools/LoL-MAPGEO-Converter/` | Convert .mapgeo to OBJ |
| LoL-NGRID-converter | `tools/LoL-NGRID-converter/` | Navmesh visualizer |
| ritobin | `tools/ritobin/` | .bin to human-readable text |
| wadtools | `tools/wadtools/` | WAD CLI (Rust) |
| Ripple | `tools/Ripple/` | Map editor (C#) |
| LeagueToolkit | `tools/LeagueToolkit/` | C# lib for LoL file formats |
| cdragon-data | `tools/cdragon-data/` | Hash tables for WAD path resolution |
| awesome-league | `tools/awesome-league/` | Curated modding tool list |
| moba-threejs | `tools/moba-threejs/` | Browser TT recreation (Three.js) |

## Assets (downloaded from CommunityDragon patch 9.22)

| Asset | Path | Size |
|-------|------|------|
| map10.bin.json | `assets/tt-reference/map-config/` | 48 KB |
| map10.bin | `assets/tt-reference/map-config/` | 37 KB |
| 2dlevelminimap.png | `assets/tt-reference/minimap/` | 284 KB |
| navmeshmask.png | `assets/tt-reference/navmesh/` | 50 KB |
| 78 map textures | `assets/tt-reference/textures/` | ~26 MB |

## Project (Rust/Bevy)

| Crate | Path | Description |
|-------|------|-------------|
| sg-core | `project/crates/sg-core/` | Types, components, constants (TT data) |
| sg-protocol | `project/crates/sg-protocol/` | Network messages |
| sg-gameplay | `project/crates/sg-gameplay/` | Combat, economy, leveling, items, abilities, buffs |
| sg-map | `project/crates/sg-map/` | Map layout, spawn scheduler |
| sg-ai | `project/crates/sg-ai/` | Minion, turret, jungle AI |
| sg-server | `project/crates/sg-server/` | Authoritative server binary |
| sg-client | `project/crates/sg-client/` | Client with rendering |

## Next Steps
1. Extract exact positions from LS4-3x3 map scripts into sg-map layout
2. Download patch 9.22 WAD and extract Map10 geometry/navmesh
3. Wire up Lightyear networking between server and client
4. Implement first playable: 1 champion moving on the map with turrets
