use bevy::prelude::*;

// ── LoL Color Palette ──────────────────────────────────────────────
pub const DARK_NAVY: Color = Color::srgb(0.04, 0.08, 0.16);       // #0A1428
pub const DARK_BG: Color = Color::srgb(0.004, 0.04, 0.075);       // #010A13
pub const PANEL_BG: Color = Color::srgb(0.12, 0.14, 0.16);        // #1E2328
pub const PANEL_BG_ALPHA: Color = Color::srgba(0.06, 0.07, 0.10, 0.85);
pub const BORDER_GRAY: Color = Color::srgb(0.24, 0.24, 0.25);     // #3C3C41
pub const GOLD: Color = Color::srgb(0.78, 0.61, 0.24);            // #C89B3C
pub const GOLD_LIGHT: Color = Color::srgb(0.94, 0.90, 0.82);      // #F0E6D2
pub const GOLD_BRIGHT: Color = Color::srgb(0.78, 0.67, 0.43);     // #C8AA6E
pub const BLUE_ACCENT: Color = Color::srgb(0.01, 0.59, 0.67);     // #0397AB
pub const BLUE_TEAM: Color = Color::srgb(0.18, 0.47, 0.78);       // #2E78C8
pub const RED_TEAM: Color = Color::srgb(0.78, 0.18, 0.25);        // #C82E40
pub const TEXT_WHITE: Color = Color::srgb(0.63, 0.61, 0.55);       // #A09B8C
pub const TEXT_BRIGHT: Color = Color::srgb(0.94, 0.90, 0.82);      // #F0E6D2
pub const RED: Color = Color::srgb(0.91, 0.25, 0.34);             // #E84057
pub const GREEN: Color = Color::srgb(0.04, 0.81, 0.51);           // #0ACF83
pub const TRANSPARENT: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);

// Font sizes
pub const FONT_TITLE: f32 = 42.0;
pub const FONT_HEADING: f32 = 28.0;
pub const FONT_SUBHEADING: f32 = 20.0;
pub const FONT_BODY: f32 = 16.0;
pub const FONT_SMALL: f32 = 13.0;
pub const FONT_TINY: f32 = 11.0;

// ── Font Resources ─────────────────────────────────────────────────

/// Holds handles to the LoL fonts (Beaufort for headings, Spiegel for body)
#[derive(Resource)]
pub struct UiFonts {
    pub beaufort_bold: Handle<Font>,
    pub beaufort_regular: Handle<Font>,
    pub beaufort_heavy: Handle<Font>,
    pub beaufort_light: Handle<Font>,
    pub spiegel_regular: Handle<Font>,
    pub spiegel_semibold: Handle<Font>,
    pub spiegel_bold: Handle<Font>,
}

/// Holds handles to common UI images
#[derive(Resource)]
pub struct UiAssets {
    pub main_background: Handle<Image>,
    pub postgame_background: Handle<Image>,
    pub league_logo: Handle<Image>,
}

pub fn load_ui_assets(app: &mut App) {
    let asset_server = app.world().resource::<AssetServer>();
    let fonts = UiFonts {
        beaufort_bold: asset_server.load("ui/fonts/beaufortforlol-bold.otf"),
        beaufort_regular: asset_server.load("ui/fonts/beaufortforlol-regular.otf"),
        beaufort_heavy: asset_server.load("ui/fonts/beaufortforlol-heavy.otf"),
        beaufort_light: asset_server.load("ui/fonts/beaufortforlol-light.otf"),
        spiegel_regular: asset_server.load("ui/fonts/spiegel-regular.otf"),
        spiegel_semibold: asset_server.load("ui/fonts/spiegel-semibold.otf"),
        spiegel_bold: asset_server.load("ui/fonts/spiegel-bold.otf"),
    };
    let ui_assets = UiAssets {
        main_background: asset_server.load("ui/backgrounds/main-background.png"),
        postgame_background: asset_server.load("ui/backgrounds/postgame-background.png"),
        league_logo: asset_server.load("ui/icons/league-logo.png"),
    };
    app.insert_resource(fonts);
    app.insert_resource(ui_assets);
}

// ── Helper: heading text (Beaufort Bold) ───────────────────────────
pub fn heading_font(fonts: &UiFonts, size: f32) -> TextFont {
    TextFont {
        font: fonts.beaufort_bold.clone(),
        font_size: size,
        ..default()
    }
}

// ── Helper: body text (Spiegel Regular) ────────────────────────────
pub fn body_font(fonts: &UiFonts, size: f32) -> TextFont {
    TextFont {
        font: fonts.spiegel_regular.clone(),
        font_size: size,
        ..default()
    }
}

// ── Helper: body semibold (Spiegel Semibold) ───────────────────────
pub fn body_semibold_font(fonts: &UiFonts, size: f32) -> TextFont {
    TextFont {
        font: fonts.spiegel_semibold.clone(),
        font_size: size,
        ..default()
    }
}

/// Spawn a gold-bordered button with text (uses LoL fonts)
pub fn spawn_button(parent: &mut ChildSpawnerCommands<'_>, text: &str, marker: impl Component, width: f32, height: f32, fonts: &UiFonts) {
    parent.spawn((
        Node {
            width: Val::Px(width),
            height: Val::Px(height),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.06, 0.02, 0.9)),
        BorderColor::all(GOLD),
        Interaction::default(),
        marker,
    )).with_children(|btn| {
        btn.spawn((
            Text::new(text),
            heading_font(fonts, FONT_BODY),
            TextColor(GOLD_LIGHT),
        ));
    });
}

/// Spawn a navigation tab button
pub fn spawn_nav_tab(parent: &mut ChildSpawnerCommands<'_>, text: &str, marker: impl Component, fonts: &UiFonts) {
    parent.spawn((
        Node {
            padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Interaction::default(),
        marker,
    )).with_children(|btn| {
        btn.spawn((
            Text::new(text),
            body_semibold_font(fonts, FONT_SMALL),
            TextColor(TEXT_WHITE),
        ));
    });
}

/// Spawn a dark panel with optional gold border
pub fn spawn_panel(parent: &mut ChildSpawnerCommands<'_>, width: Val, height: Val, bordered: bool) -> Entity {
    let cmd = parent.spawn((
        Node {
            width,
            height,
            padding: UiRect::all(Val::Px(15.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            border: if bordered { UiRect::all(Val::Px(1.0)) } else { UiRect::all(Val::Px(0.0)) },
            ..default()
        },
        BackgroundColor(PANEL_BG_ALPHA),
        BorderColor::all(if bordered { GOLD_BRIGHT } else { TRANSPARENT }),
    ));
    cmd.id()
}

/// Big primary action button (like FIND MATCH / LOCK IN)
pub fn spawn_primary_button(parent: &mut ChildSpawnerCommands<'_>, text: &str, marker: impl Component, fonts: &UiFonts) {
    parent.spawn((
        Node {
            width: Val::Px(300.0),
            height: Val::Px(70.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.12, 0.10, 0.03, 0.95)),
        BorderColor::all(GOLD),
        Interaction::default(),
        marker,
    )).with_children(|btn| {
        btn.spawn((
            Text::new(text),
            heading_font(fonts, FONT_HEADING),
            TextColor(GOLD_LIGHT),
        ));
    });
}

/// Apply hover effect on buttons
pub fn button_hover_system(
    mut interactions: Query<(&Interaction, &mut BackgroundColor, &mut BorderColor), Changed<Interaction>>,
) {
    for (interaction, mut bg, mut border) in &mut interactions {
        match interaction {
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgba(0.15, 0.12, 0.05, 0.95));
                *border = BorderColor::all(GOLD_LIGHT);
            }
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgba(0.20, 0.16, 0.06, 1.0));
                *border = BorderColor::all(Color::WHITE);
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgba(0.08, 0.06, 0.02, 0.9));
                *border = BorderColor::all(GOLD);
            }
        }
    }
}
