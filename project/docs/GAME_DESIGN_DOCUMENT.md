# Shadow Grove — Game Design Document

## Vision
Spiritual successor to League of Legends' Twisted Treeline (3v3 mode).
Phase 1: Faithful recreation using LoL assets for personal testing.
Phase 2: Replace all assets with originals for public/commercial release.
Phase 3: Innovate with new mechanics never seen in LoL 3v3.

---

## Map: The Grove (based on Twisted Treeline)

### Dimensions
- 15398 x 15398 game units
- Horizontal orientation
- 2 lanes (top, bottom) + central jungle

### Structures (per team)
| Type | Count | HP | AD |
|------|-------|-----|-----|
| Outer Turret | 2 (1/lane) | 900 (+250/champ) | 152-180 |
| Inner Turret | 2 (1/lane) | 1100 (+250/champ) | 160-188 |
| Nexus Turret | 1 | 1900 (+250/champ) | 150-250 |
| Inhibitor | 1 | — | — |
| Nexus | 1 | — | — |

### Neutral Objectives
| Objective | Spawn | Respawn | HP | Notes |
|-----------|-------|---------|-----|-------|
| Vilemaw (Spider Boss) | 10:00 | 6:00 | 5500 | Buff: Crest of Crushing Wrath (180s) |
| Wolf Camp (x2) | 1:05 | 75s | — | 3 monsters each |
| Wraith Camp (x2) | 1:05 | 75s | — | 3 monsters each |
| Golem Camp (x2) | 1:05 | 75s | — | 2 monsters each |
| Health Relic | 2:30 | — | — | Heals on pickup |
| Speed Shrine | 2:30 | — | — | Movement speed boost |

### Altars (2)
- Unlock at 2:30
- 9s channel to capture, 90s lockout, 80g per team member
- 1 altar: +10% bonus movement speed
- 2 altars: +1% max HP restored on minion/monster kill

---

## Economy

| Source | Value |
|--------|-------|
| Starting gold | 850g |
| Ambient gold | 0.95g / 5s |
| Melee minion | 20g (+0.125/90s) |
| Caster minion | 17g (+0.125/90s) |
| Siege minion | 45g (+0.35/90s) |
| Champion kill | 300g base |
| First blood bonus | +100g |

## Progression

- 18 levels, XP: 280 (L2) to 18,360 (L18)
- XP range: 1250 units
- Shared XP: 92% (solo) → 22% (5 nearby)
- Kill XP: 35 (L1) → 990 (L18), level diff multiplier: 0.16

## Death Timers
- L1: 13s → L18: 37s
- Scaling after 8:00 (+3% per 30s, cap 1.5x)

---

## Minion Waves
- First spawn: 0:45
- Interval: 45s
- Composition: 3 melee + 3 caster
- Every 3rd wave: +1 siege
- Super minion: replaces siege when enemy inhib destroyed

---

## Vision
- NO purchasable wards
- Only vision tool: trinket (sweep/scan)
- 21 brush zones

---

## Lethality
- 60% base at level 1, scales to 100% at level 18

## Perk Replacements
- Waterwalking → Scorch
- Zombie Ward → Eyeball Collection
- Ghost Poro → Eyeball Collection

---

## Vilemaw Buff: Crest of Crushing Wrath
- Duration: 180s
- Ghosted (ignore unit collision)
- 1-18% bonus damage (scales with level)
- Allied minions: +20 armor/MR, +20% AS, +15 AD, +75 range
- Minions terrorize enemies on first contact
- Lost on death

---

## Technical Stack
- Engine: Bevy 0.18 (Rust)
- Networking: Lightyear (server-authoritative, QUIC)
- Physics: Avian 0.6
- Pathfinding: Oxidized Navigation
- Tick rate: 60 Hz
- Max players: 6 (3v3)

## Surrender
- Available at 15:00

---

## Data Sources
- `map10.bin.json` (CommunityDragon patch 9.22)
- LS4-3x3 source code (map layout, spawn logic)
- LeagueSandbox GameServer (105+ champion scripts, items, buffs)
- League Wiki (structure stats, Vilemaw stats, altar mechanics)
