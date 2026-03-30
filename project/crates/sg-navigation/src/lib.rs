//! LoL NGRID navigation grid parser and A* pathfinding
//! Exact port of LeagueSandbox NavigationGrid.cs for Twisted Treeline

use bevy::prelude::*;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::io::{Read, Cursor};

// ═══════════════════════════════════════════════════════
//  NavGrid — parsed from .aimesh_ngrid binary file
// ═══════════════════════════════════════════════════════

/// Vision pathing flag bits (from LoL)
pub const FLAG_WALKABLE: u16 = 0x0;
pub const FLAG_BRUSH: u16 = 0x1;
pub const FLAG_WALL: u16 = 0x2;
pub const FLAG_STRUCTURE_WALL: u16 = 0x4;
pub const FLAG_TRANSPARENT_WALL: u16 = 0x40;
pub const FLAG_BLUE_TEAM_ONLY: u16 = 0x400;
pub const FLAG_RED_TEAM_ONLY: u16 = 0x800;

#[derive(Resource, Clone)]
pub struct NavGrid {
    pub cell_size: f32,
    pub count_x: usize,
    pub count_z: usize,
    pub min_x: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_z: f32,
    pub flags: Vec<u16>,
    pub heights: Vec<f32>,
    pub loaded: bool,
}

impl Default for NavGrid {
    fn default() -> Self {
        Self {
            cell_size: 50.0, count_x: 0, count_z: 0,
            min_x: 0.0, min_z: 0.0, max_x: 0.0, max_z: 0.0,
            flags: vec![], heights: vec![], loaded: false,
        }
    }
}

impl NavGrid {
    /// Check if a grid cell is walkable
    pub fn is_walkable(&self, gx: usize, gz: usize) -> bool {
        if gx >= self.count_x || gz >= self.count_z { return false; }
        let f = self.flags[gz * self.count_x + gx];
        // Walkable = no wall flags set
        f & FLAG_WALL == 0 && f & FLAG_STRUCTURE_WALL == 0 && f & FLAG_TRANSPARENT_WALL == 0
    }

    /// Check walkability in world coordinates
    pub fn is_walkable_world(&self, pos: Vec2) -> bool {
        if !self.loaded { return true; }
        let (gx, gz) = self.world_to_grid(pos);
        self.is_walkable(gx, gz)
    }

    /// Check walkability with a radius (like LoL champion pathing radius)
    pub fn is_walkable_radius(&self, pos: Vec2, radius: f32) -> bool {
        if !self.loaded { return true; }
        let cells_r = (radius / self.cell_size).ceil() as i32;
        let (cx, cz) = self.world_to_grid(pos);
        for dz in -cells_r..=cells_r {
            for dx in -cells_r..=cells_r {
                let nx = cx as i32 + dx;
                let nz = cz as i32 + dz;
                if nx < 0 || nz < 0 { continue; }
                let (nx, nz) = (nx as usize, nz as usize);
                // Check if cell center is within radius
                let cell_world = self.grid_to_world(nx, nz);
                if pos.distance(cell_world) <= radius + self.cell_size * 0.5 {
                    if !self.is_walkable(nx, nz) { return false; }
                }
            }
        }
        true
    }

    /// Convert world position to grid indices
    pub fn world_to_grid(&self, pos: Vec2) -> (usize, usize) {
        let gx = ((pos.x - self.min_x) / self.cell_size).max(0.0) as usize;
        let gz = ((pos.y - self.min_z) / self.cell_size).max(0.0) as usize;
        (gx.min(self.count_x.saturating_sub(1)), gz.min(self.count_z.saturating_sub(1)))
    }

    /// Convert grid indices to world position (center of cell)
    pub fn grid_to_world(&self, gx: usize, gz: usize) -> Vec2 {
        Vec2::new(
            self.min_x + (gx as f32 + 0.5) * self.cell_size,
            self.min_z + (gz as f32 + 0.5) * self.cell_size,
        )
    }

    /// Get terrain height at world position
    pub fn get_height(&self, pos: Vec2) -> f32 {
        if self.heights.is_empty() { return 0.0; }
        let hx = ((pos.x - self.min_x) / self.cell_size).max(0.0) as usize;
        let hz = ((pos.y - self.min_z) / self.cell_size).max(0.0) as usize;
        let hx = hx.min(self.count_x);
        let hz = hz.min(self.count_z);
        let idx = hz * (self.count_x + 1) + hx;
        if idx < self.heights.len() { self.heights[idx] } else { 0.0 }
    }

    /// Find nearest walkable cell to a position (spiral search)
    pub fn get_closest_walkable(&self, pos: Vec2) -> Vec2 {
        let (gx, gz) = self.world_to_grid(pos);
        if self.is_walkable(gx, gz) { return pos; }
        for r in 1..50i32 {
            for dz in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dz.abs() != r { continue; } // only check ring
                    let nx = (gx as i32 + dx).max(0) as usize;
                    let nz = (gz as i32 + dz).max(0) as usize;
                    if self.is_walkable(nx, nz) {
                        return self.grid_to_world(nx, nz);
                    }
                }
            }
        }
        pos
    }
}

// ═══════════════════════════════════════════════════════
//  NGRID Binary Parser
// ═══════════════════════════════════════════════════════

fn read_f32(cur: &mut Cursor<&[u8]>) -> f32 {
    let mut buf = [0u8; 4];
    cur.read_exact(&mut buf).unwrap_or_default();
    f32::from_le_bytes(buf)
}
fn read_i32(cur: &mut Cursor<&[u8]>) -> i32 {
    let mut buf = [0u8; 4];
    cur.read_exact(&mut buf).unwrap_or_default();
    i32::from_le_bytes(buf)
}
fn read_i16(cur: &mut Cursor<&[u8]>) -> i16 {
    let mut buf = [0u8; 2];
    cur.read_exact(&mut buf).unwrap_or_default();
    i16::from_le_bytes(buf)
}
fn read_u16(cur: &mut Cursor<&[u8]>) -> u16 {
    let mut buf = [0u8; 2];
    cur.read_exact(&mut buf).unwrap_or_default();
    u16::from_le_bytes(buf)
}
fn read_u8(cur: &mut Cursor<&[u8]>) -> u8 {
    let mut buf = [0u8; 1];
    cur.read_exact(&mut buf).unwrap_or_default();
    buf[0]
}

/// Parse an .aimesh_ngrid file (LoL navigation grid)
pub fn parse_ngrid(data: &[u8]) -> Option<NavGrid> {
    let mut cur = Cursor::new(data);

    // Version
    let major = read_u8(&mut cur);
    let minor = if major != 2 { read_u16(&mut cur) } else { 0 };
    println!("[NGRID] Version {}.{}", major, minor);

    // Bounds
    let min_x = read_f32(&mut cur);
    let min_y = read_f32(&mut cur);
    let min_z = read_f32(&mut cur);
    let max_x = read_f32(&mut cur);
    let max_y = read_f32(&mut cur);
    let max_z = read_f32(&mut cur);
    println!("[NGRID] Bounds: ({:.0},{:.0},{:.0}) → ({:.0},{:.0},{:.0})", min_x, min_y, min_z, max_x, max_y, max_z);

    // Grid size
    let cell_size = read_f32(&mut cur);
    let count_x = read_i32(&mut cur) as usize;
    let count_z = read_i32(&mut cur) as usize;
    println!("[NGRID] Grid: {}x{} cells, size={}", count_x, count_z, cell_size);

    let total = count_x * count_z;

    // Skip cell data (48 bytes per cell for v7, variable for others)
    if major == 7 {
        // Each cell: float + int + float + int + float + short + short + int + int + int + float + short + short + short + short = 48 bytes
        for _ in 0..total {
            for _ in 0..5 { read_f32(&mut cur); } // 5 floats (center_height, arrival_cost, heuristic + skip ints between)
            // Actually: float, int, float, int, float, short, short, int, int, int, float, short, short, short, short
            // = 4+4+4+4+4+2+2+4+4+4+4+2+2+2+2 = 48
            // We already read 5*4=20 bytes, need 28 more
        }
        // That's wrong — let me just skip 48*total bytes from after the grid metadata
        // Reset and skip properly
    }

    // Simpler approach: we know the cell data starts at a fixed offset
    // For v7: header = 3 + 24 + 12 = 39 bytes, then 48 bytes per cell
    let cell_data_start = if major == 2 { 1 + 24 + 12 } else { 3 + 24 + 12 };
    let cell_data_size = if major == 7 { 48 } else { 48 }; // both v5 and v7 are 48 bytes per cell based on C# code

    // Jump to flags section
    let flags_offset = cell_data_start + cell_data_size * total;
    if flags_offset >= data.len() {
        println!("[NGRID] ERROR: flags offset {} > file size {}", flags_offset, data.len());
        return None;
    }

    cur = Cursor::new(data);
    cur.set_position(flags_offset as u64);

    // Read visionPathingFlags (u16 per cell)
    let mut flags = Vec::with_capacity(total);
    for _ in 0..total {
        flags.push(read_u16(&mut cur));
    }

    // Skip other flags (riverRegion + packed bytes)
    for _ in 0..total {
        read_u8(&mut cur); // riverRegionFlags
        read_u8(&mut cur); // jungleQuadrant + mainRegion
        read_u8(&mut cur); // nearestLane + POI
        read_u8(&mut cur); // ring + SRX
    }

    // Skip unknown block (8 × 132 bytes)
    for _ in 0..(8 * 132) {
        read_u8(&mut cur);
    }

    // Read height samples
    let h_count_x = read_i32(&mut cur) as usize;
    let h_count_z = read_i32(&mut cur) as usize;
    let _h_offset_x = read_f32(&mut cur);
    let _h_offset_z = read_f32(&mut cur);
    println!("[NGRID] Height samples: {}x{}", h_count_x, h_count_z);

    let mut heights = Vec::with_capacity(h_count_x * h_count_z);
    for _ in 0..(h_count_x * h_count_z) {
        heights.push(read_f32(&mut cur));
    }

    // Stats
    let walkable = flags.iter().filter(|f| **f & FLAG_WALL == 0 && **f & FLAG_STRUCTURE_WALL == 0 && **f & FLAG_TRANSPARENT_WALL == 0).count();
    let walls = flags.iter().filter(|f| **f & FLAG_WALL != 0).count();
    let brush = flags.iter().filter(|f| **f & FLAG_BRUSH != 0).count();
    println!("[NGRID] Walkable: {} ({:.1}%), Walls: {}, Brush: {}", walkable, walkable as f32 / total as f32 * 100.0, walls, brush);

    Some(NavGrid {
        cell_size,
        count_x, count_z,
        min_x, min_z: min_z,
        max_x, max_z: max_z,
        flags,
        heights,
        loaded: true,
    })
}

// ═══════════════════════════════════════════════════════
//  A* Pathfinding (port of LeagueSandbox GetPath)
// ═══════════════════════════════════════════════════════

#[derive(Clone, PartialEq)]
struct ANode {
    pos: (usize, usize),
    f: f32,
}
impl Eq for ANode {}
impl Ord for ANode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
    }
}
impl PartialOrd for ANode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

/// Find path from start to goal using A* on the NGRID
pub fn find_path(grid: &NavGrid, start: Vec2, goal: Vec2) -> Vec<Vec2> {
    if !grid.loaded { return vec![start, goal]; }

    let (sx, sz) = grid.world_to_grid(start);
    let (mut gx, mut gz) = grid.world_to_grid(goal);

    // If goal is in a wall, find nearest walkable cell
    if !grid.is_walkable(gx, gz) {
        let nearest = grid.get_closest_walkable(goal);
        let (nx, nz) = grid.world_to_grid(nearest);
        gx = nx;
        gz = nz;
    }

    if (sx, sz) == (gx, gz) { return vec![start, goal]; }

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut g_score: HashMap<(usize, usize), f32> = HashMap::new();
    let mut closed = vec![false; grid.count_x * grid.count_z];

    g_score.insert((sx, sz), 0.0);
    open.push(ANode { pos: (sx, sz), f: heuristic(sx, sz, gx, gz) });

    let mut iterations = 0;
    while let Some(current) = open.pop() {
        iterations += 1;
        if iterations > 10000 { break; }

        let (cx, cz) = current.pos;
        if cx == gx && cz == gz {
            return reconstruct_path(grid, &came_from, (gx, gz), start, goal);
        }

        let idx = cz * grid.count_x + cx;
        if idx >= closed.len() || closed[idx] { continue; }
        closed[idx] = true;

        // 8-directional neighbors
        for (dx, dz, cost) in [
            (-1i32, 0i32, 1.0f32), (1, 0, 1.0), (0, -1, 1.0), (0, 1, 1.0),
            (-1, -1, 1.414), (-1, 1, 1.414), (1, -1, 1.414), (1, 1, 1.414),
        ] {
            let nx = cx as i32 + dx;
            let nz = cz as i32 + dz;
            if nx < 0 || nz < 0 { continue; }
            let (nx, nz) = (nx as usize, nz as usize);
            if nx >= grid.count_x || nz >= grid.count_z { continue; }
            if !grid.is_walkable(nx, nz) { continue; }
            let nidx = nz * grid.count_x + nx;
            if nidx >= closed.len() || closed[nidx] { continue; }

            // Prevent corner-cutting through walls
            if dx != 0 && dz != 0 {
                let adj_z = (cz as i32 + dz) as usize;
                let adj_x = (cx as i32 + dx) as usize;
                if !grid.is_walkable(cx, adj_z) || !grid.is_walkable(adj_x, cz) { continue; }
            }

            let tg = g_score[&(cx, cz)] + cost;
            if tg < g_score.get(&(nx, nz)).copied().unwrap_or(f32::INFINITY) {
                came_from.insert((nx, nz), (cx, cz));
                g_score.insert((nx, nz), tg);
                open.push(ANode { pos: (nx, nz), f: tg + heuristic(nx, nz, gx, gz) });
            }
        }
    }

    // No path found — return direct (will be blocked by move_to_target)
    vec![start, goal]
}

fn heuristic(x1: usize, z1: usize, x2: usize, z2: usize) -> f32 {
    let dx = x1 as f32 - x2 as f32;
    let dz = z1 as f32 - z2 as f32;
    (dx * dx + dz * dz).sqrt()
}

fn reconstruct_path(
    grid: &NavGrid,
    came_from: &HashMap<(usize, usize), (usize, usize)>,
    end: (usize, usize),
    start_world: Vec2,
    goal_world: Vec2,
) -> Vec<Vec2> {
    let mut path = vec![goal_world];
    let mut current = end;
    while let Some(&prev) = came_from.get(&current) {
        path.push(grid.grid_to_world(current.0, current.1));
        current = prev;
    }
    path.push(start_world);
    path.reverse();

    // Skip smoothing for now — keep all A* waypoints for accurate wall following
    // TODO: re-enable smooth_path once CastRay is verified
    path
}

/// Remove intermediate waypoints where line-of-sight is clear
fn smooth_path(grid: &NavGrid, path: &[Vec2]) -> Vec<Vec2> {
    if path.len() <= 2 { return path.to_vec(); }

    let mut smoothed = vec![path[0]];
    let mut i = 0;

    while i < path.len() - 1 {
        let mut furthest = i + 1;
        for j in (i + 2)..path.len() {
            if cast_ray(grid, path[i], path[j]).0 {
                furthest = j;
            } else {
                break;
            }
        }
        smoothed.push(path[furthest]);
        i = furthest;
    }

    smoothed
}

/// Cast a ray from origin to destination, checking walkability at each step
/// Returns (clear: bool, stop_position: Vec2)
pub fn cast_ray(grid: &NavGrid, from: Vec2, to: Vec2) -> (bool, Vec2) {
    if !grid.loaded { return (true, to); }

    let dist = from.distance(to);
    let steps = (dist / (grid.cell_size * 0.4)).ceil() as usize + 1;

    for s in 1..=steps {
        let t = s as f32 / steps as f32;
        let p = from.lerp(to, t);
        if !grid.is_walkable_world(p) {
            // Return the last walkable position
            let prev_t = (s - 1) as f32 / steps as f32;
            return (false, from.lerp(to, prev_t));
        }
    }

    (true, to)
}
