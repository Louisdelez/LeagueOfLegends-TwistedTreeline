use bevy::prelude::*;
use sg_core::types::*;

/// Twisted Treeline map layout — positions derived from minimap analysis & LS4-3x3 source.
/// All coordinates in game units (map is 15398x15398).

#[derive(Debug, Clone)]
pub struct MapLayout {
    pub blue_spawn: Vec2,
    pub red_spawn: Vec2,
    pub blue_fountain: Vec2,
    pub red_fountain: Vec2,
    pub turrets: Vec<TurretPlacement>,
    pub inhibitors: Vec<InhibitorPlacement>,
    pub nexuses: Vec<NexusPlacement>,
    pub altars: [AltarPlacement; 2],
    pub vilemaw_spawn: Vec2,
    pub jungle_camps: Vec<CampPlacement>,
    pub speed_shrine: Vec2,
    pub health_relics: Vec<Vec2>,
    pub brush_zones: Vec<BrushZone>,
    pub lane_paths: LanePaths,
}

#[derive(Debug, Clone)]
pub struct TurretPlacement {
    pub position: Vec2,
    pub team: Team,
    pub structure_type: StructureType,
    pub lane: Lane,
}

#[derive(Debug, Clone)]
pub struct InhibitorPlacement {
    pub position: Vec2,
    pub team: Team,
}

#[derive(Debug, Clone)]
pub struct NexusPlacement {
    pub position: Vec2,
    pub team: Team,
}

#[derive(Debug, Clone)]
pub struct AltarPlacement {
    pub position: Vec2,
    pub side: AltarSide,
}

#[derive(Debug, Clone)]
pub struct CampPlacement {
    pub position: Vec2,
    pub camp_type: CampType,
    pub team_side: Team, // which side of the map
}

#[derive(Debug, Clone)]
pub struct BrushZone {
    pub center: Vec2,
    pub radius: f32,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct LanePaths {
    pub top_blue: Vec<Vec2>,
    pub top_red: Vec<Vec2>,
    pub bottom_blue: Vec<Vec2>,
    pub bottom_red: Vec<Vec2>,
}

impl MapLayout {
    /// Create the Twisted Treeline layout based on reference data from LS4-3x3
    /// and CommunityDragon map10.bin.json.
    /// These positions will be refined once we extract exact coordinates from the WAD.
    /// Create the Twisted Treeline layout with EXACT positions from LS4-3x3 source.
    /// Coordinates are (X, Z) from the LoL coordinate system where Y is vertical height.
    /// In Bevy 3D: Vec2(x, z) maps to Vec3(x, 0.0, z) on the ground plane.
    pub fn twisted_treeline() -> Self {
        // Full top lane waypoints from LS4-3x3 __NAV_L02..L024 (23 points)
        let top_lane_waypoints: Vec<Vec2> = vec![
            Vec2::new(2626.37, 7951.97),
            Vec2::new(2524.61, 8565.65),
            Vec2::new(2597.15, 9224.34),
            Vec2::new(3097.60, 9806.08),
            Vec2::new(3675.00, 9872.94),
            Vec2::new(4221.84, 9834.41),
            Vec2::new(4822.54, 9485.89),
            Vec2::new(5401.74, 9134.70),
            Vec2::new(6093.57, 8706.35),
            Vec2::new(6592.34, 8475.71),
            Vec2::new(7200.86, 8324.82),
            Vec2::new(7708.82, 8257.57),
            Vec2::new(8216.72, 8325.00),
            Vec2::new(8825.24, 8475.71),
            Vec2::new(9324.01, 8706.35),
            Vec2::new(10015.84, 9134.70),
            Vec2::new(10645.19, 9521.44),
            Vec2::new(11215.67, 9833.19),
            Vec2::new(11775.00, 9875.00),
            Vec2::new(12319.97, 9806.08),
            Vec2::new(12814.12, 9195.29),
            Vec2::new(12887.49, 8539.19),
            Vec2::new(12742.24, 7930.22),
        ];

        // Full bottom lane waypoints from LS4-3x3 __NAV_R02..R017 (16 points)
        let bottom_lane_waypoints: Vec<Vec2> = vec![
            Vec2::new(2623.84, 6656.27),
            Vec2::new(2510.10, 6170.72),
            Vec2::new(2564.54, 5566.21),
            Vec2::new(2727.83, 4992.29),
            Vec2::new(3643.21, 4617.00),
            Vec2::new(4619.85, 4634.72),
            Vec2::new(5661.40, 4845.52),
            Vec2::new(7077.38, 5150.04),
            Vec2::new(8340.20, 5150.04),
            Vec2::new(9756.18, 4845.52),
            Vec2::new(10797.73, 4634.72),
            Vec2::new(11774.36, 4617.00),
            Vec2::new(12554.03, 5014.95),
            Vec2::new(12845.61, 5536.86),
            Vec2::new(12869.02, 6144.04),
            Vec2::new(12752.19, 6633.36),
        ];

        let top_lane_reversed: Vec<Vec2> = top_lane_waypoints.iter().copied().rev().collect();
        let bottom_lane_reversed: Vec<Vec2> = bottom_lane_waypoints.iter().copied().rev().collect();

        Self {
            // Spawn points (from __Spawn_*.sco.json)
            blue_spawn: Vec2::new(1059.62, 7297.66),
            red_spawn: Vec2::new(14321.09, 7235.35),
            // Fountain turrets
            blue_fountain: Vec2::new(295.04, 7271.23),
            red_fountain: Vec2::new(15020.64, 7301.68),

            turrets: vec![
                // === BLUE TEAM ===
                // Nexus turret (center)
                TurretPlacement { position: Vec2::new(2407.58, 7288.86), team: Team::Blue, structure_type: StructureType::NexusTurret, lane: Lane::Top },
                // Top lane inner
                TurretPlacement { position: Vec2::new(2135.52, 9264.01), team: Team::Blue, structure_type: StructureType::InnerTurret, lane: Lane::Top },
                // Top lane outer
                TurretPlacement { position: Vec2::new(4426.58, 9726.09), team: Team::Blue, structure_type: StructureType::OuterTurret, lane: Lane::Top },
                // Bottom lane inner
                TurretPlacement { position: Vec2::new(2130.30, 5241.26), team: Team::Blue, structure_type: StructureType::InnerTurret, lane: Lane::Bottom },
                // Bottom lane outer
                TurretPlacement { position: Vec2::new(4645.68, 4718.20), team: Team::Blue, structure_type: StructureType::OuterTurret, lane: Lane::Bottom },

                // === RED TEAM ===
                // Nexus turret (center)
                TurretPlacement { position: Vec2::new(13015.47, 7289.87), team: Team::Red, structure_type: StructureType::NexusTurret, lane: Lane::Top },
                // Top lane inner
                TurretPlacement { position: Vec2::new(13291.27, 9260.71), team: Team::Red, structure_type: StructureType::InnerTurret, lane: Lane::Top },
                // Top lane outer
                TurretPlacement { position: Vec2::new(10994.54, 9727.77), team: Team::Red, structure_type: StructureType::OuterTurret, lane: Lane::Top },
                // Bottom lane inner
                TurretPlacement { position: Vec2::new(13297.66, 5259.01), team: Team::Red, structure_type: StructureType::InnerTurret, lane: Lane::Bottom },
                // Bottom lane outer
                TurretPlacement { position: Vec2::new(10775.88, 4715.46), team: Team::Red, structure_type: StructureType::OuterTurret, lane: Lane::Bottom },
            ],

            inhibitors: vec![
                // Blue inhibitors (from Barracks_T1_*.sco.json)
                InhibitorPlacement { position: Vec2::new(2155.87, 8411.25), team: Team::Blue },
                InhibitorPlacement { position: Vec2::new(2147.50, 6117.34), team: Team::Blue },
                // Red inhibitors (from Barracks_T2_*.sco.json)
                InhibitorPlacement { position: Vec2::new(13284.70, 8408.06), team: Team::Red },
                InhibitorPlacement { position: Vec2::new(13295.38, 6124.81), team: Team::Red },
            ],

            nexuses: vec![
                // From HQ_T1/T2.sco.json CentralPoint
                NexusPlacement { position: Vec2::new(2981.04, 7283.01), team: Team::Blue },
                NexusPlacement { position: Vec2::new(12379.54, 7289.94), team: Team::Red },
            ],

            altars: [
                // TT_Buffplat_L and TT_Buffplat_R (approximate from map center analysis)
                AltarPlacement { position: Vec2::new(5400.0, 6400.0), side: AltarSide::Left },
                AltarPlacement { position: Vec2::new(9900.0, 6400.0), side: AltarSide::Right },
            ],

            // From NeutralMinionSpawn.cs Vilemaw camp center
            vilemaw_spawn: Vec2::new(7711.15, 10080.0),

            jungle_camps: vec![
                // From NeutralMinionSpawn.cs exact positions
                CampPlacement { position: Vec2::new(4414.48, 5774.88), camp_type: CampType::Wraith, team_side: Team::Blue },
                CampPlacement { position: Vec2::new(5088.37, 8065.55), camp_type: CampType::Golem, team_side: Team::Blue },
                CampPlacement { position: Vec2::new(6148.92, 5993.49), camp_type: CampType::Wolf, team_side: Team::Blue },
                CampPlacement { position: Vec2::new(11008.20, 5775.70), camp_type: CampType::Wraith, team_side: Team::Red },
                CampPlacement { position: Vec2::new(10341.30, 8084.77), camp_type: CampType::Golem, team_side: Team::Red },
                CampPlacement { position: Vec2::new(9239.00, 6022.87), camp_type: CampType::Wolf, team_side: Team::Red },
            ],

            // From CreateLevelProps.cs TT_Speedshrine_Gears
            speed_shrine: Vec2::new(7706.31, 6720.39),

            // From NeutralMinionSpawn.cs health relic
            health_relics: vec![
                Vec2::new(7711.15, 6722.67),
            ],

            brush_zones: Vec::new(), // TODO: extract from navmesh

            lane_paths: LanePaths {
                top_blue: top_lane_waypoints,
                top_red: top_lane_reversed,
                bottom_blue: bottom_lane_waypoints,
                bottom_red: bottom_lane_reversed,
            },
        }
    }
}
