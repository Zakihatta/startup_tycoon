use bevy::{asset::AssetPlugin, prelude::*};
use std::path::PathBuf;

mod office;
mod player;

use office::OfficePlugin;
use player::{CameraMode, PlayerController, PlayerPlugin};

fn resolve_assets_dir() -> String {
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let beside_exe = exe_dir.join("assets");
        if beside_exe.is_dir() {
            return beside_exe.to_string_lossy().into_owned();
        }
    }
    let cwd_assets = PathBuf::from("assets");
    if cwd_assets.is_dir()
        && let Ok(abs) = std::fs::canonicalize(&cwd_assets)
    {
        return abs.to_string_lossy().into_owned();
    }
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        let manifest_assets = PathBuf::from(manifest_dir).join("assets");
        if manifest_assets.is_dir() {
            return manifest_assets.to_string_lossy().into_owned();
        }
    }
    "assets".to_string()
}

fn main() {
    let assets_dir = resolve_assets_dir();
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.68, 0.80, 0.92)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.98, 0.96, 0.92),
            brightness: 450.0,
            ..default()
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: assets_dir,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Startup Tycoon - M1 Office Prototype".into(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((OfficePlugin, PlayerPlugin))
        .add_systems(Startup, setup_hud)
        .add_systems(Update, update_hud_mode)
        .run();
}

#[derive(Component)]
struct HudHintText;

fn setup_hud(mut commands: Commands) {
    // Controls hint overlay
    commands.spawn((
        HudHintText,
        Text::new(
            "Startup Tycoon (M1 Prototype) - Mode: 3rd Person\n\
             [Klik Kiri] Kunci Mouse | [ESC] Lepas Mouse\n\
             [WASD] Jalan | [Shift] Lari | [V] 1st/3rd Person | [E] Buka/Tutup Pintu",
        ),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.92)),
        BackgroundColor(Color::srgba(0.08, 0.10, 0.14, 0.65)),
        Node {
            position_type: PositionType::Absolute,
            top: px(14.0),
            left: px(14.0),
            padding: UiRect::all(px(10.0)),
            ..default()
        },
    ));

    // Simple center crosshair dot
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: px(6.0),
                    height: px(6.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));
        });
}

fn update_hud_mode(
    player_q: Query<&PlayerController, Changed<PlayerController>>,
    mut text_q: Query<&mut Text, With<HudHintText>>,
) {
    let Ok(ctrl) = player_q.single() else {
        return;
    };
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };
    let mode_label = match ctrl.camera_mode {
        CameraMode::FirstPerson => "1st Person",
        CameraMode::ThirdPerson => "3rd Person",
    };
    text.0 = format!(
        "Startup Tycoon (M1 Prototype) - Mode: {mode_label}\n\
         [Klik Kiri] Kunci Mouse | [ESC] Lepas Mouse\n\
         [WASD] Jalan | [Shift] Lari | [V] 1st/3rd Person | [E] Buka/Tutup Pintu"
    );
}
