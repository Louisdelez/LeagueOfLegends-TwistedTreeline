use bevy::prelude::*;
pub use bevy::window::PrimaryWindow;
use bevy_egui::egui;
use std::collections::HashMap;
use std::time::Instant;

pub const MAP_SIZE: f32 = 15398.0;
pub const TERRAIN_RES: usize = 128;
pub const TILE_SIZE: f32 = MAP_SIZE / TERRAIN_RES as f32;
pub const MAX_UNDO: usize = 200;

// ─── Color palette (modern dark theme) ───
pub const BG_BASE: (u8, u8, u8) = (18, 18, 22);
pub const BG_SURFACE: (u8, u8, u8) = (24, 24, 30);
pub const BG_ELEVATED: (u8, u8, u8) = (32, 32, 40);
pub const BG_HOVER: (u8, u8, u8) = (42, 42, 54);
pub const BG_ACTIVE: (u8, u8, u8) = (52, 52, 68);
pub const ACCENT: (u8, u8, u8) = (99, 132, 255);
pub const TEXT_PRIMARY: (u8, u8, u8) = (230, 230, 240);
pub const TEXT_SECONDARY: (u8, u8, u8) = (140, 140, 160);
pub const TEXT_MUTED: (u8, u8, u8) = (90, 90, 110);
pub const BORDER: (u8, u8, u8) = (45, 45, 58);
pub const SUCCESS: (u8, u8, u8) = (80, 200, 120);
pub const WARNING: (u8, u8, u8) = (240, 180, 60);

pub const CATEGORIES: &[&str] = &["All", "Lane", "Jungle", "Vilemaw", "Base", "Autel", "Transition", "Structure", "General"];

pub fn rgb(c: (u8, u8, u8)) -> egui::Color32 { egui::Color32::from_rgb(c.0, c.1, c.2) }

pub fn li(icon: lucide_icons::Icon) -> String { icon.unicode().to_string() }

pub fn cat_color(cat: &str) -> (u8, u8, u8) {
    match cat {
        "Lane" => (100, 105, 130), "Jungle" => (50, 75, 50), "Vilemaw" => (60, 40, 65),
        "Base" => (55, 65, 100), "Autel" => (95, 75, 45), "Transition" => (80, 75, 60),
        "Structure" => (90, 80, 105), "General" => (70, 80, 70), _ => (70, 70, 80),
    }
}

// ─── Enums ───

#[derive(Clone, Copy, PartialEq)]
pub enum Tool { Paint, Erase, Pick, Fill, Smooth, Hand, Select, Clone, FloodFill, Raise }

#[derive(Clone, Copy, PartialEq)]
pub enum BrushShape { Circle, Square }

#[derive(Clone, Copy, PartialEq)]
pub enum Falloff { Smooth, Linear, Sharp, Constant }

#[derive(Clone, Copy, PartialEq)]
pub enum ShadingMode { Textured, CategoryColor }

// ─── Resources ───

#[derive(Resource)]
pub struct EditorState {
    pub tex: usize, pub cells: Vec<u8>, pub tool: Tool, pub grid: bool, pub show_map: bool,
    pub cursor: Vec3, pub painted: usize, pub status: String, pub filter: String,
    pub is_painting_stroke: bool, pub stroke_cells: Vec<(usize, u8)>,
    pub stroke_heights: Vec<(usize, f32)>, // (idx, old_height)
    pub loaded: bool, pub show_help: bool, pub cat_filter: usize,
    pub open_menu: Option<&'static str>,
    pub hovered_tex: Option<usize>,
    pub autosave_timer: f32, pub last_save_time: f32,
    pub show_overlay: bool, pub cam_height: f32, pub shading_mode: ShadingMode,
    pub clone_source: Option<(i32, i32)>,
    pub heights: Vec<f32>,
    pub noise_scale: f32, pub noise_tex2: usize,
    pub show_palette: bool, pub palette_query: String,
    pub needs_refresh: bool, pub search_focused: bool,
    pub show_left_panel: bool, pub show_right_panel: bool,
}
impl Default for EditorState {
    fn default() -> Self {
        Self {
            tex: 0, cells: vec![0; TERRAIN_RES * TERRAIN_RES], tool: Tool::Paint,
            grid: true, show_map: true, cursor: Vec3::ZERO, painted: 0,
            status: "Ready".into(), filter: String::new(),
            is_painting_stroke: false, stroke_cells: Vec::new(), stroke_heights: Vec::new(), open_menu: None,
            loaded: false, show_help: false, cat_filter: 0,
            hovered_tex: None, autosave_timer: 0.0, last_save_time: 0.0,
            show_overlay: true, cam_height: 5000.0, shading_mode: ShadingMode::Textured,
            clone_source: None, heights: vec![0.0; TERRAIN_RES * TERRAIN_RES],
            noise_scale: 0.05, noise_tex2: 1,
            show_palette: false, palette_query: String::new(), needs_refresh: false, search_focused: false,
            show_left_panel: true, show_right_panel: true,
        }
    }
}

#[derive(Resource)]
pub struct BrushSettings { pub size: f32, pub opacity: f32, pub shape: BrushShape, pub falloff: Falloff, pub sym_x: bool, pub sym_z: bool }
impl Default for BrushSettings {
    fn default() -> Self { Self { size: 3.0, opacity: 1.0, shape: BrushShape::Circle, falloff: Falloff::Smooth, sym_x: false, sym_z: false } }
}

#[derive(Resource)]
pub struct TexturePalette { pub entries: Vec<TexEntry> }
pub struct TexEntry { pub name: String, pub desc: String, pub cat: String, pub image: Handle<Image> }
impl Default for TexturePalette { fn default() -> Self { Self { entries: vec![] } } }

#[derive(Resource, Default)]
pub struct UndoHistory { pub undo_stack: Vec<UndoAction>, pub redo_stack: Vec<UndoAction> }
#[derive(Clone)]
pub struct UndoAction {
    pub changes: Vec<(usize, u8, u8)>, // (cell_index, old_val, new_val)
    pub height_changes: Vec<(usize, f32, f32)>, // (cell_index, old_height, new_height)
    pub desc: String,
}

#[derive(Resource, Default)]
pub struct FpsTracker { pub frames: u32, pub last_update: Option<Instant>, pub fps: f32 }

#[derive(Resource, Default)]
pub struct TextureEguiCache { pub map: HashMap<usize, egui::TextureId>, pub initialized: bool, pub fonts_loaded: bool }

#[derive(Resource, Default)]
pub struct MaterialCache {
    pub texture_mats: HashMap<usize, Handle<StandardMaterial>>,
    pub checker_dark: Option<Handle<StandardMaterial>>,
    pub checker_light: Option<Handle<StandardMaterial>>,
}

#[derive(Resource, Default)]
pub struct UiClickZones {
    pub tool_buttons: Vec<(egui::Rect, Tool)>,
    pub texture_rows: Vec<(egui::Rect, usize)>,
    pub category_pills: Vec<(egui::Rect, usize)>,
    pub shape_circle: Option<egui::Rect>,
    pub shape_square: Option<egui::Rect>,
    pub size_minus: Option<egui::Rect>,
    pub size_plus: Option<egui::Rect>,
    pub opacity_minus: Option<egui::Rect>,
    pub opacity_plus: Option<egui::Rect>,
    pub rp_size_minus: Option<egui::Rect>,
    pub rp_size_plus: Option<egui::Rect>,
    pub rp_opacity_minus: Option<egui::Rect>,
    pub rp_opacity_plus: Option<egui::Rect>,
    pub rp_grid_toggle: Option<egui::Rect>,
    pub rp_model_toggle: Option<egui::Rect>,
    pub rp_overlay_toggle: Option<egui::Rect>,
    pub rp_sym_x: Option<egui::Rect>,
    pub rp_sym_z: Option<egui::Rect>,
    pub rp_falloff_btns: Vec<(egui::Rect, Falloff)>,
    pub nav_gizmo_btns: Vec<(egui::Rect, &'static str)>,
    pub menu_buttons: Vec<(egui::Rect, &'static str)>,
    pub menu_items: Vec<(egui::Rect, String)>,
    pub layer_rows: Vec<(egui::Rect, usize)>,
    pub layer_eye: Vec<(egui::Rect, usize)>,
    pub layer_add: Option<egui::Rect>,
    pub search_bar: Option<egui::Rect>,
    pub preset_btns: Vec<(egui::Rect, usize)>,
    pub rp_shading_btns: Vec<(egui::Rect, ShadingMode)>,
    // Asset browser
    pub browser_items: Vec<(egui::Rect, usize)>,
    pub browser_back: Option<egui::Rect>,
    pub browser_search: Option<egui::Rect>,
    pub browser_breadcrumbs: Vec<(egui::Rect, String)>,
    pub noise_scale_minus: Option<egui::Rect>,
    pub noise_scale_plus: Option<egui::Rect>,
    pub noise_tex2_btn: Option<egui::Rect>,
    pub toggle_left: Option<egui::Rect>,
    pub toggle_right: Option<egui::Rect>,
    pub toggle_browser: Option<egui::Rect>,
    pub any_panel_rect: Vec<egui::Rect>,
}

#[derive(Resource, Default)]
pub struct TerrainIndex(pub HashMap<(usize, usize), Entity>);

#[derive(Resource, Default)]
pub struct PointerOverUi(pub bool);

#[derive(Resource, Default)]
pub struct Selection {
    pub rect: Option<(usize, usize, usize, usize)>,
    pub dragging: Option<(usize, usize)>,
}

#[derive(Clone)]
pub struct BrushPreset {
    pub name: String,
    pub size: f32, pub opacity: f32, pub shape: BrushShape, pub falloff: Falloff, pub sym_x: bool, pub sym_z: bool,
}

#[derive(Resource)]
pub struct BrushPresets(pub Vec<BrushPreset>);
impl Default for BrushPresets {
    fn default() -> Self {
        Self(vec![
            BrushPreset { name: "Fine".into(), size: 1.0, opacity: 1.0, shape: BrushShape::Circle, falloff: Falloff::Constant, sym_x: false, sym_z: false },
            BrushPreset { name: "Large".into(), size: 8.0, opacity: 1.0, shape: BrushShape::Circle, falloff: Falloff::Smooth, sym_x: false, sym_z: false },
            BrushPreset { name: "Soft Edge".into(), size: 5.0, opacity: 0.5, shape: BrushShape::Circle, falloff: Falloff::Smooth, sym_x: false, sym_z: false },
            BrushPreset { name: "Symmetric".into(), size: 4.0, opacity: 1.0, shape: BrushShape::Circle, falloff: Falloff::Smooth, sym_x: true, sym_z: true },
        ])
    }
}

// ─── Layers ───

#[derive(Clone)]
pub struct Layer {
    pub name: String,
    pub cells: Vec<u8>,
    pub visible: bool,
    pub locked: bool,
}
impl Layer {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), cells: vec![0; TERRAIN_RES * TERRAIN_RES], visible: true, locked: false }
    }
}

#[derive(Resource)]
pub struct LayerStack {
    pub layers: Vec<Layer>,
    pub active: usize,
}
impl Default for LayerStack {
    fn default() -> Self {
        Self { layers: vec![Layer::new("Base")], active: 0 }
    }
}

// ─── Asset Browser ───

#[derive(Clone)]
pub struct AssetEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub category: String,
    pub thumb_stem: String, // filename without ext, for thumbnail lookup
}

#[derive(Resource)]
pub struct AssetBrowser {
    pub entries: Vec<AssetEntry>,
    pub current_path: String,
    pub search: String,
    pub search_focused: bool,
    pub show: bool,
    pub loaded: bool,
    pub selected: Option<usize>,
    pub thumbnails: HashMap<String, egui::TextureId>,
    pub thumb_handles: Vec<egui::TextureHandle>,
    pub logo: Option<egui::TextureHandle>,
    pub thumbs_loaded: bool,
}
impl Default for AssetBrowser {
    fn default() -> Self {
        Self {
            entries: Vec::new(), current_path: "ALL".into(),
            search: String::new(), search_focused: false,
            show: true, loaded: false, selected: None,
            thumbnails: HashMap::new(), thumbs_loaded: false, thumb_handles: Vec::new(), logo: None,
        }
    }
}

// ─── Components ───

#[derive(Component)] pub struct EditorCamera;
#[derive(Component)] pub struct TerrainChunk { pub gx: usize, pub gz: usize }
#[derive(Component)] pub struct BrushPreview;
#[derive(Component)] pub struct MapModel;

// ─── Helpers ───

pub fn cursor_egui_pos(wins: &Query<&Window, With<PrimaryWindow>>) -> Option<egui::Pos2> {
    let Ok(w) = wins.single() else { return None; };
    let Some(c) = w.cursor_position() else { return None; };
    Some(egui::pos2(c.x, c.y))
}

pub fn is_over_ui(wins: &Query<&Window, With<PrimaryWindow>>, zones: &UiClickZones) -> bool {
    let Some(pos) = cursor_egui_pos(wins) else { return false; };
    is_pos_over_ui(pos, zones)
}

pub fn is_pos_over_ui(pos: egui::Pos2, zones: &UiClickZones) -> bool {
    zones.any_panel_rect.iter().any(|r| r.contains(pos))
        || zones.nav_gizmo_btns.iter().any(|(r, _)| r.contains(pos))
}

// ─── UI helpers ───

pub fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).size(11.0).color(rgb(TEXT_MUTED)).strong());
        ui.add_space(4.0);
        let avail = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(avail, 1.0), egui::Sense::hover());
        ui.painter().line_segment(
            [egui::pos2(rect.left(), rect.center().y), egui::pos2(rect.right(), rect.center().y)],
            egui::Stroke::new(0.5, rgb(BORDER)),
        );
    });
}

pub fn info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).size(10.5).color(rgb(TEXT_MUTED)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value).size(10.5).color(rgb(TEXT_PRIMARY)).family(egui::FontFamily::Monospace));
        });
    });
}

pub fn separator_v(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 20.0), egui::Sense::hover());
    ui.painter().line_segment(
        [rect.left_top(), rect.left_bottom()],
        egui::Stroke::new(1.0, rgb(BORDER)),
    );
}
