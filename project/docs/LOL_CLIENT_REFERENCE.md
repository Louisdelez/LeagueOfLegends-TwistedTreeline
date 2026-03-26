# League of Legends Client Reference (Patch 9.22 Era, November 2019)

Complete breakdown of every screen, feature, and flow in the LoL client. This document serves as the authoritative reference for recreating the client UI as a fullscreen in-game interface in Bevy/Rust.

---

## Visual Design Language

### Color Palette
| Name | Hex | Usage |
|------|-----|-------|
| Dark Navy | `#0A1428` | Primary background, deep panels |
| Dark Blue-Gray | `#010A13` | Darkest backgrounds, overlays |
| Blue-Gray | `#1E2328` | Secondary panels, card backgrounds |
| Medium Blue-Gray | `#3C3C41` | Borders, dividers, inactive elements |
| Gold (Primary) | `#C89B3C` | Primary accent, headings, borders, selected states |
| Gold (Light) | `#F0E6D2` | Light text, highlights, hover states |
| Gold (Bright) | `#C8AA6E` | Button borders, active elements |
| Blue (Accent) | `#0397AB` | Links, info highlights, secondary accent |
| Blue (Light) | `#5B5A56` | Disabled text, subtle elements |
| Text White | `#A09B8C` | Body text on dark backgrounds |
| Text Bright | `#F0E6D2` | Headers, important text |
| Red (Error/Enemy) | `#E84057` | Errors, enemy team color, danger |
| Green (Ally) | `#0ACF83` | Ally team color, success states |
| Orange (Warning) | `#E89C23` | Warnings, LP changes |

### Typography
- Primary font: Beaufort for LoL (custom serif) for headings and display text
- Secondary font: Spiegel (custom sans-serif) for body text and UI elements
- Font weights: Light, Regular, Medium, Bold
- Text is generally light-on-dark throughout

### Visual Style
- Dark theme throughout, evoking a magical/fantasy atmosphere
- Gold metallic accents on borders, frames, and interactive elements
- Subtle gradient backgrounds (dark navy to slightly lighter)
- Frosted glass / translucent panels in some overlays
- Ornate corner decorations on major panels (inspired by Runeterra aesthetics)
- Button styles: gold-bordered with dark fill, hover brightens the gold, click depresses
- Scrollbars: thin, gold-tinted, minimal
- Dividers: thin gold or gray horizontal lines
- Icons: flat style with gold/white on dark backgrounds

---

## 1. Login Screen

### Layout
- Full-screen animated background (champion-themed or event-themed artwork)
- In patch 9.22 era: Worlds 2019 themed (still image with music theme, as animated login screens were reduced after patch 9.10)
- Central login form with username and password fields
- "Remember Me" checkbox
- "Sign In" button (gold accent)
- "Forgot Password?" link below
- Server/region selector in corner
- Riot Games logo and legal text at bottom

### Interactions
- Type credentials, press Enter or click Sign In
- Region can be changed before login
- After login: client loads, transitions to Home screen
- Patching progress bar shown if game update is needed

---

## 2. Main Client Shell (Persistent Elements)

### Top Navigation Bar
Permanently visible across all screens (except champion select). Left-to-right:

| Element | Position | Description |
|---------|----------|-------------|
| Riot/LoL Logo | Far left | Small logo, clicking returns to Home |
| **PLAY** button | Left side | Large, prominent, golden button to open queue selector |
| Home tab | After Play | Default landing page |
| TFT tab | After Home | Teamfight Tactics content (added mid-2019) |
| Clash tab | After TFT | Tournament system |
| Profile icon + name | Center-right area | Summoner icon, name, level; click to open Profile |
| Collection tab | Right area | Inventory management |
| Loot/Crafting icon | Right area | Hexagonal chest icon |
| Store icon | Right area | Coin stack icon |
| Currency display | Far right | Blue Essence (BE) amount + Riot Points (RP) amount |
| Settings cog | Far right corner | Opens client settings |
| Alert bell | Near settings | Notifications |

### Social Panel (Right Side)
A collapsible sidebar on the right edge of the client:
- **Friends list** at top, organized by groups
- Online/offline/in-game status indicators (green dot = online, yellow = away, gray = offline)
- Search bar to find friends by summoner name
- Friend groups (collapsible): "General", "Close Friends", custom groups
- Each friend entry shows: summoner icon, name, status message, current activity (e.g., "In Game - Summoner's Rift")
- **Clubs** section (below friends): player-created social groups with club tags (2-5 character tags appended to summoner name)
- **Chat window**: clicking a friend opens a detachable chat window at bottom
- **Voice chat** icon for party voice communication

### Bottom Bar (contextual)
- Mission/quest tracker (expandable panel from bottom-left)
- Bug report button

---

## 3. Home / Landing Screen

### Layout
The default screen after login. Full width of the viewing window.

**Hero Banner (Top)**
- Large rotating carousel of featured content
- Event announcements, new champion releases, skin lines
- Clickable banners leading to store, event pages, or patch notes
- Navigation dots at bottom of carousel

**News Grid (Below Banner)**
- 2-3 columns of news cards
- Each card: thumbnail image, title, brief description, date
- Content types: patch notes, champion spotlights, esports, community content
- Cards link to leagueoflegends.com articles

**Sidebar / Additional Panels**
- Current sales promotion
- "Your Shop" notification (periodic personalized discounts)
- Featured game mode callout (e.g., URF, One for All when active)
- Social media links (Twitter, YouTube, etc.)
- Latest patch notes quick link

### Data Shown
- Breaking news and current sales
- Latest patch notes
- Featured game modes currently active
- Event progress (if an event is running)
- Esports schedule/results

---

## 4. Profile Screen

### Access
Click summoner icon/name in the top navigation bar.

### Layout

**Background**
- Full-width champion splash art as background (defaults to most-played champion, customizable via owned skins)
- Semi-transparent overlay for readability

**Left Banner (Player Identity)**
- Summoner icon (circular, with tier border)
- Summoner name (large text)
- Optional title
- Summoner level (number in circular badge)
- Previous season rank flair (small icon/emblem)

**Bottom Section**
- **Ranked Armor**: ornate 3D-rendered emblem showing current rank tier, centered
- **Honor Level**: small icon (left side), levels 0-5, 3 checkpoint pips; hidden from other players viewing your profile
- **Mastery Score**: total champion mastery points, plus icon of most-mastered champion
- **Clash trophy/banner** (if applicable)

**Additional Info Below Banner**
- Active XP boosts indicator
- ARAM rerolls available count
- Hextech chest availability (how many chests can still be earned)

### Tabs

**Overview Tab**
- Summary view with the above elements
- Quick ranked stats snapshot

**Match History Tab**
- List of last 20 played matches
- Each match entry shows:
  - Game mode (Ranked Solo, Normal, ARAM, etc.)
  - Win/Loss indicator (blue = win, red = loss)
  - Champion played (icon)
  - KDA (Kills/Deaths/Assists)
  - CS (creep score)
  - Game duration
  - Time ago played
  - Replay download button (if available)
- **Top stats panel**: 3 most-played champions, playstyle breakdown (damage type distribution)
- Clicking a match expands to show:
  - Overview: gold graph (team gold over time), champion/objective kill timeline
  - Scoreboard: all 10 players, items built, KDA, CS, gold, damage
  - Stats: detailed damage dealt/taken, vision score, CC score
  - Graph: horizontal bar graphs comparing all players
  - Runes: rune pages used and their effectiveness stats

**Ranked Tab**
- Available after level 30
- Current rank emblem and tier (e.g., Gold II)
- LP amount and progress bar to next division
- Win/Loss record for the season
- Ranked queue selector (Solo/Duo vs Flex)
- Promotion series tracker (if in promos: shows Bo3 or Bo5 progress with checkmarks/X marks)

### Interactions
- Click settings cog to change profile background
- Click ranked armor to view ranked details
- Hover over honor level to see checkpoint progress
- Click match entries to expand details
- Download replays from match history

---

## 5. Collection Screen

### Access
"Collection" tab in top navigation bar.

### Sub-tabs (horizontal bar below main nav)
Champions | Skins | Emotes | Runes | Spells | Items | Icons | Wards | Chromas

### Champions Tab
**Layout:**
- Search bar (top)
- Filter buttons: by role (Top, Jungle, Mid, Bot, Support), by ownership (Owned/Not Owned/All)
- Sort options: Alphabetical, Mastery Rank, Date Acquired, Chest Available
- Grid of champion cards (4-5 columns)
  - Each card: champion square portrait, name below
  - Mastery badge overlay (if mastery 4+)
  - Chest icon (if chest available to earn with this champion)
  - Blue tint/lock overlay if not owned

**Clicking a champion opens:**
- Large splash art display
- Champion name, title, role tags
- Lore snippet
- Abilities preview (passive + Q/W/E/R)
- Skins owned for that champion
- Mastery level and points
- "Available Skins" section
- Purchase button (BE or RP) if not owned

### Skins Tab
**Layout:**
- Filter: by champion, by skin line/set, by tier, by ownership
- Sort: alphabetical, release date, rarity, price
- Grid of skin splash art cards
- Each card: skin name, champion name, owned indicator
- Clicking a skin shows: full splash art, purchase date (if owned), short bio, chromas available, related skins in same set

### Runes Tab
**Layout:**
- Left panel: list of saved rune pages (2 free, can buy more with BE/RP)
  - Each page: name, primary path icon, keystone icon
  - "+" button to create new page
  - 5 default preset pages (one per path)
- Right panel: rune page editor (see Section 11 for full rune system details)

### Spells Tab
- Grid of all summoner spells with icons
- Each spell shows: name, icon, cooldown, effect description, unlock level requirement
- Video/animation preview on hover or click
- Game mode availability noted (e.g., "Summoner's Rift only", "Howling Abyss only")

### Emotes Tab
- Grid of owned emotes
- Emote wheel customization (5 slots for quick-use in-game)
- Passive emote slots: Start of Game, First Blood, Ace, Victory, Defeat
- Thumbs-up emote is default/always available

### Items Tab
- Custom item set builder
- Create item sets for specific champions and maps
- Drag-and-drop item icons into build paths
- Sets appear in the in-game shop

### Icons Tab
- Grid of all summoner icons
- Hover shows: name, tier, release year, acquisition date, how it was obtained
- Click to equip as profile icon
- Filter: owned/all, by event, by type

### Wards Tab
- Grid of ward skin cosmetics
- Preview animation on hover
- Click to equip

### Chromas Tab
- Grid of chroma color variants for skins
- Filter by champion, skin, ownership
- Shows base skin + available color swaps

---

## 6. Store

### Access
Store icon (coin stack) in top navigation bar.

### Layout

**Top Bar (Store-specific)**
- Store sub-tabs: Featured | Champions | Skins | Chromas | Loot | Emotes | Accessories
- Three extra buttons: Purchase RP | Gifting Center | Account

**Featured Tab (Landing)**
- Large hero banner with featured content (new skin line, event, etc.)
- "Top Sellers" row
- "On Sale" section with countdown timer
- New releases section

**Champions Tab**
- All champions listed in grid
- Sub-section: Champion Bundles
- Filter by: role (Assassin, Mage, Marksman, Fighter, Tank, Support), price
- Sort by: name, price, release date
- Each champion card: portrait, name, price in BE and RP
- "Owned" badge if already purchased
- Price tiers: 450 BE, 1350 BE, 3150 BE, 4800 BE, 6300 BE (RP equivalents available)

**Skins Tab**
- All purchasable skins in grid
- Sub-sections: Skins | Chromas | Bundles
- Filter by: champion, release date, rarity, price
- Skin price tiers (RP): 520, 750, 975, 1350, 1820 (Legendary), 3250 (Ultimate)
- Sale items marked with red slash-through original price + discounted price

**Loot Tab**
- Hextech Chests (125 RP each)
- Hextech Keys (125 RP each)
- Masterwork Chests
- Event-specific loot items
- Little Legends (TFT)
- Event token bundles

**Accessories Tab**
- Sub-sections: Ward Skins | Summoner Icons | XP Boosts | Rune Pages | Bundles
- Rune pages: 590 RP each or 6300 BE

### Currency System

**Riot Points (RP)** - Premium currency
- Purchased with real money
- Used for: skins, champions, chromas, ward skins, emotes, boosts, rune pages, bundles
- Display: gold coin icon with amount

**Blue Essence (BE)** - Free currency
- Earned from: leveling up (champion shards disenchanted), missions, events
- Used for: champions, rune pages, mastery upgrades, name changes
- Display: blue hexagonal icon with amount

### Additional Store Features
- **Purchase RP**: opens payment window (credit card, PayPal, prepaid cards)
- **Gifting Center**: send items to friends (must be friends for 1+ day)
- **Account**: server transfers, summoner name changes, purchase history, code redemption
- **Sales**: weekly rotation every Monday at 12pm PT, 5 champions + 15 skins discounted
- **Refunds**: 3 refund tokens (1 renewed annually), within 90 days of purchase
- **Bundle system**: grouped items at reduced price vs. individual purchase

---

## 7. Loot / Hextech Crafting

### Access
Hexagonal chest icon in top navigation bar, or "Crafting" tab.

### Layout

**Top Section**
- Available chests count + keys count prominently displayed
- "Open" button (glows when a chest+key combo is available)

**Main Grid**
- All loot items displayed in a grid
- Filter tabs: All | Champions | Skins | Ward Skins | Summoner Icons | Materials

**Right Panel (Context Actions)**
- When an item is selected, shows available actions on the right

### Loot Types

**Materials:**
- Hextech Chest (requires key to open)
- Hextech Key (crafted from 3 key fragments)
- Key Fragments (earned from honor, playing games)
- Masterwork Chest (higher skin shard chance)
- Event Capsules (event-specific)
- Gemstones (rare drops, 10 = Hextech skin)

**Shards:**
- Champion Shards (blue border)
- Skin Shards (orange border)
- Ward Skin Shards
- Summoner Icon Shards
- Emote Shards

### Actions on Shards

**Champion Shards:**
- **Disenchant**: convert to Blue Essence (40% of champion's BE cost)
- **Upgrade to Permanent**: spend BE (60% of champion's BE cost) to unlock champion
- **Reroll**: combine 3 champion shards into 1 random permanent champion

**Skin Shards:**
- **Disenchant**: convert to Orange Essence (20% of skin's RP price equivalent in OE)
- **Upgrade to Permanent**: spend Orange Essence to unlock skin permanently
- **Reroll**: combine 3 skin shards/permanents into 1 random permanent skin
- **Purchase Champion**: if champion not owned, option to open collection to buy

### Essence Types
| Currency | Icon | Source | Used For |
|----------|------|--------|----------|
| Blue Essence (BE) | Blue hexagon | Disenchanting champion shards, leveling | Champions, rune pages, mastery |
| Orange Essence (OE) | Orange hexagon | Disenchanting skin shards | Upgrading skin shards to permanents |
| Gemstones | Red gem | Rare chest drops | Hextech exclusive skins (10 gems) |

### Chest Opening Animation
- Chest appears center screen
- Key inserts and turns
- Chest glows and opens with particle effects
- Loot items fly out and land, revealing rarity with colored glow borders
- Orange = skin shard, Blue = champion shard, etc.

---

## 8. Play / Lobby / Queue Selection

### Access
Click the large "PLAY" button in the top navigation bar.

### Queue Selection Modal
A large overlay/modal appears with game mode options.

**Left Side - Mode Categories:**
- PvP
- Co-op vs. AI
- Training (Practice Tool)
- Custom

**Center - Queue List:**

| Queue | Map | Players | Description |
|-------|-----|---------|-------------|
| Normal (Blind Pick) | Summoner's Rift (5v5) | 10 | Simultaneous champion selection, no bans |
| Normal (Draft Pick) | Summoner's Rift (5v5) | 10 | Ban phase + turn-based picks, role selection |
| Ranked Solo/Duo | Summoner's Rift (5v5) | 10 | Competitive, draft pick, 1 or 2 players |
| Ranked Flex | Summoner's Rift (5v5) | 10 | Competitive, any party size |
| ARAM | Howling Abyss | 10 | All Random All Mid, single lane |
| Twisted Treeline (Blind) | Twisted Treeline (3v3) | 6 | 3v3, blind pick |
| Twisted Treeline (Draft) | Twisted Treeline (3v3) | 6 | 3v3, draft pick |
| Ranked 3v3 | Twisted Treeline (3v3) | 6 | 3v3 competitive |
| TFT | Convergence | 8 | Auto-battler mode |
| Featured Game Mode | Varies | Varies | Rotating: URF, One for All, Nexus Blitz, etc. |

**For Twisted Treeline specifically:**
- Available until November 19, 2019 (removed at end of Season 9)
- 3v3 format, two lanes
- Both Blind Pick and Draft Pick queues
- Ranked 3v3 queue with separate ladder

### Lobby Screen (After Queue Selection)

**Layout:**
- Map artwork in background (showing selected map)
- **Party slots**: 2-5 circles (depends on mode) for party members
  - Your summoner icon in first slot
  - Empty slots show "+" to invite friends
  - Filled slots show friend's summoner icon + name
- **Role selection** (for draft modes only): two dropdown selectors for Primary and Secondary role (Top, Jungle, Mid, Bot, Support, Fill)
- **Estimated wait time** display
- **"Find Match" / "Start" button** (large, gold, bottom-center)
- **Invite friends** button/panel on the right
- **Chat** for lobby party
- **Settings** for queue (e.g., map selection if multiple options)

### Queue Pop (Match Found)
- Audio chime + visual notification (screen border flashes)
- Central modal: "MATCH FOUND" with Accept/Decline buttons
- Timer bar (10 seconds to accept)
- Shows how many players have accepted (e.g., "4/10 Accepted")
- If all accept: transitions to Champion Select
- If anyone declines: returns to queue with priority

---

## 9. Champion Select — Draft Pick (Ranked / Normal Draft)

### Overview
The most complex UI screen. Used for Ranked Solo/Duo, Ranked Flex, Normal Draft, and Ranked 3v3 (on TT). Full screen takeover (hides normal navigation).

### Layout

**Top Bar**
- Game mode name (e.g., "RANKED SOLO/DUO")
- Timer countdown (large, center-top)
- Phase indicator text (e.g., "BAN PHASE", "YOUR TURN TO PICK")

**Left Side - Blue Team (Ally)**
- 5 player slots (3 for TT) stacked vertically
- Each slot shows:
  - Assigned role icon (Top/Jungle/Mid/Bot/Support)
  - Summoner name
  - Champion portrait (circular) - shows intent/locked pick
  - Ban intent (small square portrait next to main)
  - Border glow when it's that player's turn
  - Skin selection indicator (after pick)

**Right Side - Red Team (Enemy)**
- 5 player slots (3 for TT) stacked vertically
- Similar layout but mirrored
- Intent picks hidden (shown as "?" during planning)
- Locked picks revealed when confirmed

**Center - Champion Grid**
- Searchable grid of all champions
- Search bar at top of grid
- Filter buttons: role filters (Assassin, Fighter, Mage, Marksman, Support, Tank)
- Champions displayed as square portraits in a scrollable grid
- Unavailable champions (banned, picked, not owned) are grayed out / darkened
- Hovering shows champion name tooltip

**Bottom Panel**
- **Lock In** button (large, center, gold) - confirms selection
- **Summoner Spells**: two spell slots (clickable to change)
- **Rune Page**: dropdown/selector to choose or edit rune page
- **Edit Runes** button: opens inline rune editor
- **Skin selector**: horizontal strip of owned skins for selected champion (appears after picking)
- **Chat**: team chat window (bottom-left)

### Draft Flow (5v5, e.g., Ranked Solo/Duo)

**Phase 1 - Ban Intent (Planning) [Varies]**
- All players can hover over champions to show intent
- Team can see each other's intent picks

**Phase 2 - Ban Phase 1 [40s per ban]**
1. Blue team players 1-3 ban simultaneously
2. Red team players 1-3 ban simultaneously
3. All bans revealed at end of phase
- 6 bans total in first round

**Phase 3 - Pick Phase 1 [40s per pick]**
1. Blue 1st pick (1 champion)
2. Red 1st + 2nd pick (2 champions)
3. Blue 2nd + 3rd pick (2 champions)

**Phase 4 - Ban Phase 2 [40s per ban]**
4. Red team players 4-5 ban
5. Blue team players 4-5 ban
- 4 more bans (10 total)

**Phase 5 - Pick Phase 2 [40s per pick]**
6. Red 3rd + 4th pick (2 champions)
7. Blue 4th + 5th pick (2 champions)
8. Red 5th pick (1 champion)

**Phase 6 - Finalization [40s]**
- All players can:
  - Select skin
  - Change summoner spells
  - Edit rune page
  - Request/accept champion trades with teammates
  - Swap pick order (before finalization)

### Draft Flow (3v3 Twisted Treeline)
- Same structure but with 3 players per team
- 6 total bans (3 per team)
- Pick order: Blue 1, Red 1-2, Blue 2-3, Red 3
- No role assignment system (roles called in chat or implicit)

### Visual States
- **Declaring intent**: champion portrait appears translucent in your slot
- **Locked in**: portrait becomes solid, "LOCKED IN" text flash, sound effect
- **Banned**: champion portrait appears with red X overlay in ban bar
- **Your turn**: slot border glows bright gold, timer text pulses
- **Not your turn**: dimmed interface, grid still browsable

### Trading
- After all picks, players can request to trade champions
- Click on ally's portrait to send trade request
- Both players must own each other's champions
- Accept/Decline buttons appear
- Champions swap between the two players

---

## 10. Champion Select — Blind Pick

### Layout
Same base layout as Draft Pick but simplified:
- No ban phase
- No pick order (everyone picks simultaneously)
- No role assignment dropdowns (roles communicated via chat)
- Center champion grid same as draft

### Phases

**Selection Phase [87s]**
- All 5 (or 3 for TT) players simultaneously browse and select champions
- Each player's pick is visible to allies only
- Only one of each champion per team (duplicate blocked with error)
- Players "call" roles in chat (e.g., "mid" "top" "jg")
- Call order traditionally respected (first come, first served)
- Lock In button to confirm selection

**Finalization Phase [10s]**
- Brief grace period
- Select skin
- Confirm summoner spells and runes
- No trading (since there's no need — everyone picked their own)

### Key Differences from Draft
- No bans at all
- Simultaneous selection (no turn order)
- Both teams can pick the same champion (mirror matches possible)
- Faster overall (87s + 10s vs. several minutes for full draft)
- Chat-based role calling instead of queue role selection

---

## 11. Runes System (Runes Reforged, Patch 9.22)

### Overview
Runes Reforged replaced the old Runes + Masteries system in Pre-Season 2018. Each player creates a rune page with:
- 1 **Primary Path** (provides keystone + 3 minor runes)
- 1 **Secondary Path** (provides 2 minor runes from a different path)
- 3 **Stat Shards** (small stat bonuses, independent of paths)

### Rune Page Editor UI

**Left column**: list of saved rune pages with names
**Main area**:
- Top: 5 path icons in a row (Precision, Domination, Sorcery, Resolve, Inspiration)
- Click one to set as **Primary Path** (highlighted with path color)
- Below: Keystone row (3-4 options, pick 1)
- Below keystone: Slot 1 row (3 options, pick 1)
- Below: Slot 2 row (3 options, pick 1)
- Below: Slot 3 row (3 options, pick 1)
- **Secondary Path** selector: remaining 4 path icons, click one
  - Shows 3 rows of minor runes from that path
  - Pick 2 total from any of the 3 rows (but only 1 per row)
- **Stat Shards** section at bottom: 3 rows of 3 small circles each

### Path Colors
| Path | Color | Theme |
|------|-------|-------|
| Precision | Gold/Yellow | `#C8AA6E` |
| Domination | Red | `#E84057` |
| Sorcery | Purple/Blue | `#9B59B6` |
| Resolve | Green | `#2ECC71` |
| Inspiration | Light Blue | `#49B4BB` |

### Complete Rune List (Patch 9.22)

NOTE: Patch 9.22 was the last patch before the preseason 10 rune changes in 9.23. Kleptomancy was still in 9.22; it was replaced by Prototype: Omnistone in 9.23.

#### Precision (Improved attacks and sustained damage)
**Keystones:**
- **Press the Attack**: 3 consecutive attacks on a champion deal bonus damage and make them vulnerable
- **Lethal Tempo**: After damaging a champion, gain attack speed boost (exceeding cap) after 1.5s delay
- **Fleet Footwork**: Energized attacks heal and grant movement speed
- **Conqueror**: Stacking adaptive force on champion combat, at max stacks heals for portion of damage dealt

**Slot 1:** Overheal | Triumph | Presence of Mind
**Slot 2:** Legend: Alacrity | Legend: Tenacity | Legend: Bloodline
**Slot 3:** Coup de Grace | Cut Down | Last Stand

#### Domination (Burst damage and target access)
**Keystones:**
- **Electrocute**: 3 separate attacks/abilities within 3s deal bonus adaptive damage
- **Predator**: Enchant boots for active: channel to gain massive movement speed toward champions
- **Dark Harvest**: Damaging low-health champions deals bonus adaptive damage and harvests a soul (stacking)
- **Hail of Blades**: First 3 attacks against champions gain massive attack speed (exceeding cap)

**Slot 1:** Cheap Shot | Taste of Blood | Sudden Impact
**Slot 2:** Zombie Ward | Ghost Poro | Eyeball Collection
**Slot 3:** Ravenous Hunter | Ingenious Hunter | Relentless Hunter | Ultimate Hunter

#### Sorcery (Empowered abilities and resource manipulation)
**Keystones:**
- **Summon Aery**: Damaging champions sends Aery to deal damage; shielding allies sends Aery to shield them
- **Arcane Comet**: Damaging a champion with an ability hurls a comet at their location
- **Phase Rush**: 3 attacks/abilities against a champion grants a burst of movement speed

**Slot 1:** Nullifying Orb | Manaflow Band | Nimbus Cloak
**Slot 2:** Transcendence | Celerity | Absolute Focus
**Slot 3:** Scorch | Waterwalking | Gathering Storm

#### Resolve (Durability and crowd control)
**Keystones:**
- **Grasp of the Undying**: Every 4s in combat, next attack on champion deals bonus magic damage, heals you, permanently increases max HP
- **Aftershock**: After immobilizing a champion, gain bonus armor/MR, then explode for damage
- **Guardian**: Guard nearby allies; when you or a guarded ally take damage, both gain a shield + movement speed

**Slot 1:** Demolish | Font of Life | Shield Bash
**Slot 2:** Conditioning | Second Wind | Bone Plating
**Slot 3:** Overgrowth | Revitalize | Unflinching

#### Inspiration (Creative tools and rule bending)
**Keystones:**
- **Glacial Augment**: Basic attacks slow champions; active item slows create a freeze ray zone
- **Kleptomancy**: After using an ability, next 2 attacks on champions grant random consumables/gold (PATCH 9.22 ONLY, removed in 9.23)
- **Unsealed Spellbook**: Swap summoner spells during the game (with cooldown)

**Slot 1:** Hextech Flashtraption | Magical Footwear | Perfect Timing
**Slot 2:** Future's Market | Minion Dematerializer | Biscuit Delivery
**Slot 3:** Cosmic Insight | Approach Velocity | Time Warp Tonic

### Stat Shards (3 independent slots)

**Slot 1 (Offense):**
- +10% Attack Speed
- +9 Adaptive Force (5.4 AD or 9 AP)
- +1-10% CDR (based on level)

**Slot 2 (Flex):**
- +9 Adaptive Force (5.4 AD or 9 AP)
- +6 Armor
- +8 Magic Resist

**Slot 3 (Defense):**
- +15-90 HP (based on level)
- +6 Armor
- +8 Magic Resist

### Twisted Treeline Rune Overrides
Per the game design document:
- Waterwalking replaced by Scorch
- Zombie Ward replaced by Eyeball Collection
- Ghost Poro replaced by Eyeball Collection

---

## 12. Summoner Spells (Patch 9.22)

### Available on Summoner's Rift (5v5) and Twisted Treeline (3v3)

| Spell | Cooldown | Unlock Level | Effect |
|-------|----------|-------------|--------|
| **Flash** | 300s | 7 | Blink 400 units in target direction |
| **Ignite** | 180s | 9 | Deal 80-505 true damage over 5s, apply Grievous Wounds |
| **Heal** | 240s | 1 | Restore 90-345 HP to self + nearest ally, +30% MS for 1s |
| **Barrier** | 180s | 4 | Shield for 115-455 for 2.5s |
| **Exhaust** | 210s | 4 | Slow target 30%, reduce damage dealt by 40% for 2.5s |
| **Ghost** | 180s | 1 | +28-45% MS for 10s (ramps up over 2s) |
| **Cleanse** | 210s | 9 | Remove all CC and summoner spell debuffs, +65% tenacity for 3s |
| **Teleport** | 360s | 7 | Channel 4.5s to teleport to allied turret/minion/ward |
| **Smite** | 90s | 9 | Deal 390-1000 true damage to monster/minion (upgradeable) |

### ARAM Only (Howling Abyss)

| Spell | Cooldown | Effect |
|-------|----------|--------|
| **Mark / Dash** | 80s | Throw snowball; if hit, reactivate to dash to target |
| **Clarity** | 240s | Restore 50% max mana to self + nearby allies |

### Spell Selection UI
- Two spell slots in champion select (bottom panel)
- Click a slot to open spell picker grid
- Shows all available spells for the current map/mode
- Selected spells have gold border
- Locked/unavailable spells are grayed out with lock icon and level requirement
- Spells persist between games (remembered per queue type)

---

## 13. Ranked / Competitive System (Season 9, 2019)

### Tier Structure

| Tier | Divisions | Color/Theme | Visual |
|------|-----------|-------------|--------|
| **Iron** | IV, III, II, I | Dark gray/rust | Rusted metal emblem |
| **Bronze** | IV, III, II, I | Bronze/brown | Bronze metallic emblem |
| **Silver** | IV, III, II, I | Silver/white | Silver metallic emblem |
| **Gold** | IV, III, II, I | Gold/yellow | Gold metallic emblem |
| **Platinum** | IV, III, II, I | Teal/green | Platinum gem emblem |
| **Diamond** | IV, III, II, I | Blue/crystal | Diamond crystal emblem |
| **Master** | None (LP ladder) | Purple | Ornate purple emblem |
| **Grandmaster** | None (LP ladder) | Red/dark | Dark crimson emblem |
| **Challenger** | None (LP ladder) | Gold/elite | Most ornate emblem |

Note: Season 9 (2019) introduced Iron and Grandmaster tiers, and reduced divisions from 5 to 4.

### League Points (LP)
- Earn LP for wins, lose LP for losses
- Amount varies based on hidden MMR (typically 15-25 LP)
- 100 LP triggers promotion series
- 0 LP + loss can trigger demotion

### Promotion Series
- **Division promo**: Best-of-3 (need 2 wins)
- **Tier promo**: Best-of-5 (need 3 wins)
- Failed promo: LP resets based on net gains during series
- **Promo Helper**: Below Gold, get free win in re-attempt after failed promo

### Demotion
- At 0 LP, losing drops you to previous division
- Demotion immunity: several games of protection after promoting
- Tier demotion (e.g., Gold to Silver) requires significantly poor performance
- Higher tiers have shorter immunity windows

### Placement Games
- 5 provisional games for new accounts or new season
- Ranked armor/regalia hidden during placements
- Provisional results determine initial placement

### Ranked Queues
| Queue | Party Size | Requirements |
|-------|-----------|--------------|
| Ranked Solo/Duo | 1-2 players | Level 30+, 20+ champions owned |
| Ranked Flex 5v5 | 1, 2, 3, or 5 players (not 4) | Level 30+, 20+ champions owned |
| Ranked 3v3 | 1-3 players | Level 30+, champions owned |

### Ranked Armor (Profile Visual)
- Ornate armor/crest displayed on profile
- Changes based on current tier
- Upgrades split-by-split throughout the season (split points add wings/details)
- Visible to other players viewing your profile

### Season Rewards (Season 9)
- **Profile Icon**: tier-specific icon
- **Loading Screen Border**: tier-specific border (visible to all in loading screen)
- **Victorious Skin**: Victorious Aatrox (Gold+ required)
- **Honor requirement**: Must be Honor level 2+ to receive rewards
- **Ranked armor/banner** retained on profile showing peak rank

---

## 14. Social / Friends System

### Friends List (Right Sidebar)

**Layout:**
- Collapsible right-side panel, always accessible
- Header: "Friends" with online count (e.g., "Friends (12/47)")
- Search bar at top
- **Friend groups** (collapsible sections):
  - "General" (default group)
  - "Close Friends"
  - Custom user-created groups
  - "Offline" (at bottom, collapsed by default)

**Per-Friend Entry:**
- Small summoner icon (left)
- Summoner name
- Status indicator dot: green (online), yellow (away), gray (offline), in-game icon
- Status message / current activity text (e.g., "In Queue", "In Game - Summoner's Rift (23:45)", "In Champion Select")
- Right-click context menu: Invite to Game, View Profile, Send Message, Remove Friend, Block, Assign to Group

### Chat
- Click on a friend to open chat window
- Chat window appears at bottom of client
- Can be detached as separate floating window
- Message history preserved within session
- Multiple chat windows can be open simultaneously
- Chat formatting: plain text, no rich formatting in 2019 era

### Party / Invite System
- Invite friends from friends list or lobby
- Party persists across multiple games
- **Open Party**: friends can join without explicit invite
- Voice chat built into party (push-to-talk or open mic)
- Party members visible in lobby with their summoner icons

### Clubs (Existed in 2019, Removed End of Season 10)
- Player-created social groups
- Club name + tag (2-5 characters, appears after summoner name in-game)
- Up to 100 members per club
- Club chat channel
- Club tag visible in loading screen and in-game
- Officers could manage membership

### Status Messages
- Custom status message visible to friends
- Preset statuses: Online, Away, Busy, Appear Offline
- Automatic status updates based on activity (In Queue, In Game, In Champion Select)

---

## 15. End of Game Screen

### Flow
After game ends: Victory/Defeat splash -> Honor Voting -> Stats Screen

### Honor Voting Screen

**Layout:**
- Full-screen overlay before stats
- "HONOR A TEAMMATE" header
- 4 teammate portraits displayed (3 in TT mode)
- 3 honor categories shown as buttons below each teammate:
  - **Stayed Cool** (green leaf icon) — for positive, calm teammates
  - **Great Shotcalling** (crown icon) — for leadership and strategy
  - **GG <3** (heart icon) — general commendation
- Timer: 60 seconds to vote (can skip)
- Select one category for one teammate, or skip
- If a player receives 3+ honors: announced to entire lobby

### Post-Game Stats Screen

**Layout:**
Dark themed panel with multiple tabs.

**Top Bar:**
- Game result: "VICTORY" (blue) or "DEFEAT" (red)
- Game mode and duration
- "Play Again" button (returns to lobby with same party)
- "Skip Waiting for Stats" / "Continue" button
- Honor indicators: if you were honored, small pips/icons shown

**Tabs:**

**Overview Tab:**
- Two graphs:
  - **Gold over time**: line graph showing team gold accumulation
  - **Objective/Kill timeline**: horizontal timeline with champion kill and objective icons
- Quick summary of game stats

**Scoreboard Tab:**
- Table with all 10 (or 6) players
- Columns: Champion icon, Summoner Name, KDA, CS, Gold Earned, Item Build (6 item icons + trinket), Damage Dealt, Wards Placed
- Your team on top, enemy team on bottom
- MVP-style highlight on best performing player (if applicable)

**Stats Tab:**
- Detailed statistics per player:
  - Total Damage Dealt to Champions
  - Physical / Magic / True damage breakdown
  - Total Damage Taken
  - Healing Done
  - Gold Earned
  - CS (Minions + Monsters)
  - Vision Score
  - Wards Placed / Destroyed
  - CC Time Applied
  - Damage to Objectives
  - Damage to Turrets

**Graph Tab:**
- Horizontal bar graphs comparing all players
- Categories selectable: damage dealt, damage taken, gold, healing, etc.

**Runes Tab:**
- Each player's rune page displayed
- Stats showing rune effectiveness (e.g., "Conqueror: dealt 3,450 bonus damage, healed 1,200")

### Rewards Display (Bottom)
- XP gained bar (showing progress to next level)
- Blue Essence earned (from champion shards at level-up)
- Mission progress updates
- First Win of the Day bonus indicator
- LP change (for ranked): "+18 LP" or "-15 LP" with rank badge
- Promotion/demotion notification if applicable
- Chest earned indicator (if S-rank on a chest-eligible champion)
- Key fragment drop notification

---

## 16. Loading Screen

### Layout
Full-screen display while the game loads. Two teams arranged side by side or top/bottom.

**Team Arrangement:**
- Blue team (left/top): 5 player cards (3 for TT)
- Red team (right/bottom): 5 player cards (3 for TT)

**Per-Player Card:**
- **Champion splash art**: cropped portrait of selected champion/skin
- **Rank border**: ornate frame around splash art, color/style based on current ranked tier (Iron = rusty, Gold = golden, Diamond = crystalline, etc.)
  - No border if unranked or below Silver
- **Summoner name**: below the splash art
- **Summoner spells**: two small circular icons below name
- **Keystone rune**: primary keystone icon displayed
- **Secondary rune path**: small circle icon showing secondary tree
- **Honor flair**: small emblem at top-center of card if honor level 3+
- **Champion mastery badge**: displayed if mastery level 4+ (level 4 = grey, level 5 = red, level 6 = purple, level 7 = blue/green)
- **Loading progress**: percentage number showing individual load progress

**Alternate View (click card to toggle):**
- Champion mastery score and tier
- Summoner level
- Previous season rank

**Center Area:**
- Map name (e.g., "Summoner's Rift", "Twisted Treeline")
- Random gameplay tip displayed in center
- Tips rotate or can be randomized: gameplay tips, lore facts, champion tips

**Visual Style:**
- Dark background
- Cards have metallic borders matching ranked tier
- Subtle particle effects on high-tier borders
- Loading progress shown as percentage on each card

---

## 17. Settings (Client)

### Access
Cog icon in top-right corner of client.

### Client Settings Categories

**General:**
- Language selection
- Client window size (1024x576, 1280x720, 1600x900, 1920x1080)
- Close client during game (minimize to tray / keep open)
- Enable low-spec mode (reduces animations)
- Enable GPU hardware acceleration

**Sound:**
- Master volume slider
- Music volume slider
- SFX volume slider
- Voice volume (for voice chat)
- Ambient sounds toggle
- Champion select music toggle

**Notifications:**
- Friend login notifications
- Party invites
- Game invites
- Gift notifications
- Mission completion alerts

**Chat:**
- Chat bubble display
- Timestamp format
- Group privacy settings

**Blocked Players:**
- List of blocked summoner names
- Unblock option

### In-Game Settings (accessible via Escape during game)

**Hotkeys:**
- Abilities: Q, W, E, R (plus Alt+Q for self-cast, etc.)
- Summoner Spells: D, F
- Items: 1-7
- Camera: Y (lock), Space (center), arrow keys or edge scroll
- Quick Cast options per ability
- Attack Move: A-click or right-click settings
- Smart Cast with indicators toggle

**Video:**
- Resolution
- Window Mode: Fullscreen, Borderless, Windowed
- Graphics Quality: Very Low, Low, Medium, High, Very High
- Individual toggles: Character Quality, Environment Quality, Effects Quality, Shadow Quality
- Frame Rate Cap: 30, 60, 80, 120, 144, 200, Uncapped
- Anti-Aliasing
- VSync
- Colorblind Mode (Red-Green, Blue-Yellow)
- HUD Scale

**Sound (In-Game):**
- Master, Music, Announcer, Voice, SFX, Ambience, Pings volume sliders
- Mute all toggle
- Disable All Sound

**Interface:**
- Minimap scale and position (left/right)
- HUD scale
- Chat scale
- Health bar display options
- Damage numbers toggle
- Show summoner names toggle
- Ability cooldown display format
- Allied/enemy chat toggle
- Timestamps in chat

**Game:**
- Mouse speed
- Camera scroll speed (mouse/keyboard)
- Auto-attack toggle
- Movement prediction (for lag)
- Camera lock mode: Per-Side Offset, Fixed Offset, Semi-Locked, Free

---

## 18. Twisted Treeline Specific Details (For This Project)

### Map Characteristics
- Horizontal layout (unlike vertical Summoner's Rift)
- 2 lanes: top and bottom
- Central jungle with 3 monster camps per side (6 total): Wolf, Wraith, Golem
- 2 neutral altars (capturable objectives)
- Vilemaw pit (center-top, spider boss replacing Baron/Dragon)
- NO wards available (vision limited to champion abilities and trinket sweep)
- Health relics and speed shrines in jungle

### Queue & Lobby Differences from 5v5
- Party size: 1-3 players
- 3 player slots in lobby (vs 5)
- No role selection system (no Top/Jungle/Mid/Bot/Support dropdowns)
- Shorter champion select (fewer players = faster draft)
- Draft pick on TT: 3 bans per team (6 total), pick order adapted for 3 players

### Champion Select on TT
- **Blind Pick**: All 3 players pick simultaneously (same as 5v5 blind but with 3 slots)
- **Draft Pick**: Ban phase (3 bans per team), then pick phase for 3 per team
  - Pick order: B1, R1-R2, B2-B3, R3
- **Ranked 3v3**: Same as draft pick with LP/ranked implications

### Exclusive Items (Not on Summoner's Rift)
- Lord Van Damm's Pillager
- Moonflair Spellblade
- Wooglet's Witchcap (replaces Rabadon's + Zhonya's)
- Arcane Sweeper (trinket, replaces ward trinkets)
- Timeworn support item variants
- Various SR items removed/replaced for balance

### Visual Differences
- Darker, more gothic atmosphere than Summoner's Rift
- Shadow Isles aesthetic: gnarled trees, ghostly fog, dark stone
- Purple/green color accents vs SR's blue/green
- Ambient effects: falling leaves, fog, spectral wisps

---

## Summary of All Client Screens (Navigation Map)

```
Login Screen
    |
    v
Main Client Shell (persistent top nav + social sidebar)
    |
    +-- Home (default landing)
    +-- Play -> Queue Selection -> Lobby -> [Queue Pop] -> Champion Select -> Loading Screen -> [IN GAME] -> End of Game (Honor -> Stats)
    +-- Profile
    |     +-- Overview
    |     +-- Match History
    |     +-- Ranked
    +-- Collection
    |     +-- Champions
    |     +-- Skins
    |     +-- Emotes
    |     +-- Runes
    |     +-- Spells
    |     +-- Items
    |     +-- Icons
    |     +-- Wards
    |     +-- Chromas
    +-- Loot / Hextech Crafting
    +-- Store
    |     +-- Featured
    |     +-- Champions
    |     +-- Skins / Chromas
    |     +-- Loot
    |     +-- Emotes
    |     +-- Accessories
    +-- Settings
    +-- Social (sidebar)
          +-- Friends List
          +-- Chat
          +-- Clubs
          +-- Voice Chat
```

---

## Sources

- [League of Legends Wiki - Client](https://wiki.leagueoflegends.com/en-us/Client)
- [League of Legends Wiki - Collection](https://wiki.leagueoflegends.com/en-us/Collection)
- [League of Legends Wiki - Summoner Profile](https://wiki.leagueoflegends.com/en-us/Summoner_profile)
- [League of Legends Wiki - Draft Pick](https://wiki.leagueoflegends.com/en-us/Draft_Pick)
- [League of Legends Wiki - Blind Pick](https://wiki.leagueoflegends.com/en-us/Blind_Pick)
- [League of Legends Wiki - Rank](https://wiki.leagueoflegends.com/en-us/Rank)
- [League of Legends Wiki - Rune](https://wiki.leagueoflegends.com/en-us/Rune)
- [League of Legends Wiki - Summoner Spell](https://wiki.leagueoflegends.com/en-us/Summoner_spell)
- [League of Legends Wiki - Honor](https://wiki.leagueoflegends.com/en-us/Honor)
- [League of Legends Wiki - Loading Screen](https://wiki.leagueoflegends.com/en-us/Loading_Screen)
- [League of Legends Wiki - Hextech Crafting](https://wiki.leagueoflegends.com/en-us/Hextech_Crafting)
- [League of Legends Wiki - Riot Store](https://wiki.leagueoflegends.com/en-us/Riot_Store)
- [League of Legends Wiki - Settings](https://wiki.leagueoflegends.com/en-us/Settings)
- [League of Legends Wiki - Twisted Treeline](https://wiki.leagueoflegends.com/en-us/Twisted_Treeline)
- [League of Legends Wiki - V9.22 Patch Notes](https://wiki.leagueoflegends.com/en-us/V9.22)
- [Riot Games - /dev diary: New Tiers & Placements in Ranked 2019](https://nexus.leagueoflegends.com/en-us/2018/09/dev-diary-new-tiers-placements-in-ranked-2019/)
- [The UI of League of Legend's Client - Medium](https://medium.com/@1537148253135/the-ui-of-league-of-legends-client-d6d8b947365a)
- [League of Legends Client UX Analysis - Medium](https://medium.com/@jiahao1604/league-of-legends-ui-ux-heuristic-analysis-review-44e677147b24)
- [Riot Games - Architecture of the League Client Update](https://technology.riotgames.com/news/architecture-league-client-update)
- [League of Legends Color Codes](https://brandpalettes.com/league-of-legends-color-codes/)
- [Riot Support - Hextech Crafting FAQ](https://support-leagueoflegends.riotgames.com/hc/en-us/articles/360036422453-Hextech-Crafting-FAQ)
- [Riot Support - Parties and Voice Chat FAQ](https://support-leagueoflegends.riotgames.com/hc/en-us/articles/360000809887-Parties-and-Voice-Chat-FAQ)
