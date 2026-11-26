use bevy::{asset::AssetPlugin, prelude::*};
use std::path::PathBuf;

mod company;
mod core;
mod economy;
mod interaction;
mod office;
mod player;
mod ui;

use company::CompanyPlugin;
use core::TimePlugin;
use economy::EconomyPlugin;
use interaction::InteractionPlugin;
use office::OfficePlugin;
use player::PlayerPlugin;
use ui::GameUiPlugin;

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
                        title: "Startup Tycoon - M3 Office Prototype".into(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            TimePlugin,
            CompanyPlugin,
            EconomyPlugin,
            OfficePlugin,
            PlayerPlugin,
            InteractionPlugin,
            GameUiPlugin,
        ))
        .run();

    // Ensure background audio/gamepad threads exit cleanly when the window closes
    std::process::exit(0);
}
